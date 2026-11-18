//! Water-heater mode decision — **pure Rust** (was `rules/water_heater.grl`).

/// Returns `"VACATION"`, `"HEAT_PUMP"` or `"TURBO"`.
///
/// Any active condition forces VACATION:
/// - `grid_connected`      : grid ON (`ac_ignore == 0`)
/// - `soc_low`             : `soc_pct < soc_min_pct`
/// - `irradiance_low`      : `irradiance < irradiance_min_wm2` (solaire disponible insuffisant)
/// - `temp_max_reached`    : tank held `>= temp_max_c` for `temp_max_hold_secs` (cuve à température cible)
/// - `battery_discharging` : battery discharging over the control cycle (see [`battery_discharging`])
///
/// All clear → TURBO when `battery_full` (see [`turbo_wanted`]), HEAT_PUMP otherwise.
/// TURBO therefore only ever replaces HEAT_PUMP: never on grid, at night, with a hot tank
/// or while the battery is discharging.
pub fn evaluate(
    grid_connected: bool,
    soc_low: bool,
    irradiance_low: bool,
    temp_max_reached: bool,
    battery_discharging: bool,
    battery_full: bool,
) -> &'static str {
    if grid_connected || soc_low || irradiance_low || temp_max_reached || battery_discharging {
        "VACATION"
    } else if battery_full {
        "TURBO"
    } else {
        "HEAT_PUMP"
    }
}

/// « Batterie pleine » with hysteresis: true above `enter_pct`, and — once the heater is
/// already in TURBO — as long as `soc_pct >= exit_pct`. TURBO draws enough to pull the SOC
/// back under the entry threshold, which without hysteresis would alternate TURBO/HEAT_PUMP
/// at every allowed mode change.
pub fn turbo_wanted(soc_pct: f64, currently_turbo: bool, enter_pct: f64, exit_pct: f64) -> bool {
    soc_pct > enter_pct || (currently_turbo && soc_pct >= exit_pct)
}

/// Battery discharging: mean battery power over the control cycle (SmartShunt, W, < 0 =
/// discharge) below `-block_w`. The solar production then does not cover the consumption
/// (water heater included). This is the reliable "production insuffisante" signal: with a
/// full battery the MPPT output is curtailed down to the consumption and says nothing about
/// the available sun (that part is `irradiance_low`).
pub fn battery_discharging(avg_power_w: f64, block_w: f64) -> bool {
    avg_power_w < -block_w
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_ok_gives_heat_pump() {
        assert_eq!(evaluate(false, false, false, false, false, false), "HEAT_PUMP");
    }

    #[test]
    fn battery_full_without_blocker_gives_turbo() {
        assert_eq!(evaluate(false, false, false, false, false, true), "TURBO");
    }

    #[test]
    fn any_condition_forces_vacation_even_when_battery_full() {
        for full in [false, true] {
            assert_eq!(evaluate(true, false, false, false, false, full), "VACATION");
            assert_eq!(evaluate(false, true, false, false, false, full), "VACATION");
            assert_eq!(evaluate(false, false, true, false, false, full), "VACATION");
            assert_eq!(evaluate(false, false, false, true, false, full), "VACATION");
            assert_eq!(evaluate(false, false, false, false, true, full), "VACATION");
        }
    }

    #[test]
    fn all_bad_forces_vacation() {
        assert_eq!(evaluate(true, true, true, true, true, true), "VACATION");
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
    fn discharge_blocks_only_beyond_tolerance() {
        assert!(!battery_discharging(1500.0, 100.0)); // en charge
        assert!(!battery_discharging(-100.0, 100.0)); // à la tolérance
        assert!(battery_discharging(-100.1, 100.0));
        assert!(battery_discharging(-2200.0, 100.0)); // Turbo non couvert par le solaire
    }
}
