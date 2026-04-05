const TemperatureNode = ({ id, data }) => {
  const {
    label = 'Station Météo',
    temperature = 22.5,
    humidity = 53,
    pressure = 1012,
    tempMin24h = 18.5,
    tempMax24h = 26.5,
    lastUpdate = 'il y a quelques secondes'
  } = data;

  return (
    <div 
      className="temperature-node"
      style={{
        backgroundColor: '#ffffff',
        borderRadius: '12px',
        padding: '8px',
        minWidth: '140px',
        border: '1px solid #e0e0e0',
        fontFamily: 'Segoe UI, monospace',
        textAlign: 'center',
        boxShadow: '0 1px 3px rgba(0,0,0,0.1)'
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: '4px', marginBottom: '6px' }}>
        <span style={{ fontSize: '14px' }}>🌡️</span>
        <span style={{ fontSize: '8px', color: '#666' }}>{label}</span>
      </div>

      <div style={{ marginBottom: '8px' }}>
        <div style={{ fontSize: '6px', color: '#888', marginBottom: '4px' }}>Température Extérieure</div>
        <div style={{ display: 'flex', alignItems: 'baseline', justifyContent: 'center', gap: '2px' }}>
          <span style={{ fontSize: '24px', fontWeight: 'bold', color: '#2196f3' }}>{temperature.toFixed(1)}</span>
          <span style={{ fontSize: '8px', color: '#999' }}>°C</span>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '8px', justifyContent: 'center', marginBottom: '8px' }}>
        <div style={{ background: '#f5f5f5', borderRadius: '30px', padding: '5px 8px', textAlign: 'center', minWidth: '55px' }}>
          <div style={{ fontSize: '12px', fontWeight: 'bold', color: '#4caf50' }}>{humidity}%</div>
          <div style={{ fontSize: '5px', color: '#888' }}>Humidité</div>
        </div>
        <div style={{ background: '#f5f5f5', borderRadius: '30px', padding: '5px 8px', textAlign: 'center', minWidth: '55px' }}>
          <div style={{ fontSize: '12px', fontWeight: 'bold', color: '#4caf50' }}>{pressure.toFixed(0)}</div>
          <div style={{ fontSize: '5px', color: '#888' }}>Pression</div>
          <div style={{ fontSize: '5px', color: '#999' }}>hPa</div>
        </div>
      </div>

      <div style={{ background: '#f5f5f5', borderRadius: '8px', padding: '5px', marginBottom: '6px' }}>
        <div style={{ fontSize: '5px', color: '#888', marginBottom: '3px' }}>Dernières 24 h</div>
        <div style={{ display: 'flex', justifyContent: 'center', gap: '10px' }}>
          <span style={{ fontSize: '7px', color: '#64b5f6' }}>{tempMin24h.toFixed(1)}°C min</span>
          <span style={{ fontSize: '7px', color: '#ff8a65' }}>{tempMax24h.toFixed(1)}°C max</span>
        </div>
      </div>

      <div style={{ fontSize: '5px', color: '#bbb', textAlign: 'center' }}>
        Dernière mise à jour<br />{lastUpdate}
      </div>
    </div>
  );
};

export default TemperatureNode;