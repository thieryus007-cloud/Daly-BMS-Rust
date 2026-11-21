//! Noms personnalisés saisis dans la console Venus OS (écran du GX, Remote Console, VRM).
//!
//! Venus écrit `/CustomName` (ou `/SwitchableOutput/0/Settings/CustomName` pour un switch) via
//! `SetValue`. Le nom est conservé dans `custom-names.json`, à côté du fichier de config
//! (`/data/daly-bms/custom-names.json` sur le GX : survit aux redémarrages, aux mises à jour de
//! Venus et aux redéploiements, qui ne touchent pas ce fichier). Il prévaut sur le nom de
//! la config et sur un `CustomName` reçu dans les payloads MQTT. Nom vide = retour au nom par
//! défaut.
//!
//! Clé : `<classe>/<instance VRM>` (ex. `switch/66`), stable tant que l'instance ne change pas.

use parking_lot::Mutex;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::{info, warn};

/// Chemins D-Bus par lesquels Venus renomme un appareil.
const NAME_PATHS: [&str; 2] = ["/CustomName", "/SwitchableOutput/0/Settings/CustomName"];

struct Store {
    path:  Option<PathBuf>,
    names: Mutex<BTreeMap<String, String>>,
}

static STORE: OnceLock<Store> = OnceLock::new();

fn store() -> &'static Store {
    STORE.get_or_init(|| Store { path: None, names: Mutex::new(BTreeMap::new()) })
}

fn key(class: &str, instance: u32) -> String {
    format!("{class}/{instance}")
}

/// Charge les noms depuis `path` (absent = aucun nom). À appeler une fois au démarrage.
pub fn init(path: &Path) {
    let names: BTreeMap<String, String> = match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_else(|e| {
            warn!(path = %path.display(), "custom-names illisible ({e}) — ignoré");
            BTreeMap::new()
        }),
        Err(_) => BTreeMap::new(),
    };
    info!(path = %path.display(), count = names.len(), "Noms personnalisés Venus chargés");
    let _ = STORE.set(Store { path: Some(path.to_path_buf()), names: Mutex::new(names) });
}

/// Nom personnalisé enregistré pour cet appareil.
pub fn get(class: &str, instance: u32) -> Option<String> {
    store().names.lock().get(&key(class, instance)).cloned()
}

/// Enregistre (ou efface si vide) le nom et réécrit le fichier (écriture atomique).
pub fn set(class: &str, instance: u32, name: &str) {
    let name = name.trim();
    let s = store();
    let snapshot = {
        let mut m = s.names.lock();
        if name.is_empty() {
            m.remove(&key(class, instance));
        } else {
            m.insert(key(class, instance), name.to_string());
        }
        m.clone()
    };
    info!(device = %key(class, instance), name = %name, "Nom personnalisé saisi dans la console Venus");
    let Some(path) = &s.path else { return };
    let tmp = path.with_extension("json.tmp");
    let res = serde_json::to_string_pretty(&snapshot)
        .map_err(std::io::Error::other)
        .and_then(|json| std::fs::write(&tmp, json))
        .and_then(|_| std::fs::rename(&tmp, path));
    if let Err(e) = res {
        warn!(path = %path.display(), "Nom personnalisé non enregistré : {e}");
    }
}

/// Chemin par lequel Venus renomme un appareil ?
pub fn is_name_path(path: &str) -> bool {
    NAME_PATHS.contains(&path)
}

/// Traite un `SetValue` sur un chemin de nom. Retourne le code D-Bus (0 = accepté).
pub fn set_from_value(class: &str, instance: u32, val: &zvariant::Value<'_>) -> i32 {
    match val {
        zvariant::Value::Str(s) => {
            set(class, instance, s.as_str());
            0
        }
        _ => 1,
    }
}

/// Applique le nom enregistré aux items D-Bus d'un appareil : remplace `/CustomName` (et le
/// nom du canal d'un switch) ; sans nom enregistré, crée `/CustomName` vide s'il manque, pour
/// que la console Venus propose le renommage (nom vide = Venus affiche `/ProductName`).
pub fn apply<T>(class: &str, instance: u32, items: &mut HashMap<String, T>, mk: impl Fn(&str) -> T) {
    match get(class, instance) {
        Some(name) => {
            for p in NAME_PATHS {
                if p == NAME_PATHS[0] || items.contains_key(p) {
                    items.insert(p.to_string(), mk(&name));
                }
            }
        }
        None => {
            items.entry(NAME_PATHS[0].to_string()).or_insert_with(|| mk(""));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stored_name_wins_and_empty_name_restores_default() {
        let mut items: HashMap<String, String> = HashMap::new();
        items.insert("/CustomName".into(), "Tongou 0615D8".into());
        items.insert("/SwitchableOutput/0/Settings/CustomName".into(), "Tongou 0615D8".into());
        apply("test-switch", 68, &mut items, str::to_string);
        assert_eq!(items["/CustomName"], "Tongou 0615D8"); // pas de nom saisi : config

        set("test-switch", 68, "  Pompe piscine ");
        let mut items2 = items.clone();
        apply("test-switch", 68, &mut items2, str::to_string);
        assert_eq!(items2["/CustomName"], "Pompe piscine");
        assert_eq!(items2["/SwitchableOutput/0/Settings/CustomName"], "Pompe piscine");

        set("test-switch", 68, "");
        let mut items3 = items.clone();
        apply("test-switch", 68, &mut items3, str::to_string);
        assert_eq!(items3["/CustomName"], "Tongou 0615D8");
    }

    #[test]
    fn missing_custom_name_is_created_empty_for_renaming() {
        let mut items: HashMap<String, String> = HashMap::new();
        items.insert("/ProductName".into(), "ET112-AC2".into());
        apply("test-meter", 31, &mut items, str::to_string);
        assert_eq!(items["/CustomName"], "");
        assert!(!items.contains_key("/SwitchableOutput/0/Settings/CustomName"));
    }

    #[test]
    fn set_from_value_accepts_strings_only() {
        assert_eq!(set_from_value("test-x", 1, &zvariant::Value::from("A")), 0);
        assert_eq!(get("test-x", 1).as_deref(), Some("A"));
        assert_eq!(set_from_value("test-x", 1, &zvariant::Value::from(3i32)), 1);
        assert!(is_name_path("/CustomName") && !is_name_path("/ProductName"));
    }
}
