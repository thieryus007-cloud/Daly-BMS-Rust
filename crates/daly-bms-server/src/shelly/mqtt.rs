//! Subscriber MQTT Shelly Pro 2PM — réception des topics natifs Shelly Gen3.
//!
//! Topics surveillés :
//!   {shelly_id}/status/switch:0   → état + mesures canal 0
//!   {shelly_id}/status/switch:1   → état + mesures canal 1
//!
//! Format JSON (par canal) :
//! ```json
//! {
//!   "id": 0, "source": "timer", "output": true,
//!   "apower": 250.0, "voltage": 230.0, "current": 1.09, "pf": 0.99,
//!   "aenergy": { "total": 1234.567 },
//!   "ret_aenergy": { "total": 0.0 }
//! }
//! ```

use crate::config::{MqttConfig, ShellyDeviceConfig};
use super::types::{ShellyChannelData, ShellyEmSnapshot};
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::{info, warn};
use chrono::Local;

/// Cache intermédiaire pour assembler les mesures multi-topics.
#[derive(Default)]
struct ShellyCache {
    ch0: ShellyChannelData,
    ch1: ShellyChannelData,
    rssi: Option<i32>,
}

/// Boucle principale d'abonnement MQTT Shelly Pro 2PM.
///
/// Reconnexion automatique après déconnexion (backoff 10 s).
/// Appelle `on_snapshot` pour chaque mise à jour reçue.
pub async fn run_shelly_mqtt_loop<F>(
    devices:         Vec<ShellyDeviceConfig>,
    mqtt_cfg:        MqttConfig,
    client_out:      Arc<Mutex<Option<AsyncClient>>>,
    mut on_snapshot: F,
)
where
    F: FnMut(ShellyEmSnapshot) + Send + 'static,
{
    if devices.is_empty() {
        return;
    }

    info!(count = devices.len(), host = %mqtt_cfg.host, "Démarrage Shelly Pro 2PM MQTT subscriber");

    let mut cache: HashMap<u8, ShellyCache> = HashMap::new();
    for dev in &devices {
        cache.insert(dev.id, ShellyCache::default());
    }

    loop {
        let mut opts = MqttOptions::new(
            format!("daly-bms-shelly-{}", uuid::Uuid::new_v4()),
            &mqtt_cfg.host,
            mqtt_cfg.port,
        );
        opts.set_keep_alive(Duration::from_secs(30));

        if let (Some(user), Some(pass)) = (&mqtt_cfg.username, &mqtt_cfg.password) {
            opts.set_credentials(user, pass);
        }

        let (client, mut eventloop) = AsyncClient::new(opts, 128);

        {
            let mut guard = client_out.lock().await;
            *guard = Some(client.clone());
        }

        // Abonnement aux topics status de chaque device configuré
        for dev in &devices {
            let t0 = format!("{}/status/switch:0", dev.shelly_id);
            let t1 = format!("{}/status/switch:1", dev.shelly_id);
            let ti = format!("{}/info", dev.shelly_id);
            let _ = client.subscribe(&t0, QoS::AtMostOnce).await;
            let _ = client.subscribe(&t1, QoS::AtMostOnce).await;
            let _ = client.subscribe(&ti, QoS::AtMostOnce).await;
        }

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Incoming::ConnAck(_))) => {
                    info!("Shelly MQTT connecté (broker {}:{})", mqtt_cfg.host, mqtt_cfg.port);
                }

                Ok(Event::Incoming(Incoming::Publish(msg))) => {
                    let topic = msg.topic.as_str();
                    let payload = match std::str::from_utf8(&msg.payload) {
                        Ok(s) => s,
                        Err(_) => continue,
                    };

                    // Trouver le device par shelly_id (préfixe du topic)
                    let dev_cfg = match devices.iter().find(|d| topic.starts_with(&d.shelly_id)) {
                        Some(d) => d,
                        None    => continue,
                    };
                    let id = dev_cfg.id;
                    let entry = cache.entry(id).or_default();

                    // {shelly_id}/status/switch:0 ou /status/switch:1
                    if topic.ends_with("/status/switch:0") || topic.ends_with("/status/switch:1") {
                        let channel: u8 = if topic.ends_with(":0") { 0 } else { 1 };

                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
                            let ch = if channel == 0 { &mut entry.ch0 } else { &mut entry.ch1 };
                            ch.output      = json["output"].as_bool().unwrap_or(false);
                            ch.power_w     = json["apower"].as_f64().unwrap_or(0.0) as f32;
                            ch.voltage_v   = json["voltage"].as_f64().unwrap_or(0.0) as f32;
                            ch.current_a   = json["current"].as_f64().unwrap_or(0.0) as f32;
                            ch.power_factor = json["pf"].as_f64().unwrap_or(0.0) as f32;
                            ch.energy_wh   = json["aenergy"]["total"].as_f64().unwrap_or(0.0);
                            ch.returned_wh = json["ret_aenergy"]["total"].as_f64().unwrap_or(0.0);
                        }

                        let total_power_w = entry.ch0.power_w + entry.ch1.power_w;
                        let snap = ShellyEmSnapshot {
                            id,
                            name:         dev_cfg.name.clone(),
                            shelly_id:    dev_cfg.shelly_id.clone(),
                            timestamp:    Local::now(),
                            channel_0:    entry.ch0.clone(),
                            channel_1:    entry.ch1.clone(),
                            rssi:         entry.rssi,
                            total_power_w,
                        };
                        on_snapshot(snap);

                    } else if topic.ends_with("/info") {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
                            entry.rssi = json["wifi"]["rssi"].as_i64()
                                .or_else(|| json["wifi_sta"]["rssi"].as_i64())
                                .map(|v| v as i32);
                        }
                    }
                }

                Ok(_) => {}

                Err(e) => {
                    warn!("Shelly MQTT erreur : {:?}", e);
                    break;
                }
            }
        }

        warn!("Shelly MQTT déconnecté — reconnexion dans 10s");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
