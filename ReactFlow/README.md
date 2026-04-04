README.md - Projet ReactFlow Énergie

```markdown
# ⚡ ReactFlow Énergie - Supervision de système électrique

## 📌 Description

Application de visualisation énergétique utilisant **React Flow** pour représenter et superviser un système électrique complet (batteries, MPPT, compteurs, interrupteurs, capteurs météo).

## 🎯 Fonctionnalités

- **7 types de nœuds personnalisés** :
  - `BatteryNode` - Batterie avec SOC, puissance, énergie importée/exportée
  - `MPPTNode` - Chargeur solaire avec support multi-MPPT
  - `ShuntNode` - Capteur de courant avec jauge circulaire
  - `SwitchNode` - Interrupteur commandable ON/OFF
  - `ET112Node` - Compteur d'énergie
  - `MeteoNode` - Irradiance et production solaire
  - `TemperatureNode` - Conditions météo (température, humidité, pression)

- **Flux dynamique** : Les handles changent de couleur et de type (source/target) selon l'état (charge/décharge)

- **Animations** : Pulsations vertes (charge) et rouges (décharge)

- **Mesures d'énergie** : Compteurs import/export pour chaque nœud

- **Thème clair compact** : Nœuds de 140px adaptés pour 15-20 éléments par page

## 📁 Structure du projet

```

src/
├── components/
│   └── nodes/
│       ├── BatteryNode.jsx + batteryAnimations.css
│       ├── MPPTNode.jsx + mpptAnimations.css
│       ├── ShuntNode.jsx + shuntAnimations.css
│       ├── SwitchNode.jsx + switchAnimations.css
│       ├── ET112Node.jsx + et112Animations.css
│       ├── MeteoNode.jsx + meteoAnimations.css
│       └── TemperatureNode.jsx + temperatureAnimations.css
├── pages/
│   └── VisualisationComplete.jsx
└── App.jsx

```

## 🚀 Installation rapide

```bash
# 1. Créer le projet
npm create vite@latest reactflow-energie -- --template react
cd reactflow-energie

# 2. Installer React Flow
npm install @xyflow/react

# 3. Copier les composants (voir sections ci-dessous)

# 4. Lancer
npm run dev
```

📦 Structure des données d'un nœud

BatteryNode

```javascript
{
  label: "BMS-360Ah",
  soc: 92,                    // %
  voltage: 52.8,              // V
  current: -17.3,             // A (négatif = décharge)
  power: -910,                // W
  energyImported: 1250.5,     // kWh
  energyExported: 890.3,      // kWh
  energyTotal: 2140.8         // kWh
}
```

MPPTNode

```javascript
{
  label: "Chargeur PV",
  totalPower: 1169,           // W
  mppts: [
    { id: "MPPT-273", voltage: 98.70, current: 1.9, power: 777 },
    { id: "MPPT-289", voltage: 98.71, current: 4.3, power: 423 }
  ],
  energyToday: 12.5,          // kWh
  energyTotal: 3450           // kWh
}
```

ShuntNode

```javascript
{
  label: "Shunt",
  status: "Décharge en cours",
  power: -1664,               // W
  soc: 90.2,                  // %
  voltage: 52.81,             // V
  current: -31.5,             // A
  timeRemaining: 13.10,       // heures
  energyDay: 24.5,            // kWh
  energyWeek: 168.2,          // kWh
  energyTotal: 12500          // kWh
}
```

SwitchNode

```javascript
{
  label: "Tongou Switch",
  deviceId: "tongou_3BC764",
  isOn: true,                 // État du toggle
  power: 2.0,                 // W
  voltage: 231.0,             // V
  current: 0.04,              // A
  cosPhi: 0.26,
  today: 4.26,                // kWh
  yesterday: 2.62,            // kWh
  total: 42.3                 // kWh
}
```

ET112Node

```javascript
{
  label: "ET112",
  deviceId: "0x07",
  power: 1664,                // W
  voltage: 230.4,             // V
  current: 7.23,              // A
  type: "pvinverter",
  imported: 760.30,           // kWh
  exported: 0.00              // kWh
}
```

MeteoNode

```javascript
{
  label: "Station Solaire",
  irradiance: 850,            // W/m²
  productionTotal: 31,        // kWh
  productionLast24h: 30.6,    // kWh
  productionDay: 31,          // kWh
  lastUpdate: "il y a quelques secondes"
}
```

TemperatureNode

```javascript
{
  label: "Station Météo",
  temperature: 10.2,          // °C
  humidity: 53,               // %
  pressure: 1007.0,           // hPa
  tempMin24h: 8.5,            // °C
  tempMax24h: 14.6,           // °C
  lastUpdate: "il y a quelques secondes"
}
```

🎨 Personnalisation des styles

Modifiez les variables dans chaque fichier CSS ou créez un common.css :

```css
:root {
  --node-min-width: 140px;
  --node-padding: 8px;
  --font-size-value-large: 22px;
  --color-charge: #4caf50;
  --color-decharge: #f44336;
}
```

🔧 Prochaines étapes (à développer)

1. Handles colorés dynamiques

· Chaque Handle change de couleur selon l'état du nœud
· Vert = charge / production
· Rouge = décharge / consommation
· Orange = idle

```jsx
<Handle 
  type={handleType}
  position={Position.Bottom}
  style={{ 
    background: currentColor,
    width: 10,
    height: 10,
    transition: 'background 0.3s ease'
  }}
/>
```

2. Edges dynamiques

· Les connexions entre nœuds changent de couleur selon le flux
· Animation du flux (points mobiles le long de l'arête)
· Mise à jour en temps réel via WebSocket

```jsx
// Exemple d'arête dynamique
const edgeStyle = {
  stroke: isCharging ? '#4caf50' : '#f44336',
  strokeWidth: 2,
  animated: true
};
```

3. Mise à jour temps réel

```javascript
// WebSocket pour données live
const ws = new WebSocket('ws://api.example.com/energy');
ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  updateNodeData(data.nodeId, data.values);
};
```

🐛 Dépannage

Erreur Solution
Failed to resolve import "./xxx.css" Déplacer le CSS dans le même dossier que le JSX
Les Handles ne s'affichent pas Vérifier handles: { left: true, right: true } dans les données
Le toggle switch ne répond pas Vérifier la fonction onToggle dans les props

📝 Notes importantes

· Les fichiers CSS doivent être dans le même dossier que les composants JSX
· Le sens du flux est déterminé par le signe du courant (positif = charge/entrée, négatif = décharge/sortie)
· Les handles sont configurables via l'objet handles dans les données

🔗 Liens utiles

· React Flow Documentation
· Vite Documentation

---

Version : 1.0
Dernière mise à jour : Avril 2026
Statut : Prêt pour ajout des handles colorés et edges dynamiques

```

---

Ce README contient **toutes les informations essentielles** pour :
- Comprendre le projet
- Installer et lancer l'application
- Connaître la structure des données de chaque nœud
- Savoir quelles sont les **prochaines étapes** (handles colorés, edges dynamiques)
- Dépanner les erreurs courantes

Vous pouvez le copier dans un fichier `README.md` à la racine de votre projet.
```
