/// Aggregates solar production (MPPT + PV inverter ET112) and posts to daly-bms-server API.
/// Daily baseline decisions for the ET112 energy counter are pure Rust (`rules::baseline_decision`).
mod rules;

use chrono::Datelike;
use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, warn};

use crate::bus::AppBus;
use crate::config::{SolarConfig, VictronConfig};
use crate::mqtt::topics::publish;
use crate::types::{EnergyState, LiveEvent, MpptState, MqttOutgoing};

pub async fn spawn(
    vic: Arc<VictronConfig>,
    cfg: SolarConfig,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let bus2   = bus.clone();
    let state2 = state.clone();
    let vic2   = vic.clone();

    crate::supervise::spawn_critical(mqtt_task(vic2, bus2, state2));
    crate::supervise::spawn_critical(writer_task(cfg, bus, state));
}

/// MPPT field fed by a `N/{portal}/solarcharger/{instance}/…` topic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MpptField { Power, Yield, MaxPower, State, PvV, DcI }

/// Venus topic → (instance, field) for every followed MPPT.
/// Paths must match `mqtt::topics::all_subscriptions`.
fn mppt_topic_map(pid: &str, instances: &[u32]) -> HashMap<String, (u32, MpptField)> {
    const PATHS: [(&str, MpptField); 6] = [
        ("Yield/Power",              MpptField::Power),
        ("History/Daily/0/Yield",    MpptField::Yield),
        ("History/Daily/0/MaxPower", MpptField::MaxPower),
        ("State",                    MpptField::State),
        ("Pv/V",                     MpptField::PvV),
        ("Dc/0/Current",             MpptField::DcI),
    ];
    let mut map = HashMap::new();
    for &i in instances {
        for (path, field) in PATHS {
            map.insert(format!("N/{pid}/solarcharger/{i}/{path}"), (i, field));
        }
    }
    map
}

async fn mqtt_task(
    vic: Arc<VictronConfig>,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let pid = &vic.portal_id;
    let pv  = vic.pvinverter_instance;

    let mppt_topics = mppt_topic_map(pid, &vic.mppt_list());
    let t_dc_pv     = format!("N/{pid}/system/0/Dc/Pv/Power");
    let t_pv_power  = format!("N/{pid}/pvinverter/{pv}/Ac/Power");
    let t_pv_energy = format!("N/{pid}/pvinverter/{pv}/Ac/Energy/Forward");
    let t_consump   = format!("N/{pid}/system/0/Ac/ConsumptionOnOutput/L1/Power");

    let mut rx = bus.subscribe_mqtt();

    loop {
        let msg = match rx.recv().await {
            Ok(m) => m,
            Err(broadcast::error::RecvError::Lagged(n)) => {
                warn!("solar_power MQTT subscriber lagged, dropped {n} message(s)");
                continue;
            }
            Err(broadcast::error::RecvError::Closed) => break,
        };
        let t = &msg.topic;

        let mut publish_baseline: Option<(i32, f64)> = None;

        {
            let mut s = state.write().await;

            if let Some(&(inst, field)) = mppt_topics.get(t.as_str()) {
                let m = s.mppts.entry(inst)
                    .or_insert_with(|| MpptState { instance: inst, ..Default::default() });
                match field {
                    MpptField::Power    => m.power_w           = msg.victron_value::<f64>(),
                    MpptField::Yield    => m.yield_today_kwh   = msg.victron_value::<f64>(),
                    MpptField::MaxPower => m.max_power_today_w = msg.victron_value::<f64>(),
                    MpptField::State    => {
                        m.state         = msg.victron_value::<i64>();
                        m.state_last_ts = Some(chrono::Utc::now());
                    }
                    MpptField::PvV      => m.pv_voltage_v      = msg.victron_value::<f64>(),
                    MpptField::DcI      => m.dc_current_a      = msg.victron_value::<f64>(),
                }
            } else if *t == t_dc_pv {
                // Source de vérité pour la somme MPPT : N/.../system/0/Dc/Pv/Power
                s.dc_pv_power_w = msg.victron_value::<f64>();
            } else if *t == t_pv_power {
                // N/.../pvinverter/32/Ac/Power (ET112 micro-onduleurs, total AC)
                s.pvinverter_power_w = msg.victron_value::<f64>();
            } else if *t == t_pv_energy {
                if let Some(kwh) = msg.victron_value::<f64>() {
                    let today = chrono::Utc::now().date_naive().num_days_from_ce();

                    // Pure-Rust decision: whether to reset and/or capture the baseline.
                    let decision = rules::baseline_decision(
                        s.pvinv_baseline_day != today,
                        s.pvinv_baseline_kwh.is_none(),
                    );

                    if decision.reset {
                        s.pvinv_baseline_kwh = None;
                        s.pvinv_baseline_day = today;
                    }
                    if decision.capture {
                        s.pvinv_baseline_kwh = Some(kwh);
                        publish_baseline      = Some((today, kwh));
                    }

                    let baseline = s.pvinv_baseline_kwh.unwrap_or(kwh);
                    s.pvinv_yield_today_kwh = (kwh - baseline).max(0.0);
                }
            } else if *t == t_consump {
                s.house_power_w = msg.victron_value::<f64>();
            } else {
                continue;
            }

            // MPPT sum = system aggregate (N/.../system/0/Dc/Pv/Power)
            let mppt_total  = s.dc_pv_power_w.unwrap_or(0.0);
            let pvinv_total = s.pvinverter_power_w.unwrap_or(0.0);
            s.solar_total_w = mppt_total + pvinv_total;

            s.mppt_yield_today_kwh  = s.mppts.values().filter_map(|m| m.yield_today_kwh).sum();
            s.total_yield_today_kwh = s.mppt_yield_today_kwh + s.pvinv_yield_today_kwh;
        }

        if let Some((day, kwh)) = publish_baseline {
            bus.publish(MqttOutgoing::raw(
                publish::PVINV_BASELINE,
                format!("{day}:{kwh:.3}"),
                true,
            )).await;
            debug!("pvinv_baseline published as retained: day={day} kwh={kwh:.3}");
        }
    }   // close loop
}

async fn writer_task(
    // `bms_server_url` n'est plus utilisé : la télémétrie part en MQTT
    // (topic `santuario/em/solar`) au lieu d'un POST HTTP 1 Hz. Le champ
    // de config est conservé pour ne pas casser les Config.toml existants
    // (détection de clés inconnues, audit 2026-06 §12).
    _cfg: SolarConfig,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;

        let (solar_total, house_power, dc_pv_w, mppts_w, pvinv_w, total_yield) = {
            let s = state.read().await;
            let mppts_w: BTreeMap<u32, f64> = s.mppts.iter()
                .map(|(i, m)| (*i, m.power_w.unwrap_or(0.0)))
                .collect();
            (
                s.solar_total_w,
                s.house_power_w.unwrap_or(0.0),
                s.dc_pv_power_w.unwrap_or(0.0),
                mppts_w,
                s.pvinverter_power_w.unwrap_or(0.0),
                s.total_yield_today_kwh,
            )
        };

        let body = json!({
            "solar_total_w":   solar_total,
            "dc_pv_power_w":   dc_pv_w,   // N/.../system/0/Dc/Pv/Power
            "pvinv_power_w":   pvinv_w,   // N/.../pvinverter/32/Ac/Power
            "mppt_power_w":    dc_pv_w,   // alias = dc_pv pour compatibilité API
            "total_yield_kwh": total_yield,
            "house_power_w":   house_power,
        });
        // Publication MQTT (QoS de transient, non retenu) — daly-bms-server
        // souscrit à `santuario/em/solar` et alimente ses RwLock + metrics-store
        // exactement comme l'ancien handler POST /api/v1/solar/mppt-yield
        // (conservé en fallback).
        bus.publish(MqttOutgoing::transient(publish::EM_SOLAR, &body)).await;

        // Topics scalaires (dashboards externes) — agrégats solaires en nombre nu.
        bus.publish(MqttOutgoing::raw("santuario/em/solar/total_w",   format!("{solar_total:.0}"), true)).await;
        bus.publish(MqttOutgoing::raw("santuario/em/solar/pv_w",      format!("{dc_pv_w:.0}"),     true)).await;
        bus.publish(MqttOutgoing::raw("santuario/em/solar/pvinv_w",   format!("{pvinv_w:.0}"),     true)).await;
        bus.publish(MqttOutgoing::raw("santuario/em/solar/yield_kwh", format!("{total_yield:.2}"), true)).await;
        bus.publish(MqttOutgoing::raw("santuario/em/solar/house_w",   format!("{house_power:.0}"), true)).await;

        bus.emit_live(LiveEvent::new("solar", json!({
            "solar_total_w": solar_total,
            "dc_pv_power_w": dc_pv_w,
            "mppt_power_w":  dc_pv_w,   // alias rétrocompat dashboard
            "mppts_w":       mppts_w,   // W par instance MPPT : {"273": …, "288": …}
            "pvinv_power_w": pvinv_w,
            "pvinv_w":       pvinv_w,   // alias rétrocompat dashboard
            "house_power_w": house_power,
        })));
    }
}

#[cfg(test)]
mod mppt_topic_tests {
    use super::*;

    #[test]
    fn maps_every_path_of_every_instance() {
        let m = mppt_topic_map("pid", &[273, 290]);
        assert_eq!(m.len(), 12);
        assert_eq!(m.get("N/pid/solarcharger/290/History/Daily/0/Yield"), Some(&(290, MpptField::Yield)));
        assert_eq!(m.get("N/pid/solarcharger/273/State"), Some(&(273, MpptField::State)));
        assert_eq!(m.get("N/pid/solarcharger/289/State"), None);
    }
}
