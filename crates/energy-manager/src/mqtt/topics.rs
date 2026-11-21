/// Build a Victron N/ topic for reading.
/// portal_id: e.g. "d81d13b31a8b"
pub fn n(portal_id: &str, path: &str) -> String {
    format!("N/{portal_id}/{path}")
}

/// Build a Victron W/ topic for writing a command.
pub fn w(portal_id: &str, path: &str) -> String {
    format!("W/{portal_id}/{path}")
}

// ---------------------------------------------------------------------------
// All topics the energy-manager subscribes to
// ---------------------------------------------------------------------------

pub fn all_subscriptions(
    portal_id: &str,
    vebus: u32,
    mppts: &[u32],
    pvinv: u32,
    shunt: u32,
    tasmota_waterheater_id: &str,
) -> Vec<String> {
    let pid = portal_id;
    let mut v = vec![
        // --- VEBus ---
        n(pid, &format!("vebus/{vebus}/Ac/State/IgnoreAcIn1")),
        n(pid, &format!("vebus/{vebus}/Ac/Out/L1/F")),
        n(pid, &format!("vebus/{vebus}/Ac/ActiveIn/Connected")),
        n(pid, &format!("vebus/{vebus}/Dc/0/Voltage")),
        n(pid, &format!("vebus/{vebus}/Dc/0/Current")),
        n(pid, &format!("vebus/{vebus}/Dc/0/Power")),
        n(pid, &format!("vebus/{vebus}/Ac/Out/L1/V")),
        n(pid, &format!("vebus/{vebus}/Ac/Out/L1/I")),
        n(pid, &format!("vebus/{vebus}/Ac/Out/L1/P")),
        n(pid, &format!("vebus/{vebus}/State")),
        n(pid, &format!("vebus/{vebus}/Energy/InverterToAcOut")),
        n(pid, &format!("vebus/{vebus}/Energy/OutToInverter")),

        // --- System aggregates ---
        n(pid, "system/0/Dc/Battery/Soc"),
        n(pid, "system/0/Dc/Battery/Current"),
        n(pid, "system/0/Dc/Battery/Power"),
        n(pid, "system/0/Dc/Battery/State"),
        n(pid, "system/0/Dc/Battery/TimeToGo"),
        n(pid, "system/0/Dc/Pv/Power"),
        n(pid, "system/0/Ac/PvOnOutput/L1/Power"),
        n(pid, "system/0/Ac/ConsumptionOnOutput/L1/Power"),

        // --- SmartShunt (battery/{shunt}) — native energy counters ---
        n(pid, &format!("battery/{shunt}/Dc/0/Voltage")),
        n(pid, &format!("battery/{shunt}/Dc/0/Current")),
        n(pid, &format!("battery/{shunt}/Dc/0/Power")),
        n(pid, &format!("battery/{shunt}/Soc")),
        n(pid, &format!("battery/{shunt}/TimeToGo")),
        n(pid, &format!("battery/{shunt}/State")),
        // Wildcards: match any battery instance so we don't need to know the
        // exact SmartShunt instance number (varies per installation).
        n(pid, "battery/+/History/ChargedEnergy"),
        n(pid, "battery/+/History/DischargedEnergy"),

    ];

    // --- MPPT : every followed charger (VictronConfig::mppt_list) ---
    // Paths must match `logic::solar_power::mppt_topic_map`.
    for m in mppts {
        for path in ["Yield/Power", "History/Daily/0/Yield", "History/Daily/0/MaxPower",
                     "State", "Pv/V", "Dc/0/Current", "MppOperationMode"] {
            v.push(n(pid, &format!("solarcharger/{m}/{path}")));
        }
    }

    v.extend([
        // --- PVInverter (ET112) ---
        n(pid, &format!("pvinverter/{pvinv}/Ac/Power")),
        n(pid, &format!("pvinverter/{pvinv}/Ac/Energy/Forward")),

        // --- Irradiance ---
        "santuario/irradiance/raw".to_string(),

        // --- Shelly (DEYE relay events) ---
        "shellypro2pm-ec62608840a4/events/rpc".to_string(),

        // --- Toshiba climatiseurs (télémétrie lecture seule, wildcard par zone).
        //     Préfixe LOCAL, non relayé par le bridge vers le GX (règle #11). ---
        "santuario/toshiba/+/state".to_string(),
        // Présence FP2 par pièce (contrôle opt-in `toshiba_ac.control_enabled`),
        // publiée par le pont `bridge/aqara-fp2-mqtt`. Préfixe LOCAL également.
        "santuario/toshiba/presence/+".to_string(),

        // --- Persist (retained baselines + DEYE state) ---
        "santuario/persist/pvinv_baseline".to_string(),
        "santuario/persist/yield_yesterday".to_string(),
        "santuario/persist/deye_state".to_string(),
    ]);

    // --- Tasmota du chauffe-eau (`victron.tasmota_waterheater_id`, logic::tasmota) ---
    if !tasmota_waterheater_id.is_empty() {
        v.push(format!("stat/{tasmota_waterheater_id}/POWER"));
        v.push(format!("tele/{tasmota_waterheater_id}/SENSOR"));
    }
    v
}

#[cfg(test)]
mod subscription_tests {
    use super::all_subscriptions;

    #[test]
    fn water_heater_tasmota_follows_config() {
        let v = all_subscriptions("pid", 276, &[273], 32, 274, "tongou_B459B8");
        assert!(v.contains(&"stat/tongou_B459B8/POWER".to_string()));
        assert!(v.contains(&"tele/tongou_B459B8/SENSOR".to_string()));
        assert!(!v.iter().any(|t| t.contains("tongou_3BC764")));
        let none = all_subscriptions("pid", 276, &[273], 32, 274, "");
        assert!(!none.iter().any(|t| t.starts_with("stat/") || t.starts_with("tele/")));
    }
}

// ---------------------------------------------------------------------------
// Publish topics (outputs)
// ---------------------------------------------------------------------------

pub mod publish {
    use super::w;

    pub fn vebus_max_charge_current(portal_id: &str, vebus: u32) -> String {
        w(portal_id, &format!("vebus/{vebus}/Dc/0/MaxChargeCurrent"))
    }

    pub fn vebus_power_assist(portal_id: &str, vebus: u32) -> String {
        w(portal_id, &format!("vebus/{vebus}/Settings/PowerAssistEnabled"))
    }

    pub fn cgwacs_max_feed_in(portal_id: &str) -> String {
        w(portal_id, "settings/0/Settings/CGwacs/MaxFeedInPower")
    }

    pub fn shelly_rpc(shelly_id: &str) -> String {
        format!("{shelly_id}/rpc")
    }

    #[allow(dead_code)]
    pub fn tasmota_cmd(tasmota_id: &str) -> String {
        format!("cmnd/{tasmota_id}/Power")
    }

    /// Commande d'une clim Toshiba (contrôle par présence FP2). Non-retained : c'est
    /// un événement, consommé par le firmware ESP32 sur `.../<zone>/command`.
    pub fn toshiba_command(zone: &str) -> String {
        format!("santuario/toshiba/{zone}/command")
    }

    pub const HEATPUMP_VENUS: &str = "santuario/heatpump/1/venus";
    pub const SWITCH_VENUS:   &str = "santuario/switch/1/venus";
    pub const PLATFORM_VENUS: &str = "santuario/platform/venus";
    pub const HEAT_VENUS:     &str = "santuario/heat/1/venus";
    pub const METEO_VENUS:    &str = "santuario/meteo/venus";
    pub const INVERTER_VENUS: &str = "santuario/inverter/venus";
    pub const SYSTEM_VENUS:   &str = "santuario/system/venus";
    /// Télémétrie solaire agrégée 1 Hz → consommée par daly-bms-server.
    /// Remplace l'ancien POST HTTP 1 Hz vers /api/v1/solar/mppt-yield : la
    /// rétention par requête HTTP côté serveur était le moteur de la fuite
    /// RSS résiduelle (cf. docs/diagnostic-depannage.md §17).
    pub const EM_SOLAR:        &str = "santuario/em/solar";
    pub const PVINV_BASELINE:  &str = "santuario/persist/pvinv_baseline";
    pub const YIELD_YESTERDAY: &str = "santuario/persist/yield_yesterday";
    pub const DEYE_STATE:      &str = "santuario/persist/deye_state";
}
