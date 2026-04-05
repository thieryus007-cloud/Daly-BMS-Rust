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
      {/* Handle HAUT - reçoit de l'Onduleur */}
      <Handle 
        type="target"
        position={Position.Top}
        id="top-input"
        style={{ 
          background: flowColor,
          width: '10px',
          height: '10px',
          top: '-5px'
        }}
      />

      {/* Handle DROIT - relié à MPPT */}
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

      {/* Handle BAS - relié aux Batteries */}
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
        <div style={{ fontSize: '14px', fontWeight: 'bold', color: flowColor }}>
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