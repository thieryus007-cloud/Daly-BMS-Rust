# ⚡ Restauration MQTT — Actions Immédiates (2026-04-10)

## 🚀 Exécuter MAINTENANT sur le Pi5

```bash
cd ~/Daly-BMS-Rust
chmod +x QUICK_MQTT_DIAGNOSIS.sh
./QUICK_MQTT_DIAGNOSIS.sh
```

### Résultats attendus :

```
[1/5] ✓ Service tourne : Active: active (running)
[2/5] ✓ Pas d'erreurs MQTT dans les logs
[3/5] ✓ Config MQTT : host = "192.168.1.120", port = 1883
[4/5] ✓ Broker accessible : 192.168.1.120:1883
[5/5] ✓ Messages MQTT reçus : JSON avec données BMS
```

---

## 🔴 Si Résultat ≠ Attendu → Actions Correctives

### **Cas 1 : Service N'EST PAS actif**
```bash
sudo systemctl restart daly-bms
sleep 2
journalctl -u daly-bms -f   # attendre 10s puis Ctrl+C
```

### **Cas 2 : Service actif MAIS pas de messages MQTT**
```bash
# Copier la config à jour
sudo cp Config.toml /etc/daly-bms/config.toml

# Redémarrer
sudo systemctl restart daly-bms
sleep 2

# Vérifier
timeout 5 mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus'
```

### **Cas 3 : Broker INACCESSIBLE (192.168.1.120)**
```bash
# Lancer la stack Docker Mosquitto
make up

# Attendre 10s
sleep 10

# Vérifier
timeout 2 bash -c 'echo | nc 192.168.1.120 1883' && echo "✓ OK"
```

### **Cas 4 : Aucun des cas précédents ne fonctionne**
```bash
# Recompiler le binaire (⏱️ ~5-10 min)
make build-arm

# Redéployer
sudo systemctl stop daly-bms
sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
sudo cp Config.toml /etc/daly-bms/config.toml
sudo systemctl start daly-bms

# Vérifier
sleep 2
journalctl -u daly-bms -f   # Ctrl+C après 10s
timeout 5 mosquitto_sub -h 192.168.1.120 -p 1883 -t 'santuario/bms/1/venus'
```

---

## ✅ Vérification Finale

Une fois les messages MQTT reçus, exécuter :

```bash
# Test Node-RED via API
curl -s http://localhost:8080/api/v1/bms | jq '.bms_count'
# Doit afficher : 2

# Vérifier Node-RED reçoit les données
# Ouvrir : http://192.168.1.141:1880
# Vérifier les nœuds MQTT (doivent être VERTS)
```

---

## 📞 Si Rien Ne Fonctionne

**Collecter les logs** :
```bash
echo "=== [1] État ==="
sudo systemctl status daly-bms

echo ""
echo "=== [2] Logs (30 lignes) ==="
journalctl -u daly-bms -n 30

echo ""
echo "=== [3] Config MQTT ==="
cat /etc/daly-bms/config.toml | grep -A10 "\[mqtt\]"

echo ""
echo "=== [4] Connectivité ==="
ping -c 2 192.168.1.120
timeout 2 bash -c 'echo | nc 192.168.1.120 1883' && echo "MQTT OK" || echo "MQTT FAILED"
```

**Puis rapporter** les résultats pour investigation.

---

## ⏱️ Temps Estimé

- **Diagnostic rapide** : 1-2 min
- **Cas 1-3** : 30 sec - 2 min
- **Cas 4** (recompilation) : 5-10 min

**Si rien ne marche : max 5-10 min pour collecter les logs**

---

## 💡 Cause Probable (Hypothèse)

Derniers changements :
- ✅ `Config.toml` modifié (section ATS)
- ✅ Fichiers de documentation modifiés
- ❓ Code Rust **inchangé** depuis le dernier commit

**→ Le problème vient probablement d'une configuration stale** ou **le service n'a pas été redémarré après un changement**. 

**Solution la plus probable : Cas 2 (copie config + redémarrage)**

---

## 🎯 Prochaines Étapes Après Restauration

1. Valider que Node-RED reçoit les messages
2. Vérifier le dashboard Pi5
3. Créer une issue/commit si un bug est découvert
4. Mettre à jour CLAUDE.md avec la procédure

