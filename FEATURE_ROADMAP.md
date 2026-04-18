# Roadmap fonctionnelle — Daly-BMS-Rust

> 6 améliorations fonctionnelles cohérentes avec l'architecture existante
> (Pi5 + NanoPi + MQTT + InfluxDB + Venus OS).
> Chaque proposition est auto-suffisante et peut être implémentée indépendamment.

---

## Vue d'ensemble

| # | Fonctionnalité | Valeur | Effort | Risque | Dépendances |
|---|----------------|--------|--------|--------|-------------|
| F1 | Rapport énergétique quotidien par email | ★★★★☆ | 4 h | LOW | `lettre` (déjà présent), InfluxDB |
| F2 | SOH & compteur de cycles BMS | ★★★★★ | 6 h | MEDIUM | SQLite (déjà présent) |
| F3 | Prévision solaire Open-Meteo | ★★★★☆ | 3 h | LOW | `reqwest` (déjà présent) |
| F4 | Export CSV historique par appareil/période | ★★★☆☆ | 3 h | LOW | InfluxDB Flux |
| F5 | Métriques santé bus RS485 par appareil | ★★★★☆ | 4 h | LOW | aucune |
| F6 | PWA mobile installable | ★★★☆☆ | 3 h | LOW | aucune |

**Total roadmap** : ~23 h de développement réparties.

---

## F1 — Rapport énergétique quotidien par email

### Valeur utilisateur

Recevoir chaque soir à 23h55 un email HTML résumant la journée solaire :
- Production solaire totale (kWh)
- Consommation totale (kWh)
- Taux d'autoconsommation (%)
- SOC batteries min/max/fin de journée
- Nombre d'alertes déclenchées
- Mini-graphe ASCII ou inline SVG de la production

### Cohérence avec l'existant

- `AlertsConfig` contient déjà `smtp_host`, `smtp_port`, `smtp_from`, `smtp_to` — **réutilisés tels quels**.
- La dépendance `lettre` est déjà dans le workspace.
- Les flux 24 h sont déjà dans InfluxDB bucket `daly_bms`.

### Plan de mise en œuvre

1. **Config** — ajouter dans `AppConfig` :
   ```toml
   [daily_report]
   enabled          = true
   send_time        = "23:55"      # hh:mm locale Pi5
   timezone         = "Europe/Paris"
   include_graph    = true         # SVG 400x100 inline dans l'email
   ```

2. **Module** — créer `crates/daly-bms-server/src/bridges/daily_report.rs` :
   - Fonction `run_daily_report(state, cfg)` lancée depuis `main.rs` en `tokio::spawn`.
   - Calcul du prochain déclenchement via `chrono::Local::now()` + parsing `send_time`.
   - `tokio::time::sleep_until` puis boucle de 24 h.
   - Query InfluxDB Flux (sur les 24 h écoulées) pour chaque KPI.
   - Rendu via template Askama `daily_report.html` (HTML email responsive).
   - Envoi via `lettre::AsyncSmtpTransport` (déjà utilisé par `alerts.rs`).

3. **Template** — `crates/daly-bms-server/templates/daily_report.html` (Askama) :
   - Table HTML inline-styled (compatibilité Gmail/Outlook).
   - SVG inline pour la courbe de puissance solaire.

4. **API manuelle** — `POST /api/v1/reports/daily?date=YYYY-MM-DD` pour relancer un rapport passé (test/replay).

### Livrable & vérification

- `cargo test -p daly-bms-server daily_report::tests` → au moins `test_parse_send_time`, `test_kpi_computation`.
- Test d'intégration : `curl -X POST /api/v1/reports/daily` → email reçu en <10 s.
- `journalctl -u daly-bms` affiche `daily_report envoyé (KPIs...)` à 23h55.

---

## F2 — SOH & compteur de cycles BMS

### Valeur utilisateur

- **SOH (State Of Health)** en % : indicateur de dégradation (référence = capacité nominale).
- **Nombre de cycles équivalents** (Ah cumulés ÷ 2 × Ah nominal).
- **Date du dernier plein** (SOC=100%) + temps depuis.
- Affichage dans dashboard BMS + publication MQTT + InfluxDB.

### Cohérence avec l'existant

- Les données brutes (courant, SOC, Vcell) circulent déjà.
- `rusqlite` est déjà dans les dépendances (utilisé par alertes).
- Topic MQTT : extension naturelle de `santuario/bms/{n}/venus`.

### Plan de mise en œuvre

1. **Schéma SQLite** — nouvelle table dans `/var/lib/daly-bms/history.db` :
   ```sql
   CREATE TABLE bms_health (
       bms_address    INTEGER PRIMARY KEY,
       ah_integrated  REAL NOT NULL DEFAULT 0.0,   -- Σ |I|·dt / 3600
       last_full_ts   INTEGER,                     -- epoch seconds
       last_empty_ts  INTEGER,
       last_full_soc  REAL,                        -- SOC vu lors du dernier plein
       full_cycles    REAL NOT NULL DEFAULT 0.0,   -- ah_integrated / (2*Ah_nom)
       updated_at     INTEGER NOT NULL
   );
   ```

2. **Module** — créer `crates/daly-bms-server/src/health.rs` :
   - `HealthTracker { db: Connection, nominal_ah: f32 }`.
   - Méthode `on_snapshot(&mut self, snap: &BmsSnapshot)` appelée depuis la loop de polling :
     - Intègre `|current_a| * dt` (dt = delta avec snapshot précédent).
     - Détecte fronts SOC : 100% → `last_full_ts`, <5% → `last_empty_ts`.
     - Persiste toutes les N minutes (throttle pour ne pas marteler le disque).
   - Méthode `compute_soh(&self, bms_addr: u8) -> Option<f32>` :
     - Entre deux "pleins" consécutifs, calcule la capacité réellement extraite.
     - SOH = capacité_extraite / capacité_nominale.

3. **Config** — par BMS dans `Config.toml` :
   ```toml
   [[bms]]
   address     = "0x01"
   nominal_ah  = 360      # NEW : capacité nominale nameplate
   ```
   (Champ optionnel — si absent, SOH non calculé, mais cycle counter reste actif.)

4. **API** :
   - `GET /api/v1/bms/{id}/health` → `{ "soh_percent": 97.2, "cycles": 123.4, "last_full_ts": 1713... , "days_since_full": 2 }`.
   - Inclus dans le `snapshot` WebSocket.

5. **MQTT** — nouveau sous-topic `santuario/bms/{n}/health` (JSON).

6. **Dashboard** — ajouter 3 KPIs dans `templates/bms_detail.html` : SOH%, cycles, jours depuis dernier plein.

### Risques & mitigations

- **Persistence** : penser à fsync + throttle (max 1 write / 60 s).
- **Démarrage à froid** : `ah_integrated=0` la première fois → SOH n'apparaît qu'après le premier cycle complet (comportement attendu, à documenter).
- **Config modifiée** : si `nominal_ah` change, ne pas reset l'historique — simplement recalculer avec la nouvelle valeur.

---

## F3 — Prévision solaire Open-Meteo

### Valeur utilisateur

- Prévision de production solaire sur les 24 h à venir (basée sur rayonnement GHI + azimut + inclinaison panneaux).
- Affichage side-by-side dans dashboard : prévu (ligne pointillée) vs réalisé (ligne pleine).
- Permet à Node-RED / Rules Engine d'anticiper (ex : "s'il pleut demain, forcer charge batterie aujourd'hui").

### Cohérence avec l'existant

- API Open-Meteo gratuite, pas de clé, pas de quota strict — cohérent avec l'esprit du projet (pas de dépendance payante).
- `reqwest` + `rustls-tls` déjà présents (utilisés pour Telegram).
- MQTT publie déjà des prévisions dans `santuario/meteo/*` — extension logique.

### Plan de mise en œuvre

1. **Config** :
   ```toml
   [forecast]
   enabled          = true
   latitude         = 43.5
   longitude        = 1.4
   azimuth          = 180       # 0=N, 180=S
   tilt             = 30        # inclinaison panneaux
   peak_power_w     = 5000      # puissance crête totale des panneaux
   refresh_min      = 60        # appel API toutes les heures
   ```

2. **Module** — `crates/daly-bms-server/src/bridges/forecast.rs` :
   - Query : `https://api.open-meteo.com/v1/forecast?latitude=...&hourly=shortwave_radiation,temperature_2m`.
   - Calcul de P(t) = GHI(t) × surface_effective × rendement simplifié.
   - Cache en mémoire (48 slots horaires).

3. **API** : `GET /api/v1/forecast/solar?horizon_h=24` → tableau `[{ts, power_w, ghi_wm2}, …]`.

4. **MQTT** : `santuario/forecast/solar` (JSON, publié après chaque refresh).

5. **Intégration dashboard** : courbe SVG superposée dans `overview.html`.

### Vérification

- `curl /api/v1/forecast/solar | jq '.[0]'` → objet non vide dans les 10 s après démarrage.
- Tolérance : si Open-Meteo down → log `warn!`, pas d'échec du service (gracefull degradation).

---

## F4 — Export CSV historique

### Valeur utilisateur

- Télécharger en 1 clic l'historique d'un appareil pour une période.
- Format standard (CSV) ouvert par Excel, LibreOffice, Python.
- Analyse hors ligne, archivage.

### Cohérence avec l'existant

- InfluxDB contient déjà toutes les séries temporelles.
- Extension naturelle des endpoints `/api/v1/{device}/history`.

### Plan de mise en œuvre

1. **Endpoints** :
   ```
   GET /api/v1/bms/{id}/export.csv?from=<rfc3339>&to=<rfc3339>&fields=soc,voltage,current
   GET /api/v1/et112/{addr}/export.csv?from=...&to=...
   GET /api/v1/irradiance/export.csv?from=...&to=...
   ```
   Réponse : `Content-Type: text/csv`, `Content-Disposition: attachment; filename="..."`.

2. **Module** — `crates/daly-bms-server/src/api/export.rs` :
   - Construit la requête Flux à partir des params.
   - Stream ligne-à-ligne (pas de buffer complet en mémoire).
   - Limite hard-codée : 31 jours max par requête (protection DoS).

3. **UI** — bouton "Export CSV" sur chaque page dashboard avec sélecteur de plage `<input type="datetime-local">`.

4. **Tests** :
   - `test_export_csv_header_correct`.
   - `test_export_csv_respects_date_range`.
   - `test_export_csv_max_31_days`.

---

## F5 — Métriques santé bus RS485 par appareil

### Valeur utilisateur

- Visibilité en temps réel sur la fiabilité du bus : quel appareil timeout ? quel appareil génère des erreurs CRC ?
- Permet de détecter précocement : câble défaillant, adresse en conflit, équipement mourant.
- Exposition Prometheus-like pour intégration Grafana.

### Cohérence avec l'existant

- La boucle `poll_loop` gère déjà timeouts et CRC — il suffit de compter.
- `monitor.rs` existe déjà pour l'état global, extension naturelle.

### Plan de mise en œuvre

1. **Structure** — dans `crates/daly-bms-core/src/poll.rs` :
   ```rust
   pub struct DeviceBusStats {
       pub address:           u8,
       pub successful_polls:  u64,
       pub timeout_count:     u64,
       pub crc_error_count:   u64,
       pub last_success_ts:   Option<i64>,
       pub last_error_ts:     Option<i64>,
       pub last_error_kind:   Option<String>,
   }
   ```
   Collection exposée via `Arc<RwLock<HashMap<u8, DeviceBusStats>>>` partagée dans `AppState`.

2. **Instrumentation** — incrémenter les compteurs dans les points de sortie existants (`poll_once` etc.), **sans** changer la logique métier.

3. **API** :
   - `GET /api/v1/system/rs485-health` → liste par adresse avec pourcentage de succès.
   - `GET /api/v1/system/rs485-health/metrics` → format Prometheus textfile (pour scrape Grafana).

4. **Dashboard** — nouveau bandeau `/dashboard/system` montrant tableau couleur (vert >99%, orange >90%, rouge ≤90%).

5. **InfluxDB** — publier 1×/min les stats dans measurement `rs485_health` (tags: address ; fields: success_rate, timeouts).

---

## F6 — PWA mobile installable

### Valeur utilisateur

- Accéder au dashboard depuis le téléphone comme une vraie app (icône home screen, plein écran).
- Cache hors ligne des assets statiques (CSS, JS, images).
- Notifications push (Phase 2 optionnelle).

### Cohérence avec l'existant

- Le dashboard est déjà servi en HTTP via Axum + templates Askama.
- Aucune dépendance externe à ajouter.

### Plan de mise en œuvre

1. **Manifest** — `crates/daly-bms-server/static/manifest.webmanifest` :
   ```json
   {
     "name": "Daly BMS Santuario",
     "short_name": "DalyBMS",
     "start_url": "/dashboard",
     "display": "standalone",
     "background_color": "#0f172a",
     "theme_color": "#16a34a",
     "icons": [
       { "src": "/static/icon-192.png", "sizes": "192x192", "type": "image/png" },
       { "src": "/static/icon-512.png", "sizes": "512x512", "type": "image/png" }
     ]
   }
   ```

2. **Service Worker** — `static/sw.js` :
   - Cache-first pour `/static/*`.
   - Network-first pour `/api/v1/*` (fallback cache si hors ligne).
   - Versionning du cache (invalidation au changement de binaire).

3. **Template** — ajout dans `templates/_base.html` :
   ```html
   <link rel="manifest" href="/static/manifest.webmanifest">
   <meta name="theme-color" content="#16a34a">
   <link rel="apple-touch-icon" href="/static/icon-192.png">
   <script>
     if ('serviceWorker' in navigator) {
       navigator.serviceWorker.register('/static/sw.js');
     }
   </script>
   ```

4. **Icônes** — générer 2 PNG (192, 512) depuis un SVG source (logo batterie stylisé).

5. **Route static** — s'assurer que `tower-http::services::ServeDir` (déjà utilisé ?) sert `/static/`.

### Vérification

- Chrome DevTools → Lighthouse → score PWA > 90.
- Test Android : "Ajouter à l'écran d'accueil" doit apparaître.

---

## Ordre d'implémentation recommandé

1. **F5** — Métriques santé RS485 (base d'observabilité, aucune dépendance).
2. **F4** — Export CSV (rapide, haute valeur quotidienne).
3. **F2** — SOH & cycles (valeur durable, prérequis pour maintenance préventive).
4. **F3** — Prévision solaire (prérequis utile pour F1 et pour automatisations Node-RED).
5. **F1** — Rapport quotidien (bénéficie des KPI de F2/F3/F5).
6. **F6** — PWA (finition UX, indépendant).

Chaque fonctionnalité fait l'objet d'une branche dédiée `feat/<F#>-<slug>`.

---

## Hors périmètre (décisions explicites)

- **Pas de framework frontend JS lourd** (React/Vue) — on reste sur Askama SSR + sprinkles JS.
- **Pas de cloud propriétaire** (AWS/Azure) — tout tourne en local Pi5.
- **Pas de remplacement de Node-RED** — Node-RED reste l'outil de composition visuelle.
- **Pas de télémétrie tierce** (Sentry, Datadog) — les logs `journalctl` + InfluxDB suffisent.
- **Pas d'authentification OAuth/OIDC** — réseau local privé, un simple API key (déjà présent) reste approprié.

---

## Checklist de validation par feature (à remplir à l'implémentation)

Pour chaque Fx :

- [ ] Branche Git `feat/F<n>-<slug>` créée depuis `main`.
- [ ] Config.toml : section ajoutée + documentée dans `Readme.md`.
- [ ] Tests unitaires (au moins 3) passent.
- [ ] `cargo clippy --all-targets -- -D warnings` propre.
- [ ] `make build-arm` OK.
- [ ] Dashboard : pas de régression visuelle sur `/dashboard`.
- [ ] Endpoint testé via `curl` avec cas nominal + cas erreur.
- [ ] Journal de session dans CLAUDE.md si info durable découverte.
- [ ] PR mergée dans `main` après revue humaine.
