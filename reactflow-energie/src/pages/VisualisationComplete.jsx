import { ReactFlow, useNodesState, useEdgesState, Background, Controls } from '@xyflow/react';
import '@xyflow/react/dist/style.css';
import BatteryNode from '../components/nodes/BatteryNode';
import MPPTNode from '../components/nodes/MPPTNode';
import ShuntNode from '../components/nodes/ShuntNode';
import SwitchNode from '../components/nodes/SwitchNode';
import ET112Node from '../components/nodes/ET112Node';
import MeteoNode from '../components/nodes/MeteoNode';
import TemperatureNode from '../components/nodes/TemperatureNode';
import ATSNode from '../components/nodes/ATSNode';
import OnduleurNode from '../components/nodes/OnduleurNode';

const nodeTypes = {
  battery: BatteryNode,
  mppt: MPPTNode,
  shunt: ShuntNode,
  switch: SwitchNode,
  et112: ET112Node,
  meteo: MeteoNode,
  temperature: TemperatureNode,
  ats: ATSNode,
  onduleur: OnduleurNode,
};

const initialNodes = [
  // ===== EN HAUT A DROITE - Météo et Température (sans handles, centrés à droite) =====
  {
    id: 'meteo-station',
    type: 'meteo',
    position: { x: 100, y: 25 },
    data: {
      label: 'Station Solaire',
      irradiance: 850,
      productionTotal: 31,
      productionLast24h: 30.6,
      productionDay: 31,
      lastUpdate: 'il y a quelques secondes'
    }
  },
  {
    id: 'temp-station',
    type: 'temperature',
    position: { x: 100, y: 250 },
    data: {
      label: 'Station Météo',
      temperature: 22.5,
      humidity: 53,
      pressure: 1012,
      tempMin24h: 18.5,
      tempMax24h: 26.5,
      lastUpdate: 'il y a quelques secondes'
    }
  },

  // ===== LIGNE HAUTE - ATS, ET112, Switch (alignés de gauche à droite) =====
  {
    id: 'ats-main',
    type: 'ats',
    position: { x: 500, y: 100 },
    data: {
      label: 'ATS',
      status: 'Source Principale',
      power: 5000
    }
  },
  {
    id: 'et112-final',
    type: 'et112',
    position: { x: 700, y: 100 },
    data: {
      label: 'ET112',
      deviceId: '0x07',
      time: '20:09:42',
      power: 4800,
      voltage: 230.4,
      current: 20.8,
      imported: 760.30,
      exported: 0.00
    }
  },
  {
    id: 'tongou-switch',
    type: 'switch',
    position: { x: 900, y: 100 },
    data: {
      label: 'Tongou Switch',
      deviceId: 'tongou_3BC764',
      time: '20:17:48',
      isOn: true,
      power: 4750,
      voltage: 231.0,
      current: 20.6,
      cosPhi: 0.95,
      today: 4.26,
      yesterday: 2.62,
      total: 42.3
    }
  },

  // ===== LIGNE MILIEU - Onduleur (sous ATS) =====
  {
    id: 'onduleur-main',
    type: 'onduleur',
    position: { x: 500, y: 300 },
    data: {
      label: 'Onduleur',
      power: 4600,
      voltage: 230,
      efficiency: 94
    }
  },

  // ===== LIGNE BAS - Shunt, MPPT, Batteries =====
  {
    id: 'shunt-main',
    type: 'shunt',
    position: { x: 500, y: 500 },
    data: {
      label: 'Shunt Principal',
      power: 4500,
      voltage: 52.85,
      current: -85.0,
      soc: 85
    }
  },
  {
    id: 'mppt-chargeur',
    type: 'mppt',
    position: { x: 900, y: 450 },
    data: {
      label: 'Chargeur PV',
      totalPower: 2500,
      mppts: [
        { id: 'MPPT-273', voltage: 98.70, current: 12.7, power: 1250 },
        { id: 'MPPT-289', voltage: 98.71, current: 12.7, power: 1250 }
      ],
      energyToday: 12.5,
      energyTotal: 3450
    }
  },
  {
    id: 'battery-360ah',
    type: 'battery',
    position: { x: 400, y: 700 },
    data: {
      label: 'BMS-360Ah',
      soc: 92,
      voltage: 52.8,
      current: -45.0,
      temperature: 14.0,
      power: -2376,
      energyImported: 1250,
      energyExported: 890
    }
  },
  {
    id: 'battery-320ah',
    type: 'battery',
    position: { x: 600, y: 700 },
    data: {
      label: 'BMS-320Ah',
      soc: 94,
      voltage: 52.9,
      current: -40.0,
      temperature: 16.0,
      power: -2116,
      energyImported: 1100,
      energyExported: 780
    }
  }
];

// ===== CONNEXIONS =====
const initialEdges = [
  // ATS (côté DROIT) → ET112 (côté GAUCHE)
  { 
    id: 'e-ats-et112', 
    source: 'ats-main', 
    sourceHandle: 'right-output',
    target: 'et112-final', 
    targetHandle: 'left-input',
    animated: true, 
    style: { stroke: '#2196f3', strokeWidth: 2 } 
  },
  
  // ET112 (côté DROIT) → Switch (côté GAUCHE)
  { 
    id: 'e-et112-switch', 
    source: 'et112-final', 
    sourceHandle: 'right-output',
    target: 'tongou-switch', 
    targetHandle: 'left-input',
    animated: true, 
    style: { stroke: '#ff9800', strokeWidth: 2 } 
  },
  
  // ATS (côté BAS) → Onduleur (côté HAUT)
  { 
    id: 'e-ats-onduleur', 
    source: 'ats-main', 
    sourceHandle: 'bottom-output',
    target: 'onduleur-main', 
    targetHandle: 'top-input',
    animated: true, 
    style: { stroke: '#4caf50', strokeWidth: 2 } 
  },
  
  // Onduleur (côté BAS) → Shunt (côté HAUT)
  { 
    id: 'e-onduleur-shunt', 
    source: 'onduleur-main', 
    sourceHandle: 'bottom-output',
    target: 'shunt-main', 
    targetHandle: 'top-input',
    animated: true, 
    style: { stroke: '#4caf50', strokeWidth: 2 } 
  },
  
  // Shunt (côté DROIT) → MPPT (côté GAUCHE)
  { 
    id: 'e-shunt-mppt', 
    source: 'shunt-main', 
    sourceHandle: 'right-output',
    target: 'mppt-chargeur', 
    targetHandle: 'left-input',
    animated: true, 
    style: { stroke: '#4caf50', strokeWidth: 2 } 
  },
  
    // Batterie 360Ah → Shunt (CORRIGÉ)
  { 
    id: 'e-battery360-shunt', 
    source: 'battery-360ah', 
    sourceHandle: 'top-output',
    target: 'shunt-main', 
    targetHandle: 'bottom-input',
    animated: true, 
    style: { stroke: '#f44336', strokeWidth: 2 } 
  },
  
  // Shunt (côté BAS) → Batterie 320Ah (côté HAUT)
  { 
    id: 'e-battery320-shunt', 
    source: 'battery-320ah', 
    sourceHandle: 'top-output',
    target: 'shunt-main', 
    targetHandle: 'bottom-input',
    animated: true, 
    style: { stroke: '#f44336', strokeWidth: 2 } 
  }
];

// Simulation de données temps réel
const simulateData = () => {
  const time = Date.now() / 1000;
  const variation = 0.9 + Math.sin(time / 10) * 0.1;
  const isDischarging = Math.sin(time / 20) > 0;

  return {
    atsPower: 5000 * variation,
    et112Power: 4800 * variation,
    switchPower: 4750 * variation,
    onduleurPower: 4600 * variation,
    shuntPower: (isDischarging ? -4500 : +4200) * variation,
    shuntCurrent: (isDischarging ? -85 : +80) * variation,
    battery360: {
      current: isDischarging ? -45 * variation : +42 * variation,
      power: isDischarging ? -2376 * variation : +2218 * variation,
      soc: Math.max(0, Math.min(100, 92 + (isDischarging ? -0.05 : +0.05)))
    },
    battery320: {
      current: isDischarging ? -40 * variation : +38 * variation,
      power: isDischarging ? -2116 * variation : +2010 * variation,
      soc: Math.max(0, Math.min(100, 94 + (isDischarging ? -0.04 : +0.04)))
    },
    mpptPower: 2500 * (0.8 + Math.random() * 0.4),
    switchState: !isDischarging
  };
};

function VisualisationComplete() {
  const [nodes, setNodes, onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  setInterval(() => {
    const newData = simulateData();
    const isDischarging = newData.battery360.current < 0;

    setNodes((nds) =>
      nds.map((node) => {
        switch (node.id) {
          case 'ats-main':
            return { ...node, data: { ...node.data, power: newData.atsPower } };
          case 'et112-final':
            return { ...node, data: { ...node.data, power: newData.et112Power, current: newData.et112Power / 230.4 } };
          case 'tongou-switch':
            return { ...node, data: { ...node.data, isOn: newData.switchState, power: newData.switchPower } };
          case 'onduleur-main':
            return { ...node, data: { ...node.data, power: newData.onduleurPower } };
          case 'shunt-main':
            return { ...node, data: { ...node.data, power: Math.abs(newData.shuntPower), current: Math.abs(newData.shuntCurrent), soc: (newData.battery360.soc + newData.battery320.soc) / 2 } };
          case 'battery-360ah':
            return { ...node, data: { ...node.data, current: newData.battery360.current, power: newData.battery360.power, soc: newData.battery360.soc } };
          case 'battery-320ah':
            return { ...node, data: { ...node.data, current: newData.battery320.current, power: newData.battery320.power, soc: newData.battery320.soc } };
          case 'mppt-chargeur':
            return { ...node, data: { ...node.data, totalPower: newData.mpptPower, mppts: node.data.mppts.map(m => ({ ...m, power: newData.mpptPower / 2, current: (newData.mpptPower / 2) / 98.7 })) } };
          default:
            return node;
        }
      })
    );

    setEdges((eds) =>
      eds.map((edge) => {
        let strokeColor = '#ff9800';
        if (edge.source === 'ats-main' || edge.source === 'onduleur-main') strokeColor = '#4caf50';
        if (edge.source === 'shunt-main' && edge.target === 'mppt-chargeur') strokeColor = '#4caf50';
        if (edge.source === 'shunt-main' && (edge.target === 'battery-360ah' || edge.target === 'battery-320ah')) strokeColor = isDischarging ? '#f44336' : '#4caf50';
        if (edge.source === 'et112-final' && edge.target === 'tongou-switch') strokeColor = '#ff9800';
        return { ...edge, style: { stroke: strokeColor, strokeWidth: 2 } };
      })
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
        🔌 ATS → ET112 → Switch | ATS → Onduleur → Shunt → MPPT | Shunt → Batteries
      </div>

      <ReactFlow
        nodes={nodes}
        edges={edges}
        nodeTypes={nodeTypes}
        onNodesChange={onNodesChange}
        onEdgesChange={onEdgesChange}
        fitView
        defaultViewport={{ x: 0, y: 0, zoom: 0.6 }}
      >
        <Background color="#ccc" gap={16} />
        <Controls />
      </ReactFlow>
    </div>
  );
}

export default VisualisationComplete;