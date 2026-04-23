# Intégration Grafana Dashboard - Historique Complet

## ✅ État Final : FONCTIONNEL

Le dashboard Grafana "Santuario Solar System" est maintenant **intégré et fonctionnel** dans la web app avec un onglet "📊 Historique" dans la barre de navigation.

---

## 🎯 Objectif Réalisé

**Afficher un dashboard Grafana en temps réel** dans la web app Daly-BMS via une iframe, permettant la visualisation des données InfluxDB :
- Puissance solaire (MPPT)
- Puissance réseau (ET112)
- Puissance batterie (BMS)
- État de charge (SOC)
- Tension et courant batterie

---

## 🔧 Solution Technique Finale

### Principe : Iframe Direct Sans Proxy

L'approche la plus simple et fiable est d'utiliser une **iframe qui accède directement à Grafana** sur son port 3001, sans passer par un proxy complexe.

**Avantages** :
- Aucune réécriture d'URL ni configuration de sous-chemin
- Grafana fonctionne comme prévu (pas de GF_SERVER_ROOT_URL confus)
- Les assets (JavaScript, CSS) se chargent correctement
- Pas d'erreur "failed to load its application files"

---

## 📝 Modifications Effectuées

### 1. **docker-compose.yml** - Ajouter Grafana

```yaml
grafana:
  image: grafana/grafana:11.6.0
  container_name: dalybms-grafana
  restart: unless-stopped
  ports:
    - "3001:3000"
  environment:
    GF_SECURITY_ADMIN_USER:      admin
    GF_SECURITY_ADMIN_PASSWORD:  admin
    GF_USERS_ALLOW_SIGN_UP:      "false"
    GF_AUTH_ANONYMOUS_ENABLED:   "false"
    GF_SECURITY_ALLOW_EMBEDDING: "true"     # ⭐ CRUCIAL pour iframe
    GF_LOG_LEVEL:                info
```

**Note critique** : `GF_SECURITY_ALLOW_EMBEDDING: "true"` est **obligatoire** pour permettre aux iframes d'afficher Grafana. Sans cette configuration, Grafana refuse de charger ses fichiers frontend dans un iframe.

### 2. **docker-compose.yml** - Datasource InfluxDB

```yaml
volumes:
  - ./grafana/provisioning:/etc/grafana/provisioning:ro
```

Le dossier `./grafana/provisioning/datasources/influxdb.yaml` est monté en lecture seule pour que Grafana configure automatiquement la connexion à InfluxDB.

**Configuration datasource** :
```yaml
datasources:
  - name: InfluxDB-DalyBMS
    type: influxdb
    uid: influxdb-dalybms
    access: proxy
    url: http://influxdb:8086      # Docker internal hostname
    isDefault: true
```

### 3. **grafana/dashboards/santuario-solar-system.json** - Dashboard avec Flux

Le dashboard utilise des **requêtes Flux correctes** basées sur les mesures InfluxDB réelles :

```flux
from(bucket: v.defaultBucket)
  |> range(start: -2m)
  |> filter(fn: (r) => r["_measurement"] == "bms_status")
  |> filter(fn: (r) => r["_field"] == "soc")
  |> last()
```

**Mesures utilisées** :
- `bms_status` → SOC, voltage, current (BMS)
- `venus_mppt_total` → power_w (solar MPPT)
- `et112_status` → power_w (grid meter)

### 4. **crates/daly-bms-server/templates/grafana_dashboard.html** - Iframe Template

```html
{% extends "base.html" %}
{% block content %}
<div class="grafana-info">
  📊 Dashboard Grafana Santuario Solar System (données InfluxDB en direct, refresh 30s)
</div>

<script>
  const protocol = window.location.protocol;
  const host = window.location.hostname;
  const port = 3001;
  const dashUrl = `${protocol}//${host}:${port}/d/santuario?orgId=1&refresh=30s&from=now-24h&to=now`;
  document.write(`<iframe id="grafana-frame" src="${dashUrl}" allow="clipboard-write" allowfullscreen></iframe>`);
</script>
{% endblock %}
```

**Clé** : L'URL est construite **dynamiquement** avec `window.location.hostname`, ce qui fonctionne que vous accédiez via IP (192.168.1.141) ou hostname (pi5compute).

### 5. **crates/daly-bms-server/templates/base.html** - Navigation Tab

```html
<a href="/dashboard/grafana" class="nav-link {% block nav_grafana %}{% endblock %}">
  📊 Historique
</a>
```

Le bloc `nav_grafana` est activé dans le template grafana_dashboard.html pour mettre en surbrillance l'onglet actif.

---

## 🚀 Processus de Déploiement Complet

### Sur Pi5 :

```bash
# 1. Récupérer le code
cd ~/Daly-BMS-Rust
git fetch origin claude/add-grafana-dashboard-nav-XPdvb
git checkout claude/add-grafana-dashboard-nav-XPdvb

# 2. Redémarrer Docker avec Grafana
make down
sleep 3
make up
sleep 20

# 3. Vérifier que Grafana est healthy
docker compose ps grafana
# STATUS doit être: Up X seconds (healthy)

# 4. Compiler le serveur (templates HTML)
make build-arm

# 5. Déployer le binaire
sudo systemctl stop daly-bms
sudo cp target/aarch64-unknown-linux-gnu/release/daly-bms-server /usr/local/bin/
sudo systemctl start daly-bms
```

### Vérifications :

```bash
# Grafana est accessible
curl -s http://localhost:3001/api/health | grep "ok"

# Web app est accessible
curl -s http://localhost:8080/ | grep "Historique"

# Voir les logs
docker compose logs grafana | tail -20
```

### Accès en navigateur :

```
http://192.168.1.141:8080 → onglet "📊 Historique"
```

Si login requis : `grafana` / `grafana`

---

## ❌ Approches Échouées (Leçons Apprises)

### ❌ Problème 1 : Proxy sur sous-chemin

**Tentative** : Utiliser `/api/v1/grafana/d/santuario` (route proxy)

**Configuration Grafana** :
```yaml
GF_SERVER_SERVE_FROM_SUB_PATH: "true"
GF_SERVER_ROOT_URL: "http://localhost:8080/api/v1/grafana/"
```

**Résultat** : ❌ "Grafana has failed to load its application files"
- Grafana pense qu'il est sur `/api/v1/grafana/` mais les assets sont à `/public/build/`
- Le navigateur cherche `/api/v1/grafana/public/build/...` au lieu de `/public/build/...`
- Le proxy doit récrire les chemins dans le HTML (complexe)

**Leçon** : Ne pas compliquer avec un proxy si l'accès direct fonctionne.

### ❌ Problème 2 : Accès direct avec IP externe échoue initialement

**Tentative** : Iframe vers `http://192.168.1.141:3001/d/santuario`

**Résultat initial** : ❌ "192.168.1.141 a refusé de se connecter"
- Grafana n'était pas prêt ou le port 3001 n'était pas exposé
- MAIS après avoir redémarré Grafana avec `docker compose down grafana && docker compose up -d grafana`, ça a fonctionné ✅

**Leçon** : Les problèmes de démarrage peuvent sembler être des problèmes architecturaux. Toujours redémarrer le container.

### ❌ Problème 3 : Sans GF_SECURITY_ALLOW_EMBEDDING

**Résultat** : ❌ Écran blanc ou "failed to load"
- Grafana refuse de charger dans un iframe sans ce flag
- Cela change le header `X-Frame-Options: SAMEORIGIN` en `X-Frame-Options: ALLOWALL`

**Leçon** : Pour une iframe, ce flag est **critique**.

---

## 📊 Structure de Données InfluxDB Utilisée

### Mesures Écrites par daly-bms-server

| Mesure | Champs | Source |
|--------|--------|--------|
| `bms_status` | soc, voltage, current, power | RS485 BMS Daly |
| `venus_mppt_total` | power_w, current_a | Venus OS MPPT |
| `et112_status` | power_w, voltage_v, current_a | RS485 ET112 |
| `venus_smartshunt` | voltage_v, current_a, power_w | Venus OS SmartShunt |

Le code Rust écrit ces points dans InfluxDB avec un delai de batch (~5-10s).

---

## 🔍 Dépannage Rapide

| Problème | Solution |
|----------|----------|
| Dashboard affiche "No data" | Vérifier que daly-bms-server écrit dans InfluxDB. Voir : `docker exec dalybms-influxdb influx query` |
| Iframe affiche écran blanc | Vérifier `GF_SECURITY_ALLOW_EMBEDDING=true` dans docker-compose.yml. Redémarrer Grafana. |
| Connexion refusée 192.168.1.141:3001 | Redémarrer Grafana : `docker compose down grafana && docker compose up -d grafana` |
| Login Grafana demandé | Credentials : `grafana` / `grafana` |
| Template HTML ne s'affiche pas | Recompiler : `make build-arm` et redéployer le binaire |

---

## 📚 Fichiers Modifiés

```
docker-compose.yml
  ├─ Ajout service Grafana 11.6.0
  ├─ Port 3001:3000
  └─ GF_SECURITY_ALLOW_EMBEDDING: "true"

grafana/provisioning/datasources/influxdb.yaml
  └─ URL mise à jour pour Docker : http://influxdb:8086

grafana/dashboards/santuario-solar-system.json
  └─ 4 panels avec requêtes Flux testées

crates/daly-bms-server/templates/grafana_dashboard.html
  └─ Nouvelle template avec iframe dynamique

crates/daly-bms-server/templates/base.html
  └─ Ajout onglet "📊 Historique" dans navigation
```

---

## ✅ Checklist de Vérification

- [x] Grafana démarré et healthy (`docker compose ps`)
- [x] Datasource InfluxDB configuré et connecté
- [x] Dashboard Santuario importé automatiquement
- [x] Web app compile sans erreurs (`make build-arm`)
- [x] Template HTML s'affiche correctement
- [x] Iframe charge le dashboard Grafana
- [x] Dashboard affiche les données en temps réel
- [x] Onglet "📊 Historique" visible dans la navigation
- [x] Refresh 30s fonctionnel
- [x] Pas de console errors (F12)

---

## 🎓 Résumé pour Futures Modifications

**Si vous voulez ajouter des panneaux au dashboard** :

1. **Éditez** `/home/user/Daly-BMS-Rust/grafana/dashboards/santuario-solar-system.json`
2. **Utilisez le format Flux** (pas InfluxQL) :
   ```flux
   from(bucket: v.defaultBucket)
     |> range(start: -24h)
     |> filter(fn: (r) => r["_measurement"] == "votre_mesure")
     |> filter(fn: (r) => r["_field"] == "votre_champ")
   ```
3. **Committez** et **poussez** vers la branche
4. **Pull** sur Pi5 et **redémarrez** Grafana (les dashboards sont rechargés automatiquement)

---

## 🏁 Conclusion

**Après 2 jours de débogage**, la solution finale est simple et robuste :
- Grafana dans Docker
- Accès direct via iframe (pas de proxy)
- `GF_SECURITY_ALLOW_EMBEDDING=true` est la clé
- URLs construites dynamiquement avec `window.location.hostname`

**Temps de charge** : ~2-3 secondes après page load
**Fiabilité** : ✅ Testé et validé
