use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use tracing::debug;

use crate::bus::AppBus;
use crate::config::{SolarConfig, VictronConfig};
use crate::types::{EnergyState, InfluxPoint, LiveEvent};

pub async fn spawn(
    vic: Arc<VictronConfig>,
    cfg: SolarConfig,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let bus2  = bus.clone();
    let state2 = state.clone();
    let vic2   = vic.clone();

    // MQTT subscriber task — updates state
    tokio::spawn(mqtt_task(vic2, bus2, state2));

    // Periodic writer task — InfluxDB + API POST
    tokio::spawn(writer_task(cfg, bus, state));
}

async fn mqtt_task(
    vic: Arc<VictronConfig>,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let pid = &vic.portal_id;
    let m1  = vic.mppt1_instance;
    let m2  = vic.mppt2_instance;
    let pv  = vic.pvinverter_instance;

    let t_m1_power   = format!("N/{pid}/solarcharger/{m1}/Yield/Power");
    let t_m2_power   = format!("N/{pid}/solarcharger/{m2}/Yield/Power");
    let t_pv_power   = format!("N/{pid}/pvinverter/{pv}/Ac/L1/Power");
    let t_pv_energy  = format!("N/{pid}/pvinverter/{pv}/Ac/Energy/Forward");
    let t_m1_yield   = format!("N/{pid}/solarcharger/{m1}/History/Daily/0/Yield");
    let t_m2_yield   = format!("N/{pid}/solarcharger/{m2}/History/Daily/0/Yield");
    let t_m1_state   = format!("N/{pid}/solarcharger/{m1}/State");
    let t_m2_state   = format!("N/{pid}/solarcharger/{m2}/State");
    let t_m1_pv_v    = format!("N/{pid}/solarcharger/{m1}/Pv/V");
    let t_m2_pv_v    = format!("N/{pid}/solarcharger/{m2}/Pv/V");
    let t_m1_dc_i    = format!("N/{pid}/solarcharger/{m1}/Dc/0/Current");
    let t_m2_dc_i    = format!("N/{pid}/solarcharger/{m2}/Dc/0/Current");
    let t_consump    = format!("N/{pid}/system/0/Ac/ConsumptionOnOutput/L1/Power");

    let mut rx = bus.subscribe_mqtt();
    loop {
        let msg = match rx.recv().await {
            Ok(m) => m,
            Err(_) => continue,
        };
        let t = &msg.topic;
        let mut s = state.write().await;

        if *t == t_m1_power {
            s.mppt_273.power_w = msg.victron_value::<f64>();
            s.mppt_power_273_w = s.mppt_273.power_w;
        } else if *t == t_m2_power {
            s.mppt_289.power_w = msg.victron_value::<f64>();
            s.mppt_power_289_w = s.mppt_289.power_w;
        } else if *t == t_pv_power {
            s.pvinverter_power_w = msg.victron_value::<f64>();
        } else if *t == t_pv_energy {
            // Store raw energy for baseline calculation (in meteo module)
            if let Some(kwh) = msg.victron_value::<f64>() {
                if s.pvinv_baseline_kwh.is_none() {
                    // First message of the day — set baseline
                    s.pvinv_baseline_kwh = Some(kwh);
                }
                let baseline = s.pvinv_baseline_kwh.unwrap_or(kwh);
                s.pvinv_yield_today_kwh = (kwh - baseline).max(0.0);
            }
        } else if *t == t_m1_yield {
            s.mppt_273.yield_today_kwh = msg.victron_value::<f64>();
        } else if *t == t_m2_yield {
            s.mppt_289.yield_today_kwh = msg.victron_value::<f64>();
        } else if *t == t_m1_state {
            s.mppt_273.state = msg.victron_value::<i64>();
        } else if *t == t_m2_state {
            s.mppt_289.state = msg.victron_value::<i64>();
        } else if *t == t_m1_pv_v {
            s.mppt_273.pv_voltage_v = msg.victron_value::<f64>();
        } else if *t == t_m2_pv_v {
            s.mppt_289.pv_voltage_v = msg.victron_value::<f64>();
        } else if *t == t_m1_dc_i {
            s.mppt_273.dc_current_a = msg.victron_value::<f64>();
        } else if *t == t_m2_dc_i {
            s.mppt_289.dc_current_a = msg.victron_value::<f64>();
        } else if *t == t_consump {
            s.house_power_w = msg.victron_value::<f64>();
        } else {
            continue;
        }

        // Recalculate totals
        let mppt_total = s.mppt_273.power_w.unwrap_or(0.0)
            + s.mppt_289.power_w.unwrap_or(0.0);
        let pvinv_total = s.pvinverter_power_w.unwrap_or(0.0);
        s.solar_total_w = mppt_total + pvinv_total;

        // Update MPPT daily totals
        s.mppt_yield_today_kwh = s.mppt_273.yield_today_kwh.unwrap_or(0.0)
            + s.mppt_289.yield_today_kwh.unwrap_or(0.0);

        s.total_yield_today_kwh = s.mppt_yield_today_kwh + s.pvinv_yield_today_kwh;
    }
}

async fn writer_task(
    cfg: SolarConfig,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let http_client = reqwest::Client::new();
    let api_url = format!("{}/api/v1/solar/mppt-yield", cfg.bms_server_url);
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;

        let (solar_total, mppt_power, house_power, host) = {
            let s = state.read().await;
            (
                s.solar_total_w,
                s.mppt_273.power_w.unwrap_or(0.0) + s.mppt_289.power_w.unwrap_or(0.0),
                s.house_power_w.unwrap_or(0.0),
                cfg.host_tag.clone(),
            )
        };

        // Write to InfluxDB
        let pt = InfluxPoint::new(&cfg.power_measurement)
            .tag("day",  chrono::Local::now().format("%Y-%m-%d").to_string())
            .tag("host", &host)
            .field_f("solar_total_w", solar_total)
            .field_f("mppt_power_w",  mppt_power)
            .field_f("pvinv_power_w", {
                let s = state.read().await;
                s.pvinverter_power_w.unwrap_or(0.0)
            })
            .field_f("house_power_w", house_power);
        bus.write_influx(pt).await;

        // POST to daly-bms-server
        let body = json!({
            "solar_total_w": solar_total,
            "mppt_power_w":  mppt_power,
            "house_power_w": house_power,
        });
        if let Err(e) = http_client
            .post(&api_url)
            .json(&body)
            .timeout(Duration::from_secs(5))
            .send()
            .await
        {
            debug!("Solar API POST error: {e}");
        }

        bus.emit_live(LiveEvent::new("solar", json!({
            "solar_total_w": solar_total,
            "mppt_power_w":  mppt_power,
            "house_power_w": house_power,
        })));
    }
}
