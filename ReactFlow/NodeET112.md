---

DOCUMENTATION TECHNIQUE - NodeType ET112 pour React Flow

Version : 1.0
Date : Avril 2026
Statut : Template réutilisable

---

1. Objectif

Créer un nœud React Flow personnalisé représentant un compteur ET112 (micro-onduleur / mesure) avec :

· 1 ou 2 handles configurables (mesure simple ou mesure entre deux nœuds)
· Affichage des métriques : Puissance, Tension, Courant, Type, Importée/Exportée
· Intégration possible dans un flux électrique (ex: entre batterie et charge)
· Modèle unique réutilisable pour plusieurs instances

---

2. Prérequis

```bash
npm install @xyflow/react
```

---

3. Structure des fichiers

```
src/
├── components/
│   └── nodes/
│       ├── BatteryNode.jsx      (déjà créé)
│       └── ET112Node.jsx         (à créer)
├── pages/
│   └── Visualisation.jsx
└── styles/
    └── et112Animations.css
```

---

4. Code complet du composant ET112Node

Fichier : src/components/nodes/ET112Node.jsx

```jsx
import { Handle, Position } from '@xyflow/react';
import './et112Animations.css';

const ET112Node = ({ id, data }) => {
  // Données d'entrée
  const {
    label = 'ET112',
    deviceId = '0x07',
    time = '20:09:42',
    power = 0,          // Puissance en W
    voltage = 230.4,    // Tension en V
    current = 0.53,     // Courant en A
    type = 'pvinverter',
    imported = 760.30,  // kWh importés
    exported = 0.00,    // kWh exportés
    // Configuration des handles
    handles = {
      left: false,      // handle gauche (entrée)
      right: false,     // handle droit (sortie)
      bottom: false     // handle bas (optionnel)
    },
    // Sens du flux (pour animation)
    isActive = false,   // Flux actif si true
    flowDirection = 'right' // 'left', 'right', 'both'
  } = data;

  // Déterminer la couleur du flux
  const isProducing = power > 0;
  const flowColor = isProducing ? '#4caf50' : (power < 0 ? '#f44336' : '#ff9800');

  return (
    <div 
      className="et112-node"
      style={{
        borderColor: isActive ? flowColor : '#333',
        boxShadow: isActive ? `0 0 8px ${flowColor}` : 'none'
      }}
    >
      {/* Handles configurables */}
      {handles.left && (
        <Handle 
          type="target"
          position={Position.Left}
          id="left-input"
          style={{ background: flowColor }}
        />
      )}
      
      {handles.right && (
        <Handle 
          type="source"
          position={Position.Right}
          id="right-output"
          style={{ background: flowColor }}
        />
      )}
      
      {handles.bottom && (
        <Handle 
          type={flowDirection === 'left' ? 'target' : 'source'}
          position={Position.Bottom}
          id="bottom-connection"
          style={{ background: flowColor }}
        />
      )}

      {/* En-tête avec badge LIVE */}
      <div className="et112-header">
        <div className="et112-title">
          <span className="et112-icon">📊</span>
          <span className="et112-label">{label}</span>
        </div>
        <div className="et112-badge">
          <span className="live-dot"></span>
          LIVE
        </div>
      </div>

      {/* ID et Time */}
      <div className="et112-id-time">
        <span className="device-id">{deviceId}</span>
        <span className="device-time">{time}</span>
      </div>

      {/* Puissance principale */}
      <div className="et112-power" style={{ color: flowColor }}>
        <span className="power-label">PUISSANCE</span>
        <span className="power-value">{Math.abs(power).toFixed(1)} W</span>
      </div>

      {/* Grille des métriques */}
      <div className="et112-metrics">
        <div className="metric">
          <span className="metric-label">TENSION</span>
          <span className="metric-value">{voltage.toFixed(1)} V</span>
        </div>
        <div className="metric">
          <span className="metric-label">COURANT</span>
          <span className="metric-value">{current.toFixed(2)} A</span>
        </div>
        <div className="metric-full">
          <span className="metric-label">TYPE</span>
          <span className="metric-value type-value">{type}</span>
        </div>
      </div>

      {/* Import/Export */}
      <div className="et112-energy">
        <div className="energy-item">
          <span className="energy-label">📥 IMPORTÉE</span>
          <span className="energy-value">{imported.toFixed(2)} kWh</span>
        </div>
        <div className="energy-item">
          <span className="energy-label">📤 EXPORTÉE</span>
          <span className="energy-value">{exported.toFixed(2)} kWh</span>
        </div>
      </div>

      {/* Détails (cliquable) */}
      <div className="et112-details">
        Détails →
      </div>
    </div>
  );
};

export default ET112Node;
```

---

5. Fichier CSS des animations

Fichier : src/components/nodes/et112Animations.css

```css
.et112-node {
  min-width: 220px;
  background: #0d1117;
  border-radius: 16px;
  padding: 14px;
  border: 1.5px solid;
  transition: all 0.3s ease;
  font-family: 'Segoe UI', monospace;
  cursor: pointer;
}

.et112-node:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.3);
}

/* En-tête */
.et112-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.et112-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.et112-icon {
  font-size: 18px;
}

.et112-label {
  font-weight: bold;
  color: #fff;
  font-size: 14px;
}

.et112-badge {
  background: #ff3b30;
  color: white;
  font-size: 9px;
  font-weight: bold;
  padding: 2px 8px;
  border-radius: 20px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.live-dot {
  width: 6px;
  height: 6px;
  background: white;
  border-radius: 50%;
  animation: livePulse 1s infinite;
}

/* ID et Time */
.et112-id-time {
  display: flex;
  justify-content: space-between;
  margin-bottom: 16px;
  font-size: 10px;
  color: #888;
  font-family: monospace;
}

/* Puissance */
.et112-power {
  text-align: center;
  margin-bottom: 16px;
  padding: 8px;
  background: #1a1f2e;
  border-radius: 12px;
}

.power-label {
  display: block;
  font-size: 10px;
  color: #888;
  letter-spacing: 1px;
}

.power-value {
  display: block;
  font-size: 28px;
  font-weight: bold;
}

/* Grille métriques */
.et112-metrics {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 8px;
  margin-bottom: 16px;
}

.metric {
  background: #1a1f2e;
  border-radius: 10px;
  padding: 6px 8px;
  text-align: center;
}

.metric-full {
  grid-column: span 2;
  background: #1a1f2e;
  border-radius: 10px;
  padding: 6px 8px;
  text-align: center;
}

.metric-label {
  display: block;
  font-size: 9px;
  color: #888;
}

.metric-value {
  display: block;
  font-size: 13px;
  font-weight: bold;
  color: #ddd;
}

.type-value {
  color: #58a6ff;
  font-family: monospace;
  text-transform: uppercase;
}

/* Import/Export */
.et112-energy {
  display: flex;
  justify-content: space-between;
  gap: 12px;
  margin-bottom: 12px;
  padding-top: 8px;
  border-top: 1px solid #2a2f3e;
}

.energy-item {
  flex: 1;
  text-align: center;
}

.energy-label {
  display: block;
  font-size: 8px;
  color: #666;
}

.energy-value {
  display: block;
  font-size: 11px;
  font-weight: bold;
  color: #ddd;
}

/* Détails */
.et112-details {
  text-align: right;
  font-size: 11px;
  color: #58a6ff;
  cursor: pointer;
  transition: opacity 0.2s;
}

.et112-details:hover {
  opacity: 0.7;
}

/* Animations */
@keyframes livePulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

/* Animation de flux pour les handles */
.handle-animation {
  animation: flowPulse 1s ease-in-out infinite;
}

@keyframes flowPulse {
  0%, 100% { transform: scale(1); opacity: 0.8; }
  50% { transform: scale(1.3); opacity: 1; }
}
```

---

6. Exemples de configuration des handles

Cas 1 : Mesure simple (1 handle, source ou target)

```jsx
// Mesure en sortie (ex: panneau solaire → ET112 → batterie)
{
  id: 'et112-simple',
  type: 'et112',
  position: { x: 300, y: 200 },
  data: {
    label: 'ET112-Solar',
    power: 1250.5,
    handles: { left: true, right: false }  // reçoit du panneau
  }
}
```

Cas 2 : Mesure entre deux nodes (2 handles)

```jsx
// Entre batterie et onduleur
{
  id: 'et112-between',
  type: 'et112',
  position: { x: 300, y: 200 },
  data: {
    label: 'ET112-Monitor',
    power: 760.30,
    handles: { left: true, right: true }  // mesure le flux entre deux
  }
}
```

Cas 3 : Avec handle bas pour dérivation

```jsx
{
  id: 'et112-with-bottom',
  type: 'et112',
  position: { x: 300, y: 200 },
  data: {
    label: 'ET112-3Way',
    handles: { left: true, right: true, bottom: true }
  }
}
```

---

7. Code d'utilisation dans la page

Fichier : src/pages/Visualisation.jsx (mise à jour)

```jsx
import { ReactFlow, useNodesState, useEdgesState, Background, Controls } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import BatteryNode from '../components/nodes/BatteryNode';
import ET112Node from '../components/nodes/ET112Node';

// Déclaration des types de nœuds
const nodeTypes = {
  battery: BatteryNode,
  et112: ET112Node,
};

const initialNodes = [
  // Batteries
  {
    id: 'bms-360ah',
    type: 'battery',
    position: { x: 100, y: 200 },
    data: {
      label: 'BMS-360Ah',
      soc: 92,
      voltage: 52.8,
      current: -17.3,
      temperature: 14.0,
      power: -0.91
    }
  },
  
  // ET112 entre batterie et charge
  {
    id: 'et112-monitor',
    type: 'et112',
    position: { x: 350, y: 200 },
    data: {
      label: 'ET112',
      deviceId: '0x07',
      time: '20:09:42',
      power: 0.0,
      voltage: 230.4,
      current: 0.53,
      type: 'pvinverter',
      imported: 760.30,
      exported: 0.00,
      handles: { left: true, right: true },  // ← deux handles
      isActive: true
    }
  },
  
  // Charge
  {
    id: 'load',
    type: 'default',
    position: { x: 600, y: 200 },
    data: { label: 'Charge AC' }
  }
];

const initialEdges = [
  { id: 'e1', source: 'bms-360ah', target: 'et112-monitor', targetHandle: 'left-input' },
  { id: 'e2', source: 'et112-monitor', target: 'load', sourceHandle: 'right-output' }
];

function Visualisation() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  return (
    <div style={{ width: '100vw', height: '100vh' }}>
      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
      >
        <Background />
        <Controls />
      </ReactFlow>
    </div>
  );
}

export default Visualisation;
```

---

8. Processus complet de mise en œuvre

Étape 1 : Créer les fichiers

```bash
# Créer le composant ET112
touch src/components/nodes/ET112Node.jsx

# Créer les styles
touch src/components/nodes/et112Animations.css
```

Étape 2 : Copier les codes

· Copier le code de la section 4 dans ET112Node.jsx
· Copier le code de la section 5 dans et112Animations.css

Étape 3 : Mettre à jour Visualisation.jsx

· Ajouter l'import de ET112Node
· Ajouter et112: ET112Node dans nodeTypes
· Ajouter des nœuds ET112 dans initialNodes

Étape 4 : Lancer l'application

```bash
npm run dev
```

---

9. Résumé des configurations possibles

Configuration Handles Cas d'usage
{ left: true } 1 (entrée) Mesure à la sortie d'un générateur
{ right: true } 1 (sortie) Mesure avant une charge
{ left: true, right: true } 2 Mesure entre deux éléments
{ left: true, right: true, bottom: true } 3 Mesure avec dérivation

---

10. Template pour futur nœud

```jsx
// Template générique pour un nouveau node type
import { Handle, Position } from '@xyflow/react';

const NewNodeType = ({ id, data }) => {
  return (
    <div className="new-node">
      {/* Handles configurables */}
      {data.handles?.top && <Handle type="target" position={Position.Top} />}
      {data.handles?.bottom && <Handle type="source" position={Position.Bottom} />}
      {data.handles?.left && <Handle type="target" position={Position.Left} />}
      {data.handles?.right && <Handle type="source" position={Position.Right} />}
      
      {/* Contenu du nœud */}
      <div>{data.label}</div>
    </div>
  );
};

export default NewNodeType;
```

---

Fin du document - Le NodeType ET112 est prêt à être utilisé avec 1 ou 2 handles selon vos besoins de mesure entre deux nœuds.
