# Transition — refonte de la page Visualisation et état du projet

Document de reprise pour une nouvelle session de travail. Il contient la spécification de la
refonte de `visualization.html`, l'état actuel de l'installation et des développements, les
points ouverts, et les procédures d'exploitation.

**État** : la refonte du §1 est en service ; le §1 reste la référence de la disposition et
des liaisons. Les points ouverts sont au §3.

**Pour reprendre** :

1. Lire ce document en entier.
2. Pour toucher à la disposition, joindre le schéma de l'utilisateur (image), dont le §1.2 est
   la transcription.

---

## 1. Refonte de la page Visualisation

### 1.1 Objectif

Réorganiser `http://192.168.1.141:8080/visualization` selon le schéma de l'utilisateur.

À conserver :

- le **bandeau supérieur** (Prod. totale, Production, Batteries, Météo, Température, heure) ;
- les **animations de flux** sur les liaisons ;
- les historiques des liaisons (`HIST_*`) : bouton 📈 au milieu de chaque liaison, qui ouvre
  la fiche détail (§1.8).

### 1.2 Schéma cible

AC en haut, DC en bas. Les lignes sont des liaisons animées. Les blocs sans ligne sont des
cartes indépendantes. Multiplus, Inverter RS, SmartShunt et les 3 BMS ont le même gabarit
(280 × 158). Les trois cartes ATS (ATS, I-AC-1, II-AC-2) et le MPPT sont agrandis ×1,5
(270 et 465 px de large).

```
               [Tasmota AC1]                      [Tasmota AC2]──[ET112-Micro]   Shelly 2PM
Chauffe-eau     (groupe I) ──[ET112-AC1]   [ET112-AC2]── (groupe II)                (DEYE)
                                 │              │                          [Climatisation]
                            [   ATS   ]         │
                        [I-AC-1]   [II-AC-2]    │      ┌──────────────────────┐
Grid                        │          │        │      │   Flux énergétique   │
                       [Multiplus]  [Inverter RS]      │      (560 × 486)     │
                            │   [Cerbo GX]   │         └──────────────────────┘
                       ═════╧═════ Lynx Distributor ═════╧══ [      MPPT      ]
[BMS-360Ah]┐                       │
[BMS-320Ah]┼─────────────────── [SmartShunt]
[BMS-620Ah]┘
```

Liaisons du schéma :

- Tasmota AC1 — ET112-AC1, puis ET112-AC1 — ATS ;
- ATS — I-AC-1 ou II-AC-2, selon la source sélectionnée (§1.5) ;
- I-AC-1 — Multiplus, et II-AC-2 — Inverter RS ;
- ET112-AC2 — Tasmota AC2, et ET112-AC2 — Inverter RS (ligne verticale) ;
- Tasmota AC2 — ET112-Micro ;
- Multiplus, Inverter RS et MPPT — Lynx Distributor ;
- Lynx Distributor — SmartShunt, et SmartShunt — les 3 BMS (barre commune).

### 1.3 Blocs : correspondance avec l'existant et données

Tous les endpoints sont servis par daly-bms-server, sur le port 8080.

| Bloc du schéma | Composant actuel (`id` dans `NODES0`) | Données | Travail |
|---|---|---|---|
| Tasmota AC1 | `tongou-group` (`TongouGroupNode`) | `/api/v1/tasmota`, filtré sur `network = "AC-1"` | Déplacer à gauche |
| ET112-AC1 | `et112-maison` (`Et112CardNode`, 0x08) | `/api/v1/et112/8/status` | Libellé « ET112-AC1 » |
| ATS | `ats-main` (`AtsMainNode`) | `/api/v1/ats/status` | Au centre, 270 de large (×1,5) |
| I-AC-1 | `ats-onduleur` (`AtsOnduleurNode`) | idem | Sous l'ATS, à gauche, 270 de large (×1,5) |
| II-AC-2 | `ats-reseau` (`AtsReseauNode`) | idem | Sous l'ATS, à droite, 270 de large (×1,5) |
| Multiplus | `onduleur` (`InverterNode`) | `/api/v1/venus/inverter` | Titre « EasySolar-II » → « Multiplus » (`viz_nodes_ats.js`, 2 occurrences) |
| Inverter RS | `inverter-rs` (`InverterRsNode`, « À venir ») | aucune tant qu'il n'est pas installé | Vrai bloc à la taille du Multiplus, toujours marqué « À venir » ; données branchées à l'installation (§3.1) |
| ET112-AC2 | `et112-reseau` (`Et112CardNode`, 0x09) | `/api/v1/et112/9/status` | Placer au-dessus de l'Inverter RS |
| Tasmota AC2 | `tongou-ac2` (`TongouGroupNode`) | `/api/v1/tasmota`, filtré sur `network = "AC-2"` | Placer à droite de l'ET112-AC2 |
| ET112-Micro | `micro-ond` (`Et112CardNode`, 0x07) | `/api/v1/et112/7/status` | Libellé « ET112-Micro » ; en haut, à droite de Tasmota AC2 |
| Shelly 2PM | `shelly` (`ShellyNode`) | `/api/v1/shelly` → `shelly[0].snapshot.channel_0` et `channel_1` (`output`, `power_w`, `voltage_v`, `current_a`, `energy_wh`) | Carte à deux canaux, DEYE-1 / DEYE-2 (§1.5) |
| Chauffe-eau | `chauffe-eau` (`WaterHeaterNode`) | Tongou `tongou_B459B8` + PAC Venus | Déplacer à gauche, sans liaison |
| Climatisation | `climatisation` (`WaterHeaterNode`) | Tongou `tongou_3BC764` | Sous ET112-Micro et le Shelly, sans liaison |
| Grid | `grid` (`BoxNode`) | aucune | Carte « Off-grid » (§1.5) |
| Cerbo GX | `cerbo` (`BoxNode`) | aucune | Boîte seule (§1.5) |
| Lynx Distributor | `lynx` (`LynxNode`) | — | Barre DC horizontale large |
| SmartShunt | `smartshunt` (`SmartShuntNode`) | `/api/v1/venus/smartshunt` | Sous le Lynx, 280 × 158 |
| BMS-360 / 320 / 620 Ah | `bms1`, `bms2`, `bms3` (`BmsNode`) | `/api/v1/bms/{1,2,3}/status` | Colonne à gauche, reliée au SmartShunt par une barre commune (`bms-bus`, `BusNode`) ; 280 × 158, puissance en W |
| MPPT | `mppt` (`MPPTGroupNode`) | `/api/v1/venus/mppt` | Sous le Flux énergétique, relié au Lynx, 465 de large (×1,5) |
| Flux énergétique | `sankey` (`SankeyNode`) | calculé dans `applyData` | Sous le Groupe II (bord gauche aligné), 560 × 486, sans liaison |

### 1.4 Liaisons et valeurs animées

La vitesse des particules est `flowValue / speedDiv`. Le diviseur vaut 500 par défaut, et 5
pour un courant en ampères. Le sens s'inverse avec `reverse`.

| Liaison | Valeur animée | Sens / remarque |
|---|---|---|
| ET112-AC1 → Tasmota AC1 | `et8.power_w` | vers les charges |
| ATS → ET112-AC1 | `et8.power_w` | vers les charges |
| I-AC-1 ou II-AC-2 → ATS | arêtes `ats-e1` / `ats-e2` (type `tripleAts`) déjà calculées selon `sw1` / `sw2` / `middleOFF` | §1.5 |
| Multiplus → I-AC-1 | `inverter.ac_output_current_a` (diviseur 5) | sortie AC du Multiplus |
| Inverter RS → II-AC-2 | à brancher à l'installation (aujourd'hui : `et9.power_w`) | — |
| Inverter RS → ET112-AC2 | `et9.power_w` | — |
| ET112-AC2 → Tasmota AC2 | somme des prises AC-2, **hors** Tongou DEYE `tongou_0616E4` (il mesure une production) | — |
| ET112-Micro → Tasmota AC2 | `et7.power_w` | production DEYE vers l'AC-2 |
| Lynx ↔ Multiplus | `abs(inverter.power_w)` (DC) | inversé selon le signe |
| Lynx ↔ Inverter RS | à brancher à l'installation | — |
| MPPT → Lynx | total MPPT (`total_power_w`) | — |
| Lynx ↔ SmartShunt | `shunt.power_w` | charge / décharge selon le signe |
| SmartShunt ↔ BMS n | `Dc.Power` de chaque BMS | sens selon le signe (logique `e-hubBMS-bN` actuelle) |

### 1.5 Décisions de l'utilisateur

- **Grid** : carte avec la seule mention « Off-grid », sans donnée ni liaison (installation
  100 % hors réseau).
- **Cerbo GX** : la boîte seule, avec son nom. Ni données ni liaison.
- **Shelly 2PM** (« DEYE — Shelly Pro 2PM ») : les **deux canaux**, chacun avec son état
  ON/OFF, sa puissance, sa tension et son énergie. Données : `/api/v1/shelly`. Pas de
  liaison. Noms des canaux : **DEYE-1** (canal 1) et **DEYE-2** (canal 0), les mêmes que
  dans Venus (§3.3) ; constante `SHELLY_DEYE_CHANNELS` de `visualization.html`.
- **ATS** : la liaison vers I-AC-1 ou vers II-AC-2 dépend de la **source sélectionnée** ;
  conserver l'animation existante. Ce sont les arêtes dynamiques `ats-e1` et `ats-e2`, ajoutées
  dans `applyData` selon `sw1` / `sw2` / `middleOFF`. Les forçages de source (boutons des blocs
  I et II) restent en place.

### 1.6 Fichiers et contraintes

| Fichier | Rôle |
|---|---|
| `crates/daly-bms-server/templates/visualization.html` | constantes de disposition (`MP_X`, `RS_X`, `LYNX_*`, `BMS_Y`… : colonnes calculées pour des liaisons verticales droites), `NODES0` (positions), `EDGES0` et historiques des liaisons `HIST_*`, contenu des fiches détail (`NODE_NAMES`, `NODE_DETAIL`, `buildDetail`, §1.8), `applyData` (valeurs, arêtes ATS dynamiques), `fetchAll` (appels API), constantes `WH_TASMOTA` / `CLIM_TASMOTA` / `DEYE_TASMOTA` / `SHELLY_DEYE_CHANNELS` |
| `viz_detail.js` | fiche détail (§1.8) : `DetailSheet`, `DetailBox` (la box agrandie), `HistChart` (graphiques ECharts), `EdgeHistoryButton` (bouton 📈 des liaisons), contexte `DetailCtx` |
| `viz_nodes_ats.js` | ATS (entrées I à gauche, II à droite, sortie en haut), I/II, `InverterNode` (Multiplus), `InverterRsNode` |
| `viz_nodes_basic.js` | `Et112CardNode`, `LynxNode`, `BusNode` (barre BMS), `BoxNode` (Grid, Cerbo GX), `BmsNode` |
| `viz_nodes_devices.js` | `TongouGroupNode` (`data.title`), `ShellyNode`, `WaterHeaterNode` |
| `viz_nodes_venus.js` | `MPPTGroupNode`, `SmartShuntNode` |
| `viz_nodes_charts.js` | `SankeyNode` (Flux énergétique ; rendu SVG, net une fois agrandi dans la fiche) |
| `viz_edges.js` | arêtes animées (`AnimatedFlowEdge`, `CustomBezierEdge`, `TripleAtsEdge`) ; une arête avec `data.history` porte le bouton 📈 |
| `visualization.css` | styles des cartes (Lynx, Grid, Cerbo, Shelly…) et de la fiche détail (classes `vd-*`) |

Contraintes :

- **Inclusion à la compilation** : les `viz_*.js` sont inclus par askama
  (`{% include %}`) dans le binaire daly-bms-server. Ne jamais écrire `{{`, `{%` ou `{#` dans le
  JavaScript. Tout changement demande une recompilation et un redémarrage de daly-bms.
- **Poignées React Flow** : une liaison relie une poignée `source` à une poignée `target`
  (`mkHandle`). Vérifier les poignées disponibles de chaque composant avant de tracer une
  liaison. `fitView` recadre automatiquement la vue.
- **Arêtes ATS** : leurs identifiants commencent par `ats-e`, et `applyData` les régénère à
  chaque rafraîchissement.

### 1.7 Vérification

1. Syntaxe JavaScript, avec les fichiers inclus assemblés (depuis
   `crates/daly-bms-server/templates`, Git Bash) :

   ```bash
   awk -v dir="$PWD" '/\{% include "[^"]+" %\}/ { match($0, /"[^"]+"/); f=substr($0, RSTART+1, RLENGTH-2);
     while ((getline l < (dir "/" f)) > 0) print l; close(dir "/" f); next } { print }' visualization.html > /tmp/viz-full.html
   awk '/^<script>/{f=1;next}/^<\/script>/{f=0}f' /tmp/viz-full.html > /tmp/viz-full.js && node --check /tmp/viz-full.js
   ```

2. **Aperçu local, sans compilation ni déploiement** — `scripts/viz-preview.py` : la page
   servie par le Pi5 reçoit la feuille de style et le script des templates locaux, et lit les
   données réelles du Pi5. Seules les lectures (GET) partent vers le Pi5 : les boutons (ATS,
   Tasmota, Shelly) sont bloqués dans l'aperçu. Capture et mesure des cartes (PowerShell) :

   ```powershell
   $O = "$env:TEMP\viz-preview"; python scripts\viz-preview.py build --out $O
   $edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
   $c = @("--headless","--no-sandbox","--disable-gpu","--disable-web-security","--allow-file-access-from-files",
          "--window-size=1900,1150","--virtual-time-budget=20000","--user-data-dir=$O\prof")
   $u = "file:///" + ($O -replace '\\','/')
   # Capture (ajouter "--force-device-scale-factor=2" pour zoomer sur un détail)
   Start-Process $edge -Wait -WindowStyle Hidden -ArgumentList ($c + @("--hide-scrollbars","--screenshot=$O\preview.png","$u/preview.html"))
   # Tailles des cartes, en px du canevas : {"id": [largeur, hauteur], …}
   Start-Process $edge -Wait -WindowStyle Hidden -RedirectStandardOutput "$O\dom.html" -RedirectStandardError "$O\err.txt" `
     -ArgumentList ($c + @("--dump-dom","$u/preview-measure.html"))
   python scripts\viz-preview.py sizes "$O\dom.html"
   # Fiche détail ouverte d'office (id d'une box, ou edge:N pour le N-ième 📈), période 24 h
   python scripts\viz-preview.py build --out $O --open bms1 --period 1440
   Start-Process $edge -Wait -WindowStyle Hidden -ArgumentList ($c + @("--hide-scrollbars","--screenshot=$O\fiche.png","$u/preview-open.html"))
   ```

   Les positions de `NODES0` sont calculées à partir des hauteurs mesurées (`H_*` en tête de
   la disposition dans `visualization.html`). Après un changement de taille d'une carte,
   reporter la nouvelle mesure dans la constante, sinon les liaisons ne sont plus droites.
3. Tests et compilation sur le Pi5 (§4.2), puis déploiement avec `scripts/deploy-pi5.sh`.
4. Capture de la page déployée, à comparer au schéma : même commande que la capture
   ci-dessus, avec l'URL `http://192.168.1.141:8080/visualization`. Juste après un
   déploiement, les MPPT et le Flux énergétique restent vides environ une minute (§4.4).

### 1.8 Fiche détail (clic sur une box ou sur une liaison)

Un clic sur une box, ou sur le bouton 📈 au milieu d'une liaison, ouvre une fiche centrée dans
la fenêtre (92 % × 88 %, au plus 1500 × 960 px). Elle est posée par-dessus la carte : sa taille
ne dépend pas du zoom. En vue d'ensemble, la carte est affichée à 58 % sur un écran
1900 × 1150 et à 35 % sur 1366 × 768.

- **À gauche** : la box elle-même, en direct (mêmes données que la carte, toutes les 2 s),
  agrandie pour remplir sa moitié de fiche ; une box BMS y fait environ 645 px de large, contre
  160 sur la carte. C'est un second React Flow qui ne contient que ce nœud (`DetailBox`). Ses
  boutons restent **actifs**, comme sur la carte : forçage de source ATS, Double OFF avec sa
  confirmation, prises Tongou (décision de l'utilisateur).
- **À droite** : les historiques de la box, un graphique par mesure (un seul axe chacun).
  Période 1 h, 6 h ou 24 h, gardée d'une fiche à l'autre ; relecture chaque minute. En-tête :
  dernier point avec son heure, min · moy · max, énergie de la période (W → kWh, A → Ah, par
  la méthode des trapèzes ; séparée en charge et décharge pour une mesure signée). Avec
  plusieurs courbes : légende cliquable (masquer / afficher), avec l'énergie de chaque courbe.
- **Liaison** : la même fiche, sans box, avec l'historique `HIST_*` de la liaison.
- **Fermeture** : ✕, Échap, clic, molette ou glisser sur le fond. Un clic sur un bouton d'une
  box de la carte déclenche son action sans ouvrir la fiche.

| Box | Graphiques |
|---|---|
| BMS (3) | courant, tension, SOC, température max |
| SmartShunt | courant, puissance, tension, SOC |
| MPPT | production totale, production par chargeur, ensoleillement |
| Multiplus | puissance DC, puissance AC en sortie, tension DC |
| Flux énergétique | production (solaire, micro-onduleurs), batteries |
| ET112-AC1, ET112-AC2, ET112-Micro | puissance, courant, tension |
| Groupes I et II | puissance par prise |
| Chauffe-eau | puissance (Tongou), température de l'eau (PAC Venus `mqtt_index` 1) |
| Climatisation | puissance (Tongou) |
| Shelly 2PM | puissance par canal (DEYE-1, DEYE-2) |
| ATS ; I-AC-1 ; II-AC-2 | tension des deux sources ; tension de la source |
| Lynx (fiche sans box) | puissances du bus DC : MPPT, Multiplus, batteries |
| Barre BMS (fiche sans box) | courant des 3 BMS |

Grid, Cerbo GX et Inverter RS (à venir) n'ouvrent pas de fiche : ils n'ont pas d'entrée dans
`NODE_DETAIL`.

**Données** : `/api/v1/query_range` (PromQL du metrics-store, comme Grafana), une requête par
courbe, avec des étiquettes qui désignent une seule série : `bms_id` (0x01 à 0x03), `address`
(0x07 à 0x09), `id` des Tongou et du Shelly, `instance` des MPPT, `mqtt_index` des PAC. Pas
de lecture : 20 s (1 h), 60 s (6 h), 180 s (24 h). Le dernier point d'un graphique peut
différer de la valeur de la box : un point par pas, contre un rafraîchissement toutes les
2 s. `/api/v1/chart/edge-history` n'est pas utilisé par la page.

**Ajouter ou modifier un graphique** : entrée de `NODE_DETAIL` dans `visualization.html`,
`{ color, box?, charts: (node, ctx) => [{ title, unit, sign?, series: [{ q, name }] }] }`
(`box: false` = fiche sans box ; `ctx.tasmota` et `ctx.shellyId` viennent du dernier
`fetchAll`). Vérifier d'abord que la requête renvoie une seule série :

```bash
now=$(date +%s); curl -s -G http://192.168.1.141:8080/api/v1/query_range \
  --data-urlencode 'query=tasmota_power_w{id="6"}' --data-urlencode start=$((now-3600)) \
  --data-urlencode end=$now --data-urlencode step=60 | python -c 'import sys,json; print(len(json.load(sys.stdin)["data"]["result"]), "série(s)")'
```

---

## 2. État de l'installation (référence)

### 2.1 Réseaux AC — installation 100 % hors réseau

Description complète : [ARCHITECTURE.md § Réseaux AC](./ARCHITECTURE.md).

- **AC-1** : MultiPlus → ATS entrée I → sortie ATS → ET112-Maison/AC1 (0x08) → groupe I.
- **AC-2** : Inverter RS Smart → ET112-AC2 (0x09) → groupe II, et entrée II de l'ATS.
- **DEYE** : sur l'AC-2, via Tongou 0616E4 → ET112 0x07 → micro-onduleurs, plus le Shelly
  Pro 2PM.
- **Avant l'installation de l'Inverter RS** : l'AC-2 est alimenté par l'AC-1.
- **Batteries** : les deux réseaux partagent les mêmes (BMS 320 / 360 / 620 Ah).
- **Comptage** : aucun compteur `grid`. Les ET112 0x08 et 0x09 sont des `acload`, et l'ESS
  tourne sans compteur réseau (`RunWithoutGridMeter = 1`). Le DVCC est désactivé sur le GX.

### 2.2 Prises Tongou (10)

| Groupe | Nom | Topic | IP | `id` | mqtt_index / instance VRM |
|---|---|---|---|---|---|
| I (AC-1) | General | `tongou_0A3C14` | .27 | 3 | 4 / 63 |
| I (AC-1) | AC-4 | `tongou_0A4040` | .28 | 4 | 5 / 64 |
| I (AC-1) | LG (chauffe-eau) | `tongou_B459B8` | .29 | 6 | 7 / 66 |
| I (AC-1) | Climatisation | `tongou_3BC764` | .30 | 1 | 2 / 61 |
| II (AC-2) | DEYE (coupure complète) | `tongou_0616E4` | .21 | 7 | 8 / 67 |
| II (AC-2) | Tongou 0615D8 | `tongou_0615D8` | .22 | 8 | 9 / 68 |
| II (AC-2) | Tongou 0616D0 | `tongou_0616D0` | .23 | 9 | 10 / 69 |
| II (AC-2) | Tongou 061630 | `tongou_061630` | .24 | 10 | 11 / 70 |
| II (AC-2) | Tongou 3ACC34 | `tongou_3ACC34` | .25 | 5 | 6 / 65 |
| II (AC-2) | Tongou 0A3FA0 | `tongou_0A3FA0` | .26 | 2 | 3 / 62 |

- Toutes les prises publient vers le broker du GX (`192.168.1.120:1883`).
- Les noms provisoires du groupe II se changent depuis la console Venus
  ([renommer-appareils.md](./renommer-appareils.md)).

### 2.3 Fonctions en service

| Fonction | État | Référence |
|---|---|---|
| Chauffe-eau LG (energy-manager) | Aucune énergie batterie. Descente d'un cran TURBO → HEAT_PUMP → VACATION sur décharge ; verrou de 30 min ; tolérance de démarrage (300 s, 40 Wh) ; production bridée (MPPT `MppOperationMode = 1`, SOC ≥ 98 %) ; mesure par le Tongou LG | [app-energy-manager.md §4.4](./app-energy-manager.md) |
| Noms des appareils | Renommage depuis la console Venus, conservé dans `/data/daly-bms/custom-names.json` (GX) ; noms des Tongou repris par les pages du Pi5 | [renommer-appareils.md](./renommer-appareils.md) |
| Base de métriques (redb, 11,8 Go) | Arrêt propre même pendant une maintenance ou une récupération (`TimeoutStopSec=30min`, compaction interruptible) | [incident-2026-07-09-reboot-redb.md §5.4](./incident-2026-07-09-reboot-redb.md) |
| ATS | Source I = « AC-1 MultiPlus », source II = « AC-2 Inverter RS » ; `/Position` Venus : 1 = I, 0 = II | [integration-materiel.md](./integration-materiel.md) |

---

## 3. Points ouverts

### 3.1 À l'installation de l'Inverter RS Smart

- **Coupure DEYE (energy-manager, `logic/deye_command`)** : elle lit aujourd'hui la fréquence de
  sortie du MultiPlus (`N/<portail>/vebus/276/Ac/Out/L1/F`). Comme les DEYE sont sur l'AC-2,
  elle devra lire la fréquence de sortie de l'Inverter RS. Commencer par relever son service et
  son instance dans Venus (`dbus -y | grep -i inverter` sur le GX).
- **Visualisation** : brancher les données du bloc Inverter RS (§1.3) et ses deux liaisons
  (II-AC-2, Lynx).
- **ATS** : l'entrée II sera alors réellement alimentée par l'Inverter RS.

### 3.2 BMS 360 Ah (0x01) : trames RS485 altérées

- **Depuis le 20/11 à 13:53**, environ 74 % des réponses du 0x01 arrivent avec des bits
  isolés modifiés (surtout 0 → 1). Les 0x02 et 0x03 sont intacts, avec le même code.
- **Impact** : environ 66 relevés complets par heure pour le 0x01, au lieu d'environ 680.
- **Redémarrage du BMS (commande 0x00)** : essayé, sans effet. La liaison RS485 du 0x01
  (câble, connecteur, module) reste à contrôler.
- **Mesure et diagnostic** : [diagnostic-depannage.md §2.1](./diagnostic-depannage.md) et
  `scripts/diag-bms-trames.sh`.

### 3.3 Autres

- **Chauffe-eau** : tant que le solaire ne couvre pas le TURBO, chaque essai peut coûter
  jusqu'à 40 Wh, rechargés ensuite par le surplus. Allonger le verrou du TURBO après un
  dépassement du budget a été proposé, sans décision.
- **Shelly DEYE dans Venus** (service natif `acload.shelly_EC62608840A4_<canal>`) : canal 0
  = instance 50 = « DEYE-2 », canal 1 = instance 51 = « DEYE-1 ». La page Visualisation
  affiche les mêmes noms ; s'ils changent dans Venus, mettre à jour `SHELLY_DEYE_CHANNELS`.
- **Healthcheck `histogram_quantile`** de `deploy-pi5.sh` : échec préexistant, connu.
- **Laissés par la migration vers le Cerbo MK3** (côté utilisateur) : Home Assistant doit
  utiliser le portail `d81d13b31a8b`, et l'installation VRM orpheline peut être supprimée.

---

## 4. Exploitation

### 4.1 Accès

Identifiants hors dépôt : ne jamais les écrire dans ce dépôt.

- Pi5 : `pi5compute@192.168.1.141` (SSH ; `plink` depuis le PC Windows).
- GX (Cerbo GX MK3) : `root@192.168.1.120` (SSH, clé d'hôte épinglée dans
  `C:\GX-migration-MK3\outils\gx.ps1`).

### 4.2 Tests et compilation

`cargo test` ne compile pas sous Windows (dépendance `sd-notify`, Unix uniquement). On teste
donc sur le Pi5, dans un dossier séparé qui ne touche pas au déploiement :

```bash
# PC : archive des sources (inclure docs/ : tests « golden » de metrics-store)
tar --exclude=./target --exclude=./.git -czf wh-src.tgz .   # puis pscp vers /tmp du Pi5
# Pi5 :
mkdir -p ~/wh-test && tar -xzf /tmp/wh-src.tgz -C ~/wh-test && cd ~/wh-test
export CARGO_TARGET_DIR=~/wh-test-target
ionice -c3 nice -n19 cargo test --locked -j2 -p <crate>
ionice -c3 nice -n19 cargo clippy --locked -j2 -p <crate> --all-targets -- -D warnings
```

### 4.3 Déploiement

- **Pi5** : `cd ~/Daly-BMS-Rust && sudo bash scripts/deploy-pi5.sh`. Le script se synchronise
  sur `origin/main`, compile, installe les unités systemd, redémarre les services modifiés et
  lance les healthchecks.
- **GX (dbus-mqtt-venus)** :
  1. Compiler le binaire Cortex-A53 sur le Pi5 :
     `CARGO_TARGET_DIR=~/venus-a53-target RUSTFLAGS="-C target-cpu=cortex-a53 -C link-arg=-Wl,--as-needed" cargo build --release --target aarch64-unknown-linux-gnu --bin dbus-mqtt-venus`.
  2. Vérifier les versions de glibc requises : `objdump -T … | grep -o 'GLIBC_[0-9.]*'` doit
     rester ≤ 2.39 (version de Venus).
  3. Copier le binaire sur le GX.
  4. Sauvegarder l'ancien binaire, puis exécuter
     `svc -d /service/dbus-mqtt-venus`, remplacer le binaire, `svc -u`.
  5. Pour la config du GX : modifier `/data/daly-bms/config.toml` (copie de
     `nanoPi/config-nanopi.toml`), puis `svc -t /service/dbus-mqtt-venus`.

### 4.4 Pièges connus

- **Fins de ligne** : git a `core.autocrlf=true`, et l'index est en LF. Les CRLF de la copie de
  travail sont normalisés au commit.
- **Git Bash** : il convertit les chemins absolus des arguments (`/lib/…` → `C:/Program
  Files/…`). Pour une commande distante, passer `MSYS_NO_PATHCONV=1` à `plink`, mais **pas** à
  `pscp` : le chemin local doit rester converti.
- **Redémarrer daly-bms** : sans risque pour la base désormais. Un arrêt pendant la phase
  d'écriture d'une compaction peut toutefois prendre quelques minutes. Vérifier ensuite
  `curl http://127.0.0.1:8080/api/v1/redb/healthy` (200 = base prête).
- **energy-manager redémarre avec daly-bms** (`PartOf=daly-bms.service` dans son unité) : tout
  déploiement qui redémarre daly-bms relance aussi energy-manager. Pendant environ une minute,
  les MPPT affichent « — », la production 0 W et le Flux énergétique « Pas de production »,
  puis tout revient sans intervention.
- **Attendre la fin d'un déploiement à distance** : lancer `deploy-pi5.sh` détaché
  (`setsid nohup … > /tmp/deploy-viz.log &`), puis attendre avec
  `while pgrep -f '[d]eploy-pi5.sh' >/dev/null; do sleep 15; done`. Avec `pgrep -f
  deploy-pi5.sh`, la boucle se trouve elle-même et ne s'arrête jamais ; tuer `plink` côté PC ne
  l'arrête pas sur le Pi5. Pour lister les processus, ne pas filtrer avec `grep -v grep` : cela
  masque aussi les lignes « while pgrep ».
