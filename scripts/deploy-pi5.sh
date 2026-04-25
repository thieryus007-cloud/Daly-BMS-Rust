#!/usr/bin/env bash
# Déploiement complet Pi5 — daly-bms-server + energy-manager
# Usage (depuis ~/Daly-BMS-Rust sur le Pi5) : bash scripts/deploy-pi5.sh

set -euo pipefail

GREEN='\033[0;32m'; YELLOW='\033[1;33m'; RED='\033[0;31m'; NC='\033[0m'
info()  { echo -e "${GREEN}[OK]${NC} $*"; }
step()  { echo -e "${YELLOW}[>>]${NC} $*"; }
error() { echo -e "${RED}[!!]${NC} $*" >&2; exit 1; }

# ── 1. Récupération du code ───────────────────────────────────────────────────
step "Synchronisation du dépôt…"
make sync || error "make sync a échoué"
info "Code à jour"

# ── 2. Compilation croisée aarch64 ───────────────────────────────────────────
step "Compilation daly-bms-server (aarch64)…"
make build-arm || error "make build-arm a échoué"
info "daly-bms-server compilé"

step "Compilation energy-manager (aarch64)…"
make build-energy-arm || error "make build-energy-arm a échoué"
info "energy-manager compilé"

# ── 3. Mise à jour de la configuration ──────────────────────────────────────
step "Déploiement Config.toml → /etc/daly-bms/config.toml…"
sudo cp Config.toml /etc/daly-bms/config.toml
info "Config.toml déployée"

# ── 4. Déploiement daly-bms-server ───────────────────────────────────────────
step "Déploiement daly-bms-server…"
sudo systemctl stop daly-bms
sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
sudo cp Config.toml /etc/daly-bms/config.toml
sudo systemctl start daly-bms
sleep 2
if systemctl is-active --quiet daly-bms; then
    info "daly-bms actif"
else
    error "daly-bms n'a pas démarré — vérifier : journalctl -u daly-bms -n 50"
fi

# ── 5. Déploiement energy-manager ────────────────────────────────────────────
step "Déploiement energy-manager…"
sudo systemctl stop energy-manager
sudo cp target/aarch64-unknown-linux-gnu/release/energy-manager /usr/local/bin/
sudo systemctl start energy-manager
sleep 2
if systemctl is-active --quiet energy-manager; then
    info "energy-manager actif"
else
    error "energy-manager n'a pas démarré — vérifier : journalctl -u energy-manager -n 50"
fi

# ── 6. Résumé ─────────────────────────────────────────────────────────────────
echo ""
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}  Déploiement terminé avec succès ✓${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
systemctl status daly-bms energy-manager --no-pager -l | grep -E "Active:|Loaded:" || true
