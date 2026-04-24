//! Bridge InfluxDB 2.x — écriture en batch des snapshots BMS.
//!
//! Chaque snapshot est converti en points InfluxDB et accumulé dans un batch.
//! Le batch est flushé soit quand il atteint `batch_size`, soit toutes les
//! `batch_flush_interval_sec` secondes.

use crate::config::InfluxConfig;
use crate::et112::Et112Snapshot;
use crate::irradiance::IrradianceSnapshot;
use crate::state::{AppState, VenusInverter, VenusSmartShunt};
use crate::tasmota::TasmotaSnapshot;
use daly_bms_core::types::BmsSnapshot;
use chrono::Utc;
use influxdb2::Client;
use influxdb2::models::DataPoint;
use tracing::{error, info, warn};
use std::time::Duration;

/// Démarre la tâche d'écriture InfluxDB en arrière-plan.
pub async fn run_influx_bridge(state: AppState, cfg: InfluxConfig) {
    if !cfg.enabled {
        info!("InfluxDB bridge désactivé (enabled = false)");
        return;
    }
    if cfg.token.is_empty() {
        warn!("InfluxDB : token vide, bridge désactivé");
        return;
    }

    info!(url = %cfg.url, org = %cfg.org, bucket = %cfg.bucket, "Démarrage InfluxDB bridge");

    let client = Client::new(&cfg.url, &cfg.org, &cfg.token);

    let mut batch: Vec<DataPoint> = Vec::with_capacity(cfg.batch_size);
    let mut rx = state.subscribe_ws();
    let flush_interval = Duration::from_secs_f64(cfg.batch_flush_interval_sec.max(1.0));
    let mut flush_ticker = tokio::time::interval(flush_interval);
    // Ticker séparé pour polling ET112 + Tasmota + irradiance (état est dans state)
    let et112_interval = Duration::from_secs(10);
    let mut et112_ticker = tokio::time::interval(et112_interval);
    let tasmota_interval = Duration::from_secs(10);
    let mut tasmota_ticker = tokio::time::interval(tasmota_interval);
    let irradiance_interval = Duration::from_secs(30);
    let mut irradiance_ticker = tokio::time::interval(irradiance_interval);
    // Ticker Venus OS (MPPT total + SmartShunt + Inverter) — pour historique des edges DC/AC.
    let venus_interval = Duration::from_secs(10);
    let mut venus_ticker = tokio::time::interval(venus_interval);

    loop {
        tokio::select! {
            Ok(snaps) = rx.recv() => {
                for snap in snaps.iter() {
                    let points = snapshot_to_points(snap);
                    batch.extend(points);
                }
                if batch.len() >= cfg.batch_size {
                    flush_batch(&client, &cfg.bucket, &mut batch).await;
                }
            }
            _ = et112_ticker.tick() => {
                // Lire les derniers snapshots ET112 et les écrire dans InfluxDB
                let et112_snaps = state.et112_latest_all().await;
                for snap in et112_snaps {
                    if let Ok(p) = et112_snapshot_to_point(&snap) {
                        batch.push(p);
                    }
                }
                if !batch.is_empty() {
                    flush_batch(&client, &cfg.bucket, &mut batch).await;
                }
            }
            _ = tasmota_ticker.tick() => {
                let tasmota_snaps = state.tasmota_latest_all().await;
                for snap in tasmota_snaps {
                    if let Ok(p) = tasmota_snapshot_to_point(&snap) {
                        batch.push(p);
                    }
                }
                if !batch.is_empty() {
                    flush_batch(&client, &cfg.bucket, &mut batch).await;
                }
            }
            _ = irradiance_ticker.tick() => {
                if let Some(snap) = state.latest_irradiance().await {
                    if let Ok(p) = irradiance_snapshot_to_point(&snap) {
                        batch.push(p);
                    }
                }
                // Production solaire journalière (total_yield_kwh + solar_total_w).
                let yield_kwh  = *state.mppt_yield_kwh.read().await;
                let solar_w    = *state.solar_total_w.read().await;
                if let Ok(p) = solar_daily_to_point(yield_kwh, solar_w) {
                    batch.push(p);
                }
                if !batch.is_empty() {
                    flush_batch(&client, &cfg.bucket, &mut batch).await;
                }
            }
            _ = venus_ticker.tick() => {
                // Total MPPT (somme de toutes les instances SolarCharger).
                let mppt_power   = state.venus_mppt_total_power().await;
                let mppt_current = state.venus_mppt_total_dc_current().await;
                if let Ok(p) = venus_mppt_total_to_point(mppt_power, mppt_current) {
                    batch.push(p);
                }
                if let Some(shunt) = state.venus_smartshunt_get().await {
                    if let Ok(p) = venus_smartshunt_to_point(&shunt) {
                        batch.push(p);
                    }
                }
                if let Some(inv) = state.venus_inverter_get().await {
                    if let Ok(p) = venus_inverter_to_point(&inv) {
                        batch.push(p);
                    }
                }
                if !batch.is_empty() {
                    flush_batch(&client, &cfg.bucket, &mut batch).await;
                }
            }
            _ = flush_ticker.tick() => {
                if !batch.is_empty() {
                    flush_batch(&client, &cfg.bucket, &mut batch).await;
                }
            }
        }
    }
}

/// Convertit un [`TasmotaSnapshot`] en un point InfluxDB.
///
/// Measurement : `tasmota_status`
/// Tags : `id`, `name`, `tasmota_id`
fn tasmota_snapshot_to_point(snap: &TasmotaSnapshot) -> anyhow::Result<DataPoint> {
    let ts_ns = snap.timestamp.timestamp_nanos_opt().unwrap_or(0);

    let point = DataPoint::builder("tasmota_status")
        .tag("id",         snap.id.to_string())
        .tag("name",       snap.name.clone())
        .tag("tasmota_id", snap.tasmota_id.clone())
        .field("power_on",            snap.power_on as i64)
        .field("power_w",             snap.power_w as f64)
        .field("voltage_v",           snap.voltage_v as f64)
        .field("current_a",           snap.current_a as f64)
        .field("apparent_power_va",   snap.apparent_power_va as f64)
        .field("power_factor",        snap.power_factor as f64)
        .field("energy_today_kwh",    snap.energy_today_kwh as f64)
        .field("energy_yesterday_kwh",snap.energy_yesterday_kwh as f64)
        .field("energy_total_kwh",    snap.energy_total_kwh as f64)
        .timestamp(ts_ns)
        .build()?;

    Ok(point)
}

/// Convertit un [`IrradianceSnapshot`] en un point InfluxDB.
///
/// Measurement : `irradiance_status`
/// Tags : `address` (hex), `name`
fn irradiance_snapshot_to_point(snap: &IrradianceSnapshot) -> anyhow::Result<DataPoint> {
    let addr_tag = format!("{:#04x}", snap.address);
    let ts_ns = snap.timestamp.timestamp_nanos_opt().unwrap_or(0);

    let point = DataPoint::builder("irradiance_status")
        .tag("address", addr_tag)
        .tag("name",    snap.name.clone())
        .field("irradiance_wm2", snap.irradiance_wm2 as f64)
        .timestamp(ts_ns)
        .build()?;

    Ok(point)
}

/// Flush le batch vers InfluxDB et vide le vecteur.
async fn flush_batch(client: &Client, bucket: &str, batch: &mut Vec<DataPoint>) {
    let points = std::mem::take(batch);
    let count = points.len();
    match client.write(bucket, futures::stream::iter(points)).await {
        Ok(_) => info!(count, "InfluxDB flush OK"),
        Err(e) => error!("InfluxDB flush erreur : {:?}", e),
    }
}

/// Convertit un [`BmsSnapshot`] en plusieurs points InfluxDB.
///
/// Measurement principal : `bms_status`
/// Tags : `address` (hex)
fn snapshot_to_points(snap: &BmsSnapshot) -> Vec<DataPoint> {
    let addr_tag = format!("{:#04x}", snap.address);
    let ts_ns = snap.timestamp.timestamp_nanos_opt().unwrap_or(0) as u128;

    let mut points = vec![
        // ── Status principal ─────────────────────────────────────────────────
        DataPoint::builder("bms_status")
            .tag("address", addr_tag.clone())
            .field("soc",         snap.soc as f64)
            .field("voltage",     snap.dc.voltage as f64)
            .field("current",     snap.dc.current as f64)
            .field("power",       snap.dc.power as f64)
            .field("capacity",    snap.capacity as f64)
            .field("consumed_ah", snap.consumed_amphours as f64)
            .field("temp_max",    snap.system.max_cell_temperature as f64)
            .field("temp_min",    snap.system.min_cell_temperature as f64)
            .field("cell_delta_mv", snap.system.cell_delta_mv() as f64)
            .field("min_cell_v",  snap.system.min_cell_voltage as f64)
            .field("max_cell_v",  snap.system.max_cell_voltage as f64)
            .field("charge_mos",   snap.io.allow_to_charge as i64)
            .field("discharge_mos", snap.io.allow_to_discharge as i64)
            .field("any_alarm",    snap.alarms.any_active() as i64)
            .field("bms_capacity", snap.bms_reported_capacity_ah as f64)
            .field("cycles",       snap.history.charge_cycles as i64)
            .timestamp(ts_ns as i64)
            .build()
            .expect("point valide"),
    ];

    // ── Tensions individuelles ─────────────────────────────────────────────
    for (name, &v) in &snap.voltages {
        if let Ok(p) = DataPoint::builder("bms_cell_voltage")
            .tag("address", addr_tag.clone())
            .tag("cell", name.clone())
            .field("voltage", v as f64)
            .timestamp(ts_ns as i64)
            .build()
        {
            points.push(p);
        }
    }

    points
}

/// Point InfluxDB « total MPPT » — somme des instances SolarCharger actives.
///
/// Measurement : `venus_mppt_total`
/// Pas de tag — singleton.
fn venus_mppt_total_to_point(power_w: f32, current_a: f32) -> anyhow::Result<DataPoint> {
    let ts_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let point = DataPoint::builder("venus_mppt_total")
        .field("power_w",   power_w as f64)
        .field("current_a", current_a as f64)
        .timestamp(ts_ns)
        .build()?;
    Ok(point)
}

/// Point InfluxDB SmartShunt.
///
/// Measurement : `venus_smartshunt`
/// Pas de tag — singleton.
fn venus_smartshunt_to_point(s: &VenusSmartShunt) -> anyhow::Result<DataPoint> {
    let ts_ns = s.timestamp.timestamp_nanos_opt().unwrap_or(0);
    let mut b = DataPoint::builder("venus_smartshunt");
    if let Some(v) = s.voltage_v           { b = b.field("voltage_v",           v as f64); }
    if let Some(v) = s.current_a           { b = b.field("current_a",           v as f64); }
    if let Some(v) = s.power_w             { b = b.field("power_w",             v as f64); }
    if let Some(v) = s.soc_percent         { b = b.field("soc_percent",         v as f64); }
    if let Some(v) = s.energy_in_kwh       { b = b.field("energy_in_kwh",       v as f64); }
    if let Some(v) = s.energy_out_kwh      { b = b.field("energy_out_kwh",      v as f64); }
    if let Some(v) = s.ah_charged_today    { b = b.field("ah_charged_today",    v as f64); }
    if let Some(v) = s.ah_discharged_today { b = b.field("ah_discharged_today", v as f64); }
    Ok(b.timestamp(ts_ns).build()?)
}

/// Point InfluxDB Onduleur/Charger Victron.
///
/// Measurement : `venus_inverter`
/// Pas de tag — singleton.
fn venus_inverter_to_point(i: &VenusInverter) -> anyhow::Result<DataPoint> {
    let ts_ns = i.timestamp.timestamp_nanos_opt().unwrap_or(0);
    let mut b = DataPoint::builder("venus_inverter");
    if let Some(v) = i.voltage_v            { b = b.field("voltage_v",            v as f64); }
    if let Some(v) = i.current_a            { b = b.field("current_a",            v as f64); }
    if let Some(v) = i.power_w              { b = b.field("power_w",              v as f64); }
    if let Some(v) = i.ac_output_voltage_v  { b = b.field("ac_output_voltage_v",  v as f64); }
    if let Some(v) = i.ac_output_current_a  { b = b.field("ac_output_current_a",  v as f64); }
    if let Some(v) = i.ac_output_power_w    { b = b.field("ac_output_power_w",    v as f64); }
    if let Some(v) = i.ac_out_frequency_hz  { b = b.field("ac_out_frequency_hz",  v as f64); }
    Ok(b.timestamp(ts_ns).build()?)
}

/// Convertit un [`Et112Snapshot`] en un point InfluxDB.
///
/// Measurement : `et112_status`
/// Tags : `address` (hex), `name`
fn et112_snapshot_to_point(snap: &Et112Snapshot) -> anyhow::Result<DataPoint> {
    let addr_tag = format!("{:#04x}", snap.address);
    let ts_ns = snap.timestamp.timestamp_nanos_opt().unwrap_or(0);

    let point = DataPoint::builder("et112_status")
        .tag("address", addr_tag)
        .tag("name",    snap.name.clone())
        .field("voltage_v",          snap.voltage_v as f64)
        .field("current_a",          snap.current_a as f64)
        .field("power_w",            snap.power_w as f64)
        .field("apparent_power_va",  snap.apparent_power_va as f64)
        .field("reactive_power_var", snap.reactive_power_var as f64)
        .field("power_factor",       snap.power_factor as f64)
        .field("frequency_hz",       snap.frequency_hz as f64)
        .field("energy_import_wh",   snap.energy_import_wh as f64)
        .field("energy_export_wh",   snap.energy_export_wh as f64)
        .timestamp(ts_ns)
        .build()?;

    Ok(point)
}

/// Point InfluxDB production solaire journalière.
///
/// Measurement : `solar_daily`
/// Écrit toutes les 30 s depuis le ticker irradiance.
fn solar_daily_to_point(yield_kwh: f32, solar_w: f32) -> anyhow::Result<DataPoint> {
    let ts_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let point = DataPoint::builder("solar_daily")
        .field("total_yield_kwh", yield_kwh as f64)
        .field("solar_total_w",   solar_w   as f64)
        .timestamp(ts_ns)
        .build()?;
    Ok(point)
}
