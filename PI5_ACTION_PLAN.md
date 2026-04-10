# Plan d'Action — Diagnostic et Restauration MQTT (Pi5)

## 🎯 Objectif
Restaurer la publication MQTT depuis daly-bms-server vers Node-RED.

---

## Phase 1 — Diagnostic Rapide (1 min)

**Exécuter sur le Pi5** :

```bash
cd ~/Daly-BMS-Rust

# Rendre le script executable
chmod +x QUICK_MQTT_DIAGNOSIS.sh

# Lancer le diagnostic
./QUICK_MQTT_DIAGNOSIS.sh
```

**Noter les résultats** :
- ✓ Service tourne ?
- ✓ Broker accessible ?
- ✓ Messages MQTT reçus ?

---

## Phase 2 — Actions Selon le Diagnostic

### Scénario A : Service actif, mais aucun message MQTT

**Cause probable** : Code Rust non compilé ou config stale

**Actions** :
```bash
cd ~/Daly-BMS-Rust

# 1. Récupérer le code à jour
git pull origin main

# 2. Vérifier la config
cat /etc/daly-bms/config.toml | grep -A5 "\[mqtt\]"

# 3. Copier la config à jour
sudo cp Config.toml /etc/daly-bms/config.toml

# 4. Redémarrer le service
sudo systemctl restart daly-bms
sleep 2

# 5. Vérifier les logs
journalctl -u daly-bms -f   # (Ctrl+C après 10s)

# 6. Tester les messages MQTT
timeout 3 mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus'
```

---

### Scénario B : Service inactif/failed

**Cause probable** : Binaire crashé ou erreur au démarrage

**Actions** :
```bash
cd ~/Daly-BMS-Rust

# 1. Voir l'erreur complète
journalctl -u daly-bms -n 50

# 2. Recompiler le binaire
make build-arm
# ⏱️ ~5-10 minutes, c'est normal

# 3. Redéployer
sudo systemctl stop daly-bms
sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
sudo cp Config.toml /etc/daly-bms/config.toml
sudo systemctl start daly-bms

# 4. Vérifier
sleep 2
journalctl -u daly-bms -f   # Ctrl+C après 10s
```

---

### Scénario C : Broker inaccessible

**Cause probable** : Réseau ou NanoPi arrêté

**Actions** :
```bash
# 1. Tester connectivité
ping -c 3 192.168.1.120

# 2. Vérifier que Mosquitto tourne sur NanoPi
ssh root@192.168.1.120 "docker ps | grep mosquitto" 2>/dev/null || \
  ssh root@192.168.1.120 "ps aux | grep mosquitto"

# 3. Si Mosquitto n'est pas actif sur NanoPi, le relancer depuis Pi5
make up

# 4. Attendre ~10s et tester
sleep 10
timeout 2 bash -c 'echo | nc 192.168.1.120 1883' && echo "OK" || echo "FAILED"
```

---

## Phase 3 — Vérification Complète

Une fois les actions correctives effectuées, exécuter cette **checklist finale** :

```bash
# 1. Service tourne
sudo systemctl status daly-bms | grep Active

# 2. Port série visible
ls -la /dev/ttyUSB0

# 3. BMS détectés (vérifier les logs)
journalctl -u daly-bms -n 20 | grep -i "bms\|address"

# 4. Broker MQTT accessible
timeout 2 bash -c 'echo | nc 192.168.1.120 1883' && echo "✓ Broker OK"

# 5. Messages MQTT reçus
timeout 5 mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus' && echo "✓ MQTT OK"

# 6. Dashboard Pi5 accessible
curl -s http://localhost:8080/api/v1/bms | jq '.bms_count' && echo "✓ API OK"

# 7. Node-RED reçoit les données
# Aller sur http://192.168.1.141:1880
# Vérifier qu'un nœud debug affiche les messages MQTT
```

---

## Phase 4 — Si Aucun Scénario Ne Correspond

**Collecter les informations détaillées** :

```bash
# Copier-coller les résultats de ces commandes
echo "=== [1] État du service ==="
sudo systemctl status daly-bms

echo ""
echo "=== [2] Logs complets (30 lignes) ==="
journalctl -u daly-bms -n 30

echo ""
echo "=== [3] Config MQTT ==="
cat /etc/daly-bms/config.toml | grep -A10 "\[mqtt\]"

echo ""
echo "=== [4] Port série ==="
ls -la /dev/ttyUSB*

echo ""
echo "=== [5] Connectivité réseau ==="
ping -c 2 192.168.1.120

echo ""
echo "=== [6] Version du binaire ==="
/usr/local/bin/daly-bms-server --version 2>&1 || echo "pas de --version"
```

Puis **rapporter ces informations** pour investigation plus approfondie.

---

## ⏱️ Temps Estimé

- **Diagnostic rapide** : 1 min
- **Copie config + redémarrage** : 30 sec
- **Recompilation** : 5-10 min
- **Redéploiement** : 30 sec
- **Vérification** : 1 min

**Total en cas de recompilation** : ~15 minutes

---

## 📋 Résumé Actions Prioritaires (par ordre)

1. ✅ Exécuter `./QUICK_MQTT_DIAGNOSIS.sh`
2. ✅ Si scénario A ou B → appliquer les actions correspondantes
3. ✅ Relancer le diagnostic pour valider
4. ✅ Vérifier Node-RED reçoit les messages (url: http://192.168.1.141:1880)

---

## 🔗 Ressources

- Guide complet : `MQTT_DEBUGGING_GUIDE.md`
- Commandes de reference : `CLAUDE.md` (section 0A-0E)
- Logs production : `journalctl -u daly-bms -f`
