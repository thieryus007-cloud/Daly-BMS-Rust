import { Handle, Position } from '@xyflow/react';

const ET112Node = ({ id, data }) => {
  const {
    label = 'ET112',
    deviceId = '0x07',
    time = '20:09:42',
    power = 0,
    voltage = 230.4,
    current = 0,
    type = 'load',
    imported = 0,
    exported = 0
  } = data;

  const flowColor = power > 0 ? '#4caf50' : '#ff9800';

  return (
    <div 
      className="et112-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '140px',
        border: '1px solid #e0e0e0',
        fontFamily: 'Segoe UI, monospace',
        boxShadow: '0 1px 3px rgba(0,0,0,0.1)'
      }}
    >
      {/* Handle GAUCHE - reçoit de ATS */}
      <Handle 
        type="target"
        position={Position.Left}
        id="left-input"
        style={{ 
          background: flowColor,
          width: '10px',
          height: '10px',
          left: '-5px'
        }}
      />

      {/* Handle DROIT - envoie vers Switch */}
      <Handle 
        type="source"
        position={Position.Right}
        id="right-output"
        style={{ 
          background: flowColor,
          width: '10px',
          height: '10px',
          right: '-5px'
        }}
      />

      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '6px' }}>
        <div style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
          <span style={{ fontSize: '12px' }}>📊</span>
          <span style={{ fontWeight: 'bold', color: '#333', fontSize: '8px' }}>{label}</span>
        </div>
        <div style={{ background: '#ff3b30', color: 'white', fontSize: '6px', fontWeight: 'bold', padding: '2px 5px', borderRadius: '10px', display: 'flex', alignItems: 'center', gap: '3px' }}>
          <span style={{ width: '4px', height: '4px', background: 'white', borderRadius: '50%', animation: 'livePulse 1s infinite' }}></span>
          LIVE
        </div>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '8px', fontSize: '7px', color: '#999', fontFamily: 'monospace' }}>
        <span>{deviceId}</span>
        <span>{time}</span>
      </div>

      <div style={{ textAlign: 'center', marginBottom: '8px', padding: '5px', background: '#f5f5f5', borderRadius: '8px' }}>
        <div style={{ fontSize: '7px', color: '#888', letterSpacing: '0.5px' }}>PUISSANCE</div>
        <div style={{ fontSize: '16px', fontWeight: 'bold', color: flowColor }}>{Math.abs(power).toFixed(1)} W</div>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '5px', marginBottom: '8px' }}>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>TENSION</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold' }}>{voltage.toFixed(1)} V</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '4px', textAlign: 'center' }}>
          <div style={{ fontSize: '6px', color: '#888' }}>COURANT</div>
          <div style={{ fontSize: '9px', fontWeight: 'bold' }}>{current.toFixed(2)} A</div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '6px', marginBottom: '6px', paddingTop: '5px', borderTop: '1px solid #e0e0e0' }}>
        <div style={{ flex: 1, textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#999' }}>📥 IMPORTÉE</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{imported.toFixed(2)} kWh</div>
        </div>
        <div style={{ flex: 1, textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#999' }}>📤 EXPORTÉE</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{exported.toFixed(2)} kWh</div>
        </div>
      </div>

      <div style={{ textAlign: 'right', fontSize: '7px', color: '#ff9800', cursor: 'pointer', paddingTop: '4px', borderTop: '1px solid #e0e0e0' }}>
        Détails →
      </div>

      <style>{`
        @keyframes livePulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
      `}</style>
    </div>
  );
};

export default ET112Node;