import { Handle, Position } from '@xyflow/react';

const ATSNode = ({ id, data }) => {
  const {
    label = 'ATS',
    status = 'Source Principale',
    power = 0
  } = data;

  return (
    <div 
      className="ats-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '120px',
        border: '2px solid #2196f3',
        fontFamily: 'Segoe UI, monospace',
        textAlign: 'center',
        boxShadow: '0 1px 3px rgba(0,0,0,0.1)'
      }}
    >
      {/* Handle HAUT - pas de liaison pour le moment */}
      <Handle 
        type="target"
        position={Position.Top}
        id="top-input"
        style={{ 
          background: '#2196f3',
          width: '10px',
          height: '10px',
          top: '-5px'
        }}
      />

      {/* Handle DROIT - relié à ET112 */}
      <Handle 
        type="source"
        position={Position.Right}
        id="right-output"
        style={{ 
          background: '#2196f3',
          width: '10px',
          height: '10px',
          right: '-5px'
        }}
      />

      {/* Handle BAS - relié à Onduleur */}
      <Handle 
        type="source"
        position={Position.Bottom}
        id="bottom-output"
        style={{ 
          background: '#2196f3',
          width: '10px',
          height: '10px',
          bottom: '-5px'
        }}
      />

      <div style={{ fontSize: '14px', marginBottom: '4px' }}>🔄</div>
      <div style={{ fontWeight: 'bold', fontSize: '11px', color: '#2196f3' }}>{label}</div>
      <div style={{ fontSize: '8px', color: '#666' }}>{status}</div>
      <div style={{ fontSize: '10px', fontWeight: 'bold', marginTop: '4px' }}>{Math.abs(power).toFixed(0)} W</div>
    </div>
  );
};

export default ATSNode;