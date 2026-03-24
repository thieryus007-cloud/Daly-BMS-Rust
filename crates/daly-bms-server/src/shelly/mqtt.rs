//! Subscriber MQTT Shelly EM — réception des topics natifs Shelly.
//!
//! Topics surveillés (wildcards) :
//!   shellies/+/emeter/0/power           → puissance canal 0 (W)
//!   shellies/+/emeter/0/reactive_power  → puissance réactive canal 0 (VAr)
//!   shellies/+/emeter/0/voltage         → tension canal 0 (V)
//!   shellies/+/emeter/0/pf              → facteur de puissance canal 0
//!   shellies/+/emeter/0/energy          → énergie cumul canal 0 (Wh)
//!   shellies/+/emeter/0/returned_energy → retour réseau canal 0 (Wh)
//!   (idem pour /emeter/1)
//!   shellies/+/info                     → JSON avec wifi.rssi

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

/// Boucle principale d'abonnement MQTT Shelly EM.
///
/// Se reconnecte automatiquement après déconnexion (backoff 10 s).
/// Appelle `on_snapshot` pour chaque mise à jour reçue.
pub async fn run_shelly_mqtt_loop<F>(
    devices:     Vec<ShellyDeviceConfig>,
    mqtt_cfg:    MqttConfig,
    client_out:  Arc<Mutex<Option<AsyncClient>>>,
    mut on_snapshot: F,
)
where
    F: FnMut(ShellyEmSnapshot) + Send + 'static,
{
    if devices.is_empty() {
        return;
    }

    info!(count = devices.len(), host = %mqtt_cfg.host, "Démarrage Shelly EM MQTT subscriber");

    // Cache par id de device
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

        // Partager le client pour les commandes (non utilisé pour Shelly — read-only)
        {
            let mut guard = client_out.lock().await;
            *guard = Some(client.clone());
        }

        // Abonnement aux wildcards Shelly
        let _ = client.subscribe("shellies/+/emeter/0/power",           QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/0/reactive_power",  QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/0/voltage",         QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/0/pf",              QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/0/energy",          QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/0/returned_energy", QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/1/power",           QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/1/reactive_power",  QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/1/voltage",         QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/1/pf",              QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/1/energy",          QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/emeter/1/returned_energy", QoS::AtMostOnce).await;
        let _ = client.subscribe("shellies/+/info",                     QoS::AtMostOnce).await;

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

                    // shellies/{device_id}/emeter/{channel}/{metric}
                    // shellies/{device_id}/info
                    let parts: Vec<&str> = topic.split('/').collect();
                    if parts.len() < 3 { continue; }
                    let device_name = parts[1];

                    // Trouver la config du device par son shelly_id
                    let dev_cfg = match devices.iter().find(|d| d.shelly_id == device_name) {
                        Some(d) => d,
                        None    => continue,
                    };
                    let id = dev_cfg.id;

                    let entry = cache.entry(id).or_default();

                    if parts.len() == 5 && parts[2] == "emeter" {
                        // parts[3] = "0" ou "1", parts[4] = métrique
                        let channel: u8 = parts[3].parse().unwrap_or(0);
                        let metric  = parts[4];
                        let val_f32: f32 = payload.trim().parse().unwrap_or(0.0);
                        let val_f64: f64 = payload.trim().parse().unwrap_or(0.0);

                        let ch = if channel == 0 { &mut entry.ch0 } else { &mut entry.ch1 };

                        match metric {
                            "power"           => ch.power_w            = val_f32,
                            "reactive_power"  => ch.reactive_power_var = val_f32,
                            "voltage"         => ch.voltage_v          = val_f32,
                            "pf"              => ch.power_factor        = val_f32,
                            "energy"          => ch.energy_wh           = val_f64,
                            "returned_energy" => ch.returned_wh         = val_f64,
                            _ => {}
                        }

                        // Émettre un snapshot à chaque mise à jour
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

                    } else if parts.len() == 3 && parts[2] == "info" {
                        // Payload JSON avec wifi.rssi
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(payload) {
                            entry.rssi = json["wifi"]["rssi"].as_i64().map(|v| v as i32);
                        }
                    }
                }

                Ok(_) => {}

                Err(e) => {
                    warn!("Shelly MQTT erreur : {:?}", e);
                    break; // sortir pour reconnecter
                }
            }
        }

        warn!("Shelly MQTT déconnecté — reconnexion dans 10s");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}
