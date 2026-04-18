#!/bin/bash
# Script de diagnostic rapide MQTT — À exécuter sur le Pi5
# Affiche 5 vérifications essentielles
set -euo pipefail

echo "═══════════════════════════════════════════════════════════════"
echo "  DIAGNOSTIC MQTT RAPIDE — Daly-BMS-Rust Pi5 (2026-04-10)"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# 1. État du service
echo "[1/5] État du service daly-bms:"
echo "─────────────────────────────────"
STATUS=$(sudo systemctl status daly-bms 2>&1 | grep "Active:")
echo "$STATUS"
if echo "$STATUS" | grep -q "active (running)"; then
    echo "✓ Service tourne"
else
    echo "✗ Service N'EST PAS actif"
fi
echo ""

# 2. Dernières erreurs
echo "[2/5] Dernières lignes de log (erreurs MQTT / RS485):"
echo "────────────────────────────────────────────────────"
journalctl -u daly-bms -n 15 --no-pager 2>&1 | tail -10
echo ""

# 3. Vérifier config MQTT
echo "[3/5] Config MQTT (host/port):"
echo "──────────────────────────────"
grep -A4 "\[mqtt\]" /etc/daly-bms/config.toml 2>/dev/null || echo "✗ Config /etc/daly-bms/config.toml non trouvée"
echo ""

# 4. Connectivité NanoPi
echo "[4/5] Connectivité vers NanoPi broker (192.168.1.120:1883):"
echo "──────────────────────────────────────────────────────────"
if timeout 2 bash -c 'echo | nc 192.168.1.120 1883' 2>/dev/null; then
    echo "✓ Broker accessible"
else
    echo "✗ Broker INACCESSIBLE"
fi
echo ""

# 5. Messages MQTT reçus
echo "[5/5] Premier message MQTT du topic santuario/bms/1/venus (5s timeout):"
echo "───────────────────────────────────────────────────────────────────────"
mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus' -W 5 2>/dev/null | head -1
if [ $? -eq 0 ]; then
    echo "✓ Messages MQTT reçus"
else
    echo "✗ Aucun message MQTT"
fi
echo ""

echo "═══════════════════════════════════════════════════════════════"
echo "  Diagnostic terminé"
echo "═══════════════════════════════════════════════════════════════"
