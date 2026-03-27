from flask import Flask, render_template_string, jsonify
import serial
import time
import threading
import logging

# Désactiver les logs Flask
logging.getLogger('werkzeug').setLevel(logging.ERROR)

app = Flask(__name__)

# Configuration
PORT = 'COM5'
BAUDRATE = 9600
ADDRESS = 6
ser = None
lock = threading.Lock()
connection_status = "Déconnecté"
last_error = ""

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
    global connection_status, last_error
    if ser is None or not ser.is_open:
        return None
    try:
        frame = build_frame(0x03, reg)
        resp = send_frame(frame)
        if resp and len(resp) >= 5 and resp[1] == 0x03:
            return (resp[3] << 8) | resp[4]
        elif resp and len(resp) >= 3 and (resp[1] & 0x80):
            last_error = f"Erreur Modbus: {resp[2]}"
        return None
    except Exception as e:
        last_error = str(e)
        return None

def write_register(reg, value):
    if ser is None or not ser.is_open:
        return False
    try:
        frame = build_frame(0x06, reg, value)
        resp = send_frame(frame)
        return resp is not None and len(resp) > 0
    except:
        return False

# Page HTML intégrée
HTML = """
<!DOCTYPE html>
<html lang="fr">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>CHINT ATS - Supervision</title>
    <style>
        * { margin: 0; padding: 0; box-sizing: border-box; }
        body { font-family: 'Segoe UI', system-ui; background: linear-gradient(135deg, #0f172a, #1e293b); color: #f1f5f9; padding: 24px; min-height: 100vh; }
        .container { max-width: 1400px; margin: 0 auto; }
        .header { background: rgba(30, 41, 59, 0.8); backdrop-filter: blur(10px); border-radius: 24px; padding: 24px 32px; margin-bottom: 24px; border: 1px solid rgba(255,255,255,0.1); }
        .header h1 { font-size: 28px; background: linear-gradient(135deg, #fbbf24, #f59e0b); -webkit-background-clip: text; -webkit-text-fill-color: transparent; margin-bottom: 8px; }
        .status-bar { display: flex; justify-content: space-between; align-items: center; margin-top: 16px; padding-top: 16px; border-top: 1px solid rgba(255,255,255,0.1); flex-wrap: wrap; gap: 12px; }
        .status { display: flex; align-items: center; gap: 12px; }
        .led { width: 12px; height: 12px; border-radius: 50%; background: #ef4444; transition: all 0.3s; }
        .led.connected { background: #22c55e; box-shadow: 0 0 8px #22c55e; }
        .btn { background: linear-gradient(135deg, #3b82f6, #2563eb); border: none; color: white; padding: 10px 24px; border-radius: 40px; font-weight: 600; cursor: pointer; transition: 0.2s; font-size: 14px; }
        .btn:hover { transform: scale(1.02); background: linear-gradient(135deg, #2563eb, #1d4ed8); }
        .grid { display: grid; grid-template-columns: repeat(auto-fit, minmax(350px, 1fr)); gap: 24px; margin-bottom: 24px; }
        .card { background: rgba(30, 41, 59, 0.8); backdrop-filter: blur(10px); border-radius: 20px; padding: 20px; border: 1px solid rgba(255,255,255,0.1); }
        .card-title { font-size: 16px; font-weight: 600; color: #94a3b8; margin-bottom: 16px; padding-bottom: 12px; border-bottom: 1px solid rgba(255,255,255,0.1); display: flex; align-items: center; gap: 8px; }
        .data-row { display: flex; justify-content: space-between; padding: 10px 0; border-bottom: 1px solid rgba(255,255,255,0.05); }
        .data-label { color: #94a3b8; font-size: 13px; }
        .data-value { font-weight: 600; font-family: monospace; font-size: 16px; }
        .badge-normal { background: rgba(34,197,94,0.2); color: #4ade80; padding: 4px 12px; border-radius: 20px; font-size: 12px; display: inline-block; }
        .badge-warning { background: rgba(245,158,11,0.2); color: #fbbf24; padding: 4px 12px; border-radius: 20px; font-size: 12px; display: inline-block; }
        .badge-danger { background: rgba(239,68,68,0.2); color: #f87171; padding: 4px 12px; border-radius: 20px; font-size: 12px; display: inline-block; }
        .action-buttons { display: flex; flex-wrap: wrap; gap: 12px; margin-top: 16px; }
        .action-btn { background: rgba(59,130,246,0.2); border: 1px solid rgba(59,130,246,0.5); color: #60a5fa; padding: 8px 16px; border-radius: 40px; font-size: 12px; cursor: pointer; transition: 0.2s; font-weight: 500; }
        .action-btn:hover { background: #3b82f6; color: white; }
        .logs-card { background: rgba(15, 23, 42, 0.9); border-radius: 20px; padding: 20px; }
        .logs-area { background: #0f172a; border-radius: 12px; padding: 12px; height: 200px; overflow-y: auto; font-family: monospace; font-size: 11px; }
        .log-entry { padding: 4px 0; border-bottom: 1px solid rgba(255,255,255,0.05); color: #94a3b8; white-space: pre-wrap; }
        .log-entry.success { color: #4ade80; }
        .log-entry.error { color: #f87171; }
        .log-entry.send { color: #60a5fa; }
        .log-entry.recv { color: #fbbf24; }
        .clear-log { margin-top: 12px; background: none; border: 1px solid rgba(255,255,255,0.2); color: #94a3b8; padding: 6px 12px; border-radius: 8px; cursor: pointer; font-size: 11px; }
        footer { text-align: center; margin-top: 24px; font-size: 12px; color: #475569; }
        .flex-between { display: flex; justify-content: space-between; align-items: center; }
        .auto-refresh { display: flex; align-items: center; gap: 8px; font-size: 12px; }
    </style>
</head>
<body>
<div class="container">
    <div class="header">
        <h1>⚡ CHINT ATS - Supervision</h1>
        <div class="status-bar">
            <div class="status">
                <div class="led" id="led"></div>
                <div id="statusText">Initialisation...</div>
                <div style="color:#64748b;">| Adresse 6 | 9600 Even | COM5</div>
            </div>
            <div class="auto-refresh">
                <input type="checkbox" id="autoRefresh" checked> <label for="autoRefresh">Auto-actualisation (5s)</label>
                <button class="btn" id="refreshBtn" onclick="refreshAll()" style="padding: 8px 20px;">🔄 Actualiser</button>
            </div>
        </div>
    </div>

    <div class="grid">
        <div class="card">
            <div class="card-title"><span>🔵</span> Source I - Tensions</div>
            <div class="data-row"><span class="data-label">Phase A (L1-N)</span><span class="data-value" id="v1a">--- V</span></div>
            <div class="data-row"><span class="data-label">Phase B (L2-N)</span><span class="data-value" id="v1b">--- V</span></div>
            <div class="data-row"><span class="data-label">Phase C (L3-N)</span><span class="data-value" id="v1c">--- V</span></div>
            <div class="data-row"><span class="data-label">Fréquence</span><span class="data-value" id="f1">--- Hz</span></div>
        </div>

        <div class="card">
            <div class="card-title"><span>🟠</span> Source II - Tensions</div>
            <div class="data-row"><span class="data-label">Phase A (L1-N)</span><span class="data-value" id="v2a">--- V</span></div>
            <div class="data-row"><span class="data-label">Phase B (L2-N)</span><span class="data-value" id="v2b">--- V</span></div>
            <div class="data-row"><span class="data-label">Phase C (L3-N)</span><span class="data-value" id="v2c">--- V</span></div>
            <div class="data-row"><span class="data-label">Fréquence</span><span class="data-value" id="f2">--- Hz</span></div>
        </div>

        <div class="card">
            <div class="card-title"><span>📊</span> État des sources</div>
            <div class="data-row"><span class="data-label">Source I - Phase A</span><span class="data-value" id="s1a">---</span></div>
            <div class="data-row"><span class="data-label">Source I - Phase B</span><span class="data-value" id="s1b">---</span></div>
            <div class="data-row"><span class="data-label">Source I - Phase C</span><span class="data-value" id="s1c">---</span></div>
            <div class="data-row"><span class="data-label">Source II - Phase A</span><span class="data-value" id="s2a">---</span></div>
            <div class="data-row"><span class="data-label">Source II - Phase B</span><span class="data-value" id="s2b">---</span></div>
            <div class="data-row"><span class="data-label">Source II - Phase C</span><span class="data-value" id="s2c">---</span></div>
        </div>

        <div class="card">
            <div class="card-title"><span>🔀</span> État du commutateur</div>
            <div class="data-row"><span class="data-label">Source I (position)</span><span class="data-value" id="sw1">---</span></div>
            <div class="data-row"><span class="data-label">Source II (position)</span><span class="data-value" id="sw2">---</span></div>
            <div class="data-row"><span class="data-label">Position double (off)</span><span class="data-value" id="swMid">---</span></div>
            <div class="data-row"><span class="data-label">Mode de fonctionnement</span><span class="data-value" id="swMode">---</span></div>
            <div class="data-row"><span class="data-label">Télécommande</span><span class="data-value" id="swRemote">---</span></div>
            <div class="data-row"><span class="data-label">Générateur</span><span class="data-value" id="swGen">---</span></div>
        </div>

        <div class="card">
            <div class="card-title"><span>📈</span> Statistiques & Produit</div>
            <div class="data-row"><span class="data-label">Commutations Source I</span><span class="data-value" id="cnt1">---</span></div>
            <div class="data-row"><span class="data-label">Commutations Source II</span><span class="data-value" id="cnt2">---</span></div>
            <div class="data-row"><span class="data-label">Temps de fonctionnement</span><span class="data-value" id="runtime">--- h</span></div>
            <div class="data-row"><span class="data-label">Version logicielle</span><span class="data-value" id="swVer">---</span></div>
            <div class="data-row"><span class="data-label">Tensions max Source I</span><span class="data-value" id="max1">---</span></div>
            <div class="data-row"><span class="data-label">Tensions max Source II</span><span class="data-value" id="max2">---</span></div>
        </div>

        <div class="card">
            <div class="card-title"><span>🎮</span> Commandes à distance</div>
            <div class="action-buttons">
                <button class="action-btn" onclick="sendCmd('remote_on')">📡 Activer télécommande</button>
                <button class="action-btn" onclick="sendCmd('remote_off')">🔒 Désactiver télécommande</button>
                <button class="action-btn" onclick="sendCmd('force_source1')">🔵 Forcer Source I</button>
                <button class="action-btn" onclick="sendCmd('force_source2')">🟠 Forcer Source II</button>
                <button class="action-btn" onclick="sendCmd('force_double')">⏹️ Forcer double déclenché</button>
            </div>
            <div class="data-row" style="margin-top: 16px;">
                <span class="data-label">⚠️ Note</span>
                <span class="data-value" style="font-size: 11px; color: #fbbf24;">Activer la télécommande avant d'utiliser les commandes de forçage</span>
            </div>
        </div>
    </div>

    <div class="logs-card">
        <div class="flex-between">
            <div class="card-title" style="border: none; margin-bottom: 0;">📋 Journal des communications</div>
            <button class="clear-log" onclick="clearLog()">🗑️ Effacer</button>
        </div>
        <div class="logs-area" id="logs">
            <div class="log-entry">✨ Initialisation en cours...</div>
        </div>
    </div>
    <footer>CHINT ATS · Modbus RTU · Mise à jour automatique toutes les 5 secondes</footer>
</div>

<script>
    let autoRefreshInterval = null;
    
    function addLog(msg, type = 'info') {
        const logs = document.getElementById('logs');
        const div = document.createElement('div');
        div.className = `log-entry ${type}`;
        const time = new Date().toLocaleTimeString();
        let icon = 'ℹ️';
        if (type === 'error') icon = '❌';
        if (type === 'success') icon = '✅';
        if (type === 'send') icon = '📤';
        if (type === 'recv') icon = '📥';
        div.innerHTML = `[${time}] ${icon} ${msg}`;
        logs.appendChild(div);
        div.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        while (logs.children.length > 150) logs.removeChild(logs.firstChild);
    }

    function clearLog() { 
        document.getElementById('logs').innerHTML = '<div class="log-entry">📋 Journal effacé</div>';
        addLog('Journal effacé');
    }

    async function apiCall(endpoint) {
        try {
            const resp = await fetch('/api/' + endpoint);
            const data = await resp.json();
            if (data.log) addLog(data.log, data.logType || 'info');
            return data;
        } catch(e) { 
            addLog('Erreur réseau: ' + e.message, 'error'); 
            return null; 
        }
    }

    async function refreshAll() {
        addLog('Lecture des données...', 'info');
        
        const endpoints = ['all'];
        
        for (const ep of endpoints) {
            const res = await apiCall(ep);
            if (res && res.success && res.values) {
                for (const [k, v] of Object.entries(res.values)) {
                    const el = document.getElementById(k);
                    if (el) el.innerHTML = v;
                }
                if (res.connected) {
                    document.getElementById('led').className = 'led connected';
                    document.getElementById('statusText').innerHTML = 'Connecté';
                } else {
                    document.getElementById('led').className = 'led';
                    document.getElementById('statusText').innerHTML = 'Déconnecté';
                }
                addLog('Données mises à jour', 'success');
            } else if (res && !res.success) {
                addLog('Erreur: ' + (res.error || 'inconnue'), 'error');
                document.getElementById('led').className = 'led';
                document.getElementById('statusText').innerHTML = 'Erreur';
            }
        }
    }

    async function sendCmd(cmd) {
        addLog(`Envoi commande: ${cmd}`, 'send');
        const res = await apiCall(cmd);
        if (res && res.success) {
            addLog(`✅ ${res.message || 'Commande exécutée'}`, 'success');
        } else {
            addLog(`❌ Échec commande: ${res?.error || 'inconnu'}`, 'error');
        }
        setTimeout(() => refreshAll(), 500);
    }

    // Auto-refresh
    function startAutoRefresh() {
        if (autoRefreshInterval) clearInterval(autoRefreshInterval);
        autoRefreshInterval = setInterval(() => {
            if (document.getElementById('autoRefresh').checked) {
                refreshAll();
            }
        }, 5000);
    }
    
    window.onload = () => {
        startAutoRefresh();
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

@app.route('/api/all')
def api_all():
    global ser, connection_status
    
    # Tentative de connexion automatique si non connecté
    if ser is None or not ser.is_open:
        try:
            ser = serial.Serial(PORT, BAUDRATE, bytesize=8, parity='E', stopbits=1, timeout=1.5)
            time.sleep(0.3)
            connection_status = "Connecté"
        except Exception as e:
            connection_status = f"Erreur: {e}"
            return jsonify({'success': False, 'error': str(e), 'connected': False})
    
    # Lecture de tous les registres
    values = {}
    
    # Tensions
    v1a = read_register(0x0006)
    v1b = read_register(0x0007)
    v1c = read_register(0x0008)
    v2a = read_register(0x0009)
    v2b = read_register(0x000A)
    v2c = read_register(0x000B)
    
    if v1a is not None: values['v1a'] = f'{v1a} V'
    if v1b is not None: values['v1b'] = f'{v1b} V'
    if v1c is not None: values['v1c'] = f'{v1c} V'
    if v2a is not None: values['v2a'] = f'{v2a} V'
    if v2b is not None: values['v2b'] = f'{v2b} V'
    if v2c is not None: values['v2c'] = f'{v2c} V'
    
    # Fréquence
    freq = read_register(0x000D)
    if freq is not None:
        values['f1'] = f'{(freq >> 8) & 0xFF} Hz'
        values['f2'] = f'{freq & 0xFF} Hz'
    
    # État des sources
    power = read_register(0x004F)
    if power is not None:
        def decode_status(bit):
            s = (power >> bit) & 0x03
            if s == 0: return '<span class="badge-normal">✅ Normal</span>'
            if s == 1: return '<span class="badge-warning">⚠️ Sous-tension</span>'
            if s == 2: return '<span class="badge-danger">⚠️ Surtension</span>'
            return '<span class="badge-danger">❌ Erreur</span>'
        values['s1a'] = decode_status(8)
        values['s1b'] = decode_status(10)
        values['s1c'] = decode_status(12)
        values['s2a'] = decode_status(0)
        values['s2b'] = decode_status(2)
        values['s2c'] = decode_status(4)
    
    # État commutateur
    switch = read_register(0x0050)
    if switch is not None:
        values['sw1'] = '✅ Fermé' if (switch & 0x02) else '⭕ Ouvert'
        values['sw2'] = '✅ Fermé' if (switch & 0x04) else '⭕ Ouvert'
        values['swMid'] = '⚠️ Oui' if (switch & 0x08) else '⭕ Non'
        values['swMode'] = '🤖 Automatique' if (switch & 0x01) else '👆 Manuel'
        values['swRemote'] = '📡 Activé' if (switch & 0x0100) else '🔒 Désactivé'
        values['swGen'] = '🟢 Marche' if (switch & 0x10) else '🔴 Arrêt'
    
    # Compteurs
    cnt1 = read_register(0x0015)
    cnt2 = read_register(0x0016)
    if cnt1 is not None: values['cnt1'] = str(cnt1)
    if cnt2 is not None: values['cnt2'] = str(cnt2)
    
    # Temps fonctionnement
    runtime = read_register(0x0017)
    if runtime is not None: values['runtime'] = f'{runtime} h'
    
    # Version logicielle
    sw = read_register(0x000C)
    if sw is not None: values['swVer'] = f'{sw/100:.2f}'
    
    # Tensions max
    max_n1_a = read_register(0x000F)
    max_n1_b = read_register(0x0010)
    max_n1_c = read_register(0x0011)
    max_n2_a = read_register(0x0012)
    max_n2_b = read_register(0x0013)
    max_n2_c = read_register(0x0014)
    if all(v is not None for v in [max_n1_a, max_n1_b, max_n1_c]):
        values['max1'] = f'{max_n1_a}/{max_n1_b}/{max_n1_c} V'
    if all(v is not None for v in [max_n2_a, max_n2_b, max_n2_c]):
        values['max2'] = f'{max_n2_a}/{max_n2_b}/{max_n2_c} V'
    
    return jsonify({'success': True, 'values': values, 'connected': True})

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
    print("  🔌 CHINT ATS - Interface de Supervision")
    print("=" * 60)
    print(f"  📡 Port série: {PORT} | 9600 bauds | Even | Adresse {ADDRESS}")
    print(f"  🌐 Ouvrez: http://localhost:5000")
    print("=" * 60)
    print("  ⚡ Connexion automatique au démarrage")
    print("=" * 60)
    app.run(host='localhost', port=5000, debug=False)
