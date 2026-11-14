# GX Venus OS — Cerbo GX MK3 (bridge `dbus-mqtt-venus`)

Le dossier s'appelle `nanoPi/` pour des raisons historiques ; il contient tout ce
qui est déployé sur le **GX Victron** du système.

| GX | Valeur |
|---|---|
| Modèle | Cerbo GX MK3 (S/N HQ2627EH6RK), machine Venus `am62xx` |
| CPU / arch | Cortex-A53, **aarch64** |
| Venus OS | v3.85 |
| IP | `192.168.1.120` (eth0, fixe) — WiFi `192.168.1.122` |
| Portal ID VRM | `d81d13b31a8b` |
| SSH | `ssh root@192.168.1.120` (clé `pi5-nanopi-deploy` du Pi5 autorisée) |

---

## Architecture

```
Pi5 (192.168.1.141) — daly-bms-server / energy-manager
   └── mosquitto-broker ── bridge pi5-nanopi-bridge ──► FlashMQ du GX (192.168.1.120:1883)
                                                          │ santuario/#  (subscribe)
                                                          ▼
                                             dbus-mqtt-venus (runit)
                                                          │
                                                          ▼ D-Bus système Venus OS
                                   com.victronenergy.battery.mqtt_1/2/3, switch.mqtt_1..6,
                                   grid.mqtt_9, acload.mqtt_8, pvinverter.mqtt_7,
                                   temperature.mqtt_1, meteo → GUI / VRM / systemcalc
```

En sens inverse, Venus publie `N/d81d13b31a8b/...` (lu par energy-manager via le
bridge) et reçoit les commandes `W/` / keepalive `R/`.

---

## Fichiers de ce dossier

| Fichier | Rôle |
|---|---|
| `config-nanopi.toml` | Config de production de `dbus-mqtt-venus` → `/data/daly-bms/config.toml` sur le GX |
| `install-venus.sh` | Déploiement complet (binaire, config si absente, service runit, `/data/rc.local`) |
| `sv/dbus-mqtt-venus/run` | Script runit → `/data/etc/sv/dbus-mqtt-venus/run` |
| `cleanup-dbus-serialbattery.sh` | Outil de nettoyage d'un ancien `dbus-serialbattery` (diagnostic + suppression interactive) |

## Implantation sur le GX

```
/data/daly-bms/dbus-mqtt-venus        binaire aarch64 Cortex-A53
/data/daly-bms/config.toml            = nanoPi/config-nanopi.toml
/data/etc/sv/dbus-mqtt-venus/run      service runit
/service/dbus-mqtt-venus -> /data/etc/sv/dbus-mqtt-venus
/data/rc.local                        recrée le lien /service au boot :
    [ -e /service/dbus-mqtt-venus ] || ln -s /data/etc/sv/dbus-mqtt-venus /service/dbus-mqtt-venus
```

---

## Déploiement (depuis le Pi5, `~/Daly-BMS-Rust`)

```bash
make install-venus     # build Cortex-A53 (target/venus-a53) + install-venus.sh 192.168.1.120
```

> ⚠ Le binaire du GX est compilé avec `-C target-cpu=cortex-a53`, **jamais**
> `target-cpu=native` : le Pi5 est un Cortex-A76, un binaire « native » plante en
> SIGILL sur le Cortex-A53 du GX. `make install-venus` s'en charge (répertoire
> cible séparé `target/venus-a53`, sans toucher au cache des binaires Pi5).

Config seule :

```bash
scp nanoPi/config-nanopi.toml root@192.168.1.120:/data/daly-bms/config.toml
ssh root@192.168.1.120 "svc -t /service/dbus-mqtt-venus"
```

---

## Diagnostic

```bash
ssh root@192.168.1.120 "svstat /service/dbus-mqtt-venus"          # up X seconds
ssh root@192.168.1.120 "dbus -y | grep victronenergy | sort"        # services présents
ssh root@192.168.1.120 "dbus -y com.victronenergy.battery.mqtt_1 /Soc GetValue"
```

Le service n'a pas de journal svlogd (pas de `log/run`). Pour voir les logs :

```bash
svc -d /service/dbus-mqtt-venus
/data/daly-bms/dbus-mqtt-venus --config /data/daly-bms/config.toml   # Ctrl+C pour arrêter
svc -u /service/dbus-mqtt-venus
```

Un service D-Bus `mqtt_*` n'apparaît qu'à la réception du premier message MQTT
correspondant (`santuario/<type>/<n>/venus`) ; il passe `Connected=0` après
`watchdog_sec` sans données.

---

## Instances VRM (D-Bus)

| Service | Instance | Appareil |
|---|---|---|
| `vebus.ttyS4` | 276 | MultiPlus-II 48/5000/70-50 (port VE.Bus du Cerbo) |
| `solarcharger.*` | 273 | SmartSolar MPPT VE.Can 250/100 « Sud » (HQ2506A3WX6) |
| `solarcharger.*` | 288 | SmartSolar MPPT 150/70 « Toiture » (HQ2552DDAMU) |
| `solarcharger.*` | 289 | SmartSolar MPPT 150/35 « Toit montagne » (HQ2506UHKWJ) |
| `solarcharger.*` | 290 | SmartSolar MPPT 150/35 « T.Nord » (HQ2515R2HDA) |
| `solarcharger.*` | 291 | SmartSolar MPPT 150/35 « T.Sud » (HQ2515APQM4) |
| `solarcharger.*` | 292 | SmartSolar MPPT 150/35 « T.milieu » (HQ2515DGNY4) |
| `battery.ttyS2` | 274 | SmartShunt 300A (HQ25134UYXJ) — moniteur batterie système |
| `switch.HQ2518CFAPP` | 50 | GX IO extender 150 |
| `acload.shelly_EC62608840A4_0/1` | 50 / 51 | Shelly Pro 2PM (Chauffe-eau / Éclairage) |
| `battery.mqtt_1/2/3` | 151 / 152 / 153 | BMS Daly 360 / 320 / 620 Ah (dbus-mqtt-venus) |
| `pvinverter.mqtt_7` | 32 | ET112 Micro-onduleurs (dbus-mqtt-venus) |
| `acload.mqtt_8` / `grid.mqtt_9` | 30 / 31 | ET112 Maison / Réseau (dbus-mqtt-venus) |
| `switch.mqtt_1..6` | 60–65 | ATS CHINT + Tongou 1–5 (dbus-mqtt-venus) |
| `temperature.mqtt_1` | 20 | Température extérieure (dbus-mqtt-venus) |
| `meteo` | 40 | Irradiance PRALRAN (dbus-mqtt-venus) |

Les instances VE.Direct (MPPT, SmartShunt) sont mémorisées par numéro de série
(`/Settings/Devices/vedirect_<serial>/ClassAndVrmInstance`) : elles suivent
l'appareil quel que soit le port. L'instance VE.Bus dépend du **port** du GX.

---

## Remplacement du GX — points à reporter

1. **Portal ID** : `Config.toml` (`[energy_manager.victron] portal_id`) **et**
   `contrib/mosquitto/mosquitto.conf` (topics `N/` `R/` `W/` du bridge).
2. **`vebus_instance`** dans `Config.toml` : relire
   `dbus -y com.victronenergy.system /VebusInstance GetValue` sur le nouveau GX.
3. **Réglages Venus** (`/data/conf/settings.xml`) : déposer le fichier de l'ancien
   GX en `/data/conf/settings.xml.import` puis redémarrer. Si l'attribut racine
   `unique-id` ne correspond pas au nouveau GX, `localsettings` applique
   `cleanup_settings()` et **supprime** `/Settings/Devices` (instances VRM) et
   `/Settings/Victron/Products` (noms des MPPT) : mettre `unique-id` = portal ID
   du nouveau GX pour les conserver.
4. **Architecture CPU** du nouveau GX (`uname -m`, `/proc/cpuinfo`) : adapter
   `VENUS_ARM_RUSTFLAGS` dans le Makefile si ce n'est plus un Cortex-A53.
5. Clé SSH du Pi5 dans `/data/home/root/.ssh/authorized_keys`, flows Node-RED,
   puis `make install-venus`.
