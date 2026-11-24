# Architecture — Daly-BMS-Rust

> **Document maître.** Vue d'ensemble du système ESS Santuario (BMS Daly, compteurs
> ET112, irradiance, ATS, Venus OS) et **index de toute la documentation** consolidée.
> Chaque domaine renvoie vers un document détaillé dans `docs/`.
>
> Stack 100 % **Rust** (workspace multi-crates), dashboard **SSR** (Askama + ECharts, sans npm),
> broker **MQTT natif systemd**, TSDB **redb** embarquée. Déploiement = binaires statiques, sans Docker.
>
> Source de vérité opérationnelle quotidienne : [`CLAUDE.md`](../CLAUDE.md) (racine, mémoire projet).
> Dernière consolidation : 2026-06-07.

## Table des matières

- [1. Index de la documentation](#1-index-de-la-documentation)
- [2. Vue d'ensemble de l'infrastructure](#2-vue-densemble-de-linfrastructure)
- [3. Rôle de chaque service](#3-rôle-de-chaque-service)
- [4. Flux MQTT par domaine](#4-flux-mqtt-par-domaine)
- [5. Flux de données détaillé](#5-flux-de-données-détaillé)
- [6. Workspace Rust (crates)](#6-workspace-rust-crates)
- [7. Inventaire matériel (résumé)](#7-inventaire-matériel-résumé)
- [8. Estimation mémoire](#8-estimation-mémoire)
- [9. Roadmap](#9-roadmap)

---

## 1. Index de la documentation

La documentation est organisée **par application** (un binaire = un document) et **par domaine
transverse**. Tous les documents vivent dans `docs/`.

### Applications (binaires du workspace)

| Document | Hôte | Contenu |
|---|---|---|
| [app-daly-bms-server.md](./app-daly-bms-server.md) | Pi5 (:8080) | Serveur principal : bus RS485, protocole Daly UART, `AppState`/ring buffer, bridges, **API REST/WebSocket complète**, dashboard SSR (Askama + ECharts), Tasmota/Shelly. |
| [app-energy-manager.md](./app-energy-manager.md) | Pi5 (:8081) | Automatisation énergie (remplace Node-RED) : 12 modules `logic/`, décisions métier en **Rust pur** (`rules.rs`), clients HTTP Open-Meteo + LG ThinQ, MQTT, WebSocket live, persistance. |
| [app-dbus-mqtt-venus.md](./app-dbus-mqtt-venus.md) | GX Cerbo GX MK3 (aarch64 Cortex-A53) | Bridge MQTT → D-Bus Venus OS (zbus pur Rust) : services `battery`/`pvinverter`/`heatpump`/`temperature`/`switch`/`meteo`/`grid`/`platform`, intégration d'un nouveau device, déploiement sur le GX (Cortex-A53). |

### Domaines transverses

| Document | Contenu |
|---|---|
| [deploiement-exploitation.md](./deploiement-exploitation.md) | Build (cibles Makefile), workflow de déploiement Pi5 + GX, services systemd, logs/rétention, restauration git, conventions Git. |
| [metriques-redb-architecture.md](./metriques-redb-architecture.md) | Moteur TSDB **redb** : schéma, encodage, write/read path, tiering & rétention. Annexe historique : migration VictoriaMetrics → redb. |
| [metriques-promql-reference.md](./metriques-promql-reference.md) | **Catalogue des métriques** (labels hex), requêtes PromQL par appareil, roadmap d'implémentation, audit de conformité PromQL. |
| [grafana-dashboards.md](./grafana-dashboards.md) | Grafana : installation, datasource (UID `daly-metrics`), provisioning, **22 dashboards**, monitoring PV. |
| [mqtt-mosquitto.md](./mqtt-mosquitto.md) | Architecture MQTT : Mosquitto natif systemd, topics `santuario/*`, bridge `pi5-nanopi`, anti-boucle, migration Docker → natif. |
| [alertes.md](./alertes.md) | **AlertEngine** natif Rust : règles + hysteresis, persistance SQLite, notifications Telegram/SMTP, API alertes. |
| [integration-materiel.md](./integration-materiel.md) | Inventaire RS485/D-Bus, **ajout d'un BMS Daly**, ATS CHINT, ET112, PRALRAN, Tasmota/Shelly. |
| [integration-toshiba-shorai-esphome.md](./integration-toshiba-shorai-esphome.md) | **Plan d'intégration Toshiba SHORAI EDGE** (×3) : ESP32 + ESPHome `toshiba_suzumi` sur CN22 → MQTT Mosquitto → module EM `logic/toshiba_ac`. BOM, câblage, phases. |
| [toshiba-suzumi-rs-plan.md](./toshiba-suzumi-rs-plan.md) | **Projet Toshiba — firmware Rust ESP32 + décisions d'intégration** (protocole SUZUMI vérifié, §18 décisions FP2/HomeKit/Homie/HomeSpan/Matter, reprise de session §0). Lourd → détail opérationnel déporté ci-dessous. |
| [toshiba-bridges.md](./toshiba-bridges.md) | **Ponts & crates Toshiba/FP2 — référence opérationnelle** : contrat MQTT, carte des composants **A–E**, pipeline présence, HomeKit **D** vs Matter **E**, commandes/tests. |
| [diagnostic-depannage.md](./diagnostic-depannage.md) | Dépannage transverse, erreurs `Checksum` BMS (`scripts/diag-bms-trames.sh`, §2.1), `netdiag` réseau, debug onduleur/SmartShunt, investigation memory-leak (en cours). |
| [TRANSITION-refonte-visualisation.md](./TRANSITION-refonte-visualisation.md) | **Reprise de session** : spécification de la refonte de la page Visualisation (schéma de l'utilisateur), état de l'installation, points ouverts, procédures d'exploitation. |
| [renommer-appareils.md](./renommer-appareils.md) | **Renommer un appareil** : renommage depuis la console Venus (conservé dans `/data/daly-bms/custom-names.json`), noms par défaut des configs, ajout d'une prise Tongou. |
| [incident-2026-07-09-reboot-redb.md](./incident-2026-07-09-reboot-redb.md) | **Post-mortem** : boucle de crash au reboot (récupération redb > `TimeoutStartSec`). Diagnostic, cause racine, 3 correctifs (timeout `infinity`, ouverture redb en tâche de fond, sauvegarde quotidienne), récupération d'historique, prévention (UPS). |

### Hors `docs/`

- [`CLAUDE.md`](../CLAUDE.md) (racine) — mémoire projet : commandes rapides, inventaire RS485/D-Bus, règles de travail. Chargé à chaque session.
- [`nanoPi/README.md`](../nanoPi/README.md) — README de composant pour la configuration Venus OS du GX.
- [`firmware/toshiba-suzumi-rs/README.md`](../firmware/toshiba-suzumi-rs/README.md) — crate détaché : firmware ESP32 (protocole SUZUMI). Composant **A**.
- [`bridge/aqara-fp2-mqtt/README.md`](../bridge/aqara-fp2-mqtt/README.md) — pont FP2 → MQTT présence (aiohomekit). Composant **C**.
- [`bridge/mqtt-homekit-occupancy/README.md`](../bridge/mqtt-homekit-occupancy/README.md) — pont MQTT → HomeKit (HAP-python). Composant **D**.
- [`bridge/matter-toshiba-rs/README.md`](../bridge/matter-toshiba-rs/README.md) — crate détaché : bridge Matter (Rust, sans Node). Composant **E**.
- [`README.md`](../README.md) (racine) — page d'accueil du dépôt.

---

## 2. Vue d'ensemble de l'infrastructure

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Raspberry Pi 5 CM  (Master)                      │
│                                                                     │
│  RS485 /dev/ttyUSB0 ── BMS-360Ah  (0x01)                            │
│                     ── BMS-320Ah  (0x02)                            │
│                     ── BMS-620Ah  (0x03)                            │
│                     ── PRALRAN     (0x05) — Irradiance / Météo      │
│                     ── ET112 PV   (0x07) — Micro-Onduleurs          │
│                     ── ET112 House (0x08) — Réseau AC-1 (maison)    │
│                     ── ET112 AC-2  (0x09) — Réseau AC-2             │
│                     ── ATS CHINT   (RS485) — bascule AC-1 / AC-2    │
│                                  │                                  │
│                                  ▼                                  │
│                          daly-bms-server :8080                      │
│                  (REST/WS + Dashboard SSR + AlertEngine             │
│                   + metrics-store redb embarqué + shim PromQL)      │
│                                  │                                  │
│  API LG ThinQ / Open-Meteo ─► energy-manager :8081                  │
│                                  │                                  │
│                          Mosquitto :1883 (+ :9001 WS)               │
│                                  │                                  │
│              metrics-store (redb : /mnt/nvme/daly-bms/metrics.redb) │
│                                  │                                  │
│                          grafana-server :3000                       │
└────────────────────────────────┬────────────────────────────────────┘
                                 │ MQTT (santuario/*/venus)
                                 │ bridge unique pi5-nanopi
                                 ▼
┌────────────────────────────────────────────────────────────────────┐
│                  GX  (Venus OS — D-Bus bridge)                     │
│   dbus-mqtt-venus  (binaire unique, zbus pur Rust, ~5–8 Mo RAM)    │
│      souscrit santuario/{bms,pvinverter,heatpump,heat,switch,…}    │
│      enregistre com.victronenergy.{battery,pvinverter,heatpump,    │
│                 temperature,switch,meteo,grid}.*                   │
│                │                                                   │
│                ▼  systemcalc-py ── VRM Portal ── Venus GUI         │
│                   hub4-control (DVCC charge/discharge)             │
└────────────────────────────────────────────────────────────────────┘
```

> **Pi5 = master** : tous les capteurs RS485 et les API cloud y sont rattachés.
> Le **GX** — Victron **Cerbo GX MK3** (Venus OS v3.85, aarch64 Cortex-A53, `192.168.1.120`,
> portal VRM `d81d13b31a8b`) — est dédié à Venus OS et n'héberge que `dbus-mqtt-venus`.
> Le dossier du dépôt s'appelle `nanoPi/` et le bridge Mosquitto `pi5-nanopi-bridge` pour des
> raisons historiques ; détails, instances VRM et remplacement du GX : [`nanoPi/README.md`](../nanoPi/README.md).
> `energy-manager` a été migré du GX vers le Pi5 pour consolider l'infrastructure.

Détails : [intégration matériel](./integration-materiel.md) · [MQTT](./mqtt-mosquitto.md) · [bridge Venus](./app-dbus-mqtt-venus.md).

---

## 3. Rôle de chaque service

| Service | Hôte | Port | Rôle | Document |
|---------|------|------|------|----------|
| **daly-bms-server** | Pi5 | 8080 | Polling RS485, REST API, WebSocket, Dashboard SSR, AlertEngine, metrics-store redb (shim PromQL) | [app-daly-bms-server.md](./app-daly-bms-server.md) |
| **energy-manager** | Pi5 | 8081 | Automatisation énergie (solaire, DEYE, chauffe-eau, charge, météo) | [app-energy-manager.md](./app-energy-manager.md) |
| **Mosquitto** | Pi5 | 1883 / 9001 | Broker MQTT natif systemd, bridge `pi5-nanopi` | [mqtt-mosquitto.md](./mqtt-mosquitto.md) |
| **metrics-store (redb)** | Pi5 | (embarqué :8080) | TSDB pure-Rust, tiering raw 30 j / hourly 365 j / daily 5 ans | [metriques-redb-architecture.md](./metriques-redb-architecture.md) |
| **grafana-server** | Pi5 | 3000 | Visualisation (datasource PromQL → :8080), 22 dashboards | [grafana-dashboards.md](./grafana-dashboards.md) |
| **dbus-mqtt-venus** | GX | — | Bridge MQTT → D-Bus Venus OS (zbus) | [app-dbus-mqtt-venus.md](./app-dbus-mqtt-venus.md) |

---

## 4. Flux MQTT par domaine

Préfixe commun : **`santuario/`**. Le bridge Mosquitto `pi5-nanopi` relaie ces topics vers le GX.

| Topic MQTT | Source (Pi5) | Cible D-Bus Venus |
|---|---|---|
| `bms/{n}/venus` | daly-bms-server | `com.victronenergy.battery.mqtt_{n}` |
| `pvinverter/{n}/venus` | daly-bms-server (ET112) | `com.victronenergy.pvinverter.mqtt_{n}` |
| `grid/{n}/venus` | daly-bms-server (ET112) | `com.victronenergy.grid.mqtt_{n}` / `acload.mqtt_{n}` |
| `heat/{n}/venus` | energy-manager (LG ThinQ) | `com.victronenergy.temperature.mqtt_{n}` |
| `switch/{n}/venus` | daly-bms-server (ATS / Tongou) | `com.victronenergy.switch.mqtt_{n}` |
| `meteo/venus` | daly-bms-server (PRALRAN) | `com.victronenergy.meteo` |

Détail complet (table topics ↔ services, validation anti-boucle) : [mqtt-mosquitto.md](./mqtt-mosquitto.md).

---

## 5. Flux de données détaillé

```
BMS UART ─► daly_bms_core::poll_loop ─► AppState::on_snapshot
                                          ├─► ring_buffer (3600 snaps/BMS)
                                          ├─► broadcast tokio ─► WebSocket /ws/bms/*
                                          ├─► MqttBridge (rumqttc) ─► Mosquitto ─► dbus-mqtt-venus
                                          ├─► metrics-store (redb)
                                          └─► AlertEngine (SQLite)

ET112 Modbus RTU (0x07/08/09) ─► et112::poll_loop ─► MQTT ─► pvinverter/grid services
PRALRAN RS485 (0x05)          ─► irradiance::poll_loop ─► MQTT ─► meteo service
ATS CHINT RS485               ─► ats (lecture + commandes) ─► MQTT ─► switch service
LG ThinQ API (PAC)            ─► energy-manager::http_clients::lg_thinq ─► MQTT ─► temperature service
```

Détails par appareil : [app-daly-bms-server.md](./app-daly-bms-server.md) · [integration-materiel.md](./integration-materiel.md).

---

## 6. Workspace Rust (crates)

| Crate / Binaire | Hôte | Rôle |
|---|---|---|
| `rs485-bus` | — | Lib : bus RS485 partagé (mutex tokio) + Modbus RTU pur Rust |
| `daly-bms-core` | — | Lib : protocole UART Daly, parsing trames, types (`BmsSnapshot`), polling |
| `daly-bms-server` | Pi5 | Binaire principal : API Axum (REST + WS) + Dashboard SSR + bridges (MQTT, redb, AlertEngine) |
| `energy-manager` | Pi5 | Binaire automatisation énergie : décisions en Rust pur (`logic/<module>/rules.rs`) + clients HTTP |
| `metrics-store` | Pi5 | Lib TSDB redb (embarquée dans daly-bms-server) + shim PromQL |
| `dbus-mqtt-venus` | GX Cerbo GX MK3 (aarch64 Cortex-A53) | Binaire MQTT → D-Bus Venus OS (zbus pur Rust) |

Structure du dépôt et arborescence `src/` détaillées dans chaque document d'application.

---

## 7. Inventaire matériel (résumé)

Bus `/dev/ttyUSB0` (RS485), instances Venus OS :

| Addr | Appareil | Type D-Bus | Topic MQTT | Instance |
|------|----------|-----------|------------|----------|
| 0x01 | BMS-360Ah | `battery.mqtt_1` | `bms/1/venus` | 151 |
| 0x02 | BMS-320Ah | `battery.mqtt_2` | `bms/2/venus` | 152 |
| 0x03 | BMS-620Ah | `battery.mqtt_3` | `bms/3/venus` | 153 |
| 0x05 | PRALRAN irradiance | `meteo` | `irradiance/raw` | 40 |
| 0x07 | ET112 Micro-Onduleurs | `pvinverter.mqtt_7` | `pvinverter/7/venus` | 32 |
| 0x08 | ET112 Maison (AC-1) | `acload.mqtt_8` | `grid/8/venus` | 30 |
| 0x09 | ET112 AC-2 | `acload.mqtt_9` | `grid/9/venus` | 31 |

Appareils Victron du GX (instances VRM) : MultiPlus-II `vebus` 276, MPPT 273 / 288–292,
SmartShunt 274, GX IO extender 50, Shelly Pro 2PM 50/51 — tableau complet dans
[`nanoPi/README.md`](../nanoPi/README.md).

### Réseaux AC (installation 100 % hors réseau)

Deux réseaux AC partagent les mêmes batteries (BMS 320 / 360 / 620 Ah) :

```
MultiPlus-II ── AC-1 ──► ATS entrée I ─┐
                                       ├─► sortie ATS ─► ET112-Maison (0x08) ─► groupe I :
Inverter RS Smart ── AC-2 ─► ET112-AC2 (0x09) ─┬─► ATS entrée II   General, AC-4, LG (chauffe-eau),
                                               │                    Climatisation
                                               └─► groupe II : DEYE (Tongou 0616E4 ─► ET112 0x07 ─►
                                                   micro-onduleurs DEYE + Shelly Pro 2PM) et 5 Tongou
```

- L'ATS bascule le groupe I sur l'Inverter RS Smart pendant une maintenance ou un arrêt du
  MultiPlus. L'Inverter RS ne charge pas les batteries ; le surplus des DEYE (couplage AC sur
  sa sortie) les recharge par décalage de fréquence automatique.
- En attendant l'installation de l'Inverter RS Smart, AC-2 est alimenté par AC-1.
- Tongou : 4 sur AC-1, 6 sur AC-2 — `[[tasmota.devices]]` (`network = "AC-1" / "AC-2"`) dans
  `Config.toml`, `[[switches]]` dans `nanoPi/config-nanopi.toml`.
- Aucun compteur `grid` : ET112 0x08 et 0x09 sont des `acload` ; ESS sans compteur réseau
  (`RunWithoutGridMeter = 1`).

Inventaire complet (switchs Tongou, ATS, températures, onduleur Victron direct) et procédures :
[integration-materiel.md](./integration-materiel.md). Référence canonique : [`CLAUDE.md` §5](../CLAUDE.md).

---

## 8. Estimation mémoire

### Pi5 (master — tout natif systemd)

| Service | RAM minimale | RAM confortable |
|---|---|---|
| daly-bms-server (redb embarqué) | ~25 MB | ~50 MB |
| energy-manager | ~30 MB | ~100 MB (`LimitMemoryMax=100M`) |
| mosquitto-broker | ~8 MB | ~15 MB |
| metrics-store (redb) | embarqué | ~0 Mo RSS additionnel (gain ~135 Mo vs ex-VictoriaMetrics) |
| OS Raspberry Pi OS Lite | ~150 MB | ~200 MB |
| **TOTAL** | **~413 MB** | **~765 MB** |

### GX (Venus OS)

| Service | RAM | Notes |
|---|---|---|
| dbus-mqtt-venus | ~5–8 MB | Binaire statique musl, zéro dépendance système |

---

## 9. Roadmap

État synthétique (détails dans l'historique git et les documents de domaine).

- **Phase 0 — Fondations Rust** ✅ workspace, types `BmsSnapshot`, protocole UART, API Axum, ring buffer/broadcast, bridges.
- **Phase 1 — Infrastructure & Intégration** ✅ Mosquitto natif, auto-détection série, dashboard SSR, Venus OS (MQTT → D-Bus).
- **Phase 2 — Production RPi5** ✅ BMS confirmés en base, rotation logs + rétention redb. ⏳ validation écritures (MOS/SOC/reset) sur hardware, tests 24 h.
- **Phase 3 — Venus OS natif Rust** ✅ `dbus-mqtt-venus` (zbus), services Victron, watchdog MQTT, keepalive D-Bus.
- **Phase 4 — Migration & Consolidation** ✅ renommage crate, migration energy-manager GX → Pi5. ⏳ nettoyage GX, validation 24 h.
- **Phase 5 — Irradiance & Météo RS485** ✅ PRALRAN intégré.
- **Phase 6 — PAC Chauffe-eau LG** ✅ LG ThinQ.
- **Phase 7 — Climatisation LG** 🔜 à étudier.
- **Phase 8 — ATS RS485** ✅ ATS CHINT intégré (lecture + commandes).
- **Vision long terme** 🔭 trait `DevicePoller` partagé, config hot-reload, dashboard SSR unifié, alertes corrélées, export Home Assistant.

---

*Référence protocole : Daly UART/485 Communications Protocol V1.21.
Runtime : tokio-serial — Axum — rumqttc — zbus — redb.*
