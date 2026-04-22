# 🌟 Grafana Dashboard Index — Quick Navigation

**Pour configurer le dashboard Grafana Santuario de A à Z, consultez ces documents dans cet ordre:**

---

## 📚 Documentation Structure

### 1️⃣ **START HERE** — Implementation Summary
**File:** `GRAFANA_IMPLEMENTATION_SUMMARY.md`

- ✅ What has been done (overview)
- ✅ Next steps (deployment checklist)
- ✅ Quick verification steps
- ✅ Troubleshooting quick reference

**Read time:** 5-10 minutes
**Best for:** Understanding what's ready and how to activate it

---

### 2️⃣ **DETAILED SETUP** — Complete Configuration Guide
**File:** `GRAFANA_SETUP.md`

- ✅ Full prerequisites checklist
- ✅ InfluxDB configuration details
- ✅ Grafana data source setup
- ✅ Dashboard import procedures (3 methods)
- ✅ Verification & testing guide
- ✅ Detailed troubleshooting (with solutions)
- ✅ Full data flow diagram
- ✅ Customization guide

**Read time:** 20-30 minutes
**Best for:** Step-by-step setup, diagnosis, fixing problems

---

### 3️⃣ **QUICK START** — Dashboard Overview
**File:** `grafana/README.md`

- ✅ Quick setup (3 options)
- ✅ Measurements covered (8 types, 52 fields)
- ✅ Dashboard access URLs
- ✅ Verification checklist
- ✅ Customization examples
- ✅ Quick troubleshooting table

**Read time:** 10-15 minutes
**Best for:** Understanding the dashboard structure

---

### 4️⃣ **AUTOMATED IMPORT** — Script-Based Setup
**File:** `scripts/import-grafana-dashboard.sh`

**What it does:**
```bash
./scripts/import-grafana-dashboard.sh
```

- ✅ Verifies Grafana connectivity
- ✅ Verifies InfluxDB data source
- ✅ Imports dashboard automatically
- ✅ Validates import success
- ✅ Displays access URLs

**Configuration:**
```bash
export GRAFANA_URL=http://192.168.1.141:3001
export GRAFANA_ADMIN_USER=admin
export GRAFANA_ADMIN_PASSWORD=admin
./scripts/import-grafana-dashboard.sh
```

---

### 5️⃣ **DASHBOARD JSON** — Raw Configuration
**File:** `grafana/dashboards/santuario-solar-system.json`

- ✅ Complete dashboard definition (1900+ lines)
- ✅ 9 panels pre-configured
- ✅ UID: `santuario` (for web app integration)
- ✅ Variables for BMS filtering
- ✅ Ready to import or deploy via provisioning

**For importing manually or via Grafana UI.**

---

## 🎯 Workflow Recommendations

### Scenario A: I just want to activate it NOW
```
1. Read: GRAFANA_IMPLEMENTATION_SUMMARY.md (5 min)
2. Run:  ./scripts/import-grafana-dashboard.sh
3. Verify: Open http://192.168.1.141:8080/dashboard/grafana
4. Done! ✨
```

### Scenario B: I want to understand everything first
```
1. Read: GRAFANA_IMPLEMENTATION_SUMMARY.md
2. Read: grafana/README.md
3. Read: GRAFANA_SETUP.md (Architecture section)
4. Run:  ./scripts/import-grafana-dashboard.sh
5. Verify & troubleshoot as needed
```

### Scenario C: I'm having issues
```
1. Check: GRAFANA_SETUP.md → "Troubleshooting" section
2. Run:   Commands from verification checklist
3. Read:  Specific error section
4. If still stuck: Run script with debug output
```

### Scenario D: I want to customize the dashboard
```
1. Read: grafana/README.md → "Customization" section
2. Read: GRAFANA_SETUP.md → "Customization" section
3. Edit: Directly in Grafana UI (Create → Edit panel)
4. Save: Button in top right
```

---

## 📊 Dashboard Content Reference

### 9 Panels Included:

| # | Panel Name | Type | Measurement | Fields Shown |
|---|-----------|------|-------------|--------------|
| 1 | BMS SOC | Gauge | bms_status | soc (0-100%) |
| 2 | BMS Voltage | Timeseries | bms_status | voltage (V) |
| 3 | BMS Current | Timeseries | bms_status | current (A) |
| 4 | BMS Power | Timeseries | bms_status | power (W) |
| 5 | BMS Temperature | Timeseries | bms_status | temp_max (°C) |
| 6 | ET112 Power | Timeseries | et112_status | power_w (W) |
| 7 | Tasmota Power | Timeseries | tasmota_status | power_w (W) |
| 8 | Solar Irradiance | Timeseries | irradiance_status | irradiance_wm2 (W/m²) |
| 9 | Venus MPPT Total | Timeseries | venus_mppt_total | power_w (W) |

### Additional Fields Available (not in main panels):
- Cell voltages (16 per BMS)
- Cell delta (imbalance)
- Temperature min/max
- MOS status & cycles
- Alarm flags
- ET112: Apparent power, reactive power, PF, frequency, energy
- Tasmota: Voltage, current, energy today/total
- Venus: SmartShunt, Inverter data

---

## 🔗 Integration Points

### Web App Tab
**Location:** `http://192.168.1.141:8080/dashboard/grafana`
- Navbar: "📊 Historique"
- Displays Grafana iframe
- Proxy-based for CORS compatibility

### Grafana Direct
**Location:** `http://192.168.1.141:3001/d/santuario/santuario-solar-system`
- Full Grafana interface
- Edit capabilities
- Admin functions

### API Proxy
**Endpoint:** `http://192.168.1.141:8080/api/v1/grafana/d/santuario`
- Backend proxy to avoid CORS
- Used by web app internally

---

## ✅ Pre-Deployment Checklist

Before activating on Pi5:

```bash
☐ Code synced to Pi5 (make sync)
☐ Binary rebuilt (make build-arm)
☐ Service restarted (systemctl restart daly-bms)
☐ InfluxDB is accessible (curl health)
☐ Grafana is running (docker ps | grep grafana)
☐ Dashboard JSON imported (script or manual)
☐ Data appears in panels (not empty)
☐ Navbar shows "📊 Historique" tab
☐ Clicking tab loads dashboard
```

---

## 🆘 Quick Help

### Import failed?
→ Check `GRAFANA_SETUP.md` → "Configuration Grafana" section

### No data in panels?
→ Check `GRAFANA_SETUP.md` → "Troubleshooting" section

### Variables not working?
→ Check `grafana/README.md` → "Variables Grafana" section

### Want to customize?
→ Read `GRAFANA_SETUP.md` → "Customization du Dashboard" section

### Script errors?
```bash
# Run with debug output
GRAFANA_URL=http://192.168.1.141:3001 \
GRAFANA_ADMIN_USER=admin \
GRAFANA_ADMIN_PASSWORD=admin \
./scripts/import-grafana-dashboard.sh -v
```

---

## 📖 Document Map

```
┌─────────────────────────────────────────────────────────┐
│ GRAFANA_INDEX.md (You are here)                         │
│ ├─ Quick navigation & workflow recommendations          │
│ └─ This file                                            │
└─────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────┐
│ GRAFANA_IMPLEMENTATION_SUMMARY.md                       │
│ ├─ What's been done (Overview)                          │
│ ├─ Next steps (Deployment)                             │
│ ├─ Verification checklist                              │
│ └─ Quick troubleshooting                               │
└─────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────┐
│ GRAFANA_SETUP.md                                        │
│ ├─ Prerequisites                                        │
│ ├─ InfluxDB configuration (detailed)                    │
│ ├─ Grafana configuration (detailed)                     │
│ ├─ Import procedures (3 methods)                        │
│ ├─ Testing & verification                              │
│ ├─ Detailed troubleshooting                            │
│ ├─ Data flow diagram                                   │
│ └─ Customization guide                                 │
└─────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────┐
│ grafana/README.md                                       │
│ ├─ Quick start (3 options)                             │
│ ├─ Structure overview                                  │
│ ├─ Data reference (52 fields)                          │
│ ├─ Data source config                                 │
│ ├─ Verification checks                                │
│ ├─ Auto-import setup (provisioning)                    │
│ └─ Quick troubleshooting                              │
└─────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────┐
│ scripts/import-grafana-dashboard.sh                     │
│ └─ Automated import with validation                    │
└─────────────────────────────────────────────────────────┘
        ↓
┌─────────────────────────────────────────────────────────┐
│ grafana/dashboards/santuario-solar-system.json          │
│ └─ Raw dashboard configuration (1900+ lines)           │
└─────────────────────────────────────────────────────────┘
```

---

## 🎯 Key Information

**Dashboard UID:** `santuario` (MUST match web app)
**Data Source:** `InfluxDB` (MUST match Grafana config)
**InfluxDB Bucket:** `daly_bms` (by default)
**Refresh Rate:** 30 seconds
**Time Range:** Last 24 hours (configurable)
**Access Points:** 3 (web app iframe, direct Grafana, API proxy)

**Measurements Covered:** 8
**Fields Visualized:** 52
**Panels:** 9
**Variables:** 2 (datasource, bms_address)

---

## 📞 Support Chain

1. **Quick question?** → Check the relevant `.md` file in the **Document Map** above
2. **Setup issue?** → Run the automated script + consult `GRAFANA_SETUP.md`
3. **Needs customization?** → Edit directly in Grafana UI (Create → Edit panel)
4. **Deep debugging?** → Consult troubleshooting sections in `GRAFANA_SETUP.md`

---

Last updated: 2026-04-22
All files ready on branch: `claude/add-grafana-dashboard-nav-XPdvb`
