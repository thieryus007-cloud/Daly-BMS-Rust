# Guide Complet — Intégration Dashboard Grafana Santuario

Ce guide vous montre comment configurer le dashboard Grafana "Santuario Solar System" pour visualiser TOUTES les données de votre installation avec InfluxDB.

---

## 📋 Table des matières

1. [Prérequis](#prérequis)
2. [Architecture de données](#architecture-de-données)
3. [Configuration InfluxDB](#configuration-influxdb)
4. [Configuration Grafana](#configuration-grafana)
5. [Import du Dashboard](#import-du-dashboard)
6. [Vérification & Test](#vérification--test)
7. [Troubleshooting](#troubleshooting)

---

## Prérequis

✅ **Obligatoire:**
- Grafana Docker running: `http://192.168.1.141:3001`
- InfluxDB Docker running: `http://192.168.1.141:8086`
- daly-bms-server en cours d'exécution (écrit les données dans InfluxDB)
- `docker-compose.yml` avec services Grafana + InfluxDB

✅ **Vérification rapide (sur Pi5):**
```bash
docker ps | grep -E "grafana|influxdb"
curl http://localhost:3001/api/health   # Grafana
curl http://localhost:8086/health       # InfluxDB
```

---

## Architecture de données

### Measurements dans InfluxDB (8 total)

| Measurement | Tags | Fields | Source | Intervalle |
|-------------|------|--------|--------|------------|
| `bms_status` | address (0x01, 0x02) | voltage, current, power, soc, temp_max, temp_min, cell_delta_mv, charge_mos, discharge_mos, any_alarm, cycles, soh, capacity, consumed_ah, min_cell_v, max_cell_v, bms_capacity | BMS Daly | À chaque snapshot |
| `bms_cell_voltage` | address, cell | voltage | BMS Daly | À chaque snapshot |
| `et112_status` | address, name | voltage_v, current_a, power_w, apparent_power_va, reactive_power_var, power_factor, frequency_hz, energy_import_wh, energy_export_wh | Compteur Gavazzi | 10s |
| `tasmota_status` | id, name, tasmota_id | power_on, power_w, voltage_v, current_a, apparent_power_va, power_factor, energy_today_kwh, energy_yesterday_kwh, energy_total_kwh | Prises Tasmota | 10s |
| `irradiance_status` | address, name | irradiance_wm2 | PRALRAN | 30s |
| `venus_mppt_total` | (aucun) | power_w, current_a | Venus (somme) | 10s |
| `venus_smartshunt` | (aucun) | voltage_v, current_a, power_w, soc_percent, energy_in_kwh, energy_out_kwh | Venus | 10s |
| `venus_inverter` | (aucun) | voltage_v, current_a, power_w, ac_output_voltage_v, ac_output_current_a, ac_output_power_w, ac_out_frequency_hz | Venus | 10s |

**Total: 52 fields distincts disponibles**

---

## Configuration InfluxDB

### 1️⃣ Accéder à InfluxDB

```bash
# Via navigateur
http://192.168.1.141:8086

# Ou via CLI Docker
docker exec -it daly-bms-influxdb influx
```

### 2️⃣ Vérifier les données (via UI InfluxDB)

1. Allez à **Data Explorer**
2. Sélectionnez le bucket **`daly_bms`** (ou votre bucket)
3. Sous **Measurements**, vous devez voir:
   - ✅ `bms_status`
   - ✅ `bms_cell_voltage`
   - ✅ `et112_status`
   - ✅ `tasmota_status`
   - ✅ `irradiance_status`
   - ✅ `venus_mppt_total`
   - ✅ `venus_smartshunt`
   - ✅ `venus_inverter`

4. **Si aucune donnée n'apparaît**: Vérifiez que daly-bms-server écrit dans InfluxDB:
   ```bash
   ssh pi5compute@192.168.1.141
   journalctl -u daly-bms -f | grep -i influx
   ```

### 3️⃣ Vérifier les données via Flux (Optionnel)

Dans InfluxDB, **Script Editor**, exécutez:

```flux
from(bucket: "daly_bms")
  |> range(start: -24h)
  |> filter(fn: (r) => r._measurement == "bms_status")
  |> limit(n: 10)
```

Vous devriez voir des résultats avec `address`, `soc`, `voltage`, `current`, etc.

---

## Configuration Grafana

### 1️⃣ Ajouter la Data Source InfluxDB

1. Allez à **Grafana** → **Configuration** (⚙️)  → **Data Sources**
2. Cliquez **Add data source**
3. Choisissez **InfluxDB**
4. Configurez:

```
Name:           InfluxDB
URL:            http://influxdb:8086          (ou http://localhost:8086)
Database:       daly_bms                      (votre bucket)
HTTP Method:    GET
Authentication: 
  - User:       (laisser vide si pas d'auth)
  - Password:   (laisser vide si pas d'auth)
Access:         Server (default)
```

5. Cliquez **Save & Test**
6. Vous devez voir: ✅ `datasource is working`

### 2️⃣ Vérifier la connexion

Si vous avez une erreur:
- ❌ `error reading influxdb response`: Vérifiez que InfluxDB est accessible
  ```bash
  docker exec daly-bms-grafana curl http://influxdb:8086/health
  ```
- ❌ `database not found`: Vérifiez le nom du bucket dans InfluxDB
- ❌ `connection refused`: Vérifiez que `http://influxdb:8086` est correct dans `docker-compose.yml`

---

## Import du Dashboard

### Option A: Import via Grafana UI (Simple, Recommandé)

1. Allez à **Grafana Dashboard** → **+ Create** → **Import**
2. Sous **"Import via panel JSON"**, collez le contenu de:
   ```
   grafana/dashboards/santuario-solar-system.json
   ```
3. Ou téléchargez le fichier et choisissez **Upload JSON file**
4. Configurez:
   - **Name:** `Santuario Solar System`
   - **UID:** `santuario` ✅ (important!)
   - **Data Source:** Sélectionnez `InfluxDB`
   - **Folder:** `Solar` (ou laisser default)
5. Cliquez **Import**

### Option B: Import via API Grafana

```bash
# Copier le JSON
curl -X POST http://192.168.1.141:3001/api/dashboards/db \
  -H "Content-Type: application/json" \
  -d @grafana/dashboards/santuario-solar-system.json
```

### Option C: Montage du fichier dans Docker (Persistent)

Modifiez votre `docker-compose.yml`:

```yaml
services:
  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/provisioning:/etc/grafana/provisioning
```

Créez `grafana/provisioning/dashboards/dashboard.yml`:

```yaml
apiVersion: 1
providers:
  - name: Santuario
    orgId: 1
    folder: 'Solar'
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /etc/grafana/provisioning/dashboards
```

Relancez Grafana:
```bash
docker-compose up -d grafana
```

---

## Vérification & Test

### 1️⃣ Accédez au Dashboard

```
http://192.168.1.141:3001/d/santuario/santuario-solar-system
```

### 2️⃣ Vous devriez voir:

**Panneaux:**
- 📊 **BMS SOC** (gauge, 0-100%)
- 📈 **BMS Voltage** (graph, V)
- 📈 **BMS Current** (graph, A)
- 📈 **BMS Power** (graph, W)
- 📈 **BMS Temperature (Max)** (graph, °C)
- 📈 **ET112 Power** (graph, W)
- 📈 **Tasmota Power** (graph, W)
- 📊 **Solar Irradiance** (graph, W/m²)
- 📈 **Venus MPPT Total** (graph, W)

**Variables:**
- 🔄 **Data Source**: `InfluxDB` (dropdown)
- 🔄 **BMS Address**: Multi-sélection des adresses BMS (0x01, 0x02, etc.)

### 3️⃣ Test des filtres

1. Sélectionnez différents **BMS Address** → Les graphiques doivent changer
2. Changez la **plage horaire** (en haut à droite) → 6h, 24h, 7j, etc.
3. Vérifiez la **frequency d'actualisation** (en haut à droite) → 30s (par défaut)

### 4️⃣ Pas de données? Diagnostiquez:

**Étape 1: Vérifiez InfluxDB**
```bash
# Sur Pi5
docker exec daly-bms-influxdb influx query 'SELECT * FROM bms_status LIMIT 1'
```

**Étape 2: Vérifiez Grafana → Data Source**
- Allez à ⚙️ Configuration → Data Sources → InfluxDB
- Cliquez **Test** → Doit afficher ✅

**Étape 3: Vérifiez le Dashboard JSON**
- Editez le dashboard (bouton crayon en haut à droite)
- Sélectionnez un panel (ex: BMS SOC)
- Allez à **Panel Options** → **Queries**
- Vérifiez que le measurement est correct (`bms_status`)

---

## Troubleshooting

### ❌ "No data found" ou graphiques vides

**Cause 1: Données non écrites**
```bash
# Vérifiez que daly-bms-server écrit dans InfluxDB
ssh pi5compute@192.168.1.141
journalctl -u daly-bms -n 20 | grep -i influxdb
```

**Cause 2: Mauvaise plage de temps**
- Par défaut: **last 24h**
- Si vous venez de démarrer: Changez à "Last 1 hour" ou "Last 30 minutes"

**Cause 3: Data Source pas connectée**
- Allez à ⚙️ Configuration → Data Sources → InfluxDB
- Cliquez **Save & Test**
- Vérifiez que vous n'avez pas d'erreur

### ❌ "Error: InfluxDB datasource is not working"

```bash
# Depuis le container Grafana, testez la connexion:
docker exec daly-bms-grafana curl -v http://influxdb:8086/health

# Ou via Pi5:
curl http://localhost:8086/health
```

**Solutions:**
1. Vérifiez que InfluxDB est running: `docker ps | grep influxdb`
2. Vérifiez l'URL: `http://influxdb:8086` (pas `localhost` dans Docker!)
3. Vérifiez le bucket: `daly_bms` (par défaut)
4. Redémarrez InfluxDB: `docker-compose restart influxdb`

### ❌ "Data source plugin not found"

Assurez-vous que Grafana a la bonne version supportant InfluxDB (v7.0+):
```bash
docker exec daly-bms-grafana grafana-cli admin list-plugins | grep influx
```

Si absent:
```bash
docker exec daly-bms-grafana grafana-cli plugins install grafana-influxdb-datasource
docker-compose restart grafana
```

### ❌ Variables BMS Address ne se remplissent pas

1. Allez au dashboard → Editer (crayon)
2. Cliquez sur **BMS Address** variable (en bas)
3. Cliquez **Query** → Doit être:
   ```
   SHOW TAG VALUES FROM "bms_status" WITH KEY = "address"
   ```
4. Cliquez **Update** → Devrait lister: `0x01`, `0x02`, etc.

---

## Personnalisation du Dashboard

### Ajouter des panneaux supplémentaires

Le dashboard est éditable. Vous pouvez:

1. **Ajouter un panel**:
   - Cliquez **+ Add panel**
   - Choisissez le type (Timeseries, Gauge, Table, etc.)
   - Sélectionnez le measurement (ex: `bms_cell_voltage`)

2. **Exemples de panels à ajouter**:
   ```
   - BMS Cell Voltages (table avec Cell1, Cell2, ...)
   - BMS Alarms (status panel)
   - ET112 Energy (cumulative)
   - Tasmota Energy Today (gauge)
   - Venus SmartShunt Current (graph)
   - Venus Inverter AC Output (graph)
   ```

3. **Sauvegarder le dashboard**:
   - Cliquez le bouton **Save** (disquette en haut)

---

## Accès depuis le navigateur web

### Via la navbar d'accueil

Dans votre application web (`http://192.168.1.141:8080`):
1. Cliquez sur l'onglet **"📊 Historique"** (ajouté dans la navbar)
2. Le dashboard Grafana s'affiche en iframe
3. Tous les panneaux sont interactifs

### URL directe Grafana

```
http://192.168.1.141:3001/d/santuario/santuario-solar-system
```

### Depuis l'API Proxy

```
http://192.168.1.141:8080/api/v1/grafana/d/santuario
```

---

## Flux de données complet

```
┌─────────────────────────────────────────────────────────────────┐
│ Pi5 Daly-BMS-Server (http://localhost:8080)                     │
├─────────────────────────────────────────────────────────────────┤
│ • Lit BMS (RS485) → Écrit bms_status + bms_cell_voltage        │
│ • Lit ET112 (Modbus) → Écrit et112_status                      │
│ • Lit Tasmota (MQTT) → Écrit tasmota_status                    │
│ • Lit Irradiance (RS485) → Écrit irradiance_status             │
│ • Lit Venus (D-Bus) → Écrit venus_mppt_total, venus_smartshunt │
└─────────┬───────────────────────────────────────────────────────┘
          │ InfluxDB Write API
          ▼
┌─────────────────────────────────────────────────────────────────┐
│ InfluxDB (http://localhost:8086)                                │
├─────────────────────────────────────────────────────────────────┤
│ Bucket: daly_bms                                                │
│ • bms_status (17 fields × 2 addresses)                          │
│ • bms_cell_voltage (16 cells × 2 addresses)                     │
│ • et112_status (9 fields × 3 devices)                          │
│ • tasmota_status (9 fields × N prises)                         │
│ • irradiance_status (1 field)                                   │
│ • venus_mppt_total (2 fields)                                   │
│ • venus_smartshunt (6 fields)                                   │
│ • venus_inverter (7 fields)                                     │
└─────────┬───────────────────────────────────────────────────────┘
          │ Grafana Query API
          ▼
┌─────────────────────────────────────────────────────────────────┐
│ Grafana (http://localhost:3001)                                 │
├─────────────────────────────────────────────────────────────────┤
│ Dashboard: Santuario Solar System (UID: santuario)              │
│ • 9 panneaux visualisant 52 fields différents                   │
│ • Variables pour filtrer par BMS Address                        │
│ • Refresh 30s                                                   │
└─────────┬───────────────────────────────────────────────────────┘
          │ Proxy (CORS bypass)
          ▼
┌─────────────────────────────────────────────────────────────────┐
│ Web App (http://192.168.1.141:8080/dashboard/grafana)           │
├─────────────────────────────────────────────────────────────────┤
│ Affiche l'iframe Grafana dans la navbar                         │
│ Utilisateurs visualisent les données en temps réel              │
└─────────────────────────────────────────────────────────────────┘
```

---

## Support & Questions

Si vous rencontrez un problème:

1. **Vérifiez les logs**:
   ```bash
   docker logs daly-bms-grafana
   docker logs daly-bms-influxdb
   ```

2. **Testez la connectivité**:
   ```bash
   docker exec daly-bms-grafana curl http://influxdb:8086/health
   curl http://192.168.1.141:8086/health
   ```

3. **Regardez le network**:
   ```bash
   docker network ls
   docker network inspect daly-bms_default
   ```

4. **Consultez CLAUDE.md** pour les configurations globales du projet

---

## Notes importantes

⚠️ **Ne pas oublier:**
- Vérifier que InfluxDB écrit les données (vérifier daly-bms logs)
- Data Source **InfluxDB** doit être nommée exactement `InfluxDB` (sensible à la casse)
- L'URL doit être `http://influxdb:8086` depuis Grafana (alias Docker), PAS `localhost`
- Le bucket s'appelle `daly_bms` (par défaut)
- Le dashboard UID **DOIT être** `santuario` pour l'intégration web

✅ **Testé avec:**
- Grafana 9.5+
- InfluxDB 2.5+
- Docker Compose
- Pi5 + aarch64

