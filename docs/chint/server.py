from flask import Flask, render_template_string, jsonify
import serial
import time
import threading

app = Flask(__name__)

# Configuration
PORT = 'COM5'
BAUDRATE = 9600
ADDRESS = 6
ser = None
lock = threading.Lock()

def calculate_crc(data):
    crc = 0xFFFF
    for byte in data:
        crc ^= byte
        for _ in range(8):
            if crc & 0x0001:
                crc = (crc >> 1) ^ 0xA001
            else:
                crc >>= 1
    return crc

def build_frame(func, reg, value=None):
    data = bytes([ADDRESS, func, (reg >> 8) & 0xFF, reg & 0xFF])
    if func == 0x03:
        data += bytes([0x00, 0x01])
    elif func == 0x06:
        data += bytes([(value >> 8) & 0xFF, value & 0xFF])
    crc = calculate_crc(data)
    return data + bytes([crc & 0xFF, (crc >> 8) & 0xFF])

def send_frame(frame):
    with lock:
        ser.write(frame)
        time.sleep(0.15)
        return ser.read(256)

def read_register(reg):
    frame = build_frame(0x03, reg)
    resp = send_frame(frame)
    if resp and len(resp) >= 5 and resp[1] == 0x03:
        return (resp[3] << 8) | resp[4]
    return None

def write_register(reg, value):
    frame = build_frame(0x06, reg, value)
    resp = send_frame(frame)
    return resp is not None and len(resp) > 0

HTML = """
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CHINT ATS - Supervision Modbus</title>
    <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        
        body {
            font-family: 'Inter', sans-serif;
            background: linear-gradient(135deg, #0f172a 0%, #1e293b 100%);
            color: #f1f5f9;
            min-height: 100vh;
            padding: 24px;
        }
        
        .container {
            max-width: 1400px;
            margin: 0 auto;
        }
        
        /* Header */
        .header {
            background: linear-gradient(135deg, #1e293b 0%, #0f172a 100%);
            border-radius: 24px;
            padding: 24px 32px;
            margin-bottom: 24px;
            border: 1px solid rgba(255,255,255,0.1);
            box-shadow: 0 8px 32px rgba(0,0,0,0.2);
        }
        
        .header h1 {
            font-size: 28px;
            font-weight: 700;
            background: linear-gradient(135deg, #fbbf24, #f59e0b);
            -webkit-background-clip: text;
            -webkit-text-fill-color: transparent;
            background-clip: text;
            margin-bottom: 8px;
        }
        
        .header .subtitle {
            color: #94a3b8;
            font-size: 14px;
        }
        
        .status-bar {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-top: 16px;
            padding-top: 16px;
            border-top: 1px solid rgba(255,255,255,0.1);
        }
        
        .status {
            display: flex;
            align-items: center;
            gap: 12px;
        }
        
        .status-led {
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: #ef4444;
            box-shadow: 0 0 8px #ef4444;
            transition: all 0.3s;
        }
        
        .status-led.connected {
            background: #22c55e;
            box-shadow: 0 0 8px #22c55e;
        }
        
        .status-text {
            font-weight: 500;
            font-size: 14px;
        }
        
        .refresh-btn {
            background: linear-gradient(135deg, #3b82f6, #2563eb);
            border: none;
            color: white;
            padding: 10px 24px;
            border-radius: 40px;
            font-weight: 600;
            font-size: 14px;
            cursor: pointer;
            transition: all 0.2s;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        
        .refresh-btn:hover {
            transform: scale(1.02);
            background: linear-gradient(135deg, #2563eb, #1d4ed8);
        }
        
        .refresh-btn:active {
            transform: scale(0.98);
        }
        
        /* Grid */
        .grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(380px, 1fr));
            gap: 24px;
            margin-bottom: 24px;
        }
        
        /* Cards */
        .card {
            background: rgba(30, 41, 59, 0.8);
            backdrop-filter: blur(10px);
            border-radius: 20px;
            padding: 20px;
            border: 1px solid rgba(255,255,255,0.1);
            transition: transform 0.2s, box-shadow 0.2s;
        }
        
        .card:hover {
            transform: translateY(-2px);
            box-shadow: 0 12px 32px rgba(0,0,0,0.3);
        }
        
        .card-title {
            font-size: 16px;
            font-weight: 600;
            color: #94a3b8;
            margin-bottom: 16px;
            display: flex;
            align-items: center;
            gap: 8px;
            border-bottom: 1px solid rgba(255,255,255,0.1);
            padding-bottom: 12px;
        }
        
        .card-title span {
            font-size: 20px;
        }
        
        .card-content {
            display: flex;
            flex-direction: column;
            gap: 12px;
        }
        
        .data-row {
            display: flex;
            justify-content: space-between;
            align-items: baseline;
            padding: 8px 0;
            border-bottom: 1px solid rgba(255,255,255,0.05);
        }
        
        .data-label {
            font-size: 13px;
            color: #94a3b8;
        }
        
        .data-value {
            font-weight: 600;
            font-size: 18px;
            font-family: 'Monaco', 'Menlo', monospace;
        }
        
        .data-value.large {
            font-size: 28px;
            font-weight: 700;
        }
        
        .data-unit {
            font-size: 12px;
            color: #64748b;
            margin-left: 4px;
        }
        
        .badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 12px;
            font-weight: 500;
        }
        
        .badge-normal {
            background: rgba(34, 197, 94, 0.2);
            color: #4ade80;
        }
        
        .badge-warning {
            background: rgba(245, 158, 11, 0.2);
            color: #fbbf24;
        }
        
        .badge-danger {
            background: rgba(239, 68, 68, 0.2);
            color: #f87171;
        }
        
        .badge-info {
            background: rgba(59, 130, 246, 0.2);
            color: #60a5fa;
        }
        
        /* Status grid inside card */
        .status-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 12px;
        }
        
        .status-item {
            text-align: center;
            padding: 12px;
            background: rgba(0,0,0,0.2);
            border-radius: 12px;
        }
        
        .status-item-label {
            font-size: 11px;
            color: #94a3b8;
            margin-bottom: 8px;
        }
        
        .status-item-value {
            font-size: 14px;
            font-weight: 600;
        }
        
        /* Buttons */
        .action-buttons {
            display: flex;
            gap: 12px;
            flex-wrap: wrap;
            margin-top: 16px;
        }
        
        .action-btn {
            background: rgba(59, 130, 246, 0.2);
            border: 1px solid rgba(59, 130, 246, 0.5);
            color: #60a5fa;
            padding: 8px 16px;
            border-radius: 40px;
            font-size: 12px;
            font-weight: 500;
            cursor: pointer;
            transition: all 0.2s;
        }
        
        .action-btn:hover {
            background: #3b82f6;
            color: white;
            border-color: #3b82f6;
        }
        
        .action-btn.warning:hover {
            background: #f59e0b;
            border-color: #f59e0b;
            color: white;
        }
        
        .action-btn.danger:hover {
            background: #ef4444;
            border-color: #ef4444;
            color: white;
        }
        
        /* Loading */
        .loading {
            opacity: 0.6;
            pointer-events: none;
        }
        
        /* Logs */
        .logs-card {
            background: rgba(15, 23, 42, 0.9);
            border-radius: 20px;
            padding: 20px;
            border: 1px solid rgba(255,255,255,0.1);
        }
        
        .logs-title {
            font-size: 16px;
            font-weight: 600;
            margin-bottom: 16px;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        
        .logs-area {
            background: #0f172a;
            border-radius: 12px;
            padding: 12px;
            height: 200px;
            overflow-y: auto;
            font-family: monospace;
            font-size: 11px;
        }
        
        .log-entry {
            padding: 4px 0;
            border-bottom: 1px solid rgba(255,255,255,0.05);
            color: #94a3b8;
        }
        
        .log-entry.success {
            color: #4ade80;
        }
        
        .log-entry.error {
            color: #f87171;
        }
        
        .clear-log {
            margin-top: 12px;
            background: none;
            border: 1px solid rgba(255,255,255,0.2);
            color: #94a3b8;
            padding: 6px 12px;
            border-radius: 8px;
            cursor: pointer;
            font-size: 11px;
        }
        
        footer {
            text-align: center;
            margin-top: 24px;
            font-size: 12px;
            color: #475569;
        }
    </style>
</head>
<body>
<div class="container">
    <div class="header">
        <h1>⚡ CHINT ATS - Supervision</h1>
        <div class="subtitle">Automatic Transfer Switch · Modbus RTU · NXZ(H)MN / NZ5(H)M</div>
        <div class="status-bar">
            <div class="status">
                <div class="status-led" id="statusLed"></div>
                <div class="status-text" id="statusText">Déconnecté</div>
                <div class="status-text" style="color:#64748b;">| Adresse: 6 | 9600 Even</div>
            </div>
            <button class="refresh-btn" id="refreshBtn" onclick="refreshAll()">
                🔄 Actualiser toutes les données
            </button>
        </div>
    </div>
    
    <div class="grid">
        <!-- Tensions Source I -->
        <div class="card">
            <div class="card-title">
                <span>🔵</span> Source I - Tensions
            </div>
            <div class="card-content">
                <div class="data-row">
                    <span class="data-label">Phase A (L1-N)</span>
                    <span class="data-value" id="voltage_n1_a">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Phase B (L2-N)</span>
                    <span class="data-value" id="voltage_n1_b">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Phase C (L3-N)</span>
                    <span class="data-value" id="voltage_n1_c">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Fréquence</span>
                    <span class="data-value" id="frequency_n1">--- <span class="data-unit">Hz</span></span>
                </div>
            </div>
        </div>
        
        <!-- Tensions Source II -->
        <div class="card">
            <div class="card-title">
                <span>🟠</span> Source II - Tensions
            </div>
            <div class="card-content">
                <div class="data-row">
                    <span class="data-label">Phase A (L1-N)</span>
                    <span class="data-value" id="voltage_n2_a">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Phase B (L2-N)</span>
                    <span class="data-value" id="voltage_n2_b">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Phase C (L3-N)</span>
                    <span class="data-value" id="voltage_n2_c">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Fréquence</span>
                    <span class="data-value" id="frequency_n2">--- <span class="data-unit">Hz</span></span>
                </div>
            </div>
        </div>
        
        <!-- État des sources -->
        <div class="card">
            <div class="card-title">
                <span>📊</span> État des sources
            </div>
            <div class="card-content">
                <div class="status-grid" id="powerStatusGrid">
                    <div class="status-item">
                        <div class="status-item-label">Source I - A</div>
                        <div class="status-item-value" id="power_n1_a">---</div>
                    </div>
                    <div class="status-item">
                        <div class="status-item-label">Source I - B</div>
                        <div class="status-item-value" id="power_n1_b">---</div>
                    </div>
                    <div class="status-item">
                        <div class="status-item-label">Source I - C</div>
                        <div class="status-item-value" id="power_n1_c">---</div>
                    </div>
                    <div class="status-item">
                        <div class="status-item-label">Source II - A</div>
                        <div class="status-item-value" id="power_n2_a">---</div>
                    </div>
                    <div class="status-item">
                        <div class="status-item-label">Source II - B</div>
                        <div class="status-item-value" id="power_n2_b">---</div>
                    </div>
                    <div class="status-item">
                        <div class="status-item-label">Source II - C</div>
                        <div class="status-item-value" id="power_n2_c">---</div>
                    </div>
                </div>
            </div>
        </div>
        
        <!-- État du commutateur -->
        <div class="card">
            <div class="card-title">
                <span>🔀</span> État du commutateur
            </div>
            <div class="card-content">
                <div class="data-row">
                    <span class="data-label">Position Source I</span>
                    <span class="data-value" id="switch_n1">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Position Source II</span>
                    <span class="data-value" id="switch_n2">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Position double (off)</span>
                    <span class="data-value" id="switch_mid">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Mode</span>
                    <span class="data-value" id="switch_mode">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Télécommande</span>
                    <span class="data-value" id="switch_remote">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Générateur</span>
                    <span class="data-value" id="switch_gen">---</span>
                </div>
            </div>
        </div>
        
        <!-- Statistiques -->
        <div class="card">
            <div class="card-title">
                <span>📈</span> Statistiques & Historique
            </div>
            <div class="card-content">
                <div class="data-row">
                    <span class="data-label">Nb commutations Source I</span>
                    <span class="data-value" id="count_n1">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Nb commutations Source II</span>
                    <span class="data-value" id="count_n2">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Temps de fonctionnement</span>
                    <span class="data-value" id="runtime">--- <span class="data-unit">heures</span></span>
                </div>
            </div>
        </div>
        
        <!-- Tensions maximales -->
        <div class="card">
            <div class="card-title">
                <span>📈</span> Tensions maximales (historique)
            </div>
            <div class="card-content">
                <div class="data-row">
                    <span class="data-label">Source I - Max A/B/C</span>
                    <span class="data-value" id="max_n1">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Source II - Max A/B/C</span>
                    <span class="data-value" id="max_n2">---</span>
                </div>
            </div>
        </div>
        
        <!-- Informations produit -->
        <div class="card">
            <div class="card-title">
                <span>ℹ️</span> Informations produit
            </div>
            <div class="card-content">
                <div class="data-row">
                    <span class="data-label">Version logicielle</span>
                    <span class="data-value" id="sw_version">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Adresse Modbus</span>
                    <span class="data-value" id="modbus_addr">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Baud rate</span>
                    <span class="data-value" id="modbus_baud">---</span>
                </div>
                <div class="data-row">
                    <span class="data-label">Parité</span>
                    <span class="data-value" id="modbus_parity">---</span>
                </div>
            </div>
        </div>
        
        <!-- Commandes -->
        <div class="card">
            <div class="card-title">
                <span>🎮</span> Commandes à distance
            </div>
            <div class="card-content">
                <div class="action-buttons">
                    <button class="action-btn" onclick="sendCommand('remote_on')">📡 Activer télécommande</button>
                    <button class="action-btn" onclick="sendCommand('remote_off')">🔒 Désactiver télécommande</button>
                    <button class="action-btn warning" onclick="sendCommand('force_source1')">🔵 Forcer Source I</button>
                    <button class="action-btn warning" onclick="sendCommand('force_source2')">🟠 Forcer Source II</button>
                    <button class="action-btn danger" onclick="sendCommand('force_double')">⏹️ Forcer double déclenché</button>
                </div>
                <div class="data-row" style="margin-top: 12px;">
                    <span class="data-label">⚠️ Attention</span>
                    <span class="data-value" style="font-size: 11px; color:#fbbf24;">Activer télécommande avant forçage</span>
                </div>
            </div>
        </div>
    </div>
    
    <!-- Logs -->
    <div class="logs-card">
        <div class="logs-title">
            <span>📋</span> Journal des communications
        </div>
        <div class="logs-area" id="logsArea">
            <div class="log-entry">✨ Prêt - Cliquez sur "Actualiser"</div>
        </div>
        <button class="clear-log" onclick="clearLogs()">🗑️ Effacer le journal</button>
    </div>
    
    <footer>
        CHINT ATS · Modbus RTU · Données temps réel · Mise à jour manuelle
    </footer>
</div>

<script>
    let logEntries = [];
    
    function addLog(msg, type = 'info') {
        const logsArea = document.getElementById('logsArea');
        const entry = document.createElement('div');
        entry.className = `log-entry ${type}`;
        const time = new Date().toLocaleTimeString();
        const prefix = type === 'error' ? '❌' : (type === 'success' ? '✅' : 'ℹ️');
        entry.innerHTML = `[${time}] ${prefix} ${msg}`;
        logsArea.appendChild(entry);
        entry.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        
        // Limiter le nombre de logs
        while (logsArea.children.length > 100) {
            logsArea.removeChild(logsArea.firstChild);
        }
    }
    
    function clearLogs() {
        document.getElementById('logsArea').innerHTML = '<div class="log-entry">✨ Journal effacé</div>';
    }
    
    function updateStatus(connected) {
        const led = document.getElementById('statusLed');
        const text = document.getElementById('statusText');
        if (connected) {
            led.className = 'status-led connected';
            text.innerHTML = 'Connecté - Données en direct';
        } else {
            led.className = 'status-led';
            text.innerHTML = 'Déconnecté';
        }
    }
    
    async function fetchData(endpoint) {
        try {
            const response = await fetch('/api/' + endpoint);
            return await response.json();
        } catch(e) {
            addLog(`Erreur API: ${e.message}`, 'error');
            return null;
        }
    }
    
    async function refreshAll() {
        addLog('Actualisation des données...', 'info');
        
        // Lectures de base
        const registers = [
            'read_voltage_n1_a', 'read_voltage_n1_b', 'read_voltage_n1_c',
            'read_voltage_n2_a', 'read_voltage_n2_b', 'read_voltage_n2_c',
            'read_frequency', 'read_power_status', 'read_switch_status',
            'read_counts', 'read_runtime', 'read_max_voltages',
            'read_sw_version', 'read_modbus_config'
        ];
        
        for (const endpoint of registers) {
            const data = await fetchData(endpoint);
            if (data && data.success) {
                if (data.values) {
                    for (const [key, val] of Object.entries(data.values)) {
                        const element = document.getElementById(key);
                        if (element) {
                            if (key.includes('voltage') || key.includes('max')) {
                                element.innerHTML = `${val} <span class="data-unit">V</span>`;
                            } else if (key === 'runtime') {
                                element.innerHTML = `${val} <span class="data-unit">heures</span>`;
                            } else if (key === 'sw_version') {
                                element.innerHTML = `${(val/100).toFixed(2)} (${val})`;
                            } else {
                                element.innerHTML = val;
                            }
                        }
                    }
                } else if (data.value !== undefined) {
                    const element = document.getElementById(endpoint.replace('read_', ''));
                    if (element) element.innerHTML = data.value;
                }
                addLog(`✅ ${endpoint}`, 'success');
            } else {
                addLog(`❌ Échec ${endpoint}`, 'error');
            }
            await new Promise(r => setTimeout(r, 100));
        }
        
        updateStatus(true);
        addLog('✅ Actualisation terminée', 'success');
    }
    
    async function sendCommand(cmd) {
        addLog(`Exécution: ${cmd}`, 'info');
        const data = await fetchData(cmd);
        if (data && data.success) {
            addLog(`✅ ${data.message || 'Commande exécutée'}`, 'success');
        } else {
            addLog(`❌ Échec commande ${cmd}`, 'error');
        }
        await new Promise(r => setTimeout(r, 500));
        refreshAll();
    }
    
    // Initialisation
    window.onload = () => {
        refreshAll();
    };
</script>
</body>
</html>
"""

# API Routes
@app.route('/')
def index():
    return render_template_string(HTML)

@app.route('/api/connect')
def api_connect():
    global ser
    try:
        ser = serial.Serial(PORT, BAUDRATE, bytesize=8, parity='E', stopbits=1, timeout=1.5)
        time.sleep(0.3)
        return jsonify({'success': True})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)})

@app.route('/api/read_voltage_n1_a')
def read_voltage_n1_a():
    val = read_register(0x0006)
    return jsonify({'success': val is not None, 'value': val, 'values': {'voltage_n1_a': val}})

@app.route('/api/read_voltage_n1_b')
def read_voltage_n1_b():
    val = read_register(0x0007)
    return jsonify({'success': val is not None, 'value': val, 'values': {'voltage_n1_b': val}})

@app.route('/api/read_voltage_n1_c')
def read_voltage_n1_c():
    val = read_register(0x0008)
    return jsonify({'success': val is not None, 'value': val, 'values': {'voltage_n1_c': val}})

@app.route('/api/read_voltage_n2_a')
def read_voltage_n2_a():
    val = read_register(0x0009)
    return jsonify({'success': val is not None, 'value': val, 'values': {'voltage_n2_a': val}})

@app.route('/api/read_voltage_n2_b')
def read_voltage_n2_b():
    val = read_register(0x000A)
    return jsonify({'success': val is not None, 'value': val, 'values': {'voltage_n2_b': val}})

@app.route('/api/read_voltage_n2_c')
def read_voltage_n2_c():
    val = read_register(0x000B)
    return jsonify({'success': val is not None, 'value': val, 'values': {'voltage_n2_c': val}})

@app.route('/api/read_frequency')
def read_frequency():
    val = read_register(0x000D)
    if val is not None:
        freq_n1 = (val >> 8) & 0xFF
        freq_n2 = val & 0xFF
        return jsonify({'success': True, 'values': {'frequency_n1': freq_n1, 'frequency_n2': freq_n2}})
    return jsonify({'success': False})

@app.route('/api/read_power_status')
def read_power_status():
    val = read_register(0x004F)
    if val is not None:
        # Décodage des bits selon PDF
        def decode_voltage_status(bit_pair):
            status = (val >> bit_pair) & 0x03
            return {0: '✅ Normal', 1: '⚠️ Sous-tension', 2: '⚠️ Surtension', 3: '??'}[status]
        
        return jsonify({'success': True, 'values': {
            'power_n1_a': decode_voltage_status(8),
            'power_n1_b': decode_voltage_status(10),
            'power_n1_c': decode_voltage_status(12),
            'power_n2_a': decode_voltage_status(0),
            'power_n2_b': decode_voltage_status(2),
            'power_n2_c': decode_voltage_status(4),
        }})
    return jsonify({'success': False})

@app.route('/api/read_switch_status')
def read_switch_status():
    val = read_register(0x0050)
    if val is not None:
        return jsonify({'success': True, 'values': {
            'switch_n1': '✅ Fermé' if (val & 0x02) else '⭕ Ouvert',
            'switch_n2': '✅ Fermé' if (val & 0x04) else '⭕ Ouvert',
            'switch_mid': '⚠️ Oui' if (val & 0x08) else '⭕ Non',
            'switch_mode': '🤖 Auto' if (val & 0x01) else '👆 Manuel',
            'switch_remote': '📡 Oui' if (val & 0x0100) else '🔒 Non',
            'switch_gen': '🟢 Marche' if (val & 0x10) else '🔴 Arrêt',
        }})
    return jsonify({'success': False})

@app.route('/api/read_counts')
def read_counts():
    count1 = read_register(0x0015)
    count2 = read_register(0x0016)
    if count1 is not None and count2 is not None:
        return jsonify({'success': True, 'values': {'count_n1': count1, 'count_n2': count2}})
    return jsonify({'success': False})

@app.route('/api/read_runtime')
def read_runtime():
    val = read_register(0x0017)
    return jsonify({'success': val is not None, 'values': {'runtime': val}})

@app.route('/api/read_max_voltages')
def read_max_voltages():
    max_n1_a = read_register(0x000F)
    max_n1_b = read_register(0x0010)
    max_n1_c = read_register(0x0011)
    max_n2_a = read_register(0x0012)
    max_n2_b = read_register(0x0013)
    max_n2_c = read_register(0x0014)
    if all(v is not None for v in [max_n1_a, max_n1_b, max_n1_c, max_n2_a, max_n2_b, max_n2_c]):
        return jsonify({'success': True, 'values': {
            'max_n1': f"{max_n1_a}/{max_n1_b}/{max_n1_c} V",
            'max_n2': f"{max_n2_a}/{max_n2_b}/{max_n2_c} V"
        }})
    return jsonify({'success': False})

@app.route('/api/read_sw_version')
def read_sw_version():
    val = read_register(0x000C)
    return jsonify({'success': val is not None, 'values': {'sw_version': val}})

@app.route('/api/read_modbus_config')
def read_modbus_config():
    addr = read_register(0x0100)
    baud = read_register(0x0101)
    parity = read_register(0x000E)
    if all(v is not None for v in [addr, baud, parity]):
        baud_map = {0: '4800', 1: '9600', 2: '19200', 3: '38400'}
        parity_map = {0: 'None', 1: 'Odd', 2: 'Even'}
        return jsonify({'success': True, 'values': {
            'modbus_addr': addr,
            'modbus_baud': baud_map.get(baud, str(baud)),
            'modbus_parity': parity_map.get(parity, str(parity))
        }})
    return jsonify({'success': False})

@app.route('/api/remote_on')
def api_remote_on():
    success = write_register(0x2800, 0x0004)
    return jsonify({'success': success, 'message': 'Télécommande activée'})

@app.route('/api/remote_off')
def api_remote_off():
    success = write_register(0x2800, 0x0000)
    return jsonify({'success': success, 'message': 'Télécommande désactivée'})

@app.route('/api/force_source1')
def api_force_source1():
    success = write_register(0x2700, 0x0000)
    return jsonify({'success': success, 'message': 'Forçage Source I'})

@app.route('/api/force_source2')
def api_force_source2():
    success = write_register(0x2700, 0x00AA)
    return jsonify({'success': success, 'message': 'Forçage Source II'})

@app.route('/api/force_double')
def api_force_double():
    success = write_register(0x2700, 0x00FF)
    return jsonify({'success': success, 'message': 'Forçage double déclenché'})

if __name__ == '__main__':
    print("=" * 60)
    print("  CHINT ATS - Interface de Supervision")
    print("=" * 60)
    print(f"  Port série: {PORT} | 9600 Even | Adresse {ADDRESS}")
    print("  Ouvrez http://localhost:5000 dans votre navigateur")
    print("=" * 60)
    app.run(host='localhost', port=5000, debug=False)
