use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::time::Duration;
use serialport::{self, SerialPort};
use std::io::{Write, Read};
use std::thread;
use std::fs::OpenOptions;
use std::io::Write as IoWrite;
use chrono::Local;

// ==================== STRUCTURES ====================

struct AppState {
    port_name: Mutex<String>,
    debug_log: Mutex<bool>,  // Activer/désactiver les logs détaillés
}

#[derive(Serialize)]
struct ModbusResponse {
    success: bool,
    values: std::collections::HashMap<String, String>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct RegValue {
    value: u16,
}

// ==================== FONCTIONS DE LOG ====================

fn write_debug_log(message: &str) {
    // Vérifier si le debug est activé - sera lu depuis AppState
    // Pour l'instant on écrit toujours, on filtrera côté appel
    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S%.3f");
    let log_line = format!("[{}] {}\n", timestamp, message);
    
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open("modbus_debug.log") {
        let _ = file.write_all(log_line.as_bytes());
    }
}

// ==================== FONCTIONS MODBUS ====================

fn calculate_crc(data: &[u8]) -> u16 {
    let mut crc = 0xFFFF;
    for &byte in data {
        crc ^= byte as u16;
        for _ in 0..8 {
            if crc & 0x0001 != 0 {
                crc = (crc >> 1) ^ 0xA001;
            } else {
                crc >>= 1;
            }
        }
    }
    crc
}

fn build_frame(addr: u8, func: u8, reg: u16, value: Option<u16>) -> Vec<u8> {
    let mut data = vec![addr, func, (reg >> 8) as u8, reg as u8];
    
    if func == 0x03 {
        data.extend_from_slice(&[0x00, 0x01]);
    } else if func == 0x06 {
        if let Some(val) = value {
            data.extend_from_slice(&[(val >> 8) as u8, val as u8]);
        }
    }
    
    let crc = calculate_crc(&data);
    data.push((crc & 0xFF) as u8);
    data.push((crc >> 8) as u8);
    data
}

fn read_register(port_name: &str, addr: u8, reg: u16, debug: bool) -> Option<u16> {
    let frame = build_frame(addr, 0x03, reg, None);
    
    if debug {
        write_debug_log(&format!("📤 READ REG 0x{:04X} | Trame: {}", reg, frame.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")));
    }
    
    let mut port = match serialport::new(port_name, 9600)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            if debug { write_debug_log(&format!("❌ Erreur ouverture port: {}", e)); }
            return None;
        }
    };
    
    if port.write_all(&frame).is_err() {
        if debug { write_debug_log("❌ Erreur écriture port"); }
        return None;
    }
    
    thread::sleep(Duration::from_millis(100));
    
    let mut buffer = vec![0u8; 256];
    match port.read(&mut buffer) {
        Ok(n) if n >= 5 => {
            let resp = &buffer[..n];
            if debug {
                write_debug_log(&format!("📥 READ REG 0x{:04X} | Réponse: {}", reg, resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")));
            }
            if resp.len() >= 5 && resp[1] == 0x03 {
                let value = ((resp[3] as u16) << 8) | resp[4] as u16;
                if debug {
                    write_debug_log(&format!("📊 Valeur: {} (0x{:04X})", value, value));
                }
                Some(value)
            } else {
                if debug { write_debug_log(&format!("⚠️ Réponse invalide (fonction: 0x{:02X})", resp[1])); }
                None
            }
        }
        Ok(n) => {
            if debug { write_debug_log(&format!("⚠️ Réponse trop courte: {} octets", n)); }
            None
        }
        Err(e) => {
            if debug { write_debug_log(&format!("❌ Erreur lecture: {}", e)); }
            None
        }
        _ => None,
    }
}

fn write_register(port_name: &str, addr: u8, reg: u16, value: u16, debug: bool) -> bool {
    let frame = build_frame(addr, 0x06, reg, Some(value));
    
    if debug {
        write_debug_log(&format!("📝 WRITE REG 0x{:04X} = {} | Trame: {}", reg, value, frame.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")));
    }
    
    let mut port = match serialport::new(port_name, 9600)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(p) => p,
        Err(e) => {
            if debug { write_debug_log(&format!("❌ Erreur ouverture port: {}", e)); }
            return false;
        }
    };
    
    if port.write_all(&frame).is_err() {
        if debug { write_debug_log("❌ Erreur écriture port"); }
        return false;
    }
    
    thread::sleep(Duration::from_millis(100));
    
    let mut buffer = vec![0u8; 256];
    match port.read(&mut buffer) {
        Ok(n) if n > 0 => {
            let resp = &buffer[..n];
            if debug {
                write_debug_log(&format!("📥 WRITE REG 0x{:04X} | Réponse: {}", reg, resp.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" ")));
            }
            true
        }
        Ok(_) => {
            if debug { write_debug_log("⚠️ Réponse vide"); }
            false
        }
        Err(e) => {
            if debug { write_debug_log(&format!("❌ Erreur lecture réponse: {}", e)); }
            false
        }
    }
}

// ==================== API ROUTES ====================

async fn read_all(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    
    let mut values = std::collections::HashMap::new();
    let addr = 6u8;
    
    let fmt_v = |x: u16| format!("{} V", x);
    let fmt_ver = |x: u16| format!("{:.2}", x as f32 / 100.0);
    let fmt_cnt = |x: u16| x.to_string();
    let fmt_h = |x: u16| format!("{} h", x);
    let fmt_s = |x: u16| format!("{} s", x);
    
    let regs: Vec<(u16, &str, Box<dyn Fn(u16) -> String>)> = vec![
        (0x0006, "v1a", Box::new(fmt_v)),
        (0x0007, "v1b", Box::new(fmt_v)),
        (0x0008, "v1c", Box::new(fmt_v)),
        (0x0009, "v2a", Box::new(fmt_v)),
        (0x000A, "v2b", Box::new(fmt_v)),
        (0x000B, "v2c", Box::new(fmt_v)),
        (0x000C, "swVer", Box::new(fmt_ver)),
        (0x0015, "cnt1", Box::new(fmt_cnt)),
        (0x0016, "cnt2", Box::new(fmt_cnt)),
        (0x0017, "runtime", Box::new(fmt_h)),
        (0x2065, "uv1", Box::new(fmt_v)),
        (0x2066, "uv2", Box::new(fmt_v)),
        (0x2067, "ov1", Box::new(fmt_v)),
        (0x2068, "ov2", Box::new(fmt_v)),
        (0x2069, "t1", Box::new(fmt_s)),
        (0x206A, "t2", Box::new(fmt_s)),
        (0x206B, "t3", Box::new(fmt_s)),
        (0x206C, "t4", Box::new(fmt_s)),
    ];
    
    for (reg, key, formatter) in regs {
        if let Some(val) = read_register(&port_name, addr, reg, debug) {
            values.insert(key.to_string(), formatter(val));
        } else {
            values.insert(key.to_string(), "---".to_string());
        }
    }
    
    // État commutateur (pour l'affichage dans le bandeau)
    if let Some(switch) = read_register(&port_name, addr, 0x0050, debug) {
        values.insert("sw1".to_string(), if switch & 0x02 != 0 { "✅ Fermé" } else { "⭕ Ouvert" }.to_string());
        values.insert("sw2".to_string(), if switch & 0x04 != 0 { "✅ Fermé" } else { "⭕ Ouvert" }.to_string());
        values.insert("swMode".to_string(), if switch & 0x01 != 0 { "🤖 Auto" } else { "👆 Manuel" }.to_string());
        values.insert("swRemote".to_string(), if switch & 0x0100 != 0 { "📡 Activé" } else { "🔒 Désactivé" }.to_string());
        let fault = (switch >> 4) & 0x07;
        values.insert("swFault".to_string(), match fault {
            0 => "Aucun", 1 => "消防联动", 2 => "电机超时", 3 => "电源I跳闸",
            4 => "电源II跳闸", 5 => "合闸信号异常", 6 => "相序异常 I", 7 => "相序异常 II",
            _ => "Inconnu",
        }.to_string());
    }
    
    // Mode fonctionnement
    if let Some(mode) = read_register(&port_name, addr, 0x206D, debug) {
        values.insert("operation_mode".to_string(), match mode {
            0 => "自投自复", 1 => "自投不自复", 2 => "互为备用",
            3 => "发电机模式", 4 => "发电机不自复", 5 => "发电机备用",
            _ => "Inconnu",
        }.to_string());
    }
    
    let success = values.values().any(|v| v != "---");
    HttpResponse::Ok().json(ModbusResponse {
        success,
        values,
        error: if success { None } else { Some("Aucune réponse".to_string()) },
    })
}

// ==================== ROUTES DE COMMANDES ====================

async fn remote_on(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2800, 0x0004, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Télécommande activée"
    }))
}

async fn remote_off(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2800, 0x0000, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Télécommande désactivée"
    }))
}

async fn force_double(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2700, 0x00FF, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Forçage double déclenché"
    }))
}

async fn force_source1(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2700, 0x0000, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Forçage Source I"
    }))
}

async fn force_source2(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2700, 0x00AA, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Forçage Source II"
    }))
}

// ==================== ROUTES DE RÉGLAGE ====================

async fn set_undervoltage1(data: web::Data<Mutex<AppState>>, query: web::Query<RegValue>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let value = query.value;
    
    if debug { write_debug_log(&format!("🔧 TENTATIVE: Régler sous-tension Source I = {} V", value)); }
    
    // Vérifier si télécommande activée
    let remote_status = read_register(&port_name, 6, 0x0050, debug).map(|s| (s & 0x0100) != 0).unwrap_or(false);
    if debug { write_debug_log(&format!("📡 Statut télécommande: {}", if remote_status { "ACTIVÉE" } else { "DÉSACTIVÉE" })); }
    
    if !remote_status {
        if debug { write_debug_log("❌ ÉCHEC: Télécommande non activée"); }
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "Activez d'abord la télécommande (bouton 'Activer télécommande')"
        }));
    }
    
    if value < 150 || value > 200 {
        if debug { write_debug_log(&format!("❌ ÉCHEC: Valeur {} hors plage 150-200V", value)); }
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "La valeur doit être entre 150 et 200 V"
        }));
    }
    
    let success = write_register(&port_name, 6, 0x2065, value, debug);
    if success {
        if debug { write_debug_log(&format!("✅ SUCCÈS: Sous-tension Source I = {} V", value)); }
    } else {
        if debug { write_debug_log("❌ ÉCHEC: Écriture registre échouée"); }
    }
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": format!("Sous-tension Source I réglée à {} V", value)
    }))
}

async fn set_undervoltage2(data: web::Data<Mutex<AppState>>, query: web::Query<RegValue>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let value = query.value;
    
    if debug { write_debug_log(&format!("🔧 TENTATIVE: Régler sous-tension Source II = {} V", value)); }
    
    let remote_status = read_register(&port_name, 6, 0x0050, debug).map(|s| (s & 0x0100) != 0).unwrap_or(false);
    if !remote_status {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "Activez d'abord la télécommande"
        }));
    }
    
    if value < 150 || value > 200 {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "La valeur doit être entre 150 et 200 V"
        }));
    }
    
    let success = write_register(&port_name, 6, 0x2066, value, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": format!("Sous-tension Source II réglée à {} V", value)
    }))
}

async fn set_overvoltage1(data: web::Data<Mutex<AppState>>, query: web::Query<RegValue>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let value = query.value;
    
    if debug { write_debug_log(&format!("🔧 TENTATIVE: Régler surtension Source I = {} V", value)); }
    
    let remote_status = read_register(&port_name, 6, 0x0050, debug).map(|s| (s & 0x0100) != 0).unwrap_or(false);
    if !remote_status {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "Activez d'abord la télécommande"
        }));
    }
    
    if value < 240 || value > 290 {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "La valeur doit être entre 240 et 290 V"
        }));
    }
    
    let success = write_register(&port_name, 6, 0x2067, value, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": format!("Surtension Source I réglée à {} V", value)
    }))
}

async fn set_overvoltage2(data: web::Data<Mutex<AppState>>, query: web::Query<RegValue>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let debug = *state.debug_log.lock().unwrap();
    let value = query.value;
    
    if debug { write_debug_log(&format!("🔧 TENTATIVE: Régler surtension Source II = {} V", value)); }
    
    let remote_status = read_register(&port_name, 6, 0x0050, debug).map(|s| (s & 0x0100) != 0).unwrap_or(false);
    if !remote_status {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "Activez d'abord la télécommande"
        }));
    }
    
    if value < 240 || value > 290 {
        return HttpResponse::Ok().json(serde_json::json!({
            "success": false,
            "error": "La valeur doit être entre 240 et 290 V"
        }));
    }
    
    let success = write_register(&port_name, 6, 0x2068, value, debug);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": format!("Surtension Source II réglée à {} V", value)
    }))
}

// ==================== ROUTES DE DEBUG ====================

async fn debug_on(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let mut debug = state.debug_log.lock().unwrap();
    *debug = true;
    write_debug_log("=== DEBUG ACTIVÉ ===");
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Mode debug activé - logs dans modbus_debug.log"
    }))
}

async fn debug_off(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let mut debug = state.debug_log.lock().unwrap();
    *debug = false;
    write_debug_log("=== DEBUG DÉSACTIVÉ ===");
    HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": "Mode debug désactivé"
    }))
}

async fn index() -> impl Responder {
    NamedFile::open_async("index.html").await.unwrap()
}

// ==================== MAIN ====================

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Créer un nouveau fichier de log au démarrage
    let _ = std::fs::write("modbus_debug.log", format!("=== DÉMARRAGE SERVEUR {} ===\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
    
    println!("========================================");
    println!("  CHINT ATS - Serveur Rust v2");
    println!("  Port: COM5 | 9600 Even | Adresse 6");
    println!("  Ouvrez http://localhost:5000");
    println!("  Actualisation automatique toutes les 5s");
    println!("  📁 Logs détaillés: modbus_debug.log");
    println!("  🔧 Activer debug: /api/debug_on");
    println!("  🔧 Désactiver debug: /api/debug_off");
    println!("========================================");
    
    let app_state = web::Data::new(Mutex::new(AppState {
        port_name: Mutex::new("COM5".to_string()),
        debug_log: Mutex::new(false),
    }));
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/api/read_all", web::get().to(read_all))
            .route("/api/remote_on", web::get().to(remote_on))
            .route("/api/remote_off", web::get().to(remote_off))
            .route("/api/force_double", web::get().to(force_double))
            .route("/api/force_source1", web::get().to(force_source1))
            .route("/api/force_source2", web::get().to(force_source2))
            .route("/api/set_undervoltage1", web::get().to(set_undervoltage1))
            .route("/api/set_undervoltage2", web::get().to(set_undervoltage2))
            .route("/api/set_overvoltage1", web::get().to(set_overvoltage1))
            .route("/api/set_overvoltage2", web::get().to(set_overvoltage2))
            .route("/api/debug_on", web::get().to(debug_on))
            .route("/api/debug_off", web::get().to(debug_off))
    })
    .bind("localhost:5000")?
    .run()
    .await
}
