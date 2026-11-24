#!/usr/bin/env bash
# =============================================================================
# diag-bms-trames.sh — diagnostic des trames RS485 des BMS Daly (Pi5)
#
# Capture les octets bruts échangés avec les BMS (journal daly_bms_core::bus au
# niveau trace) pendant N secondes, revient AUTOMATIQUEMENT au niveau info (même
# en cas d'interruption), puis affiche, par BMS et par commande, le nombre de
# trames valides / altérées et, pour les trames altérées, les octets qui
# diffèrent de la trame valide reçue juste après (nouvelle tentative).
#
# Effet en production : deux redémarrages de daly-bms (quelques secondes sans
# données BMS chacun). Rien n'est modifié dans le code ni dans la config.
# Refuse de démarrer si la base de métriques n'est pas prête ou si la 1re passe
# de maintenance depuis le dernier démarrage n'est pas terminée.
#
# Usage (sur le Pi5, depuis le dépôt) :
#   sudo bash scripts/diag-bms-trames.sh          # capture de 120 s
#   sudo bash scripts/diag-bms-trames.sh 60       # capture de 60 s
#
# Lecture du résultat : docs/diagnostic-depannage.md (« Erreurs Checksum BMS »).
# =============================================================================
set -euo pipefail

DUR="${1:-120}"
DROPIN_DIR=/etc/systemd/system/daly-bms.service.d
DROPIN="$DROPIN_DIR/zz-diag-trace.conf"
OUT="/tmp/diag-bms-trames-$(date +%Y%m%d-%H%M%S).log"

[[ $EUID -eq 0 ]] || { echo "✗ Lancer avec sudo" >&2; exit 1; }
[[ "$DUR" =~ ^[0-9]+$ ]] && (( DUR >= 10 && DUR <= 300 )) || { echo "✗ durée entre 10 et 300 s" >&2; exit 1; }

# Préconditions : ne JAMAIS redémarrer daly-bms pendant l'ouverture / la récupération de la
# base de métriques ni pendant la 1re passe de maintenance (≈ 20 min après un démarrage) :
# l'arrêt attendrait leur fin (TimeoutStopSec) ou, s'il était tué, la base repartirait en
# récupération (cf. contrib/daly-bms.service).
code=$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:8080/api/v1/redb/healthy || true)
if [[ "$code" != "200" ]]; then
    echo "✗ base de métriques pas prête (HTTP $code : ouverture ou récupération en cours) — réessayer plus tard" >&2
    exit 1
fi
SINCE="$(systemctl show -p ActiveEnterTimestamp --value daly-bms)"
if ! journalctl -u daly-bms --since "$SINCE" --no-pager | grep -q -E "compact hourly→daily (OK|failed|interrompue)"; then
    echo "✗ 1re passe de maintenance pas terminée depuis le démarrage ($SINCE) — attendre (≈ 20 min)" >&2
    exit 1
fi

revert() {
    rm -f "$DROPIN"
    rmdir "$DROPIN_DIR" 2>/dev/null || true
    systemctl daemon-reload
    systemctl restart daly-bms
    sleep 3
    echo "→ retour au niveau normal : $(systemctl show -p Environment daly-bms | grep -o 'RUST_LOG=[^ ]*'), daly-bms $(systemctl is-active daly-bms)"
}
trap revert EXIT

mkdir -p "$DROPIN_DIR"
printf '[Service]\nEnvironment=RUST_LOG=info,daly_bms_core::bus=trace\n' > "$DROPIN"
systemctl daemon-reload
systemctl restart daly-bms
T0="$(date '+%F %T')"
echo "→ capture des trames BMS pendant ${DUR} s (depuis $T0)…"
sleep "$DUR"
journalctl -u daly-bms --since "$T0" --no-pager -o cat | sed 's/\x1b\[[0-9;]*m//g' > "$OUT"
trap - EXIT
revert

python3 - "$OUT" <<'PY'
import collections, re, sys

lines = [l.rstrip("\n") for l in open(sys.argv[1], encoding="utf-8", errors="replace")]

def field(line, key):
    m = re.search(key + r'="([^"]*)"', line)
    return m.group(1) if m else None

def parse(raw):
    return [int(x, 16) for x in raw.strip("[]").split(", ")] if raw else None

def valid(b):
    return b is not None and len(b) == 13 and b[0] == 0xA5 and (sum(b[:12]) & 0xFF) == b[12]

rx = []  # (index, bms, cmd, octets, valide)
for i, l in enumerate(lines):
    if "réponse reçue" in l or "trame multi reçue" in l:
        b = parse(field(l, "raw"))
        rx.append((i, field(l, "bms"), field(l, "cmd"), b, valid(b)))

per_bms = collections.Counter((bms, ok) for _, bms, _, _, ok in rx)
print("\n== Trames reçues par BMS (valides / altérées)")
for bms in sorted({k[0] for k in per_bms}):
    ok, ko = per_bms[(bms, True)], per_bms[(bms, False)]
    print(f"   BMS {bms} : {ok:5d} valides  {ko:5d} altérées  ({100 * ko / max(1, ok + ko):.0f} %)")

bad = [r for r in rx if not r[4]]
if not bad:
    print("\nAucune trame altérée pendant la capture.")
    sys.exit(0)
print("\n== Trames altérées par (BMS, commande) :", dict(collections.Counter((r[1], r[2]) for r in bad)))

print("\n== Exemples : trame altérée → octets différents de la trame valide suivante (même BMS / commande)")
zero_to_one = one_to_zero = 0
for idx, bms, cmd, b, _ in bad[:12]:
    nxt = next((r[3] for r in rx if r[0] > idx and r[1] == bms and r[2] == cmd and r[4]), None)
    if nxt and b and len(b) == 13:
        diffs = [(x, y) for x, y in zip(nxt, b) if x != y]
        zero_to_one += sum(bin(~x & y & 0xFF).count("1") for x, y in diffs)
        one_to_zero += sum(bin(x & ~y & 0xFF).count("1") for x, y in diffs)
        d = " ".join(f"{x:02X}>{y:02X}" for x, y in diffs)
    else:
        d = "?"
    print(f"   BMS {bms} cmd {cmd} : {' '.join(f'{x:02X}' for x in b or [])}   écarts : {d}")
print(f"\n   Bits altérés dans ces exemples : 0→1 = {zero_to_one}, 1→0 = {one_to_zero}")
print("   (bits isolés altérés sur un seul BMS = signal altéré entre ce BMS et le Pi5 ;"
      " trame décalée / autre Data ID sur tous les BMS = désynchronisation logicielle)")
PY
echo "Trames brutes : $OUT"
