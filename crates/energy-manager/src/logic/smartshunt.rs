/// Aggregates SmartShunt (battery system) topics → santuario/system/venus (retained).
/// Computes daily Ah charged/discharged by integrating current over time.
use chrono::{Datelike, Utc};
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::bus::AppBus;
use crate::mqtt::topics::publish;
use crate::types::{EnergyState, InfluxPoint, LiveEvent, MqttIncoming, MqttOutgoing};

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
                // --- Ah integration ---
                let now = Utc::now();
                let day_key = now.date_naive().num_days_from_ce();

                // Midnight reset
                if s.ah_last_day != day_key {
                    s.ah_charged_today   = 0.0;
                    s.ah_discharged_today = 0.0;
                    s.ah_last_day        = day_key;
                }

                if let Some(prev_ts) = s.ah_last_ts {
                    let delta_ms = (now - prev_ts).num_milliseconds();
                    // Only integrate if interval is positive and < 10 minutes
                    if delta_ms > 0 && delta_ms < 600_000 {
                        let delta_h = delta_ms as f64 / 3_600_000.0;
                        if v > 0.0 {
                            s.ah_charged_today   += v * delta_h;
                        } else if v < 0.0 {
                            s.ah_discharged_today += (-v) * delta_h;
                        }
                    }
                }
                s.ah_last_ts = Some(now);

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
    let soc        = s.soc_pct;
    let voltage    = s.battery_voltage_v;
    let current    = s.battery_current_a;
    let power      = s.battery_power_w;
    let batt_state = s.battery_state;
    let ttg        = s.time_to_go_sec;
    let ah_charged    = s.ah_charged_today;
    let ah_discharged = s.ah_discharged_today;

    let payload = json!({
        "Soc":               soc,
        "Voltage":           voltage,
        "Current":           current,
        "Power":             power,
        "State":             batt_state,
        "TimeToGo":          ttg,
        "AhChargedToday":    ah_charged,
        "AhDischargedToday": ah_discharged,
    });
    drop(s);

    bus.publish(MqttOutgoing::retained(publish::SYSTEM_VENUS, &payload)).await;
    bus.emit_live(LiveEvent::new("battery", &payload));

    let pt = InfluxPoint::new("battery_status")
        .tag("host", "pi5")
        .field_f("soc_pct",            soc.unwrap_or(0.0))
        .field_f("voltage_v",          voltage.unwrap_or(0.0))
        .field_f("current_a",          current.unwrap_or(0.0))
        .field_f("power_w",            power.unwrap_or(0.0))
        .field_i("state",              batt_state.unwrap_or(0))
        .field_i("time_to_go_sec",     ttg.unwrap_or(0))
        .field_f("ah_charged_today",   ah_charged)
        .field_f("ah_discharged_today", ah_discharged);
    bus.write_influx(pt).await;
}
