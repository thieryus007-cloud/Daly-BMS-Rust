# Daly-BMS — Rust Edition

**Version Rust complète** — mise à jour 16 mars 2026
Remplacement total de la stack Python/FastAPI par **Rust** (workspace multi-crates : `daly-bms-core` + `daly-bms-server` + `daly-bms-cli` + `daly-bms-probe`).

> Dashboard intégré **SSR Rust** (Askama + ECharts) — aucun npm, aucun React.
> Infrastructure Docker **inchangée** (Mosquitto, InfluxDB, Grafana, Node-RED).
> Déploiement ultra-léger : **un seul binaire statique** (~12–18 Mo).
> Compatible **Windows** (testé) et **Linux/aarch64** (Raspberry Pi).

---

Raspberry Pi Compute Module 5 Wireless, 4GB RAM, 32GB eMMC
Raspberry Pi OS Lite (64-bit)

## Architecture globale

```
Pack A (0x28) ─┐
Pack B (0x29) ─┤── RS485/USB ── RPi CM5 ──[ daly-bms-server ]── Dashboard SSR (/dashboard)
               │                              (Axum natif)         WebSocket /ws/bms/stream
               │                                    │
               │                    ┌───────────────┼───────────────┐
               │                    ▼               ▼               ▼
               │               Mosquitto        InfluxDB        AlertEngine
               │               (MQTT)           (séries)        (SQLite)
               │                    │
               │      dbus-mqtt-battery-41/42 (Venus OS / NanoPi)
               │                    │
               │                    ▼
               │              D-Bus Venus OS
               │         (GUI / VRM / systemcalc)
               │
               └── [TRANSITION] dbus-canbattery.can0 (stoppé — remplacé par flux MQTT)
```

> **Architecture confirmée** : `dbus-mqtt-battery` fonctionne en mode **MQTT → D-Bus**
> (abonnement MQTT → service virtuel Venus OS). La connexion CAN directe est
> redondante une fois le RPi CM5 opérationnel.

> **Mode simulateur** : le serveur peut démarrer sans matériel (`--simulate`) pour
> tester la stack complète. **Validé sur Windows et Linux.**

### Workspace Rust

| Crate / Binaire        | Rôle |
|------------------------|------|
| `daly-bms-core`        | Protocole UART, parsing trames, types (`BmsSnapshot`), bus partagé, commandes lecture/écriture, polling |
| `daly-bms-server`      | API Axum (REST + WebSocket) + Dashboard SSR + ring buffer + simulateur + bridges (MQTT, InfluxDB, Alertes) |
| `daly-bms-cli`         | Outil CLI de diagnostic et contrôle RS485 |
| `daly-bms-probe`       | Outil diagnostic bas niveau — envoie des trames brutes, teste 3 variantes d'adressage |

### Flux de données

```
BMS UART  ──► daly_bms_core::poll_loop()   ← mode hardware
Simulateur ──► run_simulator()              ← mode --simulate (sans matériel)
                    │
                    ▼  on_snapshot(snap)
             AppState::on_snapshot()
              ┌──────┴──────────────────────┐
              ▼                             ▼
       ring_buffer                  broadcast (WebSocket)
       (3600 snaps/BMS)         ┌────────┼──────────┐
                                ▼        ▼           ▼
                           MqttBridge InfluxBridge AlertEngine
                           (rumqttc)  (influxdb2)  (rusqlite)
```

---

## Gains vs version Python

| Métrique            | Python/FastAPI | Rust/Axum | Gain |
|---------------------|----------------|-----------|------|
| RAM au repos        | 150–300 Mo     | 10–35 Mo  | ÷5–10 |
| CPU polling         | base           | ÷3 à ÷5   |       |
| Latence WebSocket   | base           | ÷5–10     |       |
| Taille binaire      | 150 Mo (venv)  | 12–18 Mo  | ÷10  |
| Démarrage           | ~3 s           | < 150 ms  | ÷20  |
| Sécurité mémoire    | GC Python      | Ownership Rust | Zéro race condition |

---

## Structure du dépôt

```
Daly-BMS-Rust/
├── Cargo.toml                 ← Workspace Rust (résolver v2)
├── Cargo.lock
├── Config.toml                ← Fichier de configuration exemple (TOML)
├── Makefile                   ← Commandes build/test/deploy/docker
├── Dockerfile                 ← Image Docker multi-stage (builder + runtime)
├── docker-compose.yml         ← Stack complète (serveur + infra)
├── docker-compose.infra.yml   ← Infra seule (Mosquitto, InfluxDB, Grafana)
├── .env                       ← Variables Docker (InfluxDB, Grafana)
├── .gitignore
│
├── crates/
│   ├── daly-bms-core/         ← Bibliothèque protocole
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── error.rs       ← DalyError, Result<T>
│   │       ├── types.rs       ← BmsSnapshot, Alarms, SystemData…
│   │       ├── protocol.rs    ← DataId, RequestFrame, checksum + 7 tests unitaires
│   │       ├── bus.rs         ← DalyPort (Mutex), DalyBusManager
│   │       ├── commands.rs    ← get_pack_status, get_cell_voltages…
│   │       ├── write.rs       ← set_charge_mos, set_soc, reset_bms
│   │       └── poll.rs        ← poll_loop + backoff + retry
│   │
│   ├── daly-bms-server/       ← Serveur principal
│   │   └── src/
│   │       ├── main.rs        ← Entrypoint, CLI flags (--simulate, --port, --bms…)
│   │       ├── config.rs      ← AppConfig (TOML → struct), per-BMS config
│   │       ├── state.rs       ← AppState, ring buffer, broadcast channel
│   │       ├── simulator.rs   ← Simulateur BMS (physique LiFePO4, sans matériel)
│   │       ├── autodetect.rs  ← Détection automatique port série + adresses BMS
│   │       ├── api/
│   │       │   ├── mod.rs     ← Router Axum (toutes les routes)
│   │       │   ├── system.rs  ← GET /api/v1/system/*
│   │       │   └── bms.rs     ← GET/POST /api/v1/bms/*, WebSocket
│   │       ├── bridges/
│   │       │   ├── mod.rs
│   │       │   ├── mqtt.rs    ← rumqttc, topics, Venus OS payload
│   │       │   ├── influx.rs  ← influxdb2-client, batch write
│   │       │   └── alerts.rs  ← AlertEngine, SQLite, Telegram/SMTP
│   │       └── dashboard/
│   │           ├── mod.rs     ← Routes /dashboard, templates Askama
│   │           └── charts.rs  ← Génération JSON ECharts (boxplot, séries…)
│   │
│   ├── daly-bms-cli/          ← Outil CLI
│   │   └── src/main.rs        ← clap, sous-commandes
│   │
│   └── daly-bms-probe/        ← Outil diagnostic bas niveau
│       └── src/main.rs        ← Trames brutes, test 3 variantes d'adressage
│
├── contrib/
│   ├── daly-bms.service       ← Service systemd
│   ├── nginx.conf             ← Reverse proxy nginx
│   ├── install-systemd.sh     ← Script d'installation
│   └── uninstall-systemd.sh
├── docker/                    ← Configs Docker (Mosquitto, Grafana…)
├── docs/
│   ├── Plan.md                ← Plan d'implémentation détaillé
│   ├── JSONData.json          ← Structure de données de référence
│   ├── Daly-UART_485-Communications-Protocol-V1.21-1.pdf
│   └── dalyModbusProtocol.xlsx
└── nanoPi/                    ← Config dbus-mqtt-battery (Venus OS)
    ├── config-bms1.ini
    ├── config-bms2.ini
    └── README.md
```

### Estimation mémoire (RPi5 / production)

| Service                    | RAM minimale | RAM confortable |
|----------------------------|-------------|-----------------|
| daly-bms-server (Rust)     | ~25 MB      | ~50 MB          |
| Mosquitto                  | ~12 MB      | ~20 MB          |
| InfluxDB 2.x (Go)          | ~200 MB     | ~350 MB         |
| Grafana                    | ~120 MB     | ~200 MB         |
| Node-RED (Node.js)         | ~150 MB     | ~250 MB         |
| OS Raspberry Pi OS Lite    | ~150 MB     | ~200 MB         |
| Docker Engine + overhead   | ~100 MB     | ~150 MB         |
| Marge tampon / cache       | ~200 MB     | ~400 MB         |
| **TOTAL**                  | **~957 MB** | **~1420 MB**    |

---

## Prérequis

| Composant | Version | Usage |
|-----------|---------|-------|
| Rust      | 1.80+   | Compilation |
| Docker    | 24+     | Infra (MQTT, InfluxDB, Grafana) |
| Docker Compose v2 | — | `make up` |
| cross     | dernière | Cross-compilation ARM (optionnel) |

> Le dashboard est **SSR (Askama + ECharts)** — Node.js/npm ne sont plus nécessaires.

**Matériel** : Raspberry Pi CM5 (ou Pi 4/5) + adaptateur USB/RS485
**OS** : Debian Bookworm / Ubuntu 24.04 (aarch64 ou x86_64), **Windows 10/11 supporté**
**Permissions Linux** : `sudo usermod -aG dialout $USER`

### Compatibilité multi-plateforme

| Plateforme | Statut | Notes |
|---|---|---|
| Windows 10/11 (x86_64) | ✅ Testé | Port COMx, auto-détection |
| Linux x86_64 | ✅ Compilé | `/dev/ttyUSB0` |
| Raspberry Pi 5 / CM5 (aarch64) | ✅ `make build-arm` | Cross-compile ou natif |
| Cerbo GX / NanoPi Venus OS | N/A | Sert le MQTT, ne fait pas tourner le serveur |

---

## Démarrage rapide

### Mode simulateur (sans matériel BMS — Windows ou Linux)

```bash
# Compiler
cargo build --release

# Lancer avec 2 BMS simulés
cargo run --bin daly-bms-server -- --simulate --sim-bms 0x28,0x29

# Ou avec Make
make run-simulate

# Accéder au dashboard
# http://localhost:8000/dashboard
```

### Infrastructure Docker (5 min)

```bash
cp .env.example .env          # adapter les tokens
make up                        # Mosquitto:1883 InfluxDB:8086 Grafana:3001 Node-RED:1880
make ps                        # vérifier l'état
```

### Configuration

```bash
sudo mkdir -p /etc/daly-bms
sudo cp Config.toml /etc/daly-bms/config.toml
sudo nano /etc/daly-bms/config.toml   # adapter port série + adresses BMS
```

### Compilation et Lancement (hardware réel)

```bash
# Développement (local)
make run-debug

# Production sur le Pi (cross-compile)
make build-arm
make deploy PI_HOST=pi@192.168.1.100
```

### Service systemd (Linux/RPi5)

```bash
make install        # copie le binaire + installe daly-bms.service
journalctl -u daly-bms -f
```

---

## Dashboard intégré

Le dashboard est **embarqué dans le binaire** (SSR Askama + ECharts). Aucun npm, aucun serveur web séparé.

| URL | Description |
|-----|-------------|
| `http://localhost:8000/dashboard` | Vue synthèse de tous les BMS |
| `http://localhost:8000/dashboard/bms/1` | Détail BMS (cellules, températures, historique) |

**Fonctionnalités :**
- Cartes par BMS : SOC, tension, courant, température, puissance
- Boxplot tensions cellules (min/max/avg) avec colorisation
- Indicateur équilibrage actif (cellules hautes/basses)
- Profil températures
- Historique temps réel (ring buffer)
- Thème clair, badge RS485 multi-BMS
- Noms personnalisés par BMS (`name = "BMS-360Ah"`)

---

## API REST — Endpoints

### Système

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/v1/system/status` | État global (BMS online, polling, version) |
| GET | `/api/v1/config` | Configuration active (sans secrets) |
| GET | `/api/v1/discover` | Découverte live sur le bus RS485 |

### BMS — Lecture

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| GET | `/api/v1/bms/{id}/status` | Snapshot complet (SOC, tension, courant…) |
| GET | `/api/v1/bms/{id}/cells` | Tensions individuelles + delta + équilibrage |
| GET | `/api/v1/bms/{id}/temperatures` | Températures par capteur |
| GET | `/api/v1/bms/{id}/alarms` | Flags d'alarme + `any_alarm` |
| GET | `/api/v1/bms/{id}/mos` | État MOS charge/décharge + cycles |
| GET | `/api/v1/bms/{id}/history` | Ring buffer (jusqu'à 3600 snapshots) |
| GET | `/api/v1/bms/{id}/history/summary` | Statistiques min/max/avg |
| GET | `/api/v1/bms/{id}/export/csv` | Export CSV du ring buffer |
| GET | `/api/v1/bms/compare` | Comparaison côte-à-côte de tous les BMS |

### BMS — Écriture (nécessite `api_key` si configurée)

| Méthode | Endpoint | Description |
|---------|----------|-------------|
| POST | `/api/v1/bms/{id}/mos` | Activer/désactiver MOS charge/décharge |
| POST | `/api/v1/bms/{id}/soc` | Calibrer SOC |
| POST | `/api/v1/bms/{id}/soc/full` | SOC → 100% |
| POST | `/api/v1/bms/{id}/soc/empty` | SOC → 0% |
| POST | `/api/v1/bms/{id}/reset` | Reset BMS (avec `confirm: true`) |

### WebSocket

| Endpoint | Description |
|----------|-------------|
| `/ws/bms/stream` | Tous les BMS, broadcast à chaque cycle |
| `/ws/bms/{id}/stream` | Un seul BMS |

---

## CLI

```bash
# Status complet
daly-bms-cli --port /dev/ttyUSB0 --addr 0x01 status

# Tensions cellules
daly-bms-cli --port /dev/ttyUSB0 --addr 0x01 cells --count 16

# Scanner le bus
daly-bms-cli --port /dev/ttyUSB0 discover --start 1 --end 10

# Polling continu
daly-bms-cli --port /dev/ttyUSB0 --addr 0x01 poll --interval 2

# Activer MOS charge
daly-bms-cli --port /dev/ttyUSB0 --addr 0x01 set-charge-mos --enable

# Calibrer SOC à 80%
daly-bms-cli --port /dev/ttyUSB0 --addr 0x01 set-soc --value 80.0
```

---

## Commandes Make

```bash
make up            # Démarrer Docker (infra)
make down          # Arrêter Docker
make build         # Compiler (release, local)
make build-arm     # Cross-compiler pour aarch64 (RPi)
make build-all     # Tous les binaires
make run           # Lancer le serveur
make run-debug     # Debug (RUST_LOG=debug)
make run-simulate  # Mode simulateur (sans matériel)
make test          # Tests unitaires
make test-core     # Tests protocole uniquement
make lint          # Clippy
make fmt           # Format code
make check         # check + fmt + clippy
make deploy        # Cross-compile + deploy SSH sur le Pi
make install       # Installer service systemd
make doc           # Générer et ouvrir la doc Rust
```

---

## Simulateur BMS

Le mode simulateur génère des données **LiFePO4 réalistes** sans matériel :

```bash
# 1 BMS simulé (adresse 0x01 par défaut)
cargo run --bin daly-bms-server -- --simulate

# 2 BMS simulés (adresses 0x28 et 0x29 comme en production)
cargo run --bin daly-bms-server -- --simulate --sim-bms 0x28,0x29
```

**Physique simulée :**
- SOC : courbe de décharge, recharge automatique à 10%, cycle à 95%
- Tension : courbe OCV LiFePO4 (44V vide → 58,4V plein pour 16 cellules)
- Courant : variation sinusoïdale autour de -8,5 A (décharge)
- Température : corrélée au courant + dérive ambiante
- Tensions cellules : déséquilibre réaliste (-15 à +15 mV par cellule)
- Équilibrage : activé automatiquement quand delta > 10 mV
- Alarmes : déclenchées sur seuils SOC/delta

Le simulateur alimente les mêmes bridges que le hardware réel : **MQTT, InfluxDB, AlertEngine, WebSocket, Dashboard**.

---

## Protocole Daly implémenté

### Commandes de lecture

| Data ID | Description | Parsing |
|---------|-------------|---------|
| 0x90 | Tension pack, courant, SOC | uint16/10, offset 30000, uint16/10 |
| 0x91 | Min/max tension cellule + numéro | uint16/1000, octet index |
| 0x92 | Min/max température + capteur | byte-40, octet index |
| 0x93 | État MOS, cycles, capacité résiduelle | bits, uint16, uint32 |
| 0x94 | Nombre cellules, capteurs, état charge | octets |
| 0x95 | Tensions individuelles (3/trame) | uint16/1000, multi-trames |
| 0x96 | Températures individuelles (7/trame) | byte-40, multi-trames |
| 0x97 | Flags équilibrage (48 max) | bits little-endian |
| 0x98 | Alarmes protection (7 octets) | flags |

### Commandes d'écriture

| Data ID | Description |
|---------|-------------|
| 0xD9 | MOS décharge ON/OFF |
| 0xDA | MOS charge ON/OFF |
| 0x21 | Calibration SOC (×10, uint16 BE) |
| 0x00 | Reset BMS |

---

## Alertes configurables

| Règle | Seuil déclenchement | Hysteresis |
|-------|---------------------|------------|
| `cell_ovp` | > 3.60 V | -50 mV |
| `cell_uvp` | < 2.90 V | +50 mV |
| `cell_imbalance` | > 100 mV | -10 mV |
| `soc_low` | < 20% | +5% |
| `soc_critical` | < 10% | +2% |
| `temp_high` | > 45°C | -2°C |
| `high_current` | > 80 A | -5 A |

Notifications : Telegram Bot + SMTP email + journal SQLite.

---

## Dépannage

```bash
# Port série
ls -l /dev/ttyUSB* && groups $USER

# Logs service
journalctl -u daly-bms -f

# Test API
curl http://localhost:8000/api/v1/system/status | jq

# Test WebSocket
wscat -c ws://localhost:8000/ws/bms/stream

# Vérifier InfluxDB
make logs

# Niveau de logs augmenté
RUST_LOG=debug daly-bms-server

# Diagnostic bas niveau (trame brute)
cargo run --bin daly-bms-probe -- --port /dev/ttyUSB0
```

---

## Roadmap

- [x] Phase 0 : Structure workspace Rust (Cargo.toml, 4 crates)
- [x] Phase 0 : Types de données (BmsSnapshot ↔ JSONData.json)
- [x] Phase 0 : Protocole UART + checksum + tests unitaires
- [x] Phase 0 : API Axum (toutes les routes définies)
- [x] Phase 0 : AppState + ring buffer + broadcast WebSocket
- [x] Phase 0 : Bridges (MQTT, InfluxDB, AlertEngine)
- [x] Phase 0 : CLI (clap, toutes les commandes)
- [x] Phase 0 : Outil probe (diagnostic bas niveau)
- [x] Phase 1 : Infrastructure Docker (Mosquitto, InfluxDB, Grafana, Node-RED)
- [x] Phase 1 : Docker complet (Dockerfile + docker-compose.yml stack complète)
- [x] Phase 1 : Simulateur BMS avec physique LiFePO4 (validé Windows + Linux)
- [x] Phase 1 : Auto-détection port série et adresses BMS
- [x] Phase 1 : Dashboard SSR intégré (Askama + ECharts, sans npm)
- [x] Phase 1 : MQTT publish_interval_sec réduit à 1s (temps réel)
- [x] Phase 1 : Architecture Venus OS confirmée (MQTT → D-Bus via dbus-mqtt-battery)
- [x] Phase 1 : Service dbus-canbattery.can0 stoppé sur NanoPi (CAN remplacé par MQTT)
- [x] Phase 1 : Compatibilité Windows 10/11 validée
- [ ] Phase 2 : Déploiement RPi5 CM — port série réel + tests sur matériel BMS physique
- [ ] Phase 2 : Validation commandes d'écriture (MOS, SOC, reset) sur hardware
- [ ] Phase 2 : Tests intégration 24h stabilité
- [ ] Phase 3 : Support Venus OS natif via D-Bus (fork dbus-serialbattery)

---

*Référence protocole : Daly UART/485 Communications Protocol V1.21*
*Runtime : [tokio-serial](https://docs.rs/tokio-serial/latest/tokio_serial/) — [Axum](https://docs.rs/axum/) — [rumqttc](https://docs.rs/rumqttc/)*
