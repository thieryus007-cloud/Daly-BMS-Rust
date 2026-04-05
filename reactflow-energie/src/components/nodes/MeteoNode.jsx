const MeteoNode = ({ id, data }) => {
  const {
    label = 'Station Solaire',
    irradiance = 850,
    productionTotal = 31,
    productionLast24h = 30.6,
    productionDay = 31,
    lastUpdate = 'il y a quelques secondes'
  } = data;

  return (
    <div 
      className="meteo-node"
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
        <span style={{ fontSize: '14px' }}>☀️</span>
        <span style={{ fontSize: '8px', color: '#666' }}>{label}</span>
      </div>

      <div style={{ marginBottom: '4px' }}>
        <span style={{ fontSize: '22px', fontWeight: 'bold', color: '#ff9800' }}>{irradiance.toFixed(1)}</span>
        <span style={{ fontSize: '8px', color: '#999' }}> W/m²</span>
      </div>

      <div style={{ marginBottom: '8px' }}>
        <span style={{ fontSize: '16px', fontWeight: 'bold', color: '#ff9800' }}>-{productionTotal}</span>
        <span style={{ fontSize: '8px', color: '#999' }}> kWh</span>
      </div>

      <div style={{ background: '#f5f5f5', borderRadius: '8px', padding: '6px', marginBottom: '6px' }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '7px', color: '#888', marginBottom: '6px' }}>
          <span>Dernières 24 h</span>
          <span style={{ color: '#ff9800', fontWeight: 'bold' }}>{productionLast24h.toFixed(1)} kWh</span>
        </div>
      </div>

      <div style={{ display: 'flex', justifyContent: 'space-between', background: '#f5f5f5', borderRadius: '6px', padding: '4px 6px', marginBottom: '6px' }}>
        <span style={{ fontSize: '7px', color: '#888' }}>Production du jour</span>
        <span style={{ fontSize: '9px', fontWeight: 'bold', color: '#ff9800' }}>{productionDay} kWh</span>
      </div>

      <div style={{ fontSize: '6px', color: '#aaa', textAlign: 'center' }}>
        Dernière mise à jour<br />{lastUpdate}
      </div>
    </div>
  );
};

export default MeteoNode;