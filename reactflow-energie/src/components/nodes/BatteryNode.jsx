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
    energyExported = 0
  } = data;

  const isCharging = current > 0;
  const isDischarging = current < 0;
  const currentColor = isCharging ? '#4caf50' : (isDischarging ? '#f44336' : '#ff9800');

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
        type="source"
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