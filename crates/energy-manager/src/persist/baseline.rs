/// Restores solar baselines and production counters at startup.
/// Primary source: MQTT retained topics (pvinv_baseline, yield_yesterday).
/// InfluxDB writes (during runtime and midnight reset) ensure persistence across restarts.
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

use crate::config::{InfluxConfig, SolarConfig};
use crate::types::EnergyState;

/// Called at startup — logs that MQTT retained will provide baseline.
pub async fn restore(
    influx_cfg: &InfluxConfig,
    _solar_cfg: &SolarConfig,
    _state: Arc<RwLock<EnergyState>>,
) {
    if influx_cfg.enabled {
        info!("Baseline restore: MQTT retained topics will provide pvinv_baseline + yield_yesterday");
    } else {
        info!("Baseline restore: InfluxDB disabled — MQTT retained only");
    }
    // Actual values arrive via MQTT retained in the first seconds after connect.
    // See on_retained_baseline() and on_retained_yield_yesterday() below.
}

/// Called when MQTT retained baseline arrives (santuario/persist/pvinv_baseline)
pub async fn on_retained_baseline(payload: &str, state: &Arc<RwLock<EnergyState>>) {
    if payload.is_empty() {
        return; // cleared at midnight
    }
    if let Ok(v) = payload.trim().parse::<f64>() {
        let mut s = state.write().await;
        if s.pvinv_baseline_kwh.is_none() {
            s.pvinv_baseline_kwh = Some(v);
            info!("Baseline restored from MQTT retained: pvinv_baseline = {v:.3} kWh");
        }
    }
}

/// Called when MQTT retained yield_yesterday arrives
pub async fn on_retained_yield_yesterday(payload: &str, state: &Arc<RwLock<EnergyState>>) {
    if payload.is_empty() {
        return;
    }
    if let Ok(v) = payload.trim().parse::<f64>() {
        let mut s = state.write().await;
        s.yield_yesterday_kwh = v;
        info!("Yield yesterday restored from MQTT retained: {v:.3} kWh");
    }
}
