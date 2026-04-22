#!/bin/bash
#
# Script d'import du dashboard Grafana Santuario
# Usage: ./scripts/import-grafana-dashboard.sh
#

set -e

# Couleurs pour output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
GRAFANA_URL="${GRAFANA_URL:-http://localhost:3001}"
GRAFANA_ADMIN_USER="${GRAFANA_ADMIN_USER:-admin}"
GRAFANA_ADMIN_PASSWORD="${GRAFANA_ADMIN_PASSWORD:-admin}"
DASHBOARD_FILE="grafana/dashboards/santuario-solar-system.json"
DASHBOARD_UID="santuario"
DATASOURCE_NAME="InfluxDB"

echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Script d'import du Dashboard Grafana - Santuario         ║${NC}"
echo -e "${BLUE}║  Solar System Visualization                               ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
echo

# Vérifications préalables
echo -e "${YELLOW}[1/5] Vérification des fichiers...${NC}"
if [ ! -f "$DASHBOARD_FILE" ]; then
    echo -e "${RED}❌ Erreur: Fichier dashboard introuvable: $DASHBOARD_FILE${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Fichier dashboard trouvé${NC}"
echo

# Vérifier la connectivité à Grafana
echo -e "${YELLOW}[2/5] Vérification de la connexion à Grafana (${GRAFANA_URL})...${NC}"
if ! curl -s -f -o /dev/null -w "%{http_code}" "$GRAFANA_URL/api/health" | grep -q "^200$"; then
    echo -e "${RED}❌ Erreur: Impossible de se connecter à Grafana (${GRAFANA_URL})${NC}"
    echo -e "${YELLOW}   Assurez-vous que Grafana est accessible à cette URL.${NC}"
    echo -e "${YELLOW}   Vous pouvez spécifier une URL différente:${NC}"
    echo -e "${YELLOW}   GRAFANA_URL=http://192.168.1.141:3001 ./scripts/import-grafana-dashboard.sh${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Grafana accessible${NC}"
echo

# Vérifier la data source InfluxDB
echo -e "${YELLOW}[3/5] Vérification de la Data Source '${DATASOURCE_NAME}'...${NC}"

# Récupérer le token admin
ADMIN_TOKEN=$(curl -s -X POST "$GRAFANA_URL/api/auth/login" \
    -H "Content-Type: application/json" \
    -d "{\"user\":\"$GRAFANA_ADMIN_USER\",\"password\":\"$GRAFANA_ADMIN_PASSWORD\"}" \
    | grep -o '"token":"[^"]*' | sed 's/"token":"//' || echo "")

if [ -z "$ADMIN_TOKEN" ]; then
    echo -e "${RED}❌ Erreur: Impossible de s'authentifier à Grafana${NC}"
    echo -e "${YELLOW}   Vérifiez les identifiants par défaut (admin/admin)${NC}"
    echo -e "${YELLOW}   Vous pouvez spécifier des identifiants:${NC}"
    echo -e "${YELLOW}   GRAFANA_ADMIN_USER=admin GRAFANA_ADMIN_PASSWORD=password ./scripts/import-grafana-dashboard.sh${NC}"
    exit 1
fi

DATASOURCE=$(curl -s "$GRAFANA_URL/api/datasources/byName/$DATASOURCE_NAME" \
    -H "Authorization: Bearer $ADMIN_TOKEN")

if echo "$DATASOURCE" | grep -q '"id"'; then
    echo -e "${GREEN}✅ Data Source '${DATASOURCE_NAME}' trouvée${NC}"
else
    echo -e "${YELLOW}⚠️  Data Source '${DATASOURCE_NAME}' non trouvée${NC}"
    echo -e "${YELLOW}   Elle devra être créée manuellement${NC}"
fi
echo

# Importer le dashboard
echo -e "${YELLOW}[4/5] Import du dashboard Grafana...${NC}"

RESPONSE=$(curl -s -X POST "$GRAFANA_URL/api/dashboards/db" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d @"$DASHBOARD_FILE")

DASHBOARD_ID=$(echo "$RESPONSE" | grep -o '"id":[0-9]*' | head -1 | sed 's/"id"://')

if [ -n "$DASHBOARD_ID" ]; then
    echo -e "${GREEN}✅ Dashboard importé avec succès${NC}"
    echo -e "${GREEN}   ID: ${DASHBOARD_ID}${NC}"
else
    ERROR=$(echo "$RESPONSE" | grep -o '"message":"[^"]*' | sed 's/"message":"//' | head -1)
    echo -e "${YELLOW}⚠️  Réponse: ${ERROR}${NC}"
    echo -e "${YELLOW}   Le dashboard peut déjà exister (c'est normal)${NC}"
fi
echo

# Vérifier l'accès
echo -e "${YELLOW}[5/5] Vérification du dashboard...${NC}"

DASHBOARD_CHECK=$(curl -s "$GRAFANA_URL/api/dashboards/uid/$DASHBOARD_UID" \
    -H "Authorization: Bearer $ADMIN_TOKEN")

if echo "$DASHBOARD_CHECK" | grep -q '"uid":"santuario"'; then
    echo -e "${GREEN}✅ Dashboard accessible${NC}"
    echo
    echo -e "${BLUE}╔════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║                   IMPORT RÉUSSI! ✨                         ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════════════╝${NC}"
    echo
    echo -e "📊 Dashboard accessible à:"
    echo -e "   ${GREEN}${GRAFANA_URL}/d/${DASHBOARD_UID}/santuario-solar-system${NC}"
    echo
    echo -e "🌐 Application Web:"
    echo -e "   ${GREEN}http://192.168.1.141:8080/dashboard/grafana${NC}"
    echo
    echo -e "📋 Prochaines étapes:"
    echo -e "   1. Vérifiez que les données apparaissent dans les panneaux"
    echo -e "   2. Si vide: Vérifiez que daly-bms-server écrit dans InfluxDB"
    echo -e "   3. Consultez GRAFANA_SETUP.md pour le troubleshooting"
    echo
else
    echo -e "${YELLOW}⚠️  Dashboard non trouvé en vérification${NC}"
    echo -e "${YELLOW}   Vérifiez l'import manuellement: ${GRAFANA_URL}/dashboards${NC}"
fi
