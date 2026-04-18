# Plan de mise en œuvre — Améliorations code Daly-BMS-Rust

> **Cible d'exécution** : Haiku 4.5, taux de réussite visé 100 %.
> **Règle absolue** : aucune modification ne doit changer un comportement fonctionnel observable
> (API publique, schéma MQTT, points InfluxDB, topics, instances D-Bus, ports réseau).
> **Branche de travail** : `claude/code-review-improvement-plan-AVIPU`.

---

## 0. Mode d'emploi pour Haiku 4.5

Pour chaque tâche, suivre **dans l'ordre** :

1. Lire intégralement le fichier cible avec l'outil `Read` **avant** toute édition.
2. Appliquer l'édition avec `Edit` en copiant-collant le bloc *AVANT* et le bloc *APRÈS* fournis ici.
3. Après chaque tâche, exécuter la vérification indiquée (`cargo check`, `bash -n`, etc.).
4. Si la vérification échoue → `git diff` + corriger. Ne **jamais** passer à la tâche suivante tant que la vérif n'est pas verte.
5. Commit Git à la fin de chaque Phase (pas après chaque tâche) avec le message indiqué.
6. À la fin de toutes les phases, un seul `git push -u origin claude/code-review-improvement-plan-AVIPU`.

**Règles dures** :
- Ne jamais toucher : `Cargo.lock`, `Config.toml`, `nanoPi/config-nanopi.toml`, `flux-nodered/*.json`, `grafana/**`, `.env`.
- Ne jamais supprimer ou renommer un type public, une fonction publique, un champ `serde`.
- Ne jamais changer une chaîne littérale utilisée comme topic MQTT, service D-Bus ou nom d'instance.
- Si une ligne du code actuel ne correspond **pas exactement** au bloc *AVANT* (numéro de ligne ou contenu), **STOP** : relire le fichier, adapter la recherche, ne jamais inventer.
- Tous les numéros de ligne sont indicatifs au moment de la rédaction ; le contenu textuel fait foi.

**Commande de vérification globale** (à lancer après chaque Phase) :
```bash
cargo fmt --all -- --check && cargo check --workspace && cargo clippy --workspace --all-targets -- -D warnings
```

---

## 1. Inventaire des améliorations retenues

| # | Catégorie | Fichier | Risque | Effort |
|---|-----------|---------|--------|--------|
| T1 | Imports inutilisés | `crates/daly-bms-server/src/bridges/mqtt.rs` | NONE | 2 min |
| T2 | Constantes seuils d'alerte | `crates/daly-bms-server/src/config.rs` | LOW | 10 min |
| T3 | Constantes `&str` pour defaults | `crates/daly-bms-server/src/config.rs` | LOW | 10 min |
| T4 | Helper `parse_modbus_address` | `crates/daly-bms-server/src/config.rs` | LOW | 15 min |
| T5 | Warning explicite sur fallback addr ET112 / Irradiance | `crates/daly-bms-server/src/config.rs` | LOW | 10 min |
| T6 | Validation `InfluxConfig::validate()` | `crates/daly-bms-server/src/config.rs` + `main.rs` | MEDIUM | 15 min |
| T7 | Log MQTT sans username en clair | `crates/daly-bms-server/src/bridges/mqtt.rs` | LOW | 5 min |
| T8 | Clone inutile dans MQTT bridge | `crates/daly-bms-server/src/bridges/mqtt.rs` | LOW | 5 min |
| T9 | Tests unitaires de parsing | `crates/daly-bms-server/src/config.rs` | NONE | 15 min |
| T10 | Scripts bash : `set -euo pipefail` | `DEPLOY.sh`, `QUICK_MQTT_DIAGNOSIS.sh`, `debug.sh` | LOW | 10 min |
| T11 | Makefile : branche courante par défaut | `Makefile` | LOW | 3 min |
| T12 | TODO obsolète nettoyé | `crates/daly-bms-server/src/api/*` | NONE | 5 min |

**Total estimé** : ~1 h 45 min.

**Hors périmètre** (proposés mais écartés pour risque trop élevé) :
- Changement de la signature publique de `parsed_address` → romprait les call sites.
- Réduction des features `tokio` → risque de compiler mais runtime qui casse une API async utilisée ailleurs.
- Refactor ring buffers en générique → changement structurel non demandé.
- Nouveaux endpoints ou templates → hors périmètre (pas de nouvelle fonctionnalité).

---

## 2. PHASE 1 — Nettoyage sans risque (≈ 15 min)

### T1 — Supprimer `use chrono::Utc` inutilisé

**Fichier** : `crates/daly-bms-server/src/bridges/mqtt.rs`
**Précondition** : vérifier `rg 'Utc' crates/daly-bms-server/src/bridges/mqtt.rs` → doit retourner uniquement la ligne `use chrono::Utc;`. Si d'autres occurrences apparaissent, **ne rien faire** et signaler.

**AVANT** (vers ligne 21) :
```rust
use chrono::Utc;
```

**APRÈS** (supprimer la ligne complète — ne pas laisser de ligne vide double) :
```rust
```

**Vérification** :
```bash
cargo check -p daly-bms-server
```

---

### T11 — Makefile : branche de déploiement = branche courante

**Fichier** : `Makefile`

**AVANT** (ligne ~177) :
```make
BRANCH ?= claude/integrate-ats-modbus-q0wyn
```

**APRÈS** :
```make
BRANCH ?= $(shell git rev-parse --abbrev-ref HEAD 2>/dev/null || echo main)
```

**Vérification** :
```bash
make -n sync   # doit afficher 'git fetch origin <branche-courante>' sans erreur
```

---

### T12 — Nettoyage éventuel d'un TODO obsolète

**Fichier** : chercher la chaîne `TODO: Phase 2 — utiliser DalyBusManager::discover()`.

```bash
# Localiser
```
Utiliser `Grep` avec `pattern: "TODO: Phase 2 — utiliser DalyBusManager::discover"`. S'il n'est **pas trouvé**, passer la tâche. Sinon :

**AVANT** :
```rust
    // TODO: Phase 2 — utiliser DalyBusManager::discover()
```

**APRÈS** :
```rust
```
(supprimer purement la ligne)

**Vérification** :
```bash
cargo check -p daly-bms-server
```

---

### Commit de la Phase 1
```bash
git add -A
git commit -m "chore(cleanup): remove unused imports, dynamic Makefile branch, stale TODO"
```

---

## 3. PHASE 2 — Constantes nommées & helpers (≈ 30 min)

### T2 — Constantes pour les seuils d'alerte

**Fichier** : `crates/daly-bms-server/src/config.rs`
**Localisation** : bloc `impl Default for AlertThresholds` (vers ligne 447).

**AVANT** :
```rust
impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cell_ovp_v:            3.60,
            cell_uvp_v:            2.90,
            cell_delta_mv:         100.0,
            soc_low_percent:       20.0,
            soc_critical_percent:  10.0,
            temp_high_c:           45.0,
            current_high_a:        80.0,
        }
    }
}
```

**APRÈS** :
```rust
// Seuils d'alerte par défaut — ces valeurs reflètent les réglages de production
// documentés dans CLAUDE.md. Ne pas changer sans mise à jour de la doc.
const DEFAULT_CELL_OVP_V:           f32 = 3.60;
const DEFAULT_CELL_UVP_V:           f32 = 2.90;
const DEFAULT_CELL_DELTA_MV:        f32 = 100.0;
const DEFAULT_SOC_LOW_PERCENT:      f32 = 20.0;
const DEFAULT_SOC_CRITICAL_PERCENT: f32 = 10.0;
const DEFAULT_TEMP_HIGH_C:          f32 = 45.0;
const DEFAULT_CURRENT_HIGH_A:       f32 = 80.0;

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            cell_ovp_v:            DEFAULT_CELL_OVP_V,
            cell_uvp_v:            DEFAULT_CELL_UVP_V,
            cell_delta_mv:         DEFAULT_CELL_DELTA_MV,
            soc_low_percent:       DEFAULT_SOC_LOW_PERCENT,
            soc_critical_percent:  DEFAULT_SOC_CRITICAL_PERCENT,
            temp_high_c:           DEFAULT_TEMP_HIGH_C,
            current_high_a:        DEFAULT_CURRENT_HIGH_A,
        }
    }
}
```

**Vérification** :
```bash
cargo check -p daly-bms-server
```
Le comportement reste strictement identique (mêmes valeurs).

---

### T3 — Remplacer les `"…".to_string()` littéraux par des constantes `&str`

**Fichier** : `crates/daly-bms-server/src/config.rs`
**Zones concernées** : lignes ~234-239, ~290-295, ~493-498, ~571-573.

**Règle** : chaque `fn default_xxx() -> String { "literal".to_string() }` devient :
```rust
const DEFAULT_XXX: &str = "literal";
fn default_xxx() -> String { DEFAULT_XXX.to_string() }
```

Appliquer cette transformation **uniquement** pour les six defaults suivants (laisser les autres tels quels) :

| Fonction | Littéral |
|----------|----------|
| `default_et112_service_type` | `"pvinverter"` |
| `default_et112_name` | `"ET112"` |
| `default_ats_name` | `"ATS CHINT"` |
| `default_irradiance_name` | `"Irradiance PRALRAN"` |
| `default_tasmota_name` | `"Tasmota"` |
| `default_tasmota_service_type` | `"switch"` |

**Exemple pour `default_et112_service_type`** — **AVANT** :
```rust
fn default_et112_service_type() -> String { "pvinverter".to_string() }
```
**APRÈS** :
```rust
const DEFAULT_ET112_SERVICE_TYPE: &str = "pvinverter";
fn default_et112_service_type() -> String { DEFAULT_ET112_SERVICE_TYPE.to_string() }
```

Répéter pour les 5 autres. **Ne pas** ajouter `pub` à la constante. **Ne pas** toucher au reste du fichier.

**Vérification** :
```bash
cargo check -p daly-bms-server
```

---

### T4 — Helper privé `parse_modbus_address`

**Fichier** : `crates/daly-bms-server/src/config.rs`

Ajouter **une seule fois, tout en haut du fichier après les `use …;`** (à vérifier via `Read` du début du fichier), le helper suivant :

```rust
/// Parse une adresse Modbus au format "0x05" ou "5" en u8.
/// Retourne `None` si le format est invalide.
fn parse_modbus_address(s: &str) -> Option<u8> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix("0x").or_else(|| s.strip_prefix("0X")) {
        u8::from_str_radix(hex, 16).ok()
    } else {
        s.parse::<u8>().ok()
    }
}
```

Puis réécrire les **trois** `impl …::parsed_address` **en gardant la signature publique existante** (ne pas changer le type de retour) :

**Pour `BmsDeviceConfig::parsed_address` (retourne déjà `Option<u8>`)** :

AVANT :
```rust
    pub fn parsed_address(&self) -> Option<u8> {
        let s = self.address.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u8::from_str_radix(&s[2..], 16).ok()
        } else {
            s.parse::<u8>().ok()
        }
    }
```
APRÈS :
```rust
    pub fn parsed_address(&self) -> Option<u8> {
        parse_modbus_address(&self.address)
    }
```

**Pour `Et112DeviceConfig::parsed_address` (retourne `u8`, fallback `3`)** — conserver le fallback **à l'identique** :

AVANT :
```rust
    pub fn parsed_address(&self) -> u8 {
        let s = self.address.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u8::from_str_radix(&s[2..], 16).unwrap_or(3)
        } else {
            s.parse::<u8>().unwrap_or(3)
        }
    }
```
APRÈS :
```rust
    pub fn parsed_address(&self) -> u8 {
        parse_modbus_address(&self.address).unwrap_or_else(|| {
            tracing::warn!(addr = %self.address, "ET112 address invalide, fallback 0x03");
            3
        })
    }
```

**Pour `IrradianceConfig::parsed_address` (retourne `u8`, fallback `5`)** :

AVANT :
```rust
    pub fn parsed_address(&self) -> u8 {
        let s = self.address.trim();
        if s.starts_with("0x") || s.starts_with("0X") {
            u8::from_str_radix(&s[2..], 16).unwrap_or(5)
        } else {
            s.parse::<u8>().unwrap_or(5)
        }
    }
```
APRÈS :
```rust
    pub fn parsed_address(&self) -> u8 {
        parse_modbus_address(&self.address).unwrap_or_else(|| {
            tracing::warn!(addr = %self.address, "Irradiance address invalide, fallback 0x05");
            5
        })
    }
```

> Le fallback *numérique* reste strictement identique. Seul un log `warn!` s'ajoute quand l'entrée est invalide — comportement observable inchangé dans le cas nominal (adresse valide).

**T5 est couvert par T4** (les `warn!` ajoutés *sont* T5).

**Vérification** :
```bash
cargo check -p daly-bms-server
cargo clippy -p daly-bms-server --all-targets -- -D warnings
```

---

### Commit de la Phase 2
```bash
git add -A
git commit -m "refactor(config): named constants for thresholds + shared parse_modbus_address helper"
```

---

## 4. PHASE 3 — Validation & logs (≈ 25 min)

### T6 — `InfluxConfig::validate()` appelée au démarrage

**Fichier 1** : `crates/daly-bms-server/src/config.rs`

Ajouter la méthode **immédiatement après** la définition de `impl Default for InfluxConfig` (ou, si elle n'existe pas, immédiatement après la `struct InfluxConfig`) :

```rust
impl InfluxConfig {
    /// Valide la configuration InfluxDB au démarrage.
    /// Ne renvoie une erreur que si `enabled = true` et un champ critique est vide.
    pub fn validate(&self) -> anyhow::Result<()> {
        if !self.enabled {
            return Ok(());
        }
        if self.url.trim().is_empty() {
            anyhow::bail!("InfluxDB enabled mais [influxdb].url est vide");
        }
        if self.token.trim().is_empty() {
            anyhow::bail!("InfluxDB enabled mais [influxdb].token est vide");
        }
        Ok(())
    }
}
```

> **Important** : ne pas modifier les champs de `InfluxConfig`. Si la `struct` est dans un autre crate, **abandonner cette tâche** et consigner en note (ne **pas** forcer).

**Fichier 2** : `crates/daly-bms-server/src/main.rs`

Trouver l'endroit où `AppConfig::load_default()?` est appelé (juste après le load). Ajouter **juste après** :

```rust
    config.influxdb.validate()?;
```

Si `config` est nommé autrement (`cfg`, `app_cfg`…), adapter. **Ne pas** déplacer d'autres lignes.

**Vérification** :
```bash
cargo check -p daly-bms-server
```

---

### T7 — MQTT : ne pas logger `username` en clair au niveau `info`

**Fichier** : `crates/daly-bms-server/src/bridges/mqtt.rs`

**AVANT** (vers ligne 42-53) :
```rust
    info!(host = %cfg.host, port = cfg.port, "Démarrage MQTT bridge");

    let mut opts = MqttOptions::new(
        format!("daly-bms-{}", uuid::Uuid::new_v4()),
        &cfg.host,
        cfg.port,
    );
    opts.set_keep_alive(Duration::from_secs(30));

    if let (Some(user), Some(pass)) = (&cfg.username, &cfg.password) {
        opts.set_credentials(user, pass);
    }
```

**APRÈS** :
```rust
    info!(
        host = %cfg.host,
        port = cfg.port,
        authenticated = cfg.username.is_some() && cfg.password.is_some(),
        "Démarrage MQTT bridge"
    );

    let mut opts = MqttOptions::new(
        format!("daly-bms-{}", uuid::Uuid::new_v4()),
        &cfg.host,
        cfg.port,
    );
    opts.set_keep_alive(Duration::from_secs(30));

    if let (Some(user), Some(pass)) = (&cfg.username, &cfg.password) {
        debug!(username = %user, "MQTT credentials configurés");
        opts.set_credentials(user, pass);
    }
```

`debug` est déjà importé (ligne 28 : `use tracing::{debug, error, info, warn};`).

**Vérification** :
```bash
cargo check -p daly-bms-server
```

---

### T8 — Supprimer un `clone()` inutile dans mqtt.rs

**Fichier** : `crates/daly-bms-server/src/bridges/mqtt.rs`
**Localisation** : vers ligne 78-81.

**AVANT** :
```rust
            let topic_id = addr_map
                .get(&snap.address)
                .cloned()
                .unwrap_or_else(|| snap.address.to_string());
```

**APRÈS** :
```rust
            let topic_id = addr_map
                .get(&snap.address)
                .map(String::as_str)
                .map(|s| s.to_string())
                .unwrap_or_else(|| snap.address.to_string());
```

> Note : l'appel suivant `publish_snapshot(&client, &cfg, snap, &topic_id)` prend `&str`/`&String`, le type `String` reste donc adéquat. Le gain est cosmétique (pas de `.cloned()` implicite sur `Option<&String>`). Si Clippy proteste, **annuler** cette tâche et laisser le code d'origine.

**Vérification** :
```bash
cargo clippy -p daly-bms-server --all-targets -- -D warnings
```

---

### Commit de la Phase 3
```bash
git add -A
git commit -m "fix(observability): validate influx config at boot + avoid logging mqtt username at info level"
```

---

## 5. PHASE 4 — Tests (≈ 15 min)

### T9 — Tests unitaires pour `parse_modbus_address` et `AlertThresholds::default`

**Fichier** : `crates/daly-bms-server/src/config.rs`

Ajouter **tout à la fin du fichier** (après la dernière accolade, bien en dehors de tout bloc `impl`) :

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_modbus_address_hex_lowercase() {
        assert_eq!(parse_modbus_address("0x05"), Some(5));
        assert_eq!(parse_modbus_address("0xff"), Some(255));
    }

    #[test]
    fn parse_modbus_address_hex_uppercase() {
        assert_eq!(parse_modbus_address("0X05"), Some(5));
        assert_eq!(parse_modbus_address("0XFF"), Some(255));
    }

    #[test]
    fn parse_modbus_address_decimal() {
        assert_eq!(parse_modbus_address("5"), Some(5));
        assert_eq!(parse_modbus_address("255"), Some(255));
    }

    #[test]
    fn parse_modbus_address_whitespace_trimmed() {
        assert_eq!(parse_modbus_address("  0x07 "), Some(7));
        assert_eq!(parse_modbus_address(" 40 "), Some(40));
    }

    #[test]
    fn parse_modbus_address_invalid_returns_none() {
        assert_eq!(parse_modbus_address(""), None);
        assert_eq!(parse_modbus_address("0xZZ"), None);
        assert_eq!(parse_modbus_address("300"), None);   // overflow u8
        assert_eq!(parse_modbus_address("not-a-number"), None);
    }

    #[test]
    fn alert_thresholds_defaults_unchanged() {
        let t = AlertThresholds::default();
        assert_eq!(t.cell_ovp_v, 3.60);
        assert_eq!(t.cell_uvp_v, 2.90);
        assert_eq!(t.cell_delta_mv, 100.0);
        assert_eq!(t.soc_low_percent, 20.0);
        assert_eq!(t.soc_critical_percent, 10.0);
        assert_eq!(t.temp_high_c, 45.0);
        assert_eq!(t.current_high_a, 80.0);
    }
}
```

**Vérification** :
```bash
cargo test -p daly-bms-server --lib config::tests
```
Tous les tests doivent passer (6 tests OK).

---

### Commit de la Phase 4
```bash
git add -A
git commit -m "test(config): cover parse_modbus_address + AlertThresholds default values"
```

---

## 6. PHASE 5 — Scripts bash (≈ 10 min)

### T10 — `set -euo pipefail` pour tous les scripts

Appliquer exactement la même modification aux **trois** fichiers :

1. `DEPLOY.sh`
2. `QUICK_MQTT_DIAGNOSIS.sh`
3. `debug.sh`

Pour chacun :

**AVANT** (première ligne non-shebang) :
```bash
#!/bin/bash
...
```

**Règle** :
- Si le fichier contient `set -e` → le remplacer par `set -euo pipefail`.
- Si le fichier ne contient **pas** `set -e` → insérer `set -euo pipefail` **juste après la première ligne shebang + ligne de commentaire optionnelle**.

**Exemple pour `DEPLOY.sh`** (contient déjà `set -e` en ligne 5) :

AVANT :
```bash
#!/bin/bash
# SCRIPT DE DÉPLOIEMENT COMPLET & VALIDATION
# À exécuter sur Pi5: bash ~/Daly-BMS-Rust/DEPLOY.sh

set -e
```

APRÈS :
```bash
#!/bin/bash
# SCRIPT DE DÉPLOIEMENT COMPLET & VALIDATION
# À exécuter sur Pi5: bash ~/Daly-BMS-Rust/DEPLOY.sh

set -euo pipefail
```

**Exemple pour `QUICK_MQTT_DIAGNOSIS.sh`** (pas de `set -e` actuel) :

AVANT :
```bash
#!/bin/bash
# Script de diagnostic rapide MQTT — À exécuter sur le Pi5
# Affiche 5 vérifications essentielles

echo "═══════════════════════════════════════════════════════════════"
```
APRÈS :
```bash
#!/bin/bash
# Script de diagnostic rapide MQTT — À exécuter sur le Pi5
# Affiche 5 vérifications essentielles
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
```

Pour `debug.sh`, procéder de la même manière (insérer `set -euo pipefail` après le bloc de commentaires de tête si absent).

> **Attention** : avec `-u`, toute variable référencée sans valeur par défaut est une erreur. Dans `QUICK_MQTT_DIAGNOSIS.sh`, le test `if [ $? -eq 0 ]` après `mosquitto_sub` reste valide (code de retour, pas une variable). Dans `DEPLOY.sh`, les `|| true` en aval des `test_*` protègent déjà contre les échecs non fatals.

**Vérification** (pour chacun des trois) :
```bash
bash -n DEPLOY.sh
bash -n QUICK_MQTT_DIAGNOSIS.sh
bash -n debug.sh
```
Chaque commande doit sortir sans message (syntaxe OK).

---

### Commit de la Phase 5
```bash
git add -A
git commit -m "chore(scripts): enable set -euo pipefail across deploy and diagnostic scripts"
```

---

## 7. PHASE 6 — Vérification finale et push

### Commandes à exécuter dans l'ordre

```bash
cargo fmt --all
cargo fmt --all -- --check
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
git log --oneline -10
git branch --show-current    # doit afficher: claude/code-review-improvement-plan-AVIPU
git push -u origin claude/code-review-improvement-plan-AVIPU
```

Si `cargo clippy -D warnings` échoue sur du code **non modifié** par ce plan, ajouter à la fin du dernier commit un `#[allow(clippy::xxxx)]` local **uniquement sur les lignes existantes en échec** et consigner en PR, **ne pas** refactorer.

---

## 8. Critères de succès

- Tous les `cargo` (fmt, check, clippy, test) passent.
- `bash -n` passe sur les trois scripts.
- Aucun fichier listé dans la liste noire (§0) n'apparaît dans `git diff origin/main...HEAD --stat`.
- Aucun topic MQTT, nom de service D-Bus, ou instance numérique n'apparaît dans le diff (vérifier :
  `git diff origin/main...HEAD | grep -E 'santuario/|com.victronenergy|battery.mqtt_|pvinverter.mqtt_|heatpump.mqtt_'` → **vide**).
- 6 commits au total, noms exacts ceux listés dans les phases.
- Push effectué sur `claude/code-review-improvement-plan-AVIPU`.

---

## 9. Rollback

En cas d'échec détecté après push :
```bash
git reset --hard origin/main
git push -f -u origin claude/code-review-improvement-plan-AVIPU
```
(autorisé uniquement sur cette branche Claude dédiée, **jamais** sur `main`).

---

## 10. Journal à remplir par Haiku pendant l'exécution

| Tâche | État | Note |
|-------|------|------|
| T1 — unused import | ☐ | |
| T11 — Makefile branch | ☐ | |
| T12 — stale TODO | ☐ | |
| T2 — threshold consts | ☐ | |
| T3 — default str consts | ☐ | |
| T4 — parse helper | ☐ | |
| T5 — warn fallbacks | ☐ | (intégré à T4) |
| T6 — influx validate | ☐ | |
| T7 — mqtt log | ☐ | |
| T8 — clone removal | ☐ | |
| T9 — unit tests | ☐ | |
| T10 — bash hardening | ☐ | |
| Vérif finale + push | ☐ | |

Cocher (☒) uniquement après vérification locale verte.
