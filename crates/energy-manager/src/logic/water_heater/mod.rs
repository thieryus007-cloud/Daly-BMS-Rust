//crates/energy-manager/src/logic/water_heater/mod.rs

mod rules;
use chrono::{DateTime, Utc};
use serde_json::json;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio::time::{interval, sleep, Duration, MissedTickBehavior};
use tracing::{debug, error, info, warn};
use crate::bus::AppBus;
use crate::config::WaterHeaterConfig;
use crate::http_clients::lg_thinq::LgThinqClient;
use crate::mqtt::topics::publish;
use crate::types::{EnergyState, LiveEvent, MqttOutgoing, WaterHeaterMode};

pub async fn spawn(
    wh_cfg: Arc<RwLock<WaterHeaterConfig>>,
    lg: Option<Arc<LgThinqClient>>,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
) {
    // keepalive_secs est lu une seule fois (l'intervalle du ticker est fixé au démarrage).
    let keepalive_secs = wh_cfg.read().await.keepalive_secs;
    crate::supervise::spawn_critical(keepalive_task(keepalive_secs, bus.clone(), state.clone()));
    if let Some(lg_client) = lg {
        let window = Arc::new(Mutex::new(PowerWindow::default()));
        crate::supervise::spawn_critical(battery_sampler_task(window.clone(), state.clone()));
        crate::supervise::spawn_critical(control_task(wh_cfg, lg_client, bus, state, window));
    } else {
        info!("Water heater auto-control disabled (no LG ThinQ client)");
    }
}

/// Échantillons de puissance batterie (SmartShunt, W, < 0 = décharge) : cumul du cycle de
/// contrôle (règle « décharge batterie » à 5 min) + dernière minute (coupure rapide).
#[derive(Default)]
struct PowerWindow {
    sum_w: f64,
    n: u32,
    /// Derniers échantillons frais (≤ BATTERY_MIN_SAMPLES, soit 1 min).
    recent: VecDeque<f64>,
    /// Échantillons encore à ignorer après un changement de mode (SETTLE_SAMPLES).
    skip: u32,
    /// Énergie fournie par la batterie depuis le dernier changement de mode (Wh, décharges
    /// seules) : budget de la tolérance de démarrage.
    discharge_wh: f64,
}

impl PowerWindow {
    fn push(&mut self, p: f64) {
        if self.skip > 0 {
            self.skip -= 1;
            return;
        }
        if p < 0.0 {
            self.discharge_wh += -p * BATTERY_SAMPLE_SECS as f64 / 3600.0;
        }
        self.sum_w += p;
        self.n += 1;
        self.recent.push_back(p);
        while self.recent.len() > BATTERY_MIN_SAMPLES as usize {
            self.recent.pop_front();
        }
    }

    /// Mesure absente ou périmée : la dernière minute n'est plus continue → vidée, pour
    /// qu'une valeur ancienne ne puisse pas déclencher une coupure.
    fn clear_recent(&mut self) {
        self.recent.clear();
    }

    /// Changement de mode envoyé : les échantillons passés décrivent l'ancienne consommation
    /// → cumul du cycle et dernière minute vidés, et les SETTLE_SAMPLES suivants ignorés (le
    /// temps que le LG applique le mode). Sans cela, après TURBO → HEAT_PUMP, la minute
    /// contiendrait encore la décharge du TURBO et couperait aussi la HEAT_PUMP.
    fn reset_after_change(&mut self) {
        self.sum_w = 0.0;
        self.n = 0;
        self.recent.clear();
        self.skip = SETTLE_SAMPLES;
        self.discharge_wh = 0.0;
    }

    /// Énergie fournie par la batterie depuis le dernier changement de mode (Wh).
    fn discharge_wh(&self) -> f64 {
        self.discharge_wh
    }

    /// Moyenne de la dernière minute (None tant qu'elle n'est pas complète).
    fn recent_avg(&self) -> Option<f64> {
        (self.recent.len() >= BATTERY_MIN_SAMPLES as usize)
            .then(|| self.recent.iter().sum::<f64>() / self.recent.len() as f64)
    }

    /// Moyenne du cycle écoulé (None sous BATTERY_MIN_SAMPLES), puis remise à zéro du cumul.
    fn take_cycle_avg(&mut self) -> Option<f64> {
        let avg = (self.n >= BATTERY_MIN_SAMPLES).then(|| self.sum_w / f64::from(self.n));
        self.sum_w = 0.0;
        self.n = 0;
        avg
    }
}

/// Période de la coupure rapide (vérification de la dernière minute de batterie).
const FAST_GUARD_SECS: u64 = 30;
/// Après un échec d'envoi par la coupure rapide, délai avant nouvel essai (API LG).
const FAST_GUARD_RETRY_SECS: i64 = 120;

const BATTERY_SAMPLE_SECS: u64 = 10;
/// Au-delà, la dernière mesure SmartShunt est périmée : elle n'est pas échantillonnée.
const BATTERY_SAMPLE_MAX_AGE_SECS: i64 = 30;
/// Échantillons minimum (1 min) pour décider sur la moyenne batterie du cycle.
const BATTERY_MIN_SAMPLES: u32 = 6;
/// Échantillons ignorés après un changement de mode (20 s). Mesuré le 18/11 : TURBO coupé à
/// 12:58:06, batterie encore à −65 W à 12:58:10, +1591 W à 12:58:20.
const SETTLE_SAMPLES: u32 = 2;
/// Au-delà, `/State` ou `/MppOperationMode` d'un MPPT est périmé (Venus les republie toutes
/// les ~25 s) : ce MPPT ne compte pas pour la production bridée.
const MPPT_MAX_AGE_SECS: i64 = 120;

/// Production solaire bridée batterie pleine, sur les MPPT aux mesures fraîches.
fn production_curtailed_now(s: &EnergyState, now: DateTime<Utc>, full_soc_pct: f64) -> bool {
    let fresh = |t: Option<DateTime<Utc>>| t.is_some_and(|t| (now - t).num_seconds() <= MPPT_MAX_AGE_SECS);
    let mppts: Vec<(i64, i64)> = s
        .mppts
        .values()
        .filter(|m| fresh(m.mpp_mode_last_ts) && fresh(m.state_last_ts))
        .filter_map(|m| Some((m.mpp_mode?, m.state?)))
        .collect();
    rules::production_curtailed(&mppts, s.soc_pct, full_soc_pct)
}

/// Verrou après une descente pour décharge : mode maximal autorisé et échéance.
type DischargeLock = Option<(WaterHeaterMode, DateTime<Utc>)>;

/// Verrou encore actif à `now` (None une fois l'échéance passée).
fn active_lock(lock: DischargeLock, now: DateTime<Utc>) -> DischargeLock {
    lock.filter(|&(_, until)| now < until)
}

/// Ce que le verrou interdit, pour les journaux.
fn lock_desc(ceiling: WaterHeaterMode) -> &'static str {
    match ceiling {
        WaterHeaterMode::Turbo => "aucun mode interdit",
        WaterHeaterMode::HeatPump => "TURBO interdit",
        WaterHeaterMode::Vacation => "HEAT_PUMP/TURBO interdits",
    }
}

fn set_lock_status(s: &mut EnergyState, lock: DischargeLock) {
    s.water_heater_rule.lock_ceiling = lock.map(|(m, _)| m.to_lg_str().to_string());
    s.water_heater_rule.lock_until = lock.map(|(_, t)| t);
}

fn set_decision(s: &mut EnergyState, target: WaterHeaterMode, reason: &str, now: DateTime<Utc>) {
    s.water_heater_rule.target = Some(target.to_lg_str().to_string());
    s.water_heater_rule.reason = Some(reason.to_string());
    s.water_heater_rule.decided_ts = Some(now);
}

/// Puissance batterie SmartShunt si la mesure est fraîche.
fn fresh_battery_power(s: &EnergyState, now: DateTime<Utc>) -> Option<f64> {
    let fresh = s
        .shunt_last_seen_ts
        .is_some_and(|t| (now - t).num_seconds() <= BATTERY_SAMPLE_MAX_AGE_SECS);
    if fresh { s.battery_power_w } else { None }
}

/// Échantillonne la puissance batterie toutes les 10 s : la décision porte sur la moyenne
/// du cycle, un pic bref (nuage, démarrage du compresseur) ne déclenche pas le blocage.
async fn battery_sampler_task(window: Arc<Mutex<PowerWindow>>, state: Arc<RwLock<EnergyState>>) {
    let mut ticker = interval(Duration::from_secs(BATTERY_SAMPLE_SECS));
    loop {
        ticker.tick().await;
        let sample = fresh_battery_power(&*state.read().await, Utc::now());
        let mut w = window.lock().await;
        match sample {
            Some(p) => w.push(p),
            None => w.clear_recent(),
        }
    }
}

async fn keepalive_task(interval_secs: u64, bus: AppBus, state: Arc<RwLock<EnergyState>>) {
    let mut ticker = interval(Duration::from_secs(interval_secs));
    loop {
        ticker.tick().await;
        publish_to_venus(&bus, &state).await;
    }
}

async fn publish_to_venus(bus: &AppBus, state: &Arc<RwLock<EnergyState>>) {
    let s = state.read().await;
    let mode   = s.water_heater_mode.to_venus_state();
    let temp   = s.water_heater_temp_c;
    let target = s.water_heater_target_c;
    let payload = json!({
        "State": mode,
        "Temperature": temp,
        "TargetTemperature": target,
        "Position": 0,
    });
    drop(s);
    bus.publish(MqttOutgoing::retained(publish::HEATPUMP_VENUS, &payload)).await;
    // Topics scalaires (dashboards externes type Dashboard Studio) : °C et mode 0/1/2.
    if let Some(t) = temp {
        bus.publish(MqttOutgoing::raw("santuario/em/water_heater/temp", format!("{t:.1}"), true)).await;
    }
    if let Some(t) = target {
        bus.publish(MqttOutgoing::raw("santuario/em/water_heater/target", format!("{t:.1}"), true)).await;
    }
    bus.publish(MqttOutgoing::raw("santuario/em/water_heater/mode", mode.to_string(), true)).await;
    bus.emit_live(LiveEvent::new("water_heater_venus", &payload));
}

/// Ancre (ou réarme) le chrono « cuve à température cible » en fonction de la
/// dernière température lue, stockée au préalable dans `water_heater_temp_c`.
///
/// Appelée à **chaque** lecture de température — par le poller LG ThinQ ET par
/// le `control_task` — afin que le chrono démarre dès l'instant où la cuve passe
/// ≥ `temp_max_c`, sans attendre le prochain tick de 5 min du control_task.
///
/// - Temp ≥ seuil : on date le premier franchissement (`get_or_insert`).
/// - Temp < seuil : la cible n'est plus tenue → on réarme.
/// - Temp inconnue (erreur API/timeout transitoire) : on PRÉSERVE le chrono en
///   cours pour ne pas perdre le temps accumulé.
pub(crate) fn anchor_temp_max(s: &mut EnergyState, now: DateTime<Utc>, temp_max_c: f64) {
    match s.water_heater_temp_c {
        Some(t) if t >= temp_max_c => {
            s.water_heater_temp_max_since.get_or_insert(now);
        }
        Some(_) => s.water_heater_temp_max_since = None,
        None => {}
    }
}

/// Met à jour le suivi « température cible atteinte » dans l'état partagé et
/// renvoie `true` si la température de la cuve est restée ≥ `temp_max_c`
/// pendant au moins `hold_secs`.
///
/// L'ancrage du chrono (`water_heater_temp_max_since`) est délégué à
/// [`anchor_temp_max`] ; cette fonction ne fait que mesurer la durée écoulée
/// depuis cet ancrage pour décider du passage en VACATION.
fn update_temp_max_tracking(
    s: &mut EnergyState,
    now: DateTime<Utc>,
    temp_max_c: f64,
    hold_secs: u64,
) -> bool {
    anchor_temp_max(s, now, temp_max_c);
    match (s.water_heater_temp_c, s.water_heater_temp_max_since) {
        (Some(t), Some(since)) if t >= temp_max_c => {
            (now - since).num_seconds().max(0) as u64 >= hold_secs
        }
        _ => false,
    }
}

async fn write_wh_metrics(vm_url: &str, mode: WaterHeaterMode, temp: Option<f64>, target: Option<f64>) {
    let ts_ms = Utc::now().timestamp_millis();
    let mut lines = Vec::new();
    lines.push(format!("wh_mode{{}} {} {}", mode.to_venus_state(), ts_ms));
    if let Some(t) = temp {
        lines.push(format!("wh_current_temp_c{{}} {} {}", t, ts_ms));
    }
    if let Some(t) = target {
        lines.push(format!("wh_target_temp_c{{}} {} {}", t, ts_ms));
    }
    let body = lines.join("\n");
    let url = format!("{}/api/v1/import/prometheus", vm_url);
    if let Err(e) = crate::http_clients::shared_client()
        .post(&url)
        .body(body)
        .send()
        .await
    {
        warn!("Water heater VM write error: {e}");
    }
}

/// Envoie le mode au LG puis, après `temp_set_delay_secs`, la consigne associée ; met à jour
/// l'état partagé et Venus, et remet à zéro la mesure batterie (elle décrivait l'ancien mode).
/// Commun au cycle de contrôle et à la coupure rapide.
/// Retourne `false` si l'envoi du mode a échoué (rien n'est modifié).
async fn apply_mode(
    lg: &Arc<LgThinqClient>,
    bus: &AppBus,
    state: &Arc<RwLock<EnergyState>>,
    cfg: &WaterHeaterConfig,
    window: &Arc<Mutex<PowerWindow>>,
    target_mode: WaterHeaterMode,
    now: DateTime<Utc>,
) -> bool {
    if let Err(e) = lg.set_mode(target_mode).await {
        error!("LG set_mode error: {e}");
        return false;
    }
    window.lock().await.reset_after_change();
    {
        let mut s = state.write().await;
        s.water_heater_mode = target_mode;
        s.water_heater_last_change = Some(now);
        s.water_heater_send_count += 1;
        s.water_heater_rule.battery_minute_avg_w = None;
    }
    publish_to_venus(bus, state).await;

    let delay_secs = cfg.temp_set_delay_secs;
    let target_temp = match target_mode {
        WaterHeaterMode::HeatPump => cfg.heat_pump_target_c,
        WaterHeaterMode::Turbo    => cfg.turbo_target_c,
        WaterHeaterMode::Vacation => cfg.vacation_target_c,
    };
    let lg2 = lg.clone();
    let bus2 = bus.clone();
    let state2 = state.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(delay_secs)).await;
        if let Err(e) = lg2.set_target_temp(target_temp).await {
            error!("LG set_target_temp error: {e}");
            return;
        }
        {
            let mut s = state2.write().await;
            s.water_heater_target_c = Some(target_temp);
        }
        publish_to_venus(&bus2, &state2).await;
    });
    true
}

/// Coupure rapide « aucune énergie batterie pour le chauffe-eau » (toutes les 30 s) :
/// chauffe-eau en TURBO/HEAT_PUMP et décharge imputable → un cran en dessous immédiatement
/// (TURBO → HEAT_PUMP, HEAT_PUMP → VACATION), sans attendre la fin du cycle ni le délai entre
/// changements. Imputable = budget de démarrage dépassé pendant la tolérance de démarrage,
/// sinon décharge sur la dernière minute hors production bridée ([`rules::fast_cut`]). Le mode
/// quitté reste ensuite interdit `discharge_lockout_secs` (pas de cycles courts quand le
/// solaire est tout juste suffisant). Rafraîchit aussi l'état exposé par /api/rules-status.
#[allow(clippy::too_many_arguments)]
async fn fast_discharge_cut(
    wh_cfg: &Arc<RwLock<WaterHeaterConfig>>,
    lg: &Arc<LgThinqClient>,
    bus: &AppBus,
    state: &Arc<RwLock<EnergyState>>,
    window: &Arc<Mutex<PowerWindow>>,
    last_change: &mut Option<DateTime<Utc>>,
    lock: &mut DischargeLock,
    start_grace_until: &mut Option<DateTime<Utc>>,
    retry_after: &mut Option<DateTime<Utc>>,
) {
    let now = Utc::now();
    *lock = active_lock(*lock, now);
    *start_grace_until = start_grace_until.filter(|&t| now < t);
    let cfg = wh_cfg.read().await.clone();
    let (recent, used_wh) = {
        let w = window.lock().await;
        (w.recent_avg(), w.discharge_wh())
    };
    let (mode, curtailed) = {
        let mut s = state.write().await;
        let curtailed = production_curtailed_now(&s, now, cfg.full_battery_soc_pct);
        s.water_heater_rule.battery_minute_avg_w = recent;
        s.water_heater_rule.production_curtailed = curtailed;
        s.water_heater_rule.start_grace_until = *start_grace_until;
        s.water_heater_rule.start_used_wh = start_grace_until.map(|_| used_wh);
        set_lock_status(&mut s, *lock);
        (s.water_heater_mode, curtailed)
    };
    if retry_after.is_some_and(|t| now < t) {
        return;
    }
    let budget = start_grace_until.map(|_| (used_wh, cfg.start_grace_max_wh));
    let Some(target) = rules::fast_cut(mode, recent, cfg.discharge_block_w, curtailed, budget) else {
        return;
    };
    let (why, reason) = match budget {
        Some((used, max)) => (
            format!("démarrage : la batterie a fourni {used:.0} Wh (budget {max:.0} Wh)"),
            "coupure rapide : budget de démarrage dépassé",
        ),
        None => (
            format!(
                "batterie {:.0} W en moyenne sur la dernière minute (seuil −{:.0} W), production non bridée",
                recent.unwrap_or(0.0), cfg.discharge_block_w
            ),
            "coupure rapide : batterie en décharge sur la dernière minute",
        ),
    };
    warn!(
        "Water heater: COUPURE RAPIDE {mode:?} → {target:?} — {why} ; {} pendant {} s",
        lock_desc(target), cfg.discharge_lockout_secs
    );
    if apply_mode(lg, bus, state, &cfg, window, target, now).await {
        *last_change = Some(now);
        *lock = Some((target, now + chrono::Duration::seconds(cfg.discharge_lockout_secs as i64)));
        *start_grace_until = None;
        *retry_after = None;
        let mut s = state.write().await;
        set_decision(&mut s, target, reason, now);
        set_lock_status(&mut s, *lock);
        s.water_heater_rule.start_grace_until = None;
        s.water_heater_rule.start_used_wh = None;
    } else {
        *retry_after = Some(now + chrono::Duration::seconds(FAST_GUARD_RETRY_SECS));
    }
}

async fn control_task(
    wh_cfg: Arc<RwLock<WaterHeaterConfig>>,
    lg: Arc<LgThinqClient>,
    bus: AppBus,
    state: Arc<RwLock<EnergyState>>,
    window: Arc<Mutex<PowerWindow>>,
) {
    let mut last_change: Option<DateTime<Utc>> = None;
    let mut consecutive_fails: u32 = 0;
    let mut ticker = interval(Duration::from_secs(300));
    let mut guard = interval(Duration::from_secs(FAST_GUARD_SECS));
    guard.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let mut lock: DischargeLock = None;
    let mut start_grace_until: Option<DateTime<Utc>> = None;
    let mut guard_retry_after: Option<DateTime<Utc>> = None;

    info!("Water heater control task started (5min interval, coupure rapide batterie toutes les {FAST_GUARD_SECS}s)");

    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = guard.tick() => {
                fast_discharge_cut(
                    &wh_cfg, &lg, &bus, &state, &window,
                    &mut last_change, &mut lock, &mut start_grace_until, &mut guard_retry_after,
                ).await;
                continue;
            }
        }
        let now = Utc::now();

        // Snapshot des seuils courants : un clone par tick (toutes les 5 min) →
        // coût négligeable. Rechargeables à chaud (POST /api/v1/em/rules/reload) :
        // la nouvelle valeur s'applique au tick suivant.
        let cfg = wh_cfg.read().await.clone();

        // Read actual state from LG ThinQ before deciding
        debug!("Water heater: calling lg.get_state()...");
        let lg_snapshot = match lg.get_state().await {
            Ok(snap) => {
                debug!(
                    "Water heater: LG get_state OK → mode={:?}, temp={:?}°C, target={:?}°C",
                    snap.mode, snap.current_temp_c, snap.target_temp_c
                );
                {
                    let mut s = state.write().await;
                    s.water_heater_mode     = snap.mode;
                    s.water_heater_temp_c   = snap.current_temp_c;
                    s.water_heater_target_c = snap.target_temp_c;
                    s.water_heater_last_read = Some(now);
                }
                write_wh_metrics(
                    &cfg.vm_url,
                    snap.mode,
                    snap.current_temp_c,
                    snap.target_temp_c,
                ).await;
                Some(snap)
            }
            Err(e) => {
                error!("LG ThinQ get_state FAILED: {e} — will skip mode comparison this tick");
                None
            }
        };

        // Suivi « cuve à température cible » : si la température (lue à l'instant
        // ci-dessus, sinon valeur en cache rafraîchie par le poller LG) reste
        // ≥ temp_max_c pendant temp_max_hold_secs → on forcera VACATION.
        let (temp_max_reached, temp_max_since) = {
            let mut s = state.write().await;
            let reached = update_temp_max_tracking(&mut s, now, cfg.temp_max_c, cfg.temp_max_hold_secs);
            (reached, s.water_heater_temp_max_since)
        };
        if temp_max_reached {
            info!(
                "Water heater: température ≥ {:.1}°C tenue ≥ {}s (depuis {:?}) → cible atteinte, VACATION forcé",
                cfg.temp_max_c, cfg.temp_max_hold_secs, temp_max_since
            );
        }

        // Puissance batterie moyenne du cycle écoulé (fenêtre vidée à chaque tick et à chaque
        // changement de mode). Jamais de décision sur une mesure instantanée : batterie pleine,
        // la régulation des MPPT donne des creux brefs (−189 W relevés au démarrage le 18/11)
        // qui forceraient VACATION pour toute la durée du cooldown. Moins de
        // BATTERY_MIN_SAMPLES → cycle sauté.
        let battery_avg_w = window.lock().await.take_cycle_avg();

        // Read energy conditions — skip evaluation if MQTT data not yet available
        let (ac_ignore_opt, soc_opt, irradiance) = {
            let s = state.read().await;
            (s.ac_ignore, s.soc_pct, s.irradiance_wm2)
        };

        debug!(
            "Water heater tick — ac_ignore={:?}, soc={:?}, irradiance={:?} W/m² (min={})",
            ac_ignore_opt, soc_opt, irradiance, cfg.irradiance_min_wm2
        );

        if ac_ignore_opt.is_none() || soc_opt.is_none() {
            warn!(
                "Water heater: MQTT data missing (ac_ignore={:?}, soc={:?}) — skipping evaluation. \
                 Check that energy-manager subscriptions are received from broker.",
                ac_ignore_opt, soc_opt
            );
            continue;
        }

        let Some(battery_avg_w) = battery_avg_w else {
            if last_change.is_some_and(|t| (now - t).num_seconds() < 300) {
                debug!(
                    "Water heater: changement de mode il y a moins d'un cycle — mesure batterie \
                     remise à zéro, évaluation sautée, mode actuel conservé"
                );
            } else {
                warn!(
                    "Water heater: moins de {} échantillons SmartShunt frais sur le cycle (démarrage du \
                     service, ou mesure périmée > {}s) — évaluation sautée, mode actuel conservé",
                    BATTERY_MIN_SAMPLES, BATTERY_SAMPLE_MAX_AGE_SECS
                );
            }
            continue;
        };

        let ac_ignore = ac_ignore_opt.unwrap_or(0);
        let soc       = soc_opt.unwrap_or(0.0);
        let irradiance_low = match irradiance {
            Some(w) => {
                let low = w < cfg.irradiance_min_wm2;
                debug!("Water heater: irradiance={:.1} W/m², min={}, irradiance_low={}", w, cfg.irradiance_min_wm2, low);
                low
            }
            None => {
                warn!(
                    "Water heater: irradiance_wm2=None (topic 'santuario/irradiance/raw' not received yet) \
                     — treating as irradiance_low=true → target will be VACATION"
                );
                true
            }
        };
        let grid_connected = ac_ignore == 0;
        let soc_low = soc < cfg.soc_min_pct;
        debug!(
            "Water heater: ac_ignore={}, grid_connected={}, soc={:.1}% (min={}%, soc_low={}), irradiance_low={}",
            ac_ignore, grid_connected, soc, cfg.soc_min_pct, soc_low, irradiance_low
        );

        // Use actual LG state (from readback) as the reference for comparison — also needed
        // by the TURBO hysteresis (already in TURBO → stay while SOC ≥ turbo_soc_exit_pct) and
        // by the discharge step-down (one level below the current mode).
        let actual_mode = lg_snapshot
            .as_ref()
            .map(|s| s.mode)
            .unwrap_or_else(|| {
                warn!("Water heater: LG readback failed, using cached state for comparison");
                state.try_read().map(|s| s.water_heater_mode).unwrap_or(WaterHeaterMode::Vacation)
            });

        let battery_full = rules::turbo_wanted(
            soc,
            actual_mode == WaterHeaterMode::Turbo,
            cfg.turbo_soc_enter_pct,
            cfg.turbo_soc_exit_pct,
        );
        // Après une descente pour décharge, le mode quitté reste interdit jusqu'à l'échéance.
        lock = active_lock(lock, now);
        start_grace_until = start_grace_until.filter(|&t| now < t);
        // Décharge imputable au chauffe-eau : hors production bridée (MPPT limités batterie
        // pleine) et hors tolérance de démarrage (bornée par son budget, coupure rapide).
        let curtailed = production_curtailed_now(&*state.read().await, now, cfg.full_battery_soc_pct);
        let starting = start_grace_until.is_some();
        let inputs = rules::Inputs {
            grid_connected,
            soc_low,
            irradiance_low,
            temp_max_reached,
            battery_discharging: rules::discharge_counts(
                rules::battery_discharging(battery_avg_w, cfg.discharge_block_w),
                curtailed,
                starting,
            ),
            battery_full,
            current: actual_mode,
            ceiling: lock.map(|(m, _)| m),
        };
        let (target_mode, reason) = rules::evaluate(&inputs);
        {
            let mut s = state.write().await;
            set_decision(&mut s, target_mode, reason, now);
            s.water_heater_rule.battery_cycle_avg_w = Some(battery_avg_w);
            s.water_heater_rule.production_curtailed = curtailed;
            set_lock_status(&mut s, lock);
        }
        // Démoté info!→debug! (2026-06) : la décision est ré-évaluée à chaque tick
        // (toutes les 5 min) → 288 lignes/jour même sans changement. Le SEND réel
        // (changement de mode effectif) reste en info! plus bas.
        debug!(
            "Water heater: decision → {target_mode:?} ({reason}) — {inputs:?}, battery_avg={:.0}W, \
             curtailed={curtailed}, starting={starting}, lock={:?}",
            battery_avg_w, lock
        );

        let should_send = actual_mode != target_mode;

        if !should_send {
            debug!(
                "Water heater: target={:?} matches actual={:?}, no command needed",
                target_mode, actual_mode
            );
            consecutive_fails = 0;
            continue;
        }

        let can_change = last_change
            .map(|t| (now - t).num_seconds() as u64 >= cfg.mode_change_min_secs)
            .unwrap_or(true);

        if !can_change {
            let secs_left = cfg.mode_change_min_secs.saturating_sub(
                last_change.map(|t| (now - t).num_seconds() as u64).unwrap_or(0)
            );
            debug!("Water heater: cooldown active, {}s remaining — target={:?}", secs_left, target_mode);
            continue;
        }

        info!(
            "Water heater: SEND {:?} — {} (actual={:?}, soc={:.1}%, grid={}, irradiance_low={}, battery_avg={:.0}W, production_bridée={}, battery_full={}, lock={:?})",
            target_mode, reason, actual_mode, soc, grid_connected, irradiance_low, battery_avg_w, curtailed, battery_full, lock
        );

        if !apply_mode(&lg, &bus, &state, &cfg, &window, target_mode, now).await {
            consecutive_fails += 1;
            if consecutive_fails >= 3 {
                warn!(
                    "Water heater: {} consecutive send failures — LG ThinQ may be unreachable!",
                    consecutive_fails
                );
            }
            continue;
        }

        consecutive_fails = 0;
        last_change = Some(now);

        // Démarrage (mode qui consomme plus) : tolérance de démarrage, bornée par
        // start_grace_max_wh (coupure rapide). Toute descente la referme.
        start_grace_until = rules::is_increase(actual_mode, target_mode)
            .then(|| now + chrono::Duration::seconds(cfg.start_grace_secs as i64));
        if let Some(until) = start_grace_until {
            info!(
                "Water heater: démarrage {actual_mode:?} → {target_mode:?} — décharge tolérée jusqu'à {until} \
                 dans la limite de {:.0} Wh",
                cfg.start_grace_max_wh
            );
        }

        // Descente pour décharge vue sur le cycle (TURBO → HEAT_PUMP, HEAT_PUMP → VACATION) :
        // même verrou qu'après une coupure rapide.
        if inputs.battery_discharging && target_mode == rules::step_down(actual_mode) {
            lock = Some((target_mode, now + chrono::Duration::seconds(cfg.discharge_lockout_secs as i64)));
            info!(
                "Water heater: descente pour décharge (cycle) — {} pendant {} s",
                lock_desc(target_mode), cfg.discharge_lockout_secs
            );
        }
        {
            let mut s = state.write().await;
            set_lock_status(&mut s, lock);
            s.water_heater_rule.start_grace_until = start_grace_until;
            s.water_heater_rule.start_used_wh = start_grace_until.map(|_| 0.0);
        }
    }   // close loop
}

#[cfg(test)]
mod power_window_tests {
    use super::{PowerWindow, BATTERY_MIN_SAMPLES, SETTLE_SAMPLES};

    #[test]
    fn recent_minute_is_rolling_and_needs_full_minute() {
        let mut w = PowerWindow::default();
        for _ in 0..BATTERY_MIN_SAMPLES - 1 {
            w.push(-300.0);
        }
        assert_eq!(w.recent_avg(), None);
        w.push(-300.0);
        assert_eq!(w.recent_avg(), Some(-300.0));
        for _ in 0..BATTERY_MIN_SAMPLES {
            w.push(60.0); // les anciens échantillons sortent de la minute
        }
        assert_eq!(w.recent_avg(), Some(60.0));
    }

    #[test]
    fn stale_measure_clears_minute_but_not_cycle() {
        let mut w = PowerWindow::default();
        for _ in 0..BATTERY_MIN_SAMPLES {
            w.push(-500.0);
        }
        w.clear_recent();
        assert_eq!(w.recent_avg(), None);
        assert_eq!(w.take_cycle_avg(), Some(-500.0));
        assert_eq!(w.take_cycle_avg(), None); // cumul remis à zéro
    }

    #[test]
    fn mode_change_forgets_old_mode_and_skips_settling() {
        let mut w = PowerWindow::default();
        for _ in 0..BATTERY_MIN_SAMPLES {
            w.push(-800.0); // TURBO non couvert par le solaire
        }
        w.reset_after_change(); // TURBO → HEAT_PUMP envoyé
        assert_eq!(w.recent_avg(), None);
        for _ in 0..SETTLE_SAMPLES {
            w.push(-800.0); // résistance pas encore coupée : ignoré
        }
        for _ in 0..BATTERY_MIN_SAMPLES {
            w.push(100.0); // HEAT_PUMP couverte par le solaire
        }
        assert_eq!(w.recent_avg(), Some(100.0));
        assert_eq!(w.take_cycle_avg(), Some(100.0));
    }

    #[test]
    fn start_budget_counts_discharge_since_last_change() {
        let mut w = PowerWindow::default();
        w.push(-3600.0); // 10 s à −3600 W = 10 Wh, avant le changement de mode
        w.reset_after_change();
        assert_eq!(w.discharge_wh(), 0.0);
        for _ in 0..SETTLE_SAMPLES {
            w.push(-3600.0); // pendant l'attente de 20 s : ignoré
        }
        w.push(-1800.0); // 5 Wh
        w.push(1800.0); // une recharge ne rembourse pas le budget
        w.push(-360.0); // 1 Wh
        assert!((w.discharge_wh() - 6.0).abs() < 1e-9);
    }
}
