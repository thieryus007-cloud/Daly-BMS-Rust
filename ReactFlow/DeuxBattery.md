## 1. Fichier : src/components/nodes/BatteryNode.jsx

```jsx
import { Handle, Position } from '@xyflow/react';

const BatteryNode = ({ id, data }) => {
  const {
    label = 'BMS',
    soc = 0,
    voltage = 0,
    current = 0,
    temperature = 0,
    power = 0,
    energyImported = 0,
    energyExported = 0,
    energyTotal = 0
  } = data;

  const isCharging = current > 0;
  const isDischarging = current < 0;
  const currentColor = isCharging ? '#4caf50' : (isDischarging ? '#f44336' : '#ff9800');
  
  // Handle sur le HAUT du nœud (sortie vers Shunt)
  const handleType = 'source';

  return (
    <div 
      className="battery-node"
      style={{ 
        borderColor: currentColor, 
        boxShadow: `0 0 4px ${currentColor}`,
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '140px',
        border: '1.5px solid',
        fontFamily: 'Segoe UI, monospace'
      }}
    >
      {/* Handle HAUT - connexion vers le Shunt */}
      <Handle 
        type={handleType}
        position={Position.Top}
        id="top-output"
        style={{ 
          background: currentColor,
          width: '10px',
          height: '10px',
          top: '-5px'
        }}
      />

      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px' }}>
        <span>🔋 {label}</span>
        <span style={{ color: currentColor, fontSize: '9px' }}>
          {isCharging ? 'CHARGE' : (isDischarging ? 'DÉCHARGE' : 'IDLE')}
        </span>
      </div>

      <div style={{ textAlign: 'center', marginBottom: '8px' }}>
        <span style={{ fontSize: '18px', fontWeight: 'bold' }}>{soc}%</span>
        <div style={{ background: '#e0e0e0', borderRadius: '6px', height: '5px', marginTop: '4px' }}>
          <div style={{ width: `${soc}%`, height: '5px', background: currentColor, borderRadius: '6px' }} />
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '5px', marginBottom: '8px' }}>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>TENSION</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold' }}>{voltage.toFixed(1)}V</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>COURANT</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold', color: currentColor }}>{Math.abs(current).toFixed(1)}A</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>TEMP.</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold' }}>{temperature.toFixed(1)}°C</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>PUISSANCE</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold', color: currentColor }}>{Math.abs(power).toFixed(0)}W</div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '4px', borderTop: '1px solid #e0e0e0', paddingTop: '5px' }}>
        <div style={{ flex: 1, background: '#f5f5f5', borderRadius: '5px', padding: '3px', textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#888' }}>IMPORTÉ</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{energyImported.toFixed(0)} kWh</div>
        </div>
        <div style={{ flex: 1, background: '#f5f5f5', borderRadius: '5px', padding: '3px', textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#888' }}>EXPORTÉ</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{energyExported.toFixed(0)} kWh</div>
        </div>
      </div>
    </div>
  );
};

export default BatteryNode;
```

---

## 2. Fichier : src/components/nodes/ShuntNode.jsx

```jsx
import { Handle, Position } from '@xyflow/react';

const ShuntNode = ({ id, data }) => {
  const {
    label = 'Shunt',
    power = 0,
    voltage = 0,
    current = 0,
    soc = 0
  } = data;

  const isCharging = current > 0;
  const isDischarging = current < 0;
  const flowColor = isCharging ? '#4caf50' : (isDischarging ? '#f44336' : '#ff9800');

  return (
    <div 
      className="shunt-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '140px',
        border: '1.5px solid',
        borderColor: flowColor,
        fontFamily: 'Segoe UI, monospace',
        boxShadow: `0 0 4px ${flowColor}`
      }}
    >
      {/* Handle BAS - reçoit des batteries */}
      <Handle 
        type="target"
        position={Position.Bottom}
        id="bottom-input"
        style={{ 
          background: flowColor,
          width: '10px',
          height: '10px',
          bottom: '-5px'
        }}
      />

      <div style={{ textAlign: 'center', marginBottom: '8px' }}>
        <span style={{ fontSize: '9px', color: '#888' }}>{label}</span>
        <div style={{ fontSize: '22px', fontWeight: 'bold', color: flowColor }}>
          {Math.abs(power).toFixed(0)} W
        </div>
      </div>

      <div style={{ textAlign: 'center', marginBottom: '8px' }}>
        <div style={{ position: 'relative', width: '60px', height: '60px', margin: '0 auto' }}>
          <svg viewBox="0 0 100 100" style={{ width: '100%', height: '100%', transform: 'rotate(-90deg)' }}>
            <circle cx="50" cy="50" r="45" fill="none" stroke="#e0e0e0" strokeWidth="8" />
            <circle 
              cx="50" cy="50" r="45" fill="none" stroke={flowColor} strokeWidth="8"
              strokeDasharray={`${(soc / 100) * 283} 283`}
              strokeLinecap="round"
            />
          </svg>
          <div style={{ position: 'absolute', top: '50%', left: '50%', transform: 'translate(-50%, -50%)', fontSize: '11px', fontWeight: 'bold' }}>
            {soc}%
          </div>
        </div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '5px' }}>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>TENSION</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold' }}>{voltage.toFixed(2)} V</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>COURANT</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold', color: flowColor }}>{Math.abs(current).toFixed(1)} A</div>
        </div>
      </div>
    </div>
  );
};

export default ShuntNode;
```

---

## 3. Fichier : src/pages/VisualisationComplete.jsx

```jsx
import { ReactFlow, useNodesState, useEdgesState, Background, Controls } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import BatteryNode from '../components/nodes/BatteryNode';
import ShuntNode from '../components/nodes/ShuntNode';

// Types de nœuds personnalisés
const nodeTypes = {
  battery: BatteryNode,
  shunt: ShuntNode,
};

// Données initiales - Deux batteries + Shunt
const initialNodes = [
  // Batterie 360Ah
  {
    id: 'battery-360ah',
    type: 'battery',
    position: { x: 150, y: 300 },
    data: {
      label: 'BMS-360Ah',
      soc: 92,
      voltage: 52.8,
      current: -17.3,
      temperature: 14.0,
      power: -910,
      energyImported: 1250,
      energyExported: 890
    }
  },
  
  // Batterie 320Ah
  {
    id: 'battery-320ah',
    type: 'battery',
    position: { x: 450, y: 300 },
    data: {
      label: 'BMS-320Ah',
      soc: 94,
      voltage: 52.9,
      current: -13.1,
      temperature: 16.0,
      power: -690,
      energyImported: 1100,
      energyExported: 780
    }
  },
  
  // Shunt (central)
  {
    id: 'shunt-main',
    type: 'shunt',
    position: { x: 300, y: 100 },
    data: {
      label: 'Shunt Principal',
      power: 1600,
      voltage: 52.85,
      current: -30.4,
      soc: 93
    }
  }
];

// Connexions : Batteries (handle HAUT) → Shunt (handle BAS)
const initialEdges = [
  {
    id: 'edge-360ah-shunt',
    source: 'battery-360ah',
    sourceHandle: 'top-output',
    target: 'shunt-main',
    targetHandle: 'bottom-input',
    style: { stroke: '#f44336', strokeWidth: 2 },
    animated: true
  },
  {
    id: 'edge-320ah-shunt',
    source: 'battery-320ah',
    sourceHandle: 'top-output',
    target: 'shunt-main',
    targetHandle: 'bottom-input',
    style: { stroke: '#f44336', strokeWidth: 2 },
    animated: true
  }
];

// Simulation de données temps réel
const simulateData = () => {
  // Valeurs fluctuantes pour la simulation
  const time = Date.now() / 1000;
  const variation = 0.95 + Math.sin(time / 10) * 0.05;
  
  return {
    battery360: {
      current: -17.3 * variation,
      power: -910 * variation,
      soc: 92 - (1 - variation) * 0.5
    },
    battery320: {
      current: -13.1 * variation,
      power: -690 * variation,
      soc: 94 - (1 - variation) * 0.3
    },
    shunt: {
      current: (-17.3 - 13.1) * variation,
      power: (-910 - 690) * variation,
      soc: (92 + 94) / 2 - (1 - variation) * 0.4
    }
  };
};

function VisualisationComplete() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  // Mise à jour temps réel toutes les 2 secondes
  setInterval(() => {
    const newData = simulateData();
    
    setNodes((nds) =>
      nds.map((node) => {
        if (node.id === 'battery-360ah') {
          return {
            ...node,
            data: {
              ...node.data,
              current: newData.battery360.current,
              power: newData.battery360.power,
              soc: Math.max(0, Math.min(100, newData.battery360.soc))
            }
          };
        }
        if (node.id === 'battery-320ah') {
          return {
            ...node,
            data: {
              ...node.data,
              current: newData.battery320.current,
              power: newData.battery320.power,
              soc: Math.max(0, Math.min(100, newData.battery320.soc))
            }
          };
        }
        if (node.id === 'shunt-main') {
          return {
            ...node,
            data: {
              ...node.data,
              current: newData.shunt.current,
              power: newData.shunt.power,
              soc: Math.max(0, Math.min(100, newData.shunt.soc))
            }
          };
        }
        return node;
      })
    );

    // Mise à jour des couleurs des edges en fonction du flux
    const isDischarging = newData.battery360.current < 0 && newData.battery320.current < 0;
    setEdges((eds) =>
      eds.map((edge) => ({
        ...edge,
        style: { stroke: isDischarging ? '#f44336' : '#4caf50', strokeWidth: 2 },
        animated: true
      }))
    );
  }, 2000);

  return (
    <div style={{ width: '100vw', height: '100vh', backgroundColor: '#f0f2f5' }}>
      <div style={{
        position: 'absolute',
        top: 10,
        left: 10,
        zIndex: 10,
        background: '#fff',
        padding: '6px 12px',
        borderRadius: 8,
        color: '#333',
        fontSize: 12,
        fontFamily: 'monospace',
        boxShadow: '0 1px 3px rgba(0,0,0,0.1)'
      }}>
        🔋 Deux batteries connectées au Shunt | Données temps réel
      </div>

      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        defaultViewport={{ x: 0, y: 0, zoom: 0.8 }}
      >
        <Background color="#ccc" gap={16} />
        <Controls />
      </ReactFlow>
    </div>
  );
}

export default VisualisationComplete;
```

---

## 4. Fichier : src/App.jsx

```jsx
import VisualisationComplete from './pages/VisualisationComplete';

function App() {
  return <VisualisationComplete />;
}

export default App;
```

---

# 5. Récapitulatif des connexions

```
                    ┌─────────────────┐
                    │   Shunt Main    │
                    │    1600 W       │
                    │     93%         │
                    └────────┬────────┘
                             │ (Handle BAS - target)
              ┌──────────────┼──────────────┐
              │              │              │
    (Handle Haut)     (Handle Haut)   (Handle Haut)
         source            source          source
              │              │              │
    ┌─────────┴─────────┐ ┌──┴──────────────┐
    │  Battery 360Ah    │ │  Battery 320Ah  │
    │     -910 W        │ │     -690 W      │
    │      92%          │ │      94%        │
    └───────────────────┘ └─────────────────┘
```

---

Résumé des modifications

Fichier Modifications
BatteryNode.jsx Handle sur Position.Top (type source)
ShuntNode.jsx Handle sur Position.Bottom (type target)
VisualisationComplete.jsx Positions des nœuds, edges connectés, simulation temps réel
App.jsx Point d'entrée simplifié

---

Copiez ces 4 fichiers dans votre projet et exécutez npm run dev. Les deux batteries seront connectées au shunt avec des edges animés et colorés selon le sens du flux.
