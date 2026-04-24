/// Aggregates SmartShunt (battery system) topics → santuario/system/venus (retained).
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bus::AppBus;
use crate::mqtt::topics::publish;
use crate::types::{EnergyState, LiveEvent, MqttIncoming, MqttOutgoing};

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
        if handle(&msg, &state).await {
            publish_state(&bus, &state).await;
        }
    }
}

async fn handle(msg: &MqttIncoming, state: &Arc<RwLock<EnergyState>>) -> bool {
    let t = &msg.topic;
    if !t.contains("/system/0/Dc/Battery/") && !t.contains("/vebus/") {
        return false;
    }

    let mut updated = false;
    {
        let mut s = state.write().await;
        if t.ends_with("Battery/Soc") {
            if let Some(v) = msg.victron_value::<f64>() {
                s.soc_pct = Some(v);
                updated = true;
            }
        } else if t.ends_with("Battery/Current") {
            if let Some(v) = msg.victron_value::<f64>() {
                s.battery_current_a = Some(v);
                updated = true;
            }
        } else if t.ends_with("Battery/State") {
            if let Some(v) = msg.victron_value::<i64>() {
                s.battery_state = Some(v);
                updated = true;
            }
        } else if t.ends_with("Battery/TimeToGo") {
            if let Some(v) = msg.victron_value::<i64>() {
                s.time_to_go_sec = Some(v);
                updated = true;
            }
        } else if t.ends_with("/Dc/0/Voltage") && t.contains("/vebus/") {
            if let Some(v) = msg.victron_value::<f64>() {
                s.battery_voltage_v = Some(v);
                updated = true;
            }
        } else if t.ends_with("/Dc/0/Power") && t.contains("/vebus/") {
            if let Some(v) = msg.victron_value::<f64>() {
                s.battery_power_w = Some(v);
                updated = true;
            }
        }
    }
    updated
}

async fn publish_state(bus: &AppBus, state: &Arc<RwLock<EnergyState>>) {
    let s = state.read().await;
    let payload = json!({
        "Soc":      s.soc_pct,
        "Voltage":  s.battery_voltage_v,
        "Current":  s.battery_current_a,
        "Power":    s.battery_power_w,
        "State":    s.battery_state,
        "TimeToGo": s.time_to_go_sec,
    });
    drop(s);
    bus.publish(MqttOutgoing::retained(publish::SYSTEM_VENUS, &payload)).await;
    bus.emit_live(LiveEvent::new("battery", &payload));
}
