# Renommer un appareil (Venus OS, VRM, pages web du Pi5)

Ce document dit **où chaque nom est défini** et **comment le changer**, pour les appareils
publiés par le Pi5 sur le GX (Cerbo GX MK3) : prises Tongou, ATS, ET112, BMS, sonde de
température, irradiance.

## 1. Où sont les noms

| Affichage | Source du nom | Comment le changer |
|---|---|---|
| Console Venus (écran du GX, Remote Console), VRM | Nom saisi dans la console, sinon `custom_name` / `name` de `/data/daly-bms/config.toml` sur le GX | **Depuis la console Venus** (§2) |
| Pages web du Pi5 — prises Tongou (visualisation, page Tasmota, `/api/v1/tasmota`) | Nom Venus de la prise, sinon `name` de `/etc/daly-bms/config.toml` | **Depuis la console Venus** (§2) : repris automatiquement |
| Pages web du Pi5 — autres cartes (ET112, BMS, ATS, chauffe-eau, climatisation) | Libellés de `Config.toml` ou de la page | Fichier (§4) |
| Topics MQTT, métriques (`tasmota_id`, `id`, adresses RS485) | Identifiants techniques | Ne pas changer : l'historique des métriques en dépend |

## 2. Renommer depuis la console Venus (méthode recommandée)

1. Ouvrir la console : écran du GX, Remote Console (VRM → Remote Console) ou
   `http://192.168.1.120`.
2. **Liste des appareils** → choisir l'appareil → champ **Nom** (pour une prise Tongou, on peut
   aussi renommer la sortie depuis la carte des interrupteurs).
3. Saisir le nouveau nom et valider.

Effet :

- Le nom est enregistré par dbus-mqtt-venus dans `/data/daly-bms/custom-names.json` (sur le
  GX). Il survit aux redémarrages du GX, aux mises à jour de Venus OS et aux redéploiements.
- Il prévaut sur le nom des fichiers de config.
- Pour une prise Tongou, les pages web du Pi5 affichent le nouveau nom au relevé suivant de la
  prise (quelques secondes à quelques minutes ; rechargement de la page Tasmota).

**Revenir au nom d'origine** : effacer le champ **Nom** (nom vide) et valider.

## 3. Le fichier des noms saisis (GX)

`/data/daly-bms/custom-names.json`, clé `<classe>/<instance VRM>` :

```json
{
  "switch/66": "Chauffe-eau LG",
  "ac-meter/31": "Compteur AC-2"
}
```

| Classe | Appareils |
|---|---|
| `switch` | Prises Tongou (instances 61–70), ATS (60) |
| `ac-meter` | ET112 0x08 (30) et 0x09 (31) |
| `pvinverter` | ET112 micro-onduleurs 0x07 (32) |
| `battery` | BMS 360 / 320 / 620 Ah (151 / 152 / 153) |
| `temperature` | Sonde de température (20) |
| `meteo` | Capteur d'irradiance (40) |
| `heatpump` | Pompe à chaleur (si déclarée) |

Modification manuelle (SSH sur le GX), si la console n'est pas disponible :

```bash
ssh root@192.168.1.120
vi /data/daly-bms/custom-names.json          # JSON valide, guillemets droits
svc -t /service/dbus-mqtt-venus             # relit le fichier au redémarrage
```

Sauvegarde : copier ce fichier avec la config du GX. Un appareil qui change d'instance VRM
(remplacement, nouvelle config) perd son nom saisi : le ressaisir.

## 4. Noms dans les fichiers de config (valeurs par défaut)

À utiliser pour un appareil **nouveau** ou pour les cartes du Pi5 qui ne suivent pas Venus.

**Sur le GX** — `/data/daly-bms/config.toml` (copie de `nanoPi/config-nanopi.toml` du dépôt) :
`custom_name` des `[[switches]]`, `name` des `[[grids]]`, `[[bms]]`, etc. Puis :

```bash
svc -t /service/dbus-mqtt-venus
```

`nanoPi/install-venus.sh` conserve le `config.toml` existant du GX : reporter aussi la
modification dans `nanoPi/config-nanopi.toml` du dépôt pour qu'elle serve de référence.

**Sur le Pi5** — `/etc/daly-bms/config.toml` (référence : `Config.toml` du dépôt) : `name` des
`[[tasmota.devices]]`, `[[et112.devices]]`, `[[bms]]`. Puis :

```bash
sudo systemctl restart daly-bms
```

Les libellés fixes de la visualisation (cartes ET112, BMS, ATS) sont dans
`crates/daly-bms-server/templates/visualization.html` et `viz_nodes_*.js` (recompilation de
daly-bms-server).

## 5. Ajouter une prise Tongou

1. **Tasmota** (page web de la prise) : *Configuration → MQTT* → Host `192.168.1.120`, port
   `1883`, Topic `tongou_<6 derniers caractères de la MAC>`. L'enregistrement redémarre la
   prise (coupure brève du relais ; `PowerOnState 3` restaure l'état).
2. **Pi5** — `/etc/daly-bms/config.toml` et `Config.toml` du dépôt : un `[[tasmota.devices]]`
   avec un `id` nouveau (ne jamais renuméroter les existants), `tasmota_id`, `name`,
   `network = "AC-1"` ou `"AC-2"`, `mqtt_index` et `device_instance` libres. Puis
   `sudo systemctl restart daly-bms`.
3. **GX** — `/data/daly-bms/config.toml` et `nanoPi/config-nanopi.toml` : un `[[switches]]`
   avec les mêmes `mqtt_index` / `device_instance`, `command_topic =
   "cmnd/tongou_<…>/Power"`, `custom_name`, `group = "Groupe I (AC-1)"` ou
   `"Groupe II (AC-2)"`. Puis `svc -t /service/dbus-mqtt-venus`.
4. Le switch apparaît dans Venus au premier relevé de la prise ; le renommer ensuite depuis la
   console (§2) si besoin.
