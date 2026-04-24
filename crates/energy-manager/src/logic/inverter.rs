/// Aggregates VEBus topics into santuario/inverter/venus (retained).
use serde_json::json;
use tokio::sync::RwLock;
use std::sync::Arc;

use crate::bus::AppBus;
use crate::config::VictronConfig;
use crate::mqtt::topics::publish;
use crate::types::{EnergyState, LiveEvent, MqttIncoming, MqttOutgoing};

pub async fn spawn(
    cfg: Arc<VictronConfig>,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    tokio::spawn(run(cfg, bus, state));
}

async fn run(
    cfg: Arc<VictronConfig>,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let pid = &cfg.portal_id;
    let vb  = cfg.vebus_instance;

    let pfx_vebus  = format!("N/{pid}/vebus/{vb}/");
    let pfx_system = format!("N/{pid}/system/0/");

    let mut rx = bus.subscribe_mqtt();
    loop {
        let msg = match rx.recv().await {
            Ok(m) => m,
            Err(_) => continue,
        };

        if handle(&msg, &pfx_vebus, &pfx_system, &state).await {
            publish_state(&bus, &state).await;
        }
    }
}

async fn handle(
    msg: &MqttIncoming,
    _pfx_vebus: &str,
    _pfx_system: &str,
    state: &Arc<RwLock<EnergyState>>,
) -> bool {
    let t = &msg.topic;

    if t.contains("/vebus/") && t.ends_with("/Dc/0/Voltage") {
        if let Some(v) = msg.victron_value::<f64>() {
            state.write().await.dc_voltage_v = Some(v);
            return true;
        }
    } else if t.contains("/vebus/") && t.ends_with("/Dc/0/Current") {
        if let Some(v) = msg.victron_value::<f64>() {
            state.write().await.dc_current_a = Some(v);
            return true;
        }
    } else if t.contains("/vebus/") && t.ends_with("/Dc/0/Power") {
        if let Some(v) = msg.victron_value::<f64>() {
            state.write().await.dc_power_w = Some(v);
            return true;
        }
    } else if t.contains("/vebus/") && t.ends_with("/Ac/Out/L1/V") {
        if let Some(v) = msg.victron_value::<f64>() {
            state.write().await.ac_out_voltage_v = Some(v);
            return true;
        }
    } else if t.contains("/vebus/") && t.ends_with("/Ac/Out/L1/I") {
        if let Some(v) = msg.victron_value::<f64>() {
            state.write().await.ac_out_current_a = Some(v);
            return true;
        }
    } else if t.contains("/vebus/") && t.ends_with("/State") {
        if let Some(v) = msg.victron_value::<i64>() {
            state.write().await.vebus_state = Some(v);
            return true;
        }
    } else if t.contains("/vebus/") && t.ends_with("/Ac/State/IgnoreAcIn1") {
        if let Some(v) = msg.victron_value::<i64>() {
            state.write().await.ac_ignore = Some(v);
            return true;
        }
    }
    false
}

async fn publish_state(bus: &AppBus, state: &Arc<RwLock<EnergyState>>) {
    let s = state.read().await;

    let ac_power = match (s.ac_out_voltage_v, s.ac_out_current_a) {
        (Some(v), Some(i)) => Some(v * i),
        _ => None,
    };

    let payload = json!({
        "Voltage":     s.dc_voltage_v,
        "Current":     s.dc_current_a,
        "Power":       s.dc_power_w,
        "AcVoltage":   s.ac_out_voltage_v,
        "AcCurrent":   s.ac_out_current_a,
        "AcPower":     ac_power,
        "AcFrequency": s.ac_frequency_hz,
        "State":       "on",
        "Mode":        "inverter",
        "IgnoreAcIn":  s.ac_ignore,
        "VebusState":  s.vebus_state,
    });

    drop(s);

    let msg = MqttOutgoing::retained(publish::INVERTER_VENUS, &payload);
    bus.publish(msg).await;
    bus.emit_live(LiveEvent::new("inverter", &payload));
}
