#!/usr/bin/env python3
"""
deploy-nodered-flows.py
Déploie les flows Node-RED depuis flux-nodered/*.json vers Node-RED via l'API REST.

GIT = SOURCE DE VÉRITÉ : les flows git remplacent ENTIÈREMENT Node-RED.
Aucune fusion, aucun nœud "conservé" depuis l'état courant.

Usage :
  python3 contrib/deploy-nodered-flows.py
  NODERED_URL=http://192.168.1.141:1880 python3 contrib/deploy-nodered-flows.py

Principe :
  1. Lit flux-nodered/*.json → flows git (déduplication par ID)
  2. POST /flows         → remplacement complet (équivalent "Deploy All")

Pas de redémarrage Docker nécessaire.
"""

import json
import os
import glob
import sys
import urllib.request
import urllib.error
import time

NODERED_URL = os.environ.get("NODERED_URL", "http://localhost:1880")
FLOWS_DIR   = os.path.join(os.path.dirname(__file__), "..", "flux-nodered")


def post_flows(flows):
    """Envoie les flows à Node-RED et déclenche un déploiement complet."""
    body = json.dumps(flows).encode("utf-8")
    req  = urllib.request.Request(f"{NODERED_URL}/flows", data=body, method="POST")
    req.add_header("Content-Type", "application/json")
    req.add_header("Node-RED-Deployment-Type", "full")
    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            return resp.status
    except urllib.error.HTTPError as e:
        print(f"ERREUR HTTP {e.code} lors du déploiement : {e.read().decode()}")
        sys.exit(1)
    except urllib.error.URLError as e:
        print(f"ERREUR réseau lors du déploiement : {e}")
        sys.exit(1)


def load_git_flows():
    """Lit UNIQUEMENT All-nodered-flow.json — source de vérité unique."""
    all_flow = os.path.join(FLOWS_DIR, "All-nodered-flow.json")
    if not os.path.exists(all_flow):
        print(f"ERREUR : {all_flow} introuvable")
        sys.exit(1)
    try:
        with open(all_flow, encoding="utf-8") as f:
            nodes = json.load(f)
        if not isinstance(nodes, list):
            print(f"ERREUR : All-nodered-flow.json n'est pas un tableau JSON")
            sys.exit(1)
        print(f"  [OK]   All-nodered-flow.json ({len(nodes)} nœuds)")
        return nodes
    except Exception as e:
        print(f"ERREUR lecture All-nodered-flow.json : {e}")
        sys.exit(1)


def wait_for_nodered(max_wait=30):
    """Attend que Node-RED soit disponible."""
    print(f"Attente de Node-RED ({NODERED_URL})...", end="", flush=True)
    for _ in range(max_wait):
        try:
            urllib.request.urlopen(f"{NODERED_URL}", timeout=2)
            print(" OK")
            return
        except Exception:
            print(".", end="", flush=True)
            time.sleep(1)
    print("\nErreur : Node-RED n'a pas répondu dans les délais.")
    sys.exit(1)


def main():
    print("=" * 60)
    print("Déploiement des flows Node-RED depuis git")
    print(f"  Source  : {os.path.abspath(FLOWS_DIR)}")
    print(f"  Cible   : {NODERED_URL}")
    print("  Mode    : REMPLACEMENT COMPLET (git = source de vérité)")
    print("=" * 60)

    wait_for_nodered()

    print("\nLecture des flows depuis git :")
    git_nodes = load_git_flows()
    print(f"  Total : {len(git_nodes)} nœuds uniques")

    print(f"\nDéploiement ({len(git_nodes)} nœuds) → Node-RED...")
    status = post_flows(git_nodes)
    print(f"\n{'='*60}")
    print(f"✓ Flows déployés avec succès (HTTP {status})")
    print(f"  Ouvrir http://192.168.1.141:1880 pour vérifier")
    print("=" * 60)


if __name__ == "__main__":
    main()
