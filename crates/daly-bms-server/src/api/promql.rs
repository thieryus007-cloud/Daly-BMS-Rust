//! Endpoints PromQL compatibles Prometheus HTTP API.
//!
//! Expose l'historique Tsink via :
//!   GET /api/v1/query        — requête instantanée
//!   GET /api/v1/query_range  — requête sur plage temporelle

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use tracing::warn;

use crate::state::AppState;
use crate::tsink_db::TsinkError;
use tsink::promql::{PromqlValue, Sample, Series};

// =============================================================================
// Paramètres de requête
// =============================================================================

#[derive(Deserialize)]
pub struct InstantQueryParams {
    pub query: String,
    /// Timestamp d'évaluation en millisecondes (défaut : maintenant)
    pub time: Option<i64>,
}

#[derive(Deserialize)]
pub struct RangeQueryParams {
    pub query: String,
    /// Début de plage en millisecondes
    pub start: i64,
    /// Fin de plage en millisecondes
    pub end: i64,
    /// Pas en millisecondes
    pub step: i64,
}

// =============================================================================
// Réponse d'erreur
// =============================================================================

#[derive(serde::Serialize)]
pub struct ApiError {
    status: String,
    error: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    error_type: Option<String>,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, Json(self)).into_response()
    }
}

fn tsink_error_to_api(e: TsinkError) -> ApiError {
    match e {
        TsinkError::PromQL(msg) => ApiError {
            status: "error".into(),
            error: msg,
            error_type: Some("bad_data".into()),
        },
        other => {
            warn!("Tsink internal error: {}", other);
            ApiError {
                status: "error".into(),
                error: "internal_error".into(),
                error_type: Some("internal".into()),
            }
        }
    }
}

// =============================================================================
// Conversion PromqlValue → JSON Prometheus
// =============================================================================

/// Timestamp Tsink (ms) → secondes en f64 (format Prometheus)
#[inline]
fn ts_to_secs(ts_ms: i64) -> f64 {
    ts_ms as f64 / 1000.0
}

fn sample_to_prometheus(s: &Sample) -> Value {
    let mut metric = serde_json::Map::new();
    metric.insert("__name__".into(), Value::String(s.metric.clone()));
    for label in &s.labels {
        metric.insert(label.name.clone(), Value::String(label.value.clone()));
    }
    json!({
        "metric": metric,
        "value": [ts_to_secs(s.timestamp), s.value.to_string()]
    })
}

fn series_to_prometheus(s: &Series) -> Value {
    let mut metric = serde_json::Map::new();
    metric.insert("__name__".into(), Value::String(s.metric.clone()));
    for label in &s.labels {
        metric.insert(label.name.clone(), Value::String(label.value.clone()));
    }
    let values: Vec<Value> = s.samples.iter().map(|(ts, v)| {
        json!([ts_to_secs(*ts), v.to_string()])
    }).collect();
    json!({
        "metric": metric,
        "values": values
    })
}

fn promql_value_to_json(value: PromqlValue) -> Value {
    match value {
        PromqlValue::Scalar(v, ts) => json!({
            "status": "success",
            "data": {
                "resultType": "scalar",
                "result": [ts_to_secs(ts), v.to_string()]
            }
        }),
        PromqlValue::InstantVector(samples) => {
            let result: Vec<Value> = samples.iter().map(sample_to_prometheus).collect();
            json!({
                "status": "success",
                "data": {
                    "resultType": "vector",
                    "result": result
                }
            })
        }
        PromqlValue::RangeVector(series) => {
            let result: Vec<Value> = series.iter().map(series_to_prometheus).collect();
            json!({
                "status": "success",
                "data": {
                    "resultType": "matrix",
                    "result": result
                }
            })
        }
        PromqlValue::String(s, ts) => json!({
            "status": "success",
            "data": {
                "resultType": "string",
                "result": [ts_to_secs(ts), s]
            }
        }),
    }
}

// =============================================================================
// Handlers
// =============================================================================

/// `GET /api/v1/query` — Requête PromQL instantanée.
///
/// Exemple : `/api/v1/query?query=bms_voltage{bms_id="0x01"}&time=1700000000000`
pub async fn query_instant(
    State(state): State<AppState>,
    Query(params): Query<InstantQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let tsink = state.tsink.as_ref().ok_or_else(|| ApiError {
        status: "error".into(),
        error: "Tsink storage is not enabled".into(),
        error_type: Some("unavailable".into()),
    })?;

    let time_ms = params
        .time
        .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

    tsink
        .query_instant(params.query, time_ms)
        .await
        .map(|v| Json(promql_value_to_json(v)))
        .map_err(tsink_error_to_api)
}

/// `GET /api/v1/query_range` — Requête PromQL sur plage temporelle.
///
/// Exemple : `/api/v1/query_range?query=bms_soc{bms_id="0x01"}&start=1700000000000&end=1700086400000&step=60000`
pub async fn query_range(
    State(state): State<AppState>,
    Query(params): Query<RangeQueryParams>,
) -> Result<Json<Value>, ApiError> {
    let tsink = state.tsink.as_ref().ok_or_else(|| ApiError {
        status: "error".into(),
        error: "Tsink storage is not enabled".into(),
        error_type: Some("unavailable".into()),
    })?;

    if params.start > params.end {
        return Err(ApiError {
            status: "error".into(),
            error: "start must be before end".into(),
            error_type: Some("bad_data".into()),
        });
    }
    if params.step <= 0 {
        return Err(ApiError {
            status: "error".into(),
            error: "step must be positive".into(),
            error_type: Some("bad_data".into()),
        });
    }

    tsink
        .query_range(params.query, params.start, params.end, params.step)
        .await
        .map(|v| Json(promql_value_to_json(v)))
        .map_err(tsink_error_to_api)
}

/// `GET /api/v1/labels` — Liste des métriques connues (pour autocomplete frontend).
pub async fn list_metrics(State(state): State<AppState>) -> Json<Value> {
    Json(json!({
        "status": "success",
        "data": if state.tsink.is_some() {
            vec![
                "bms_voltage", "bms_current", "bms_power", "bms_soc",
                "bms_capacity_ah", "bms_cell_delta_mv", "bms_temp_max", "bms_temp_min",
                "bms_charge_mos", "bms_discharge_mos", "bms_cell_voltage",
                "et112_voltage_v", "et112_current_a", "et112_power_w",
                "et112_energy_import_wh", "et112_energy_export_wh",
                "irradiance_wm2",
                "venus_shunt_voltage_v", "venus_shunt_current_a", "venus_shunt_power_w",
                "venus_shunt_soc_percent", "venus_shunt_ah_charged_today",
                "venus_inverter_power_w", "venus_inverter_voltage_v",
            ]
        } else {
            vec![]
        }
    }))
}
