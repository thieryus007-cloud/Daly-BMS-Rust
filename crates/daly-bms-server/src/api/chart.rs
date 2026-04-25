//! Endpoint historique graphique — proxy InfluxDB pour le dashboard overview.
//!
//! GET /api/v1/chart/history?minutes=60
//! Retourne { solar:[{t,v}], soc:[{t,v}], load:[{t,v}] }
//!
//! GET /api/v1/chart/edge-history?measurement=bms_status&field=current&address=0x01&minutes=360
//! Retourne { ok, series:[{t,v}], unit }

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct HistoryParams {
    pub minutes: Option<u32>,
}

#[derive(Deserialize)]
pub struct EdgeHistoryParams {
    pub measurement: String,
    pub field: String,
    /// Optionnel — requis uniquement pour les measurements taggés par adresse
    /// (`bms_status`, `et112_status`). Absent pour les singletons Venus.
    pub address: Option<String>,
    pub minutes: Option<u32>,
}

/// GET /api/v1/chart/history?minutes=X
pub async fn get_chart_history(
    State(state): State<AppState>,
    Query(q): Query<HistoryParams>,
) -> impl IntoResponse {
    let minutes = q.minutes.unwrap_or(60).clamp(1, 720);
    let cfg = &state.config.influxdb;

    if !cfg.enabled || cfg.token.is_empty() {
        return Json(json!({"solar": [], "soc": [], "load": [], "ok": false}));
    }

    let window = if minutes <= 60 { "1m" } else if minutes <= 360 { "5m" } else { "10m" };
    let b = &cfg.bucket;

    let solar_q = format!(
        "from(bucket: \"{b}\") |> range(start: -{minutes}m) \
         |> filter(fn: (r) => r._measurement == \"solar_power\" and r._field == \"solar_total\") \
         |> aggregateWindow(every: {window}, fn: mean, createEmpty: false)"
    );

    let soc_q = format!(
        "from(bucket: \"{b}\") |> range(start: -{minutes}m) \
         |> filter(fn: (r) => r._measurement == \"bms_status\" and r._field == \"soc\") \
         |> aggregateWindow(every: {window}, fn: mean, createEmpty: false)"
    );

    let load_q = format!(
        "from(bucket: \"{b}\") |> range(start: -{minutes}m) \
         |> filter(fn: (r) => r._measurement == \"et112_status\" and r._field == \"power_w\") \
         |> aggregateWindow(every: {window}, fn: mean, createEmpty: false)"
    );

    let url  = format!("{}/api/v2/query?org={}", cfg.url, cfg.org);
    let auth = format!("Token {}", cfg.token);
    let client = reqwest::Client::new();

    let (solar_r, soc_r, load_r) = tokio::join!(
        influx_query(&client, &url, &auth, &solar_q),
        influx_query(&client, &url, &auth, &soc_q),
        influx_query(&client, &url, &auth, &load_q),
    );

    Json(json!({
        "ok":    true,
        "solar": solar_r.unwrap_or_default(),
        "soc":   soc_r.map(average_by_time).unwrap_or_default(),
        "load":  load_r.map(sum_by_time).unwrap_or_default(),
    }))
}

async fn influx_query(
    client: &reqwest::Client,
    url: &str,
    auth: &str,
    flux: &str,
) -> Option<Vec<(String, f64)>> {
    let resp = client
        .post(url)
        .header("Authorization", auth)
        .header("Content-Type", "application/vnd.flux")
        .header("Accept", "application/csv")
        .body(flux.to_string())
        .send()
        .await
        .ok()?;

    if !resp.status().is_success() {
        return None;
    }

    let csv = resp.text().await.ok()?;
    Some(parse_influx_csv(&csv))
}

fn parse_influx_csv(csv: &str) -> Vec<(String, f64)> {
    let mut result = Vec::new();
    let mut time_idx:  Option<usize> = None;
    let mut value_idx: Option<usize> = None;
    let mut in_header = false;

    for raw_line in csv.lines() {
        let line = raw_line.trim_end_matches('\r');

        if line.is_empty() || line.starts_with('#') {
            in_header = false;
            time_idx  = None;
            value_idx = None;
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();

        if !in_header {
            for (i, f) in fields.iter().enumerate() {
                match *f {
                    "_time"  => time_idx  = Some(i),
                    "_value" => value_idx = Some(i),
                    _ => {}
                }
            }
            in_header = true;
            continue;
        }

        if let (Some(ti), Some(vi)) = (time_idx, value_idx) {
            if let (Some(t), Some(v_str)) = (fields.get(ti), fields.get(vi)) {
                if let Ok(v) = v_str.parse::<f64>() {
                    // ISO 8601 → "HH:MM" (chars 11..16)
                    let t_fmt = if t.len() >= 16 { &t[11..16] } else { t };
                    result.push((t_fmt.to_string(), v));
                }
            }
        }
    }

    result
}

/// Moyenne des valeurs par timestamp (SOC multi-BMS → une seule série).
fn average_by_time(rows: Vec<(String, f64)>) -> Vec<Value> {
    let mut map: BTreeMap<String, (f64, usize)> = BTreeMap::new();
    for (t, v) in rows {
        let e = map.entry(t).or_insert((0.0, 0));
        e.0 += v;
        e.1 += 1;
    }
    map.into_iter()
        .map(|(t, (sum, n))| json!({"t": t, "v": (sum / n as f64).round()}))
        .collect()
}

/// Somme des valeurs par timestamp (charges ET112 multi-appareils).
fn sum_by_time(rows: Vec<(String, f64)>) -> Vec<Value> {
    let mut map: BTreeMap<String, f64> = BTreeMap::new();
    for (t, v) in rows {
        *map.entry(t).or_insert(0.0) += v;
    }
    map.into_iter()
        .map(|(t, v)| json!({"t": t, "v": v.round()}))
        .collect()
}

/// GET /api/v1/chart/edge-history?measurement=...&field=...&address=...&minutes=360
///
/// Renvoie la série temporelle brute d'un champ (courant) pour un appareil donné.
/// Utilisé par les overlays de graphique sur les edges de la page visualisation.
pub async fn get_edge_history(
    State(state): State<AppState>,
    Query(q): Query<EdgeHistoryParams>,
) -> impl IntoResponse {
    let minutes = q.minutes.unwrap_or(360).clamp(1, 1440);
    let cfg = &state.config.influxdb;

    if !cfg.enabled || cfg.token.is_empty() {
        return Json(json!({ "ok": false, "series": [], "reason": "influxdb_disabled" }));
    }

    // Whitelist stricte pour éviter l'injection Flux.
    // address_required indique si le filtre `r.address == …` doit être appliqué.
    let (measurement, address_required) = match q.measurement.as_str() {
        "bms_status"        => ("bms_status",        true),
        "et112_status"      => ("et112_status",      true),
        // energy-manager real measurement names
        "battery_status"    => ("battery_status",    false),
        "inverter_status"   => ("inverter_status",   false),
        "solar_power"       => ("solar_power",       false),
        // legacy aliases (kept for backward compat — never written, return empty series)
        "venus_mppt_total"  => ("venus_mppt_total",  false),
        "venus_smartshunt"  => ("venus_smartshunt",  false),
        "venus_inverter"    => ("venus_inverter",    false),
        _ => return Json(json!({ "ok": false, "series": [], "reason": "bad_measurement" })),
    };
    let field = match q.field.as_str() {
        "current"              => "current",
        "current_a"            => "current_a",
        // inverter_status field names (energy-manager)
        "dc_current_a"         => "dc_current_a",
        "ac_out_current_a"     => "ac_out_current_a",
        "dc_power_w"           => "dc_power_w",
        "ac_out_power_w"       => "ac_out_power_w",
        // solar_power field names (energy-manager)
        "mppt_power_w"         => "mppt_power_w",
        // legacy / generic
        "ac_output_current_a"  => "ac_output_current_a",
        "power_w"              => "power_w",
        _ => return Json(json!({ "ok": false, "series": [], "reason": "bad_field" })),
    };

    // Adresse : requise (et re-normalisée) seulement pour les measurements tagués.
    let address = if address_required {
        let raw = match q.address.as_deref() {
            Some(s) if !s.is_empty() => s,
            _ => return Json(json!({ "ok": false, "series": [], "reason": "missing_address" })),
        };
        let addr_num: u32 = if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
            match u32::from_str_radix(hex, 16) {
                Ok(n) => n,
                Err(_) => return Json(json!({ "ok": false, "series": [], "reason": "bad_address" })),
            }
        } else {
            match raw.parse::<u32>() {
                Ok(n) => n,
                Err(_) => return Json(json!({ "ok": false, "series": [], "reason": "bad_address" })),
            }
        };
        Some(format!("{:#04x}", addr_num))
    } else {
        None
    };

    let window = if minutes <= 60 { "1m" } else if minutes <= 360 { "3m" } else { "10m" };
    let b = &cfg.bucket;

    let addr_filter = match &address {
        Some(a) => format!(" and r.address == \"{a}\""),
        None    => String::new(),
    };
    let flux = format!(
        "from(bucket: \"{b}\") |> range(start: -{minutes}m) \
         |> filter(fn: (r) => r._measurement == \"{measurement}\" and r._field == \"{field}\"{addr_filter}) \
         |> aggregateWindow(every: {window}, fn: mean, createEmpty: false)"
    );

    let url  = format!("{}/api/v2/query?org={}", cfg.url, cfg.org);
    let auth = format!("Token {}", cfg.token);
    let client = reqwest::Client::new();

    let series: Vec<Value> = match influx_query(&client, &url, &auth, &flux).await {
        Some(rows) => rows.into_iter()
            .map(|(t, v)| json!({ "t": t, "v": (v * 100.0).round() / 100.0 }))
            .collect(),
        None => Vec::new(),
    };

    let unit = match field {
        "current" | "current_a" | "dc_current_a" | "ac_out_current_a" | "ac_output_current_a" => "A",
        "power_w" | "mppt_power_w" | "dc_power_w" | "ac_out_power_w" => "W",
        _ => "",
    };

    Json(json!({
        "ok":      true,
        "series":  series,
        "unit":    unit,
        "address": address,
        "field":   field,
        "measurement": measurement,
        "minutes": minutes,
    }))
}
