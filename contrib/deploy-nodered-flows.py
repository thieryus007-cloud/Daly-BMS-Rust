#!/usr/bin/env python3
"""
deploy-nodered-flows.py
Déploie les flows Node-RED depuis flux-nodered/All-nodered-flow.json.

Stratégie en deux temps :
  1. Essai via l'API REST POST /flows (déploiement à chaud, sans redémarrage).
  2. Si l'API échoue (EBUSY ou autre), repli sur : stop → docker cp → start.
     Cette méthode est fiable sur tout type de volume Docker.

Usage :
  make deploy-nodered
  NODERED_URL=http://192.168.1.141:1880 python3 contrib/deploy-nodered-flows.py
"""

import json
import os
import subprocess
import sys
import urllib.request
import urllib.error
import time

NODERED_URL       = os.environ.get("NODERED_URL", "http://localhost:1880")
NODERED_CONTAINER = os.environ.get("NODERED_CONTAINER", "dalybms-nodered")
COMPOSE_FILE      = os.path.join(os.path.dirname(__file__), "..", "docker-compose.infra.yml")
FLOWS_DIR         = os.path.join(os.path.dirname(__file__), "..", "flux-nodered")
ALL_FLOW_FILE     = os.path.join(FLOWS_DIR, "All-nodered-flow.json")


# ─────────────────────────────────────────────────────────────────────────────
# Lecture du fichier source
# ─────────────────────────────────────────────────────────────────────────────

def load_git_flows():
    """Lit UNIQUEMENT All-nodered-flow.json — source de vérité unique."""
    if not os.path.exists(ALL_FLOW_FILE):
        print(f"ERREUR : {ALL_FLOW_FILE} introuvable")
        sys.exit(1)
    try:
        with open(ALL_FLOW_FILE, encoding="utf-8") as f:
            nodes = json.load(f)
        if not isinstance(nodes, list):
            print("ERREUR : All-nodered-flow.json n'est pas un tableau JSON")
            sys.exit(1)
        print(f"  [OK]   All-nodered-flow.json ({len(nodes)} nœuds)")
        return nodes
    except Exception as e:
        print(f"ERREUR lecture All-nodered-flow.json : {e}")
        sys.exit(1)


# ─────────────────────────────────────────────────────────────────────────────
# Méthode 1 : API REST (déploiement à chaud)
# ─────────────────────────────────────────────────────────────────────────────

def wait_for_nodered(max_wait=60):
    """Attend que Node-RED soit disponible."""
    print(f"Attente de Node-RED ({NODERED_URL})...", end="", flush=True)
    for _ in range(max_wait):
        try:
            urllib.request.urlopen(f"{NODERED_URL}/flows", timeout=2)
            print(" OK")
            return True
        except Exception:
            print(".", end="", flush=True)
            time.sleep(1)
    print("\n  Node-RED n'a pas répondu dans les délais.")
    return False


def post_flows_api(flows):
    """
    Envoie les flows via POST /flows.
    Retourne True si succès, False si EBUSY ou erreur récupérable.
    """
    body = json.dumps(flows).encode("utf-8")
    req  = urllib.request.Request(f"{NODERED_URL}/flows", data=body, method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Node-RED-Deployment-Type", "full")
    try:
        with urllib.request.urlopen(req, timeout=30) as resp:
            print(f"  API REST → HTTP {resp.status} ✓")
            return True
    except urllib.error.HTTPError as e:
        body_txt = e.read().decode()
        if "EBUSY" in body_txt or e.code == 400:
            print(f"  API REST → HTTP {e.code} ({body_txt[:80]}) — repli sur docker cp")
            return False
        print(f"ERREUR HTTP {e.code} : {body_txt}")
        sys.exit(1)
    except urllib.error.URLError as e:
        print(f"ERREUR réseau : {e}")
        return False


# ─────────────────────────────────────────────────────────────────────────────
# Méthode 2 : docker cp (fiable même si le volume pose problème)
# ─────────────────────────────────────────────────────────────────────────────

def _run(cmd, check=True):
    print(f"  $ {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True)
    if check and result.returncode != 0:
        print(f"ERREUR : {result.stderr.strip()}")
        sys.exit(1)
    return result


def deploy_via_docker_cp():
    """
    Arrête Node-RED, écrit flows.json via docker cp, redémarre.
    Contourne totalement l'API et le rename() qui cause EBUSY.
    """
    print("  Arrêt de Node-RED...")
    _run(["docker", "compose", "-f", os.path.abspath(COMPOSE_FILE), "stop", "nodered"])

    print("  Copie de flows.json dans le container (docker cp)...")
    _run(["docker", "cp", os.path.abspath(ALL_FLOW_FILE),
          f"{NODERED_CONTAINER}:/data/flows.json"])

    print("  Démarrage de Node-RED...")
    _run(["docker", "compose", "-f", os.path.abspath(COMPOSE_FILE), "up", "-d", "nodered"])

    print("  Attente du démarrage...")
    time.sleep(15)
    if not wait_for_nodered(max_wait=60):
        print("AVERTISSEMENT : Node-RED n'est pas encore prêt — vérifier les logs (make logs)")
    else:
        print("  Node-RED démarré avec les nouveaux flows ✓")


# ─────────────────────────────────────────────────────────────────────────────
# Main
# ─────────────────────────────────────────────────────────────────────────────

def main():
    print("=" * 60)
    print("Déploiement des flows Node-RED depuis git")
    print(f"  Source  : {os.path.abspath(ALL_FLOW_FILE)}")
    print(f"  Cible   : {NODERED_URL}")
    print("=" * 60)

    print("\nLecture des flows depuis git :")
    git_nodes = load_git_flows()
    print(f"  Total : {len(git_nodes)} nœuds")

    # ── Tentative 1 : API REST (à chaud) ────────────────────────────────────
    print(f"\n[1/2] Déploiement via API REST...")
    if wait_for_nodered(max_wait=30):
        if post_flows_api(git_nodes):
            print(f"\n{'='*60}")
            print(f"✓ Flows déployés via API REST")
            print(f"  http://192.168.1.141:1880")
            print("=" * 60)
            return

    # ── Tentative 2 : docker cp (stop → cp → start) ──────────────────────────
    print(f"\n[2/2] Déploiement via docker cp (stop → cp → start)...")
    deploy_via_docker_cp()

    print(f"\n{'='*60}")
    print(f"✓ Flows déployés via docker cp")
    print(f"  http://192.168.1.141:1880")
    print("=" * 60)


if __name__ == "__main__":
    main()
