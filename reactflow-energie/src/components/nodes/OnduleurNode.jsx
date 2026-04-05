import { Handle, Position } from '@xyflow/react';

const OnduleurNode = ({ id, data }) => {
  const {
    label = 'Onduleur',
    power = 0,
    voltage = 230,
    efficiency = 94
  } = data;

  const flowColor = power > 0 ? '#4caf50' : '#ff9800';

  return (
    <div 
      className="onduleur-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '120px',
        border: '1.5px solid',
        borderColor: flowColor,
        fontFamily: 'Segoe UI, monospace',
        textAlign: 'center',
        boxShadow: `0 0 4px ${flowColor}`
      }}
    >
      {/* Handle HAUT - reçoit de ATS */}
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

      {/* Handle BAS - envoie vers Shunt */}
      <Handle 
        type="source"
        position={Position.Bottom}
        id="bottom-output"
        style={{ 
          background: flowColor,
          width: '10px',
          height: '10px',
          bottom: '-5px'
        }}
      />

      <div style={{ fontSize: '14px', marginBottom: '4px' }}>⚡</div>
      <div style={{ fontWeight: 'bold', fontSize: '10px', color: '#333' }}>{label}</div>
      <div style={{ fontSize: '18px', fontWeight: 'bold', color: flowColor }}>{Math.abs(power).toFixed(0)}</div>
      <div style={{ fontSize: '8px', color: '#888' }}>W</div>
      <div style={{ display: 'flex', justifyContent: 'space-between', marginTop: '6px', paddingTop: '4px', borderTop: '1px solid #e0e0e0' }}>
        <div>
          <div style={{ fontSize: '6px', color: '#888' }}>Tension</div>
          <div style={{ fontSize: '8px', fontWeight: 'bold' }}>{voltage}V</div>
        </div>
        <div>
          <div style={{ fontSize: '6px', color: '#888' }}>Rendement</div>
          <div style={{ fontSize: '8px', fontWeight: 'bold', color: '#4caf50' }}>{efficiency}%</div>
        </div>
      </div>
    </div>
  );
};

export default OnduleurNode;