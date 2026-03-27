from flask import Flask, render_template_string, jsonify, request
from pymodbus.client import ModbusSerialClient
import threading
import time

app = Flask(__name__)

# Configuration Modbus
client = None
connected = False

HTML = """
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <title>CHINT ATS - Modbus Control</title>
    <style>
        body { font-family: monospace; background: #1a2a3a; color: #eee; padding: 20px; }
        .container { max-width: 800px; margin: 0 auto; background: #1e2a32; border-radius: 16px; padding: 20px; }
        button { background: #2c3e50; border: none; color: white; padding: 10px 20px; border-radius: 8px; cursor: pointer; margin: 5px; }
        button:hover { background: #ffaa44; color: #1e2a32; }
        .connected { background: #1f6d3a; padding: 10px; border-radius: 8px; }
        .disconnected { background: #7f2a1f; padding: 10px; border-radius: 8px; }
        .log-area { background: #0a1219; border-radius: 8px; padding: 10px; height: 250px; overflow-y: auto; font-size: 12px; }
        .result { background: #0f1a1f; padding: 15px; border-radius: 8px; margin: 10px 0; text-align: center; font-size: 18px; }
        .flex { display: flex; flex-wrap: wrap; gap: 10px; margin: 15px 0; }
        input { background: #0f1a1f; border: 1px solid #3a5a6a; color: #eee; padding: 8px; border-radius: 6px; width: 80px; }
    </style>
</head>
<body>
<div class="container">
    <h2>⚡ CHINT ATS - Modbus RTU (Python Backend)</h2>
    <div id="status" class="disconnected">🔌 Déconnecté</div>
    
    <div class="flex">
        <button onclick="connect()">🔌 Connecter (COM5)</button>
        <button onclick="disconnect()">⛔ Déconnecter</button>
        <button onclick="readPower()">📡 Lire état sources (0x004F)</button>
        <button onclick="readSwitch()">🔀 Lire commutateur (0x0050)</button>
        <button onclick="readVoltage()">📊 Lire tension A Source I</button>
        <button onclick="remoteOn()">📡 Activer télécommande</button>
        <button onclick="remoteOff()">🔒 Désactiver télécommande</button>
        <button onclick="forceDouble()">⏹️ Forcer double</button>
        <button onclick="forceSource1()">🔵 Forcer Source I</button>
        <button onclick="forceSource2()">🟠 Forcer Source II</button>
    </div>
    
    <div id="result" class="result">-- En attente --</div>
    
    <div class="log-area" id="log">
        <div>✨ Prêt - Cliquez sur Connecter</div>
    </div>
    <button onclick="clearLog()">🗑️ Effacer journal</button>
</div>

<script>
    async function apiCall(endpoint, data = {}) {
        try {
            const resp = await fetch('/api/' + endpoint, {
                method: 'POST',
                headers: {'Content-Type': 'application/json'},
                body: JSON.stringify(data)
            });
            return await resp.json();
        } catch(e) {
            addLog('Erreur API: ' + e.message, 'error');
        }
    }
    
    function addLog(msg, type = 'info') {
        const logDiv = document.getElementById('log');
        const entry = document.createElement('div');
        const time = new Date().toLocaleTimeString();
        const prefix = type === 'error' ? '❌' : (type === 'success' ? '✅' : 'ℹ️');
        entry.innerHTML = `[${time}] ${prefix} ${msg}`;
        logDiv.appendChild(entry);
        entry.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
    
    async function connect() {
        const result = await apiCall('connect', { port: 'COM5' });
        if (result.success) {
            document.getElementById('status').className = 'connected';
            document.getElementById('status').innerHTML = '✅ Connecté sur COM5 | 9600 Even | Adresse 6';
            addLog('Connecté avec succès!', 'success');
        } else {
            addLog('Échec connexion: ' + result.error, 'error');
        }
    }
    
    async function disconnect() {
        const result = await apiCall('disconnect');
        document.getElementById('status').className = 'disconnected';
        document.getElementById('status').innerHTML = '🔌 Déconnecté';
        addLog('Déconnecté');
    }
    
    async function readPower() {
        addLog('Lecture état sources (0x004F)...');
        const result = await apiCall('read', { register: 0x004F });
        if (result.success) {
            document.getElementById('result').innerHTML = `📊 État sources = 0x${result.value.toString(16).padStart(4,'0')} (${result.value})`;
            addLog(`Valeur: 0x${result.value.toString(16)}`, 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function readSwitch() {
        addLog('Lecture état commutateur (0x0050)...');
        const result = await apiCall('read', { register: 0x0050 });
        if (result.success) {
            document.getElementById('result').innerHTML = `🔀 État commutateur = 0x${result.value.toString(16).padStart(4,'0')} (${result.value})`;
            addLog(`Valeur: 0x${result.value.toString(16)}`, 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function readVoltage() {
        addLog('Lecture tension phase A Source I (0x0006)...');
        const result = await apiCall('read', { register: 0x0006 });
        if (result.success) {
            document.getElementById('result').innerHTML = `📊 Tension Phase A Source I = ${result.value} V`;
            addLog(`Tension: ${result.value}V`, 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function remoteOn() {
        addLog('Activation télécommande...');
        const result = await apiCall('write', { register: 0x2800, value: 0x0004 });
        if (result.success) {
            document.getElementById('result').innerHTML = '✅ Télécommande activée';
            addLog('Télécommande activée', 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function remoteOff() {
        addLog('Désactivation télécommande...');
        const result = await apiCall('write', { register: 0x2800, value: 0x0000 });
        if (result.success) {
            document.getElementById('result').innerHTML = '🔒 Télécommande désactivée';
            addLog('Télécommande désactivée', 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function forceDouble() {
        addLog('Forçage position double...');
        const result = await apiCall('write', { register: 0x2700, value: 0x00FF });
        if (result.success) {
            document.getElementById('result').innerHTML = '⏹️ Forçage double envoyé';
            addLog('Forçage double effectué', 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function forceSource1() {
        addLog('Forçage Source I...');
        const result = await apiCall('write', { register: 0x2700, value: 0x0000 });
        if (result.success) {
            document.getElementById('result').innerHTML = '🔵 Forçage Source I envoyé';
            addLog('Forçage Source I', 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    async function forceSource2() {
        addLog('Forçage Source II...');
        const result = await apiCall('write', { register: 0x2700, value: 0x00AA });
        if (result.success) {
            document.getElementById('result').innerHTML = '🟠 Forçage Source II envoyé';
            addLog('Forçage Source II', 'success');
        } else {
            addLog('Erreur: ' + result.error, 'error');
        }
    }
    
    function clearLog() {
        document.getElementById('log').innerHTML = '<div>Journal effacé</div>';
    }
</script>
</body>
</html>
"""

# API Routes
@app.route('/')
def index():
    return render_template_string(HTML)

@app.route('/api/connect', methods=['POST'])
def api_connect():
    global client, connected
    try:
        data = request.json
        port = data.get('port', 'COM5')
        
        client = ModbusSerialClient(
            method='rtu',
            port=port,
            baudrate=9600,
            bytesize=8,
            parity='E',
            stopbits=1,
            timeout=1
        )
        
        if client.connect():
            connected = True
            return jsonify({'success': True})
        else:
            return jsonify({'success': False, 'error': 'Connexion échouée'})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)})

@app.route('/api/disconnect', methods=['POST'])
def api_disconnect():
    global client, connected
    if client:
        client.close()
    connected = False
    return jsonify({'success': True})

@app.route('/api/read', methods=['POST'])
def api_read():
    global client, connected
    if not client or not connected:
        return jsonify({'success': False, 'error': 'Non connecté'})
    
    try:
        data = request.json
        register = data.get('register')
        
        result = client.read_holding_registers(register, 1, slave=6)
        
        if result.isError():
            return jsonify({'success': False, 'error': str(result)})
        
        return jsonify({'success': True, 'value': result.registers[0]})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)})

@app.route('/api/write', methods=['POST'])
def api_write():
    global client, connected
    if not client or not connected:
        return jsonify({'success': False, 'error': 'Non connecté'})
    
    try:
        data = request.json
        register = data.get('register')
        value = data.get('value')
        
        result = client.write_register(register, value, slave=6)
        
        if result.isError():
            return jsonify({'success': False, 'error': str(result)})
        
        return jsonify({'success': True})
    except Exception as e:
        return jsonify({'success': False, 'error': str(e)})

if __name__ == '__main__':
    print("=== Serveur Modbus CHINT ATS ===")
    print("1. Assurez-vous que l'adaptateur RS485 est sur COM5")
    print("2. Ouvrez http://localhost:5000 dans Chrome")
    print("================================")
    app.run(host='localhost', port=5000, debug=False)
