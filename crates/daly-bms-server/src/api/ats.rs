//! API REST pour l'ATS CHINT NXZB/NXZBN.
//!
//! ## Endpoints
//!
//! | Méthode | URL                          | Description                    |
//! |---------|------------------------------|-------------------------------|
//! | GET     | /api/v1/ats/status           | Dernier snapshot ATS           |
//! | POST    | /api/v1/ats/remote_on        | Activer télécommande           |
//! | POST    | /api/v1/ats/remote_off       | Désactiver télécommande        |
//! | POST    | /api/v1/ats/force_source1    | Forcer Onduleur (source 1)     |
//! | POST    | /api/v1/ats/force_source2    | Forcer Réseau (source 2)       |
//! | POST    | /api/v1/ats/force_double     | Forcer double déclenché        |

use crate::ats::{execute_ats_command, AtsCommand};
use crate::state::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;

// =============================================================================
// Lecture
// =============================================================================

/// GET /api/v1/ats/status
///
/// Retourne le dernier snapshot ATS au format JSON.
/// 404 si aucun snapshot disponible (ATS non configuré ou pas encore de données).
pub async fn get_ats_status(State(state): State<AppState>) -> impl IntoResponse {
    match state.ats_latest().await {
        Some(snap) => Json(json!({
            "success": true,
            "data": snap,
        }))
        .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "success": false,
                "error": "Aucune donnée ATS disponible — ATS non configuré ou en attente de données"
            })),
        )
            .into_response(),
    }
}

// =============================================================================
// Commandes d'écriture
// =============================================================================

/// Exécute une commande ATS via le bus RS485 dédié.
async fn run_command(state: &AppState, cfg_addr: u8, cmd: AtsCommand) -> impl IntoResponse {
    let bus_guard = state.ats_bus.read().await;
    let bus = match bus_guard.as_ref() {
        Some(b) => b.clone(),
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(json!({
                    "success": false,
                    "error": "Bus ATS non disponible — ATS non configuré ou port série non ouvert"
                })),
            )
                .into_response();
        }
    };
    drop(bus_guard);

    match execute_ats_command(&bus, cfg_addr, cmd).await {
        Ok(()) => Json(json!({
            "success": true,
            "message": cmd.label(),
            "command": format!("{:?}", cmd),
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "success": false,
                "error": e.to_string(),
                "command": format!("{:?}", cmd),
            })),
        )
            .into_response(),
    }
}

/// POST /api/v1/ats/remote_on — Activer la télécommande
pub async fn ats_remote_on(State(state): State<AppState>) -> impl IntoResponse {
    let addr = ats_addr(&state);
    run_command(&state, addr, AtsCommand::RemoteOn).await
}

/// POST /api/v1/ats/remote_off — Désactiver la télécommande
pub async fn ats_remote_off(State(state): State<AppState>) -> impl IntoResponse {
    let addr = ats_addr(&state);
    run_command(&state, addr, AtsCommand::RemoteOff).await
}

/// POST /api/v1/ats/force_source1 — Forcer Onduleur (source 1)
pub async fn ats_force_source1(State(state): State<AppState>) -> impl IntoResponse {
    let addr = ats_addr(&state);
    run_command(&state, addr, AtsCommand::ForceSource1).await
}

/// POST /api/v1/ats/force_source2 — Forcer Réseau (source 2)
pub async fn ats_force_source2(State(state): State<AppState>) -> impl IntoResponse {
    let addr = ats_addr(&state);
    run_command(&state, addr, AtsCommand::ForceSource2).await
}

/// POST /api/v1/ats/force_double — Forcer double déclenché (position centrale)
pub async fn ats_force_double(State(state): State<AppState>) -> impl IntoResponse {
    let addr = ats_addr(&state);
    run_command(&state, addr, AtsCommand::ForceDouble).await
}

// =============================================================================
// Helper
// =============================================================================

fn ats_addr(state: &AppState) -> u8 {
    state
        .config
        .ats
        .as_ref()
        .map(|c| c.address)
        .unwrap_or(6)
}
