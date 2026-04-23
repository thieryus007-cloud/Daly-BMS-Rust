# 📊 Grafana Dashboards — Santuario Solar System

Dashboards Grafana pour visualiser TOUTES les données du système solaire Santuario.

---

## 📁 Structure

```
grafana/
├── dashboards/
│   └── santuario-solar-system.json          # UID: "santuario"
│       └── Panneaux complets (9 panels)
│           ├── BMS SOC (gauge)
│           ├── BMS Voltage (timeseries)
│           ├── BMS Current (timeseries)
│           ├── BMS Power (timeseries)
│           ├── BMS Temperature (timeseries)
│           ├── ET112 Power (timeseries)
│           ├── Tasmota Power (timeseries)
│           ├── Solar Irradiance (timeseries)
│           └── Venus MPPT Total (timeseries)
├── provisioning/                            # (Optionnel) Configuration auto-import
│   └── dashboards/dashboard.yml
├── README.md                                # (Ce fichier)
```

---

## 🚀 Quick Start

### 1️⃣ Via Script automatique (Recommandé)

```bash
cd /home/user/Daly-BMS-Rust

# Configuration (optionnel si Grafana sur localhost:3001)
export GRAFANA_URL=http://192.168.1.141:3001
export GRAFANA_ADMIN_USER=admin
export GRAFANA_ADMIN_PASSWORD=admin

influxDB
supersecretchangeit

grafana
autre_supersecret

# Lancer l'import
./scripts/import-grafana-dashboard.sh
```

✅ Le script va:
- Vérifier la connexion à Grafana
- Vérifier la data source InfluxDB
- Importer le dashboard JSON
- Afficher un lien d'accès direct

### 2️⃣ Via Grafana UI

1. Ouvrez **Grafana** → `http://192.168.1.141:3001`
2. **Create** → **Import**
3. **Upload JSON file** → Sélectionnez `grafana/dashboards/santuario-solar-system.json`
4. Cliquez **Import**

### 3️⃣ Via API REST

```bash
curl -X POST http://192.168.1.141:3001/api/dashboards/db \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_GRAFANA_API_TOKEN" \
  -d @grafana/dashboards/santuario-solar-system.json
```

---

## 📊 Données visualisées

### Measurements couverts:
- ✅ **bms_status** — Voltage, courant, puissance, SOC, température
- ✅ **et112_status** — Compteurs (puissances, énergies)
- ✅ **tasmota_status** — Prises connectées
- ✅ **irradiance_status** — Irradiance solaire (W/m²)
- ✅ **venus_mppt_total** — Production MPPT totale
- ✅ **venus_smartshunt** — Batterie ShartShunt (optionnel)
- ✅ **venus_inverter** — Onduleur (optionnel)

**Total: 52 fields distincts**

---

## 🔧 Configuration de la Data Source

Grafana **doit** avoir une data source InfluxDB configurée:

**Nom exact:** `InfluxDB` (sensible à la casse!)

**Configuration:**
- **URL:** `http://influxdb:8086` (depuis Docker) ou `http://localhost:8086`
- **Database:** `daly_bms` (le bucket InfluxDB par défaut)
- **Authentication:** Laisser vide (par défaut)

---

## 📖 Documentation complète

Consultez **`GRAFANA_SETUP.md`** pour:
- Architecture de données détaillée
- Configuration InfluxDB
- Procédures d'import pas à pas
- Troubleshooting
- Personnalisation du dashboard

```bash
cat ../GRAFANA_SETUP.md
```

---

## 🌐 Accès

### Via Application Web
```
http://192.168.1.141:8080/dashboard/grafana
```
(Affiche le dashboard dans une iframe)

### Via Grafana direct
```
http://192.168.1.141:3001/d/santuario/santuario-solar-system
```

### Via API Proxy
```
http://192.168.1.141:8080/api/v1/grafana/d/santuario
```

---

## ✅ Vérifications

### 1. Vérifier les données InfluxDB

```bash
# SSH Pi5
ssh pi5compute@192.168.1.141

# Logs daly-bms
journalctl -u daly-bms -n 50 | grep -i influx

# Ou via API InfluxDB directe
curl http://localhost:8086/health
```

### 2. Vérifier la connexion Grafana ↔ InfluxDB

```bash
# Dans Grafana UI: Configuration → Data Sources → InfluxDB → Test
# Doit afficher: ✅ datasource is working
```

### 3. Vérifier le dashboard

```bash
curl http://192.168.1.141:3001/api/dashboards/uid/santuario \
  -H "Authorization: Bearer YOUR_TOKEN"
```

---

## 🎯 Variables Grafana

Le dashboard inclut des **variables dynamiques**:

| Variable | Options | Usage |
|----------|---------|-------|
| `datasource` | InfluxDB, Prometheus, etc | Sélectionner la source |
| `bms` | 0x01, 0x02, ... | Filtrer par adresse BMS |

---

## 🔄 Auto-Import (Optionnel)

Pour importer automatiquement le dashboard au démarrage de Grafana:

1. Modifiez `docker-compose.yml`:

```yaml
services:
  grafana:
    image: grafana/grafana:latest
    volumes:
      - ./grafana/provisioning:/etc/grafana/provisioning
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
```

2. Créez `grafana/provisioning/dashboards/dashboard.yml`:

```yaml
apiVersion: 1
providers:
  - name: 'Santuario'
    orgId: 1
    folder: ''
    type: file
    disableDeletion: false
    editable: true
    options:
      path: /etc/grafana/provisioning/dashboards
```

3. Redémarrez:
```bash
docker-compose up -d grafana
```

---

## 🛠️ Personnalisation

Le dashboard est **entièrement éditable**. Vous pouvez:

- ➕ **Ajouter des panneaux** → Click **+ Add panel**
- ✏️ **Modifier les requêtes** → Click panel → **Edit**
- 📊 **Changer les types** → Gauge, Table, Heatmap, etc
- 💾 **Sauvegarder** → Click **Save** en haut

Exemples de panels à ajouter:
```
- BMS Cell Voltages (table)
- BMS Alarms Status (stat)
- ET112 Energy Cumulative (gauge)
- Tasmota State (status)
- Venus SmartShunt SOC (gauge)
- ATS Active Source (stat)
```

---

## 📝 Notes importantes

⚠️ **UID du dashboard DOIT être `santuario`**
- C'est l'identifiant utilisé par l'intégration web
- Ne pas renommer!

⚠️ **Data Source DOIT s'appeler `InfluxDB`**
- Case-sensitive!
- Utilisée par tous les panels

⚠️ **Les données doivent être présentes dans InfluxDB**
- Vérifiez que daly-bms-server écrit actuel
- Vérifiez les logs: `journalctl -u daly-bms -f`

---

## 🔗 Liens utiles

- **Grafana Docs**: https://grafana.com/docs/grafana/latest/
- **InfluxDB Flux**: https://docs.influxdata.com/flux/v0.x/
- **GRAFANA_SETUP.md**: Procédures détaillées
- **CLAUDE.md**: Configuration globale du projet

---

## 🚨 Troubleshooting rapide

| Problème | Solution |
|----------|----------|
| Pas de données | Vérifiez InfluxDB écrit (journalctl daly-bms) |
| Erreur data source | Vérifiez que InfluxDB est accessible (curl health) |
| Dashboard pas trouvé | Vérifiez l'UID: `santuario` (sensible à la casse) |
| Variables vides | Vérifiez que les measurements existent dans InfluxDB |

---

Dernière mise à jour: 2026-04-22
