use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::bus::AppBus;
use crate::config::LgThinqConfig;
use crate::types::{LiveEvent, WaterHeaterMode};

// ---------------------------------------------------------------------------
// API types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct LgStateResponse {
    result: Option<LgStateResult>,
}

#[derive(Debug, Deserialize)]
struct LgStateResult {
    data: Option<LgDeviceData>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LgDeviceData {
    operation: Option<LgOperation>,
    temperature: Option<LgTemperature>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LgOperation {
    #[serde(rename = "waterHeaterOperationMode")]
    mode: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LgTemperature {
    current_temp: Option<f64>,
    target_temp: Option<f64>,
}

// ---------------------------------------------------------------------------
// Public snapshot
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct LgSnapshot {
    pub mode: WaterHeaterMode,
    pub current_temp_c: Option<f64>,
    pub target_temp_c: Option<f64>,
}

// ---------------------------------------------------------------------------
// Client
// ---------------------------------------------------------------------------

pub struct LgThinqClient {
    http: reqwest::Client,
    cfg: LgThinqConfig,
}

impl LgThinqClient {
    pub fn new(cfg: LgThinqConfig) -> Self {
        Self {
            http: reqwest::Client::new(),
            cfg,
        }
    }

    fn state_url(&self) -> String {
        format!("{}/devices/{}/state", self.cfg.base_url, self.cfg.device_id)
    }

    fn control_url(&self) -> String {
        format!("{}/devices/{}/control", self.cfg.base_url, self.cfg.device_id)
    }

    fn auth_headers(&self) -> reqwest::header::HeaderMap {
        use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.cfg.bearer_token)).unwrap(),
        );
        if !self.cfg.api_key.is_empty() {
            headers.insert(
                "x-api-key",
                HeaderValue::from_str(&self.cfg.api_key).unwrap(),
            );
        }
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    pub async fn get_state(&self) -> Result<LgSnapshot> {
        let resp = self.http
            .get(self.state_url())
            .headers(self.auth_headers())
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .context("LG ThinQ GET state")?
            .error_for_status()
            .context("LG ThinQ GET state HTTP error")?;

        let body: LgStateResponse = resp.json().await.context("LG ThinQ parse")?;
        let data = body.result
            .and_then(|r| r.data)
            .unwrap_or(LgDeviceData { operation: None, temperature: None });

        let mode_str = data.operation.and_then(|o| o.mode).unwrap_or_default();
        let mode = WaterHeaterMode::from_lg_str(&mode_str);
        let current_temp_c = data.temperature.as_ref().and_then(|t| t.current_temp);
        let target_temp_c  = data.temperature.as_ref().and_then(|t| t.target_temp);

        Ok(LgSnapshot { mode, current_temp_c, target_temp_c })
    }

    pub async fn set_mode(&self, mode: WaterHeaterMode) -> Result<()> {
        let payload = json!({
            "operation": {
                "waterHeaterOperationMode": mode.to_lg_str()
            }
        });
        self.http
            .post(self.control_url())
            .headers(self.auth_headers())
            .json(&payload)
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .context("LG ThinQ POST control (mode)")?
            .error_for_status()
            .context("LG ThinQ POST control HTTP error")?;
        info!("LG ThinQ: mode set to {:?}", mode);
        Ok(())
    }

    pub async fn set_target_temp(&self, temp_c: f64) -> Result<()> {
        let payload = json!({
            "temperature": {
                "targetTemperature": temp_c
            }
        });
        self.http
            .post(self.control_url())
            .headers(self.auth_headers())
            .json(&payload)
            .timeout(Duration::from_secs(15))
            .send()
            .await
            .context("LG ThinQ POST control (temp)")?
            .error_for_status()
            .context("LG ThinQ POST control temp HTTP error")?;
        info!("LG ThinQ: target temperature set to {temp_c}°C");
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Polling task (read state every N minutes)
// ---------------------------------------------------------------------------

pub async fn spawn_poller(
    cfg: LgThinqConfig,
    bus: AppBus,
    state: std::sync::Arc<tokio::sync::RwLock<crate::types::EnergyState>>,
) -> Option<LgThinqClient> {
    if !cfg.enabled {
        info!("LG ThinQ integration disabled");
        return None;
    }

    if cfg.device_id.is_empty() || cfg.bearer_token.is_empty() {
        warn!("LG ThinQ enabled but credentials missing (LG_DEVICE_ID / LG_BEARER_TOKEN)");
        return None;
    }

    info!("LG ThinQ poller started (device={}, interval={}s)",
        cfg.device_id, cfg.poll_interval_secs);

    let client = LgThinqClient::new(cfg.clone());

    // Spawn background polling
    let cfg2 = cfg.clone();
    let bus2 = bus.clone();
    let state2 = state.clone();
    tokio::spawn(async move {
        let poller = LgThinqClient::new(cfg2);
        let mut ticker = interval(Duration::from_secs(poller.cfg.poll_interval_secs));
        loop {
            ticker.tick().await;
            match poller.get_state().await {
                Ok(snap) => {
                    debug!("LG ThinQ: {:?}", snap);
                    {
                        let mut s = state2.write().await;
                        s.water_heater_mode     = snap.mode;
                        s.water_heater_temp_c   = snap.current_temp_c;
                        s.water_heater_target_c = snap.target_temp_c;
                    }
                    bus2.emit_live(LiveEvent::new("water_heater", &snap));
                }
                Err(e) => error!("LG ThinQ poll error: {e}"),
            }
        }
    });

    Some(client)
}
