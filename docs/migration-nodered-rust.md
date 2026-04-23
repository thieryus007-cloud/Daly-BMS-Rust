
# Cahier des charges – Migration Node-RED → Application Rust

## 1. Contexte et objectif

Remplacer l’intégralité des flows Node-RED par une application monolithique écrite en **Rust**, exécutée sur un Raspberry Pi 5 (ou équivalent). L’application doit :

- Assurer la même logique métier (gestion d’énergie, pilotage d’équipements, relevés météo, production solaire, etc.).
- Conserver les interactions avec :
  - Un broker MQTT (Mosquitto sur `192.168.1.141`).
  - L’API REST de Victron Venus OS (via bridge MQTT → D‑Bus).
  - Les API HTTP externes (LG ThinQ, Open‑Meteo).
  - InfluxDB 2.x (pour la persistance et la restauration d’états).
  - Des équipement Shelly (MQTT RPC) et Tasmota (MQTT).
- Offrir une **diffusion live** (WebSocket) pour les données temps réel, en plus du stockage dans InfluxDB.
- Maintenir la **persistance** des compteurs et baselines (production solaire, état du chauffe‑eau, etc.) à travers les redémarrages.

## 2. Architecture cible (application Rust)

L’application Rust s’articule autour des composants suivants :

1. **Client MQTT** (`rumqttc`) :
   - Se connecte au broker MQTT local (`192.168.1.141:1883`).
   - Souscrit à tous les topics utilisés par les flows (préfixes `N/`, `santuario/irradiance/raw`, `shelly…/events`, etc.).
   - Publie les commandes sur les topics `W/`, `cmnd/…`, `shelly…/rpc`, `santuario/…`.

2. **Serveur HTTP / WebSocket** (`axum`) :
   - Endpoints pour recevoir des commandes manuelles / externes (optionnel).
   - WebSocket pour diffuser en direct certaines variables (PV power, SOC, AC state, etc.) **avant** écriture dans InfluxDB.

3. **Bus interne (Tokio MPSC / broadcast)** :
   - Un canal `broadcast` pour le flux live (abonné par WebSocket).
   - Un canal `mpsc` pour l’écriture asynchrone dans InfluxDB et les actions non critiques.

4. **Tâches dédiées (tokio::spawn)** :
   - Polling des API externes (Open‑Meteo, LG ThinQ) avec timers.
   - Logique métier (machines à états, hystérésis, anti‑oscillation, délais).
   - Écriture par lots dans InfluxDB (toutes les 5 secondes ou au fil de l’eau).
   - Keep‑alive MQTT pour les services Venus OS.

5. **Stockage** :
   - **InfluxDB** (bucket `daly_bms`) : écriture des mesures `solar_persist`, `solar_power`.
   - Variables d’environnement : `INFLUX_TOKEN`, `INFLUX_ORG`, `INFLUX_BUCKET`.

6. **Configuration** :
   - Fichier `config.toml` ou variables d’environnement pour :
     - adresses IP, ports, credentials API (LG, clés Open‑Meteo, etc.).
     - Seuils (fréquences, hystérésis, délais, production minimale, etc.).
     - Identifiants des devices (Portal ID `c0619ab9929a`, instances VEBus, etc.).

## 3. Sources de données (entrées)

### 3.1 Topics MQTT (broker `192.168.1.141`)

| Topic | Type payload | Description | Utilisé par |
|-------|--------------|-------------|--------------|
| `N/c0619ab9929a/vebus/275/Ac/State/IgnoreAcIn1` | JSON `{value:0/1}` | AC input ignoré (off‑grid) | Chargement batterie, chauffe‑eau, DEYE |
| `N/c0619ab9929a/vebus/275/Dc/0/MaxChargeCurrent` | - | Valeur de consigne courante (non utilisé en entrée) | (référence) |
| `N/c0619ab9929a/system/0/Ac/PvOnOutput/L1/Power` | JSON | Puissance PV côté AC | Excédent PV → charge batterie |
| `N/c0619ab9929a/system/0/Ac/ConsumptionOnOutput/L1/Power` | JSON | Consommation sur la sortie AC | Excédent PV, monitoring maison |
| `N/c0619ab9929a/vebus/275/Ac/Out/L1/F` | JSON | Fréquence secteur | Commande DEYE (anti‑oscillation) |
| `N/c0619ab9929a/vebus/275/Ac/ActiveIn/Connected` | JSON | Connexion au réseau (0/1) | Commande DEYE |
| `santuario/irradiance/raw` | string (int) | Irradiance solaire (W/m²) | Météo, chauffe‑eau |
| `N/c0619ab9929a/solarcharger/273/Yield/Power` | JSON | Puissance MPPT 273 | Production solaire |
| `N/c0619ab9929a/solarcharger/289/Yield/Power` | JSON | Puissance MPPT 289 | Production solaire |
| `N/c0619ab9929a/pvinverter/32/Ac/L1/Power` | JSON | Puissance micro‑onduleurs | Production solaire |
| `N/c0619ab9929a/solarcharger/+/History/Daily/0/Yield` | JSON | Énergie journalière MPPT | Production cumulée |
| `N/c0619ab9929a/pvinverter/32/Ac/Energy/Forward` | JSON | Énergie cumulée micro‑onduleurs | Delta journalier + baseline |
| `shellypro2pm-ec62608840a4/events/rpc` | JSON | Événements Shelly (interrupteurs) | État des switches (DEYE) |
| `stat/tongou_3BC764/POWER` | `ON` / `OFF` | État du relais Tasmota | Commande chauffe‑eau |
| `tele/tongou_3BC764/SENSOR` | JSON | Puissance, énergie Tasmota | Monitoring |
| `N/c0619ab9929a/system/0/Dc/Battery/Soc` | JSON | SOC batterie | Chauffe‑eau, commande DEYE |
| `N/c0619ab9929a/system/0/Dc/Battery/Current` | JSON | Courant batterie | Chauffe‑eau (décharge) |
| `N/c0619ab9929a/system/0/Dc/Pv/Power` | JSON | Puissance DC totale des MPPT | Chauffe‑eau |
| `N/c0619ab9929a/vebus/275/Dc/0/Voltage` | JSON | Tension batterie (inverter) | Inverter JSON |
| `N/c0619ab9929a/vebus/275/Dc/0/Current` | JSON | Courant batterie (inverter) | Inverter JSON |
| `N/c0619ab9929a/vebus/275/Dc/0/Power` | JSON | Puissance DC inverter | Inverter JSON |
| `N/c0619ab9929a/vebus/275/Ac/Out/L1/V` | JSON | Tension AC out | Inverter JSON |
| `N/c0619ab9929a/vebus/275/Ac/Out/L1/I` | JSON | Courant AC out | Inverter JSON |
| `N/c0619ab9929a/vebus/275/Energy/InverterToAcOut` | JSON | Énergie inverter → AC | (monitoring) |
| `N/c0619ab9929a/vebus/275/Energy/OutToInverter` | JSON | Énergie AC → inverter | (monitoring) |
| `N/c0619ab9929a/vebus/275/State` | JSON | État du VEBus (mode) | Inverter JSON |
| `N/c0619ab9929a/system/0/Dc/Battery/State` | JSON | État batterie (smartshunt) | Inverter JSON |
| `N/c0619ab9929a/system/0/Dc/Battery/TimeToGo` | JSON | Autonomie restante | (monitoring) |
| `N/c0619ab9929a/solarcharger/273/State`, `Pv/V`, `Dc/0/Current`, `History/…` | JSON | Détails MPPT 273 | Production détaillée |
| `N/c0619ab9929a/solarcharger/289/…` | JSON | Détails MPPT 289 | Production détaillée |

### 3.2 Appels HTTP externes

| API / Endpoint | Méthode | Fréquence | Description |
|----------------|---------|-----------|-------------|
| `https://api.open-meteo.com/v1/forecast` (Badalucco) | GET | 5 min | Température, humidité, pression, vent |
| `https://api-eic.lgthinq.com/devices/{id}/state` | GET | 10 min | État du chauffe‑eau LG (mode, températures) |
| `https://api-eic.lgthinq.com/devices/{id}/control` | POST | sur événement | Commande du chauffe‑eau (mode VACATION/HEAT_PUMP/TURBO, température) |

### 3.3 Injection / timers internes (remplacés par des tâches tokio)

- **Every 2 min** : vérification manuelle `IgnoreAcIn1` (déjà géré par MQTT, mais inject pour forcer ?)
- **Keepalive 25s** : republication du dernier état du chauffe‑eau sur MQTT.
- **Keepalive 60s** pour la pompe à chaleur (heatpump) et pour le service platform.
- **Reset minuit** (cron) : remise à zéro des compteurs journaliers.
- **Démarrage** : restauration des baselines depuis InfluxDB et MQTT retained.

## 4. Logiques métier (par onglet)

### 4.1 Onglet `Charge Current` (gestion du courant de charge batterie)

- **Entrées** : `Ac/State/IgnoreAcIn1`, `Ac/PvOnOutput/L1/Power`, `Ac/ConsumptionOnOutput/L1/Power`.
- **Calcul** : excédent PV = PV_on_output - consommation. Si excédent > 50 W, flag `pvExcess` true.
- **Règle** :
  - Si grid déconnecté (`IgnoreAcIn1 == 1`) → envoyer `70` A au `MaxChargeCurrent`.
  - Si grid connecté :
    - Si `pvExcess` → courant = 4 A.
    - Sinon → courant = 0 A.
- **Actions** : publier sur `W/.../MaxChargeCurrent` et `W/.../PowerAssistEnabled` (1 si off‑grid, 0 sinon). Publier aussi `W/.../MaxFeedInPower = 0` si grid connecté.
- **Anti‑répétition** : ne publier que si la valeur change (mémoire d’état).

### 4.2 Onglet `Commande DEYE` (pilotage du relais Shelly pour l’onduleur)

- **Entrées** : fréquence AC out (`Ac/Out/L1/F`), état grid (`Ac/ActiveIn/Connected`).
- **Machine à états** (off‑grid uniquement) :
  - Seuil haut : `52.0` Hz (coupe après 15 s).
  - Seuil bas : `50.3` Hz (rallume après 45 s).
  - **Anti‑oscillation** : interdiction de rallumer pendant 120 s après une coupure.
- **Action** : envoyer commande `Switch.Set` au Shelly (MQTT RPC) sur le topic `shellypro2pm-ec62608840a4/rpc` (payload JSON) pour l’interrupteur **0** (DEYE).
- **État** : maintenir en mémoire l’état actuel (`ON`/`OFF`), les timers de validation.

### 4.3 Onglet `Manage waterHeater` (commande automatique du chauffe‑eau LG)

- **Entrées** :
  - `IgnoreAcIn1` → `acIgnored` (0 = grid connecté)
  - `Soc` batterie
  - Courant batterie (signe : négatif = décharge)
  - Production solaire totale (DC + AC)
- **Paramètres** :
  - Production min : 2000 W (seuil sous lequel on passe en mode VACATION).
  - Délai de confirmation : 5 minutes pour les conditions instables (décharge, production insuffisante).
  - Délai minimum entre deux changements de mode : 15 minutes.
- **Règles** : basculer en mode **VACATION** si l’une des conditions suivantes est remplie (après débounce) :
  1. Grid connecté (`acIgnored == 0`)
  2. SOC < 95%
  3. Batterie en décharge depuis au moins 5 min
  4. Production solaire ≤ 2000 W depuis au moins 5 min
- Sinon, mode **HEAT_PUMP**.
- **Actions** :
  - Appel API LG (POST `/control`) pour changer le mode (`VACATION` ou `HEAT_PUMP`).
  - Après un changement vers `HEAT_PUMP` : attendre 15 s puis régler la température cible à 60°C.
  - Après un changement vers `VACATION` : attendre 15 s puis régler la température à 45°C (économie).
- **Persistance** : stocker le mode actuel et l’horodatage du dernier changement (pour respecter le délai de 15 min). Au démarrage, lire l’état depuis MQTT retained ou API.

### 4.4 Onglet `Set waterHeater` (lecture périodique et keepalive pour Venus OS)

- **Toutes les 10 min** : appel API GET `/state` du chauffe‑eau LG.
- **Parser** le mode (`HEAT_PUMP`/`TURBO`/`VACATION`) et les températures.
- **Mapper** vers le service D‑Bus Venus `com.victronenergy.heatpump.mqtt_1` :
  - `State` : 0 = VACATION, 1 = HEAT_PUMP, 2 = TURBO.
  - `Temperature` : température courante.
  - `TargetTemperature` : consigne.
- **Publier** sur MQTT `santuario/heatpump/1/venus` (retained).
- **Keepalive 25 s** : republier la dernière valeur connue (watchdog Venus 30 s).

### 4.5 Onglet `Heatpump / Chauffe-eau` (simulation / test)

- **Inject manuels** pour envoyer des états ON/OFF factices (State=2 / 0).
- **Keepalive 60 s** : publication périodique d’un payload fixe (retained).
- À conserver pour compatibilité, mais pourra être fusionné avec l’onglet Set waterHeater.

### 4.6 Onglet `Switch / ATS` (gestion des sources AC)

- **Commandes manuelles** (position 0 = réseau, position 1 = générateur).
- Publication sur `santuario/switch/1/venus` (retained) avec `Position` et `State`.
- Actuellement piloté par inject, peut être étendu pour automatisation (futur). À migrer tel quel.

### 4.7 Onglet `Platform Pi5` (statut de backup)

- Publication périodique (60 s) du statut de backup (`Status`: 0 idle, 1 running, 2 OK, 3 error) sur `santuario/platform/venus`.
- Pour l’instant valeurs factices. À migrer en tant que service optionnel.

### 4.8 Onglet `Irradiance RS485`

- **Entrée MQTT** : `santuario/irradiance/raw` (entier W/m²) publié par un script Python externe.
- **Stockage** dans `global.irradiance_wm2` (variable d’état).
- **Validation** : plage 0‑2000 W/m².
- Utilisé par l’onglet Météo.

### 4.9 Onglet `Meteo` (pivot central)

- **Polling Open‑Meteo** toutes les 5 min → température, humidité, pression, vent.
- **Collecte** : irradiation (via `irradiance_wm2`), production solaire journalière (`total_yield_today`), production de la veille (`yield_yesterday`).
- **Publication** (retained) :
  - `santuario/heat/1/venus` : température, humidité, pression (service D‑Bus Venus).
  - `santuario/meteo/venus` : irradiation, production du jour, production veille, vent.
- **Persistance des compteurs** :
  - **MPPT** : cumul des `History/Daily/0/Yield` de chaque MPPT → `mppt_yield_today`.
  - **PVInverter (ET112)** : énergie cumulée → `pvinv_yield_today` via baseline stockée.
  - **Baseline PVInverter** : valeur d’énergie absolue au début de la journée. Persistée dans InfluxDB (measurement `solar_persist`, tag `day`) et en MQTT retained (`santuario/persist/pvinv_baseline`).
  - **Reset minuit** :
    - Sauvegarder `total_yield_today` dans `yield_yesterday`.
    - Remettre à zéro tous les compteurs et la baseline.
    - Nettoyer la baseline retained MQTT (payload vide).
- **Restauration au démarrage** :
  - Lire la baseline depuis InfluxDB (dernière valeur pour la date du jour).
  - Lire `yield_yesterday` depuis MQTT retained.
  - Si aucune baseline, attendre le premier message PVInverter de la journée.

### 4.10 Onglet `Inverter/Charger → santuario/inverter/venus`

- **Collecte** de multiples topics MQTT :
  - Tension, courant, puissance DC.
  - Tension, courant, puissance AC out.
  - Fréquence AC, `IgnoreAcIn1`, état VEBus.
- **Fusion** des données dans un JSON unique et publication sur `santuario/inverter/venus` (retained).
- Structure du JSON final :
```json
{
  "Voltage": number,
  "Current": number,
  "Power": number,
  "AcVoltage": number,
  "AcCurrent": number,
  "AcPower": number,
  "AcFrequency": number,
  "State": "on",             // constant
  "Mode": "inverter",       // constant
  "IgnoreAcIn": 0/1,
  "VebusState": number
}
```

4.11 Onglet SmartShunt → santuario/system/venus

· Collecte : SOC, tension, courant, puissance, State (code numérique) et TimeToGo (secondes).
· Publication sur santuario/system/venus (retained) :

```json
{
  "Soc": number,
  "Voltage": number,
  "Current": number,
  "Power": number,
  "State": number,
  "TimeToGo": number
}
```

4.12 Onglet Tasmota (commande et état du relais chauffe‑eau)

· Commande : publication sur cmnd/tongou_3BC764/Power avec payload ON/OFF/TOGGLE.
· État : souscription à stat/tongou_3BC764/POWER (payload ON/OFF) → stocker dans une variable tongou_state.
· Télémétrie : souscription à tele/tongou_3BC764/SENSOR (JSON ENERGY) → extraire Power, Voltage, Current, Today, Total.
· Ces valeurs peuvent être exposées via WebSocket ou écrites dans InfluxDB.

4.13 Onglet Solar_power (puissance et monitoring)

· Entrées : puissance MPPT 273, MPPT 289, PVInverter 32, et consommation maison (Ac/ConsumptionOnOutput).
· Calcul : solar_total_w = somme des trois puissances. mppt_power_w = somme des deux MPPT.
· Écriture dans InfluxDB (measurement solar_power, avec tags day,host) toutes les secondes (au fil de l’eau).
· Publication sur santuario/meteo/venus (enrichi avec MpptPower, SolarTotal, et liste détaillée des MPPT avec leurs attributs : Instance, State, PvVoltage, DcCurrent, Power, YieldToday).
· Envoi au serveur daly-bms-server via POST http://192.168.1.141:8080/api/v1/solar/mppt-yield avec solar_total_w, mppt_power_w, house_power_w.

5. Sorties et actions

5.1 Commandes MQTT (broker 192.168.1.141)

Topic Payload Description
W/c0619ab9929a/vebus/275/Dc/0/MaxChargeCurrent {value: numerique} Courant de charge max
W/c0619ab9929a/vebus/275/Settings/PowerAssistEnabled {value: 0/1} Activation PowerAssist
W/c0619ab9929a/settings/0/Settings/CGwacs/MaxFeedInPower {value: 0} Limite injection réseau
W/c0619ab9929a/settings/0/Settings/CGwacs/AcExportLimit {value: 0} (rare)
shellypro2pm-ec62608840a4/rpc JSON RPC (Switch.Set) Commande Shelly (DEYE)
cmnd/tongou_3BC764/Power ON/OFF/TOGGLE Commande Tasmota
santuario/heatpump/1/venus JSON (State,Temperature,TargetTemperature,Position) Interface Venus OS
santuario/switch/1/venus {"Position":0/1,"State":0/1/2} ATS
santuario/platform/venus {"Backup":{"Status":0-3,"LastRun":ts},"Restore":...} Statut backup
santuario/heat/1/venus {"Temperature":...,"Humidity":...,"Pressure":...} Météo heat service
santuario/meteo/venus JSON complet (irradiance, yields, vent, mppt) Météo + solaire
santuario/inverter/venus JSON state inverter Monitoring
santuario/system/venus JSON battery system Monitoring
santuario/persist/pvinv_baseline string (baseline) ou vide Persistance retained
santuario/persist/yield_yesterday string (kWh) Persistance retained

5.2 Requêtes HTTP

URL (POST) Payload Fréquence
https://api-eic.lgthinq.com/…/control mode ou température sur changement détecté
http://192.168.1.141:8080/api/v1/solar/mppt-yield {solar_total_w, mppt_power_w, house_power_w} à chaque mise à jour solaire (~1s)
(InfluxDB écriture) line protocol à chaque donnée pertinente (solar_power, solar_persist)

5.3 WebSocket live (nouveau)

· Diffuser en temps réel les variables suivantes (dès réception, avant stockage) :
  · Puissance PV totale, consommation maison, SOC batterie, courant batterie, fréquence, état grid, mode chauffe‑eau, état Shelly DEYE, etc.
· Les clients WebSocket s’abonnent à un flux JSON par topic (ex: live/solar, live/battery).

6. Persistance et restauration

6.1 InfluxDB (mesures)

· solar_persist : enregistre chaque nuit le pvinv_baseline, mppt_yield_today, total_yield_today (tags: day, host). Écriture lors du reset ou périodiquement.
· solar_power : enregistre toutes les secondes les puissances instantanées (W) des trois sources + total.

6.2 MQTT retained (fallback)

· santuario/persist/pvinv_baseline : valeur absolue de l’énergie PVInverter au début de la journée.
· santuario/persist/yield_yesterday : production de la veille.
· santuario/heatpump/1/venus : dernier état du chauffe‑eau (keepalive).
· santuario/meteo/venus : dernières données météo.
· santuario/inverter/venus, santuario/system/venus : états courants.

6.3 Comportement au démarrage

1. Se connecter au broker MQTT, souscrire à tous les topics nécessaires.
2. Lire les topics retained (baseline, yield_yesterday, dernier état du chauffe‑eau, etc.) pour initialiser les variables globales.
3. Interroger InfluxDB pour récupérer la baseline de la journée en cours (si existe).
4. Démarrer les tâches périodiques (Open‑Meteo, API LG, keepalives, reset minuit).
5. Lancer le serveur WebSocket pour le live.

7. Exigences non fonctionnelles

· Latence : le live WebSocket doit avoir une latence < 50 ms (hors writing InfluxDB).
· Robustesse : l’application doit continuer à fonctionner si InfluxDB est indisponible (écriture en mémoire tampon limitée, fallback MQTT retained).
· Consommation mémoire : < 50 MB (typique pour une app Rust bien écrite).
· CPU : utilisation minimale (< 5% sur Pi5 en régime normal).
· Logs : structurés (JSON) avec niveaux (debug, info, error) orientés tracing.
· Maintenabilité : configuration externe (fichier TOML) pour tous les seuils (fréquences, délais, production min, etc.). Pas de valeurs codées en dur.

8. Contraintes techniques

· Langage : Rust (édition 2021).
· Runtime asynchrone : Tokio.
· MQTT : rumqttc.
· WebSocket / HTTP : axum + tokio-tungstenite.
· HTTP client : reqwest.
· InfluxDB : influxdb (ou influxdb_rust pour v2) + serde_json.
· Sérialisation : serde.
· Variables d’environnement : dotenvy pour les secrets (tokens API, clés).
· Cron : tokio-cron-scheduler ou simple tokio::time::interval.
· Tests unitaires : au minimum pour les machines à états (DEYE, water heater).
· Déploiement : binaire statique, service systemd.

9. Livrables attendus (par l’IA)

· Code Rust complet, structuré en modules (mqtt, http, influx, logic, live_ws, config).
· Fichier Cargo.toml avec toutes les dépendances.
· Fichier de configuration exemple (config.toml.sample) avec tous les seuils documentés.
· Fichier README.md expliquant le lancement, la configuration, et l’architecture.
· (Optionnel) Un script systemd pour le service.

10. Notes complémentaires

· Les topics MQTT avec préfixe N/ sont des données entrantes du broker (via bridge Venus OS → Pi5). Les topics W/ sont des commandes sortantes vers Venus OS.
· Le client MQTT doit avoir un client_id unique pour éviter les conflits.
· Certains keepalives (25s, 60s) sont critiques : les services Venus OS ont un watchdog de 30 s. Ne pas les espacer trop.
· L’API LG ThinQ nécessite des tokens d’authentification (bearer, api‑key). Ils resteront dans le fichier .env.
· Toutes les valeurs numériques (températures, puissances, seuils) doivent être configurables sans recompilation.

```
