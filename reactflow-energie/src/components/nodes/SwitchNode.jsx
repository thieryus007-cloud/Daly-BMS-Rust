import { useState } from 'react';
import { Handle, Position } from '@xyflow/react';

const SwitchNode = ({ id, data, onToggle }) => {
  const [isOn, setIsOn] = useState(data.isOn !== undefined ? data.isOn : true);

  const {
    label = 'Tongou Switch',
    deviceId = 'tongou_3BC764',
    time = '20:17:48',
    power = 0,
    voltage = 0,
    current = 0,
    cosPhi = 0,
    today = 0,
    yesterday = 0,
    total = 0
  } = data;

  const switchColor = isOn ? '#4caf50' : '#f44336';
  const statusText = isOn ? 'ON' : 'OFF';

  const handleToggle = () => {
    const newState = !isOn;
    setIsOn(newState);
    if (onToggle) onToggle(id, newState);
  };

  return (
    <div 
      className="switch-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '140px',
        border: '1.5px solid',
        borderColor: switchColor,
        fontFamily: 'Segoe UI, monospace',
        boxShadow: isOn ? `0 0 4px ${switchColor}` : 'none',
        opacity: isOn ? 1 : 0.7
      }}
    >
      {/* Handle GAUCHE - reçoit de ET112 */}
      <Handle 
        type="target"
        position={Position.Left}
        id="left-input"
        style={{ 
          background: switchColor,
          width: '10px',
          height: '10px',
          left: '-5px'
        }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: '6px', marginBottom: '6px' }}>
        <div style={{ padding: '2px 6px', borderRadius: '10px', fontSize: '7px', fontWeight: 'bold', backgroundColor: switchColor, color: 'white' }}>
          {statusText}
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '3px' }}>
          <span style={{ fontSize: '10px' }}>🔌</span>
          <span style={{ fontWeight: 'bold', color: '#333', fontSize: '8px' }}>{label}</span>
        </div>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '6px', fontSize: '6px', color: '#999', fontFamily: 'monospace' }}>
        <span>{deviceId}</span>
        <span>{time}</span>
      </div>

      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '8px', padding: '4px 0', borderTop: '1px solid #e0e0e0', borderBottom: '1px solid #e0e0e0' }}>
        <button 
          onClick={handleToggle}
          style={{
            width: '36px',
            height: '18px',
            background: isOn ? '#4caf50' : '#ccc',
            borderRadius: '18px',
            border: 'none',
            cursor: 'pointer',
            position: 'relative',
            transition: 'background 0.2s ease',
            padding: 0
          }}
        >
          <span style={{
            position: 'absolute',
            width: '14px',
            height: '14px',
            background: 'white',
            borderRadius: '50%',
            top: '2px',
            left: isOn ? '20px' : '2px',
            transition: 'left 0.2s ease'
          }} />
        </button>
        <span style={{ fontSize: '7px', fontWeight: 'bold', color: switchColor }}>
          {isOn ? 'COMMANDÉ ON' : 'COMMANDÉ OFF'}
        </span>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(4, 1fr)', gap: '4px', marginBottom: '8px' }}>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '3px', textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#888' }}>PUISSANCE</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold', color: switchColor }}>{power.toFixed(1)} W</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '3px', textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#888' }}>TENSION</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{voltage.toFixed(1)} V</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '3px', textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#888' }}>COURANT</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{current.toFixed(2)} A</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '6px', padding: '3px', textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#888' }}>COS Φ</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{cosPhi.toFixed(2)}</div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '6px', marginBottom: '6px', padding: '4px', background: '#f5f5f5', borderRadius: '8px' }}>
        <div style={{ flex: 1, textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#999' }}>AUJOURD'HUI</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{today.toFixed(2)} kWh</div>
        </div>
        <div style={{ flex: 1, textAlign: 'center' }}>
          <div style={{ fontSize: '5px', color: '#999' }}>HIER</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{yesterday.toFixed(2)} kWh</div>
        </div>
        <div style={{ flex: 1, textAlign: 'center', borderLeft: '1px solid #ddd' }}>
          <div style={{ fontSize: '5px', color: '#999' }}>TOTAL</div>
          <div style={{ fontSize: '7px', fontWeight: 'bold', color: '#ff9800' }}>{total.toFixed(1)} kWh</div>
        </div>
      </div>

      <div style={{ textAlign: 'right', fontSize: '7px', color: '#ff9800', cursor: 'pointer', paddingTop: '4px', borderTop: '1px solid #e0e0e0' }}>
        Details →
      </div>
    </div>
  );
};

export default SwitchNode;