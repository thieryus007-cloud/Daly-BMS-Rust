#!/bin/bash
# Quick diagnostic script for onduleur/smartshunt issues
# Run on Pi5: bash ~/Daly-BMS-Rust/debug.sh

echo "🔍 DIAGNOSTIC RAPIDE - ONDULEUR & SMARTSHUNT"
echo "=============================================="
echo ""

# Check 1: Services running
echo "1️⃣  SERVICES ACTIFS?"
echo "---"
systemctl is-active daly-bms >/dev/null && echo "✅ daly-bms: ACTIF" || echo "❌ daly-bms: INACTIF"
docker ps | grep -q mosquitto && echo "✅ mosquitto: ACTIF" || echo "❌ mosquitto: INACTIF"
docker ps | grep -q nodered && echo "✅ nodered: ACTIF" || echo "❌ nodered: INACTIF"
echo ""

# Check 2: MQTT topics
echo "2️⃣  TOPICS MQTT PUBLIÉS?"
echo "---"
timeout 5 docker exec mosquitto mosquitto_sub -h localhost -p 1883 -t 'santuario/inverter/venus' -C 1 >/dev/null 2>&1 && echo "✅ santuario/inverter/venus: ACTIF" || echo "❌ santuario/inverter/venus: ABSENT"
timeout 5 docker exec mosquitto mosquitto_sub -h localhost -p 1883 -t 'santuario/system/venus' -C 1 >/dev/null 2>&1 && echo "✅ santuario/system/venus: ACTIF" || echo "❌ santuario/system/venus: ABSENT"
echo ""

# Check 3: API endpoints
echo "3️⃣  API ENDPOINTS RÉPONDENT?"
echo "---"
INVERTER=$(curl -s http://localhost:8080/api/v1/venus/inverter 2>/dev/null | jq -r '.connected // "ERROR"' 2>/dev/null)
SMARTSHUNT=$(curl -s http://localhost:8080/api/v1/venus/smartshunt 2>/dev/null | jq -r '.connected // "ERROR"' 2>/dev/null)

case "$INVERTER" in
  "true") echo "✅ /api/v1/venus/inverter: CONNECTÉ" ;;
  "false") echo "⚠️  /api/v1/venus/inverter: DISCONNECTÉ (MQTT pas reçu)" ;;
  *) echo "❌ /api/v1/venus/inverter: ERREUR ($INVERTER)" ;;
esac

case "$SMARTSHUNT" in
  "true") echo "✅ /api/v1/venus/smartshunt: CONNECTÉ" ;;
  "false") echo "⚠️  /api/v1/venus/smartshunt: DISCONNECTÉ (MQTT pas reçu)" ;;
  *) echo "❌ /api/v1/venus/smartshunt: ERREUR ($SMARTSHUNT)" ;;
esac
echo ""

# Check 4: Recent logs
echo "4️⃣  LOGS BMS (dernières erreurs)"
echo "---"
ERRORS=$(journalctl -u daly-bms -n 50 --no-pager 2>/dev/null | grep -i "error\|mqtt\|updated inverter\|updated smartshunt" | head -10)
if [ -z "$ERRORS" ]; then
  echo "✅ Aucune erreur visible"
else
  echo "$ERRORS"
fi
echo ""

# Check 5: Node-RED flows
echo "5️⃣  NODE-RED FLOWS DÉPLOYÉS?"
echo "---"
INVERTER_FLOW=$(curl -s http://localhost:1880/api/flows | jq '.[] | select(.label=="Inverter MultiPlus") | .active' 2>/dev/null)
SMARTSHUNT_FLOW=$(curl -s http://localhost:1880/api/flows | jq '.[] | select(.label=="SmartShunt") | .active' 2>/dev/null)

case "$INVERTER_FLOW" in
  "true") echo "✅ inverter.json: DÉPLOYÉ" ;;
  "false") echo "⚠️  inverter.json: PRÉSENT MAIS INACTIF" ;;
  *) echo "❌ inverter.json: ABSENT" ;;
esac

case "$SMARTSHUNT_FLOW" in
  "true") echo "✅ smartshunt.json: DÉPLOYÉ" ;;
  "false") echo "⚠️  smartshunt.json: PRÉSENT MAIS INACTIF" ;;
  *) echo "❌ smartshunt.json: ABSENT" ;;
esac
echo ""

# Summary
echo "📊 RÉSUMÉ"
echo "---"
if [ "$INVERTER" = "true" ] && [ "$SMARTSHUNT" = "true" ]; then
  echo "✅ TOUT FONCTIONNE - Vérifier le navigateur (cache?)"
  echo ""
  echo "Actions:"
  echo "  1. F5 (refresh) sur le dashboard"
  echo "  2. Vider cache: Ctrl+Shift+Del → tout cocher → Clear"
  echo "  3. Ou accès incognito: http://192.168.1.141:8080/visualization"
elif [ "$INVERTER" = "false" ] || [ "$SMARTSHUNT" = "false" ]; then
  echo "⚠️  MQTT reçu mais AppState vide"
  echo ""
  echo "Actions:"
  echo "  1. Vérifier code compiled récemment"
  echo "  2. make build-arm && sudo systemctl restart daly-bms"
  echo "  3. Attendre 5s puis relancer ce script"
else
  echo "❌ MQTT topics pas publiés"
  echo ""
  echo "Actions:"
  echo "  1. Node-RED: http://192.168.1.141:1880"
  echo "  2. Importer flux-nodered/inverter.json et smartshunt.json"
  echo "  3. Cliquer Deploy (bouton rouge)"
  echo "  4. Attendre 5s puis relancer ce script"
fi
echo ""

echo "📖 Pour investigation complète:"
echo "   Lire: cat ~/Daly-BMS-Rust/DEBUG_ONDULEUR_SMARTSHUNT.md"
