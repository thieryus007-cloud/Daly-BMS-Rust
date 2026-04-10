# Guide de Dépannage MQTT — Pi5 → Node-RED (2026-04-10)

## 🔴 Problème
Aucun message MQTT ne provient du Pi5 vers Node-RED depuis le dernier changement.

## ✅ Vérifications Préalables
- Mosquitto (NanoPi) fonctionne bien
- Autres services RS485 fonctionnent bien
- Donc le problème vient de **daly-bms-server (Pi5)** qui ne publie plus

---

## Processus de Dépannage Systématique

### Étape 1 — État du Service daly-bms-server (Pi5)

**Commande** :
```bash
sudo systemctl status daly-bms
```

**Interprétation** :
- `active (running)` → service tourne ✓
- `inactive` → service arrêté ✗
- `failed` → service a crashé ✗

**Si inactif/failed** :
```bash
# Voir les 50 dernières lignes d'erreur
journalctl -u daly-bms -n 50

# Redémarrer et observer les logs
sudo systemctl restart daly-bms
sleep 2
journalctl -u daly-bms -f   # (Ctrl+C pour quitter)
```

---

### Étape 2 — Vérifier la Config MQTT (Pi5)

**Fichiers** :
- Config repo : `/home/pi5compute/Daly-BMS-Rust/Config.toml`
- Config production : `/etc/daly-bms/config.toml` ← **celle-ci est utilisée par le service**

**Vérifier que la production est à jour** :
```bash
# Option A — copier depuis le repo
sudo cp /home/pi5compute/Daly-BMS-Rust/Config.toml /etc/daly-bms/config.toml

# Redémarrer le service
sudo systemctl restart daly-bms
sleep 2
```

**Afficher la section MQTT** :
```bash
cat /etc/daly-bms/config.toml | grep -A5 "\[mqtt\]"
```

**Doit afficher** :
```toml
[mqtt]
host = "192.168.1.120"    # IP du NanoPi (broker)
port = 1883               # port MQTT
topic = "santuario/bms"   # préfixe topics
format = "venus"          # format payload
```

**Si le host/port est incorrect** → corriger et redémarrer :
```bash
# Éditer la config
sudo nano /etc/daly-bms/config.toml

# Redémarrer
sudo systemctl restart daly-bms
```

---

### Étape 3 — Logs daly-bms-server (Pi5)

**Afficher les logs en temps réel** :
```bash
journalctl -u daly-bms -f
```

**Chercher les erreurs MQTT** :
- `connection refused` → broker MQTT ne répond pas
- `timeout` → broker inaccessible (firewall / réseau)
- `Cannot connect to MQTT` → erreur connexion
- `Published to` → messages publiés ✓

**Chercher les erreurs RS485** :
- `ttyUSB0` → voir si port série est accessible
- `timeout` → appareils RS485 ne répondent pas

**Quitter les logs** : `Ctrl+C`

---

### Étape 4 — Vérifier la Connectivité Réseau Pi5 → NanoPi

**Ping NanoPi depuis Pi5** :
```bash
ping -c 3 192.168.1.120
```

**Doit répondre** :
```
PING 192.168.1.120 (192.168.1.120) 56(84) bytes of data.
64 bytes from 192.168.1.120: icmp_seq=1 ttl=64 time=5.23 ms
```

**Si pas de réponse** → problème réseau, vérifier câble Ethernet

---

### Étape 5 — Vérifier que le Broker MQTT (NanoPi) Reçoit les Messages

**Depuis le Pi5, s'abonner au broker MQTT** :
```bash
mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/#' -v
```

**Doit afficher les messages MQTT** :
```
santuario/bms/1/venus {"Voltage": 55.2, ...}
santuario/bms/2/venus {"Voltage": 48.1, ...}
```

**Si rien n'apparaît** → daly-bms-server ne publie pas

**Quitter** : `Ctrl+C`

---

### Étape 6 — Vérifier le Port Série (Pi5)

**Lister les appareils connectés** :
```bash
ls -la /dev/ttyUSB*
```

**Doit afficher** :
```
/dev/ttyUSB0 → BMS (RS485)
/dev/ttyUSB1 → ET112 ou capteur irradiance
```

**Vérifier les permissions** :
```bash
sudo usermod -a -G dialout pi5compute
```

**Puis se reconnecter SSH ou redémarrer** :
```bash
sudo reboot
```

---

### Étape 7 — Recompiler daly-bms-server (si code a changé)

**Compilation** :
```bash
cd /home/pi5compute/Daly-BMS-Rust
make build-arm
```

**Dépannage erreur de compilation** :
```bash
# Voir le dernier commit
git log --oneline -1

# Voir les changements
git diff HEAD~1

# Si erreur Rust, forcer un clean build
cargo clean
make build-arm
```

**Redéployer** :
```bash
sudo systemctl stop daly-bms
sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
sudo systemctl start daly-bms
sleep 2
journalctl -u daly-bms -f
```

---

### Étape 8 — Vérifier depuis Node-RED (Pi5)

**Accès Node-RED** : `http://192.168.1.141:1880`

**Vérifier les nœuds MQTT** :
1. Cliquer sur un nœud **mqtt in** ou **mqtt out**
2. Afficher les propriétés (double-clic)
3. Vérifier :
   - **Broker** : `192.168.1.120:1883` ✓
   - **Status** du nœud : doit être vert (connecté)
   - **Topics** : `santuario/bms/+/venus`, etc.

**Si nœud MQTT rouge** :
- Double-clic → **Serveur MQTT**
- Vérifier IP/port
- Cliquer **Redéployer** (en haut à droite)

---

## 🔧 Checklist de Dépannage Complet

### Niveau 1 — État global
- [ ] `sudo systemctl status daly-bms` → **active (running)**
- [ ] `ping 192.168.1.120` → répond ✓
- [ ] `journalctl -u daly-bms -f` → pas d'erreurs de connexion MQTT

### Niveau 2 — Configuration
- [ ] `/etc/daly-bms/config.toml` contient `host = "192.168.1.120"`
- [ ] `/etc/daly-bms/config.toml` contient `port = 1883`
- [ ] `/etc/daly-bms/config.toml` contient `format = "venus"`

### Niveau 3 — Connectivité MQTT
- [ ] `mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/#' -v` → affiche les messages ✓
- [ ] Node-RED : nœuds MQTT affichés en **vert** (connectés)
- [ ] Dashboard Pi5 : données visibles (ou API `/api/v1/bms`)

### Niveau 4 — Compilation & Binaire
- [ ] Code Rust sans erreurs : `make build-arm`
- [ ] Binaire déployé : `sudo cp target/aarch64.../release/daly-bms-server /usr/local/bin/`
- [ ] Service redémarré : `sudo systemctl restart daly-bms`

### Niveau 5 — RS485 / Port Série
- [ ] `/dev/ttyUSB0` existe et est accessible
- [ ] BMS détectés en polling (logs `journalctl`)
- [ ] Données valides dans API `/api/v1/bms`

---

## 📍 Diagnostics Rapides (1-2 min)

### A — Service tourne-t-il?
```bash
sudo systemctl status daly-bms | grep Active
```
**Réponse attendue** : `Active: active (running)`

### B — Erreurs de connexion?
```bash
journalctl -u daly-bms -n 20 | grep -i "mqtt\|error\|connection"
```
**Réponse attendue** : Aucune ligne MQTT, ou `Published to santuario/...`

### C — Broker accessible?
```bash
timeout 2 bash -c 'echo | nc 192.168.1.120 1883' && echo "✓ Broker accessible" || echo "✗ Broker inaccessible"
```
**Réponse attendue** : `✓ Broker accessible`

### D — Messages publiés?
```bash
timeout 5 mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus'
```
**Réponse attendue** : JSON avec données BMS (Voltage, etc.)

---

## ⚠️ Causes Usuelles

| Symptôme | Cause | Solution |
|---|---|---|
| Service `failed` | Code Rust non compilé ou erreur runtime | `make build-arm` + redéployer |
| Service `inactive` | Pas lancé automatiquement | `sudo systemctl restart daly-bms` |
| Config ignorée | `/etc/daly-bms/config.toml` pas à jour | `sudo cp Config.toml /etc/daly-bms/config.toml` |
| Pas de messages MQTT | Serveur ne démarre pas ou config MQTT fausse | Checker logs + config |
| Broker inaccessible | Réseau ou firewall | `ping 192.168.1.120` |
| Port série /dev/ttyUSB0 absent | BMS non connecté ou USB débranché | Vérifier câbles USB |
| Anciennes données dans Node-RED | Cache MQTT retained | `make reset` (destructif) ou nettoyer manuellement |

---

## 📊 Workflow Mise à Jour Complet

```bash
# 1. Sur Claude Code / Pi5 — récupérer code à jour
git pull origin main

# 2. Compiler
make build-arm

# 3. Redéployer
sudo systemctl stop daly-bms
sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
sudo cp Config.toml /etc/daly-bms/config.toml
sudo systemctl start daly-bms

# 4. Vérifier
sleep 2
journalctl -u daly-bms -f   # Observer 5-10s
mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus' -C 1
```

---

## 🎯 Prochaines Étapes

1. **Immédiatement** : Exécuter la **Checklist Niveau 1** (60 secondes)
2. **Si niveau 1 OK** : Passer à **Niveau 2-3** (config + connectivité)
3. **Si tout échoue** : Passer à **Niveau 4-5** (compilation + RS485)
4. **Rapporter les résultats** avec les **5 diagnostics rapides (A-E)** pour analyse

---

## ❓ Questions pour Rapporter

Répondre à ces questions aide à diagnostiquer rapidement :

1. **Statut service** :
   ```bash
   sudo systemctl status daly-bms | grep -E "Active|running"
   ```

2. **Dernières erreurs** :
   ```bash
   journalctl -u daly-bms -n 10
   ```

3. **Config MQTT** :
   ```bash
   grep -A3 "\[mqtt\]" /etc/daly-bms/config.toml
   ```

4. **Connectivité** :
   ```bash
   timeout 2 bash -c 'echo | nc 192.168.1.120 1883' && echo OK || echo FAILED
   ```

5. **Messages MQTT** :
   ```bash
   timeout 3 mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus'
   ```
