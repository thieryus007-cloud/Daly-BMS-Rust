# NanoPi — Configuration Venus OS (Cerbo GX)

Fichiers à copier sur le NanoPi Venus OS pour les deux instances du driver
[dbus-mqtt-battery](https://github.com/mr-manuel/venus-os_dbus-mqtt-battery).

---

## Architecture confirmée (15 mars 2026)

Le flux de données a été vérifié en production :

```
RPi CM5 (Rust RS485)
       │
       ▼ MQTT publish (toutes les 1s)
Mosquitto / FlashMQ
  santuario/bms/1/venus
  santuario/bms/2/venus
       │
       ▼ subscribe (dbus-mqtt-battery)
dbus-mqtt-battery-41  →  com.victronenergy.battery.mqtt_battery_141
dbus-mqtt-battery-42  →  com.victronenergy.battery.mqtt_battery_142
       │
       ▼ D-Bus Venus OS
  GUI / VRM Portal / systemcalc / hub4control
```

> `dbus-mqtt-battery` fonctionne en mode **MQTT → D-Bus** (abonnement MQTT,
> création d'un service batterie virtuel sur le bus système Venus OS).

---

## Migration depuis CAN

Lors du passage de `dbus-serialbattery.py can0` vers le flux MQTT du RPi CM5,
arrêter le service CAN sur le NanoPi :

```bash
# Arrêter sans désinstaller (récupère ~60 MB RAM + 6% CPU)
svc -d /service/dbus-canbattery.can0

# Vérifier l'état
svstat /service/dbus-canbattery.can0

# Relancer si besoin
svc -u /service/dbus-canbattery.can0
```

---

## Structure sur le NanoPi

```
/data/etc/
  dbus-mqtt-battery-41/
    config.ini          ← copier le contenu de config-bms1.ini
  dbus-mqtt-battery-42/
    config.ini          ← copier le contenu de config-bms2.ini
```

## Déploiement

```bash
# Depuis le RPi CM5 (adapter l'IP du NanoPi)
scp nanopi/config-bms1.ini root@192.168.1.120:/data/etc/dbus-mqtt-battery-41/config.ini
scp nanopi/config-bms2.ini root@192.168.1.120:/data/etc/dbus-mqtt-battery-42/config.ini

# Redémarrer les drivers sur le NanoPi
ssh root@192.168.1.120 "svc -t /service/dbus-mqtt-battery-41 /service/dbus-mqtt-battery-42"
```

## Topics MQTT publiés par le RPi CM5

```
santuario/bms/1/venus   → dbus-mqtt-battery-41  (BMS-360Ah, adresse 0x28)
santuario/bms/2/venus   → dbus-mqtt-battery-42  (BMS-320Ah, adresse 0x29)
```

Intervalle de publication : **1 seconde** (`publish_interval_sec = 1` dans Config.toml).

---

## Surveillance ressources NanoPi (BusyBox compatible)

Venus OS utilise BusyBox — utiliser ces commandes adaptées :

```bash
# Vue CPU + RAM temps réel (meilleure option)
top -b -n 1 | head -n 20

# Surveillance continue refresh 2s
watch -n 2 'top -b -n 1 | head -n 20'

# RAM disponible
cat /proc/meminfo | grep -E "MemTotal|MemFree|MemAvailable"
```

### État du système (15 mars 2026)

| Process | RAM | CPU | Notes |
|---|---|---|---|
| `node-red` | ~229 MB | ~4% | Flow Grid déconnexion actif |
| `gui` (VNC) | ~164 MB | ~6% | Interface locale |
| `dbus-modbus-client` | ~91 MB | 0% | Modbus TCP |
| `dbus-canbattery.can0` | ~60 MB | 0% | **STOPPÉ** — remplacé par MQTT |
| `flashmq` | ~49 MB | 0% | Broker MQTT local |
| `dbus-mqtt-battery-41/42` | ~42 MB x2 | 1% | Bridge MQTT → D-Bus |

RAM disponible après arrêt du service CAN : **~77 MB** (free + cache).

---

## Vérification sur le NanoPi

```bash
# Vérifier que les drivers sont actifs
svstat /service/dbus-mqtt-battery-41
svstat /service/dbus-mqtt-battery-42

# Voir les données reçues sur D-Bus
dbus -y com.victronenergy.battery.mqtt_battery_141 / GetValue

# Logs des drivers
tail -f /var/log/dbus-mqtt-battery-41/current
tail -f /var/log/dbus-mqtt-battery-42/current

# Vérifier le service CAN (doit être stoppé)
svstat /service/dbus-canbattery.can0
```
