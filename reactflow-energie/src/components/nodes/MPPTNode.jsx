import { Handle, Position } from '@xyflow/react';

const MPPTNode = ({ id, data }) => {
  const {
    label = 'Chargeur PV',
    totalPower = 0,
    mppts = [],
    energyToday = 0,
    energyTotal = 0
  } = data;

  const totalColor = totalPower > 0 ? '#4caf50' : '#888';

  return (
    <div 
      className="mppt-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '140px',
        border: '1.5px solid',
        borderColor: totalColor,
        fontFamily: 'Segoe UI, monospace',
        boxShadow: totalPower > 0 ? `0 0 4px ${totalColor}` : 'none'
      }}
    >
      {/* Handle GAUCHE - reçoit du Shunt */}
      <Handle 
        type="target"
        position={Position.Left}
        id="left-input"
        style={{ 
          background: totalColor,
          width: '10px',
          height: '10px',
          left: '-5px'
        }}
      />

      <div style={{ display: 'flex', alignItems: 'center', gap: '5px', marginBottom: '6px', paddingBottom: '4px', borderBottom: '1px solid #e0e0e0' }}>
        <span style={{ fontSize: '10px' }}>☀️</span>
        <span style={{ fontWeight: 'bold', color: '#333', fontSize: '8px', flex: 1 }}>{label}</span>
        {Math.round(totalPower) > 0 && (
          <span style={{ fontSize: '6px', padding: '1px 5px', borderRadius: '10px', backgroundColor: totalColor, color: 'white' }}>ACTIF</span>
        )}
      </div>

      <div style={{ textAlign: 'center', marginBottom: '8px', padding: '5px', background: '#f5f5f5', borderRadius: '10px' }}>
        <span style={{ fontSize: '22px', fontWeight: 'bold', color: totalColor }}>{Math.round(totalPower)}</span>
        <span style={{ fontSize: '8px', marginLeft: '2px' }}>W</span>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '6px', marginBottom: '8px' }}>
        {mppts.map((mppt, index) => {
          const mpptColor = mppt.power > 0 ? '#4caf50' : '#888';
          return (
            <div key={mppt.id || index} style={{ background: '#f5f5f5', borderRadius: '8px', padding: '5px' }}>
              <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '4px' }}>
                <span style={{ fontSize: '7px', fontWeight: 'bold', color: '#ff9800', fontFamily: 'monospace' }}>{mppt.id}</span>
                <span style={{ fontSize: '9px', fontWeight: 'bold', color: mpptColor }}>{Math.round(mppt.power)} W</span>
              </div>
              <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '5px' }}>
                <div style={{ background: '#ffffff', borderRadius: '5px', padding: '3px', textAlign: 'center', border: '1px solid #e0e0e0' }}>
                  <div style={{ fontSize: '5px', color: '#888' }}>Tension</div>
                  <div style={{ fontSize: '8px', fontWeight: 'bold' }}>{mppt.voltage.toFixed(2)} V</div>
                </div>
                <div style={{ background: '#ffffff', borderRadius: '5px', padding: '3px', textAlign: 'center', border: '1px solid #e0e0e0' }}>
                  <div style={{ fontSize: '5px', color: '#888' }}>Courant</div>
                  <div style={{ fontSize: '8px', fontWeight: 'bold', color: mpptColor }}>{mppt.current.toFixed(1)} A</div>
                </div>
              </div>
            </div>
          );
        })}
      </div>

      {(energyToday > 0 || energyTotal > 0) && (
        <div style={{ display: 'flex', gap: '6px', marginTop: '6px', paddingTop: '5px', borderTop: '1px solid #e0e0e0' }}>
          {energyToday > 0 && (
            <div style={{ flex: 1, background: '#f5f5f5', borderRadius: '5px', padding: '3px', textAlign: 'center' }}>
              <div style={{ fontSize: '5px', color: '#888' }}>Aujourd'hui</div>
              <div style={{ fontSize: '7px', fontWeight: 'bold' }}>{energyToday.toFixed(1)} kWh</div>
            </div>
          )}
          {energyTotal > 0 && (
            <div style={{ flex: 1, background: '#f5f5f5', borderRadius: '5px', padding: '3px', textAlign: 'center' }}>
              <div style={{ fontSize: '5px', color: '#888' }}>Total</div>
              <div style={{ fontSize: '7px', fontWeight: 'bold', color: '#ff9800' }}>{energyTotal.toFixed(0)} kWh</div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};

export default MPPTNode;