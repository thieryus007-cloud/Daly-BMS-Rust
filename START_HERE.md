# 🚀 START HERE — Grafana Dashboard Quick Start

**Everything you need is on this page.** No other doc needed to get started.

---

## ⚡ 30-Second Summary

✅ **Complete Grafana dashboard created** — Specific to your installation with all 52 data fields
✅ **Integrated with web app** — "📊 Historique" tab in navbar
✅ **Fully documented** — 2000+ lines of guides
✅ **Ready to deploy** — Just follow 4 simple steps

---

## 📋 Pre-Deployment Checklist (On Pi5)

Before you start, verify these are running:

```bash
# SSH to Pi5
ssh pi5compute@192.168.1.141

# Check daly-bms-server
systemctl status daly-bms           # Should show "active (running)"

# Check Docker
docker ps | grep -E "grafana|influxdb"
# Should show both grafana and influxdb containers running

# Verify InfluxDB is accessible
curl http://localhost:8086/health
# Should return: {"status":"ok"}
```

If anything fails, **STOP** and debug before proceeding.

---

## 🎯 Deploy in 4 Steps

### Step 1: Sync Code (2 min)
```bash
ssh pi5compute@192.168.1.141
cd ~/Daly-BMS-Rust
make sync
```

### Step 2: Build & Deploy (5-10 min)
```bash
make build-arm

sudo systemctl stop daly-bms

sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/

sudo systemctl start daly-bms

# Verify it started
journalctl -u daly-bms -n 5
```

### Step 3: Import Dashboard (1 min) — AUTOMATED

```bash
# Still on Pi5, in ~/Daly-BMS-Rust directory

export GRAFANA_URL=http://192.168.1.141:3001
export GRAFANA_ADMIN_USER=admin
export GRAFANA_ADMIN_PASSWORD=admin

./scripts/import-grafana-dashboard.sh
```

**Expected output:**
```
✅ File dashboard trouvé
✅ Grafana accessible
✅ Data Source 'InfluxDB' trouvée
✅ Dashboard importé avec succès
✅ Dashboard accessible

📊 Dashboard accessible à:
   http://192.168.1.141:3001/d/santuario/santuario-solar-system

🌐 Application Web:
   http://192.168.1.141:8080/dashboard/grafana
```

### Step 4: Verify (2 min)

Open your browser:

**Option A: Via Web App** (Recommended first-time)
```
http://192.168.1.141:8080
→ Click "📊 Historique" tab in navbar
→ Should display Grafana dashboard in iframe
```

**Option B: Direct Grafana**
```
http://192.168.1.141:3001/d/santuario/santuario-solar-system
→ Should display 9 panels with live data
```

---

## ✅ Quick Verification

In the dashboard, you should see:

- [ ] **BMS SOC** panel shows a percentage (e.g., 75%)
- [ ] **BMS Voltage** graph shows a line trending up/down
- [ ] **BMS Current** graph shows data points
- [ ] **BMS Power** graph shows values in watts
- [ ] **BMS Temperature** graph shows temperature
- [ ] **ET112 Power** graph shows your meter power
- [ ] **Tasmota Power** graph shows outlet power
- [ ] **Solar Irradiance** graph shows W/m² (should be 0 at night)
- [ ] **Venus MPPT Total** graph shows solar generation

**All panels empty?** → Go to **"Troubleshooting"** section below

---

## 🛠️ Troubleshooting Quick Fix

### Problem: "No data found" in dashboard

**Check 1: Is data being written to InfluxDB?**
```bash
journalctl -u daly-bms -f | grep -i influx
# Should show write operations
```

**Check 2: Is Grafana connected to InfluxDB?**
```bash
# In Grafana UI (http://192.168.1.141:3001)
# Go to: ⚙️ Configuration → Data Sources → InfluxDB
# Click "Test" button
# Should show: ✅ datasource is working
```

**Check 3: Change time range**
- Default is "Last 24h"
- If you just started: Try "Last 1 hour"
- Button at top-right of dashboard

### Problem: "Grafana not accessible"

```bash
# Check if container is running
docker ps | grep grafana

# If not running
docker-compose up -d grafana

# Wait 10 seconds
sleep 10

# Try accessing
curl http://localhost:3001/api/health
```

### Problem: Dashboard import failed

```bash
# Try manual import instead:
# 1. Open http://192.168.1.141:3001
# 2. Click: Create → Import
# 3. Upload: ~/Daly-BMS-Rust/grafana/dashboards/santuario-solar-system.json
# 4. Click: Import
```

---

## 📚 Next Steps (Optional)

Once verified, you can:

1. **Customize the dashboard**
   - Click edit button (pencil) in top-right
   - Add more panels
   - Change colors, units, etc.
   - Save when done

2. **Read full documentation** (if needed)
   - **GRAFANA_INDEX.md** — Navigation hub
   - **GRAFANA_SETUP.md** — Complete guide (500+ lines)
   - **GRAFANA_IMPLEMENTATION_SUMMARY.md** — What was done

3. **Configure auto-import** (so dashboard imports on restart)
   - Edit: `docker-compose.yml`
   - Mount provisioning directory
   - See: **GRAFANA_SETUP.md** → "Provisioning" section

---

## 🔑 Key Information

| Item | Value |
|------|-------|
| **Dashboard UID** | `santuario` |
| **Data Source** | `InfluxDB` |
| **InfluxDB Bucket** | `daly_bms` |
| **Web App URL** | `http://192.168.1.141:8080/dashboard/grafana` |
| **Direct Grafana** | `http://192.168.1.141:3001/d/santuario/santuario-solar-system` |
| **Refresh Rate** | 30 seconds |
| **Data Covered** | 8 measurements, 52 fields |
| **Panels** | 9 (BMS, ET112, Tasmota, Solar, Venus) |

---

## ❓ Still Stuck?

1. **Check logs:**
   ```bash
   journalctl -u daly-bms -n 50 | grep -i error
   docker logs daly-bms-grafana | tail -20
   ```

2. **Read full troubleshooting:**
   ```bash
   cat GRAFANA_SETUP.md | grep -A 20 "Troubleshooting"
   ```

3. **Run verification checklist:**
   ```bash
   # From GRAFANA_IMPLEMENTATION_SUMMARY.md
   # (Copy the verification checklist and run each command)
   ```

---

## 📊 What Was Delivered

**On GitHub branch `claude/add-grafana-dashboard-nav-XPdvb`:**

| File | Purpose |
|------|---------|
| `grafana/dashboards/santuario-solar-system.json` | Complete dashboard (1900+ lines) |
| `GRAFANA_SETUP.md` | Full setup guide (500+ lines) |
| `GRAFANA_INDEX.md` | Navigation hub for all docs |
| `GRAFANA_IMPLEMENTATION_SUMMARY.md` | Deployment guide |
| `grafana/README.md` | Quick reference |
| `scripts/import-grafana-dashboard.sh` | Automated import script |
| `crates/daly-bms-server/templates/grafana_dashboard.html` | Web app template |
| `crates/daly-bms-server/src/api/grafana.rs` | Grafana proxy backend |
| `crates/daly-bms-server/templates/base.html` | Updated navbar with "📊 Historique" |

---

## 🎯 Expected Result

After following these 4 steps, you should have:

✅ Web app showing "📊 Historique" tab in navbar
✅ Clicking tab displays Grafana dashboard in iframe
✅ 9 panels showing live data from InfluxDB
✅ BMS voltage, current, power displayed
✅ ET112 meter power displayed
✅ Tasmota outlet power displayed
✅ Solar irradiance displayed
✅ Venus MPPT power displayed
✅ All data updating every 30 seconds
✅ Time range selector working
✅ BMS address filter working

---

## 🎉 You're Done!

That's it. Everything is ready to go.

**Total time:** ~20 minutes (mostly build time)
**Effort:** ~5 minutes of actual work
**Result:** Complete solar system dashboard with all your data

---

## 📞 Questions?

- **"What if data doesn't appear?"** → See Troubleshooting section above
- **"Can I customize it?"** → Yes, edit in Grafana UI (pencil button)
- **"Can I add more panels?"** → Yes, Create → Add panel
- **"Where's the full documentation?"** → Read `GRAFANA_INDEX.md`

---

**Ready? Follow the 4 steps above! 🚀**
