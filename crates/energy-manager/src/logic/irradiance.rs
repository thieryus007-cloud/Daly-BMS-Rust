/// Receives santuario/irradiance/raw → validates → stores in state.
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::bus::AppBus;
use crate::types::EnergyState;

const TOPIC: &str = "santuario/irradiance/raw";
const MAX_WM2: f64 = 2000.0;

pub async fn spawn(bus: AppBus, state: Arc<RwLock<EnergyState>>) {
    tokio::spawn(run(bus, state));
}

async fn run(bus: AppBus, state: Arc<RwLock<EnergyState>>) {
    let mut rx = bus.subscribe_mqtt();
    loop {
        let msg = match rx.recv().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        if msg.topic != TOPIC {
            continue;
        }
        let raw = msg.payload_str().trim().parse::<f64>().unwrap_or(-1.0);
        if raw < 0.0 || raw > MAX_WM2 {
            debug!("Irradiance out of range: {raw}");
            continue;
        }
        debug!("Irradiance: {raw} W/m²");
        state.write().await.irradiance_wm2 = Some(raw);
        bus.emit_live(crate::types::LiveEvent::new(
            "irradiance",
            serde_json::json!({ "irradiance_wm2": raw }),
        ));
    }
}
