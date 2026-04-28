//! Endpoint de santé `/health` pour supervision externe.

use axum::{extract::State, Json};
use serde::Serialize;
use serde_json::json;
use std::sync::atomic::Ordering;

use crate::state::AppState;

#[derive(Serialize)]
#[allow(dead_code)]
pub struct HealthResponse {
    pub status: String,
    pub tsink: TsinkStatus,
    pub bms_polling: String,
    pub ws_clients: usize,
}

#[derive(Serialize)]
pub struct TsinkStatus {
    pub enabled: bool,
    pub memory_used_mb: Option<f64>,
    pub memory_budget_mb: Option<f64>,
}

/// `GET /health` — Statut global du serveur.
pub async fn health_check(State(state): State<AppState>) -> Json<serde_json::Value> {
    let polling = state.polling_active.load(Ordering::Relaxed);
    let ws_clients = state.ws_tx.receiver_count();

    let tsink_status = match &state.tsink {
        None => TsinkStatus {
            enabled:           false,
            memory_used_mb:    None,
            memory_budget_mb:  None,
        },
        Some(tsink) => TsinkStatus {
            enabled:          true,
            memory_used_mb:   Some(tsink.memory_used_bytes() as f64 / 1_048_576.0),
            memory_budget_mb: Some(tsink.memory_budget_bytes() as f64 / 1_048_576.0),
        },
    };

    let overall = if tsink_status.enabled && polling {
        "healthy"
    } else if !polling {
        "degraded"
    } else {
        "ok"
    };

    Json(json!({
        "status": overall,
        "tsink": tsink_status,
        "bms_polling": if polling { "active" } else { "inactive" },
        "ws_clients": ws_clients,
    }))
}
