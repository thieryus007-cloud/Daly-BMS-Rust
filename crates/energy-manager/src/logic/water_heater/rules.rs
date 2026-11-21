//! Water-heater mode decision — **pure Rust** (was `rules/water_heater.grl`).
//!
//! Principe : aucune énergie batterie pour le chauffe-eau. Batterie en décharge → on descend
//! d'un cran ([`step_down`] : TURBO → HEAT_PUMP → VACATION) et le mode quitté reste interdit
//! `discharge_lockout_secs` (plafond `Inputs::ceiling`, posé par le control_task).
//!
//! Deux décharges ne sont pas imputées au chauffe-eau ([`discharge_counts`]) :
//! - le **démarrage** (VACATION → HEAT_PUMP/TURBO, HEAT_PUMP → TURBO) : la batterie fournit
//!   l'appoint le temps que les MPPT remontent, borné par un budget en Wh ([`fast_cut`]) ;
//! - la **production bridée** batterie pleine ([`production_curtailed`]) : MPPT limités en
//!   tension, du solaire est disponible ; la batterie redescend vers sa tension de float avec
//!   ou sans chauffe-eau.

use crate::types::WaterHeaterMode;

/// Entrées de la décision du cycle de contrôle (5 min).
#[derive(Debug, Clone, Copy, Default)]
pub struct Inputs {
    /// Réseau connecté (`ac_ignore == 0`).
    pub grid_connected: bool,
    /// `soc_pct < soc_min_pct`.
    pub soc_low: bool,
    /// `irradiance < irradiance_min_wm2` (solaire disponible insuffisant).
    pub irradiance_low: bool,
    /// Cuve tenue `>= temp_max_c` pendant `temp_max_hold_secs` (cuve à température cible).
    pub temp_max_reached: bool,
    /// Batterie en décharge sur le cycle écoulé, imputable au chauffe-eau (voir
    /// [`discharge_counts`]).
    pub battery_discharging: bool,
    /// Batterie pleine (voir [`turbo_wanted`]).
    pub battery_full: bool,
    /// Mode LG relu avant la décision.
    pub current: WaterHeaterMode,
    /// Verrou après une descente pour décharge : mode maximal autorisé (None = pas de verrou).
    pub ceiling: Option<WaterHeaterMode>,
}

/// Mode cible du cycle et sa raison :
/// 1. réseau connecté, SOC bas, irradiance faible ou cuve chaude → VACATION ;
/// 2. batterie en décharge → un cran sous le mode actuel ([`step_down`]) ;
/// 3. sinon TURBO si batterie pleine, HEAT_PUMP sinon ;
///
/// puis limité au plafond du verrou de décharge. TURBO ne fait donc que remplacer HEAT_PUMP :
/// jamais sur réseau, la nuit, cuve chaude, batterie en décharge ni sous verrou.
pub fn evaluate(i: &Inputs) -> (WaterHeaterMode, &'static str) {
    use WaterHeaterMode::{HeatPump, Turbo, Vacation};
    let (wanted, why) = if i.grid_connected {
        (Vacation, "réseau connecté")
    } else if i.soc_low {
        (Vacation, "SOC bas")
    } else if i.irradiance_low {
        (Vacation, "irradiance faible")
    } else if i.temp_max_reached {
        (Vacation, "cuve à température cible")
    } else if i.battery_discharging {
        (step_down(i.current), "batterie en décharge")
    } else if i.battery_full {
        (Turbo, "batterie pleine")
    } else {
        (HeatPump, "conditions OK")
    };
    match i.ceiling {
        Some(c) if rank(wanted) > rank(c) => (c, "verrou après coupure pour décharge"),
        _ => (wanted, why),
    }
}

/// Un cran de consommation en dessous : TURBO → HEAT_PUMP, HEAT_PUMP → VACATION.
pub fn step_down(mode: WaterHeaterMode) -> WaterHeaterMode {
    match mode {
        WaterHeaterMode::Turbo => WaterHeaterMode::HeatPump,
        WaterHeaterMode::HeatPump | WaterHeaterMode::Vacation => WaterHeaterMode::Vacation,
    }
}

/// Ordre de consommation : VACATION < HEAT_PUMP < TURBO.
fn rank(mode: WaterHeaterMode) -> u8 {
    match mode {
        WaterHeaterMode::Vacation => 0,
        WaterHeaterMode::HeatPump => 1,
        WaterHeaterMode::Turbo => 2,
    }
}

/// Passage vers un mode qui consomme plus (démarrage) : ouvre la tolérance de démarrage.
pub fn is_increase(from: WaterHeaterMode, to: WaterHeaterMode) -> bool {
    rank(to) > rank(from)
}

/// MPPT Victron : `MppOperationMode` 1 = limité en tension ou en courant (production bridée).
const MPP_LIMITED: i64 = 1;
/// MPPT Victron : `State` 4 = absorption, 5 = float, 6 = storage (batterie pleine).
const MPPT_FULL_STATES: [i64; 3] = [4, 5, 6];

/// Production solaire bridée batterie pleine : au moins un MPPT (mesures fraîches, `(mode,
/// state)`) limité en tension en absorption/float, et SOC ≥ `full_soc_pct`. Du solaire est
/// alors disponible mais non utilisé : la batterie se décharge parce qu'elle est au-dessus de
/// la tension de régulation des MPPT, pas faute de soleil. En bulk, un MPPT limité l'est en
/// courant (pas de réserve) : il ne compte pas. Un vrai manque de soleil fait baisser la
/// tension sous leur consigne : les MPPT passent alors en suivi MPP (mode 2) → false.
pub fn production_curtailed(mppts: &[(i64, i64)], soc_pct: Option<f64>, full_soc_pct: f64) -> bool {
    soc_pct.is_some_and(|soc| soc >= full_soc_pct)
        && mppts.iter().any(|&(mode, state)| mode == MPP_LIMITED && MPPT_FULL_STATES.contains(&state))
}

/// Décharge imputable au chauffe-eau : décharge mesurée, hors production bridée et hors
/// tolérance de démarrage (le démarrage est borné par son budget, voir [`fast_cut`]).
pub fn discharge_counts(discharging: bool, curtailed: bool, starting: bool) -> bool {
    discharging && !curtailed && !starting
}

/// « Batterie pleine » with hysteresis: true above `enter_pct`, and — once the heater is
/// already in TURBO — as long as `soc_pct >= exit_pct`. TURBO draws enough to pull the SOC
/// back under the entry threshold, which without hysteresis would alternate TURBO/HEAT_PUMP
/// at every allowed mode change.
pub fn turbo_wanted(soc_pct: f64, currently_turbo: bool, enter_pct: f64, exit_pct: f64) -> bool {
    soc_pct > enter_pct || (currently_turbo && soc_pct >= exit_pct)
}

/// Battery discharging: mean battery power (SmartShunt, W, < 0 = discharge) below
/// `-block_w`. The solar production then does not cover the consumption (water heater
/// included). This is the reliable "production insuffisante" signal: with a full battery the
/// MPPT output is curtailed down to the consumption and says nothing about the available sun
/// (that part is `irradiance_low`).
pub fn battery_discharging(avg_power_w: f64, block_w: f64) -> bool {
    avg_power_w < -block_w
}

/// Coupure rapide (vérifiée toutes les 30 s) : chauffe-eau en TURBO/HEAT_PUMP → mode d'un cran
/// en dessous, tout de suite, sans attendre la fin du cycle ni le délai entre changements :
/// - pendant la tolérance de démarrage (`start_budget` = `(Wh fournis par la batterie depuis
///   le démarrage, budget Wh)`) : seulement si le budget est dépassé ;
/// - sinon : batterie en décharge sur la dernière minute, hors production bridée.
///
/// None : rien à couper (déjà VACATION, décharge non imputable, ou minute incomplète).
pub fn fast_cut(
    current: WaterHeaterMode,
    recent_avg_w: Option<f64>,
    block_w: f64,
    curtailed: bool,
    start_budget: Option<(f64, f64)>,
) -> Option<WaterHeaterMode> {
    let cut = match start_budget {
        Some((used_wh, max_wh)) => used_wh > max_wh,
        None => discharge_counts(
            recent_avg_w.is_some_and(|avg| battery_discharging(avg, block_w)),
            curtailed,
            false,
        ),
    };
    (cut && current != WaterHeaterMode::Vacation).then(|| step_down(current))
}

#[cfg(test)]
mod tests {
    use super::*;
    use WaterHeaterMode::{HeatPump, Turbo, Vacation};

    fn ok_from(current: WaterHeaterMode) -> Inputs {
        Inputs { current, ..Inputs::default() }
    }

    #[test]
    fn all_ok_gives_heat_pump() {
        assert_eq!(evaluate(&ok_from(Vacation)).0, HeatPump);
    }

    #[test]
    fn battery_full_without_blocker_gives_turbo() {
        let i = Inputs { battery_full: true, ..ok_from(HeatPump) };
        assert_eq!(evaluate(&i).0, Turbo);
    }

    #[test]
    fn blockers_force_vacation_from_any_mode() {
        for current in [Vacation, HeatPump, Turbo] {
            for full in [false, true] {
                let base = Inputs { battery_full: full, ..ok_from(current) };
                for i in [
                    Inputs { grid_connected: true, ..base },
                    Inputs { soc_low: true, ..base },
                    Inputs { irradiance_low: true, ..base },
                    Inputs { temp_max_reached: true, ..base },
                ] {
                    assert_eq!(evaluate(&i).0, Vacation, "{i:?}");
                }
            }
        }
    }

    #[test]
    fn discharge_steps_down_one_level() {
        for full in [false, true] {
            let d = |current| Inputs { battery_discharging: true, battery_full: full, ..ok_from(current) };
            assert_eq!(evaluate(&d(Turbo)).0, HeatPump); // le solaire couvre peut-être la PAC seule
            assert_eq!(evaluate(&d(HeatPump)).0, Vacation);
            assert_eq!(evaluate(&d(Vacation)).0, Vacation); // consommation maison : pas de démarrage
        }
    }

    #[test]
    fn lock_caps_the_target() {
        let full = Inputs { battery_full: true, ..ok_from(HeatPump) };
        assert_eq!(evaluate(&Inputs { ceiling: Some(HeatPump), ..full }).0, HeatPump);
        assert_eq!(evaluate(&Inputs { ceiling: Some(Vacation), ..full }).0, Vacation);
        assert_eq!(evaluate(&Inputs { ceiling: Some(Turbo), ..full }).0, Turbo);
        // le plafond ne fait jamais monter : un bloqueur garde VACATION
        let blocked = Inputs { irradiance_low: true, ceiling: Some(HeatPump), ..full };
        assert_eq!(evaluate(&blocked), (Vacation, "irradiance faible"));
    }

    #[test]
    fn turbo_enters_strictly_above_98() {
        assert!(!turbo_wanted(98.0, false, 98.0, 95.0));
        assert!(turbo_wanted(98.1, false, 98.0, 95.0));
    }

    #[test]
    fn turbo_holds_down_to_95_only_when_already_turbo() {
        assert!(turbo_wanted(96.0, true, 98.0, 95.0));
        assert!(turbo_wanted(95.0, true, 98.0, 95.0));
        assert!(!turbo_wanted(94.9, true, 98.0, 95.0));
        assert!(!turbo_wanted(96.0, false, 98.0, 95.0));
    }

    #[test]
    fn fast_cut_steps_down_turbo_then_heat_pump() {
        assert_eq!(fast_cut(Turbo, Some(-300.0), 50.0, false, None), Some(HeatPump));
        assert_eq!(fast_cut(HeatPump, Some(-300.0), 50.0, false, None), Some(Vacation));
        assert_eq!(fast_cut(Vacation, Some(-2000.0), 50.0, false, None), None); // rien à couper
        assert_eq!(fast_cut(Turbo, Some(-40.0), 50.0, false, None), None); // dans la tolérance
        assert_eq!(fast_cut(Turbo, None, 50.0, false, None), None); // minute incomplète
    }

    #[test]
    fn fast_cut_ignores_regulation_discharge_when_production_curtailed() {
        // 18–20/11 : MPPT limités batterie pleine, HEAT_PUMP coupée à −51…−62 W → plus de coupure
        assert_eq!(fast_cut(HeatPump, Some(-140.0), 50.0, true, None), None);
        assert_eq!(fast_cut(HeatPump, Some(-140.0), 50.0, false, None), Some(Vacation));
    }

    #[test]
    fn fast_cut_during_start_only_on_budget() {
        // 18/11 11:01 : −1538 W au démarrage du TURBO, 24 Wh avant que les MPPT couvrent
        assert_eq!(fast_cut(Turbo, Some(-1500.0), 50.0, false, Some((24.0, 40.0))), None);
        assert_eq!(fast_cut(Turbo, Some(-200.0), 50.0, true, Some((40.1, 40.0))), Some(HeatPump));
        assert_eq!(fast_cut(Vacation, None, 50.0, false, Some((99.0, 40.0))), None);
    }

    #[test]
    fn curtailed_needs_full_battery_and_a_voltage_limited_mppt() {
        let limited_float = [(2, 3), (1, 5)];
        assert!(production_curtailed(&limited_float, Some(100.0), 98.0));
        assert!(!production_curtailed(&limited_float, Some(97.9), 98.0)); // filet SOC
        assert!(!production_curtailed(&limited_float, None, 98.0));
        assert!(!production_curtailed(&[(2, 4), (2, 5)], Some(100.0), 98.0)); // suivi MPP : tout est utilisé
        assert!(!production_curtailed(&[(1, 3)], Some(100.0), 98.0)); // bulk : limité en courant
        assert!(!production_curtailed(&[], Some(100.0), 98.0)); // MPPT inconnus / périmés
    }

    #[test]
    fn discharge_counts_only_outside_start_and_curtailment() {
        assert!(discharge_counts(true, false, false));
        assert!(!discharge_counts(true, true, false));
        assert!(!discharge_counts(true, false, true));
        assert!(!discharge_counts(false, false, false));
        assert!(is_increase(Vacation, HeatPump) && is_increase(HeatPump, Turbo));
        assert!(!is_increase(Turbo, HeatPump) && !is_increase(HeatPump, HeatPump));
    }

    #[test]
    fn discharge_blocks_only_beyond_tolerance() {
        assert!(!battery_discharging(1500.0, 50.0)); // en charge
        assert!(!battery_discharging(-50.0, 50.0)); // à la tolérance
        assert!(battery_discharging(-50.1, 50.0));
        assert!(battery_discharging(-2200.0, 50.0)); // Turbo non couvert par le solaire
    }
}
