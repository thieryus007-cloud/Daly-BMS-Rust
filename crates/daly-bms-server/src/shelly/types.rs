//! Types de données pour les compteurs d'énergie Shelly EM (2 canaux).

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Données d'un canal Shelly EM.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShellyChannelData {
    pub power_w:            f32,
    pub reactive_power_var: f32,
    pub voltage_v:          f32,
    pub power_factor:       f32,
    pub energy_wh:          f64,
    pub returned_wh:        f64,
}

/// Snapshot complet d'un Shelly EM (2 canaux).
#[derive(Debug, Clone, Serialize)]
pub struct ShellyEmSnapshot {
    pub id:           u8,
    pub name:         String,
    pub shelly_id:    String,
    pub timestamp:    DateTime<Local>,
    pub channel_0:    ShellyChannelData,
    pub channel_1:    ShellyChannelData,
    pub rssi:         Option<i32>,
    pub total_power_w: f32,
}
