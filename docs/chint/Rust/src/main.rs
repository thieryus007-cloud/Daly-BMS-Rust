use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use actix_files::NamedFile;
use serde::{Serialize, Deserialize};
use std::sync::Mutex;
use std::time::Duration;
use serialport::prelude::*;
use std::io::{Write, Read};
use std::thread;

// ==================== STRUCTURES ====================

struct AppState {
    port_name: Mutex<String>,
}

#[derive(Serialize)]
struct ModbusResponse {
    success: bool,
    values: std::collections::HashMap<String, String>,
    error: Option<String>,
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

fn read_register(port_name: &str, addr: u8, reg: u16) -> Option<u16> {
    let frame = build_frame(addr, 0x03, reg, None);
    
    // Ouverture du port série
    let mut port = match serialport::new(port_name, 9600)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(p) => p,
        Err(_) => return None,
    };
    
    // Envoi
    if port.write_all(&frame).is_err() {
        return None;
    }
    
    // Attente réponse
    thread::sleep(Duration::from_millis(100));
    
    // Lecture
    let mut buffer = vec![0u8; 256];
    match port.read(&mut buffer) {
        Ok(n) if n >= 5 => {
            let resp = &buffer[..n];
            if resp[1] == 0x03 && resp.len() >= 5 {
                Some(((resp[3] as u16) << 8) | resp[4] as u16)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn write_register(port_name: &str, addr: u8, reg: u16, value: u16) -> bool {
    let frame = build_frame(addr, 0x06, reg, Some(value));
    
    let mut port = match serialport::new(port_name, 9600)
        .data_bits(serialport::DataBits::Eight)
        .parity(serialport::Parity::Even)
        .stop_bits(serialport::StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
    {
        Ok(p) => p,
        Err(_) => return false,
    };
    
    if port.write_all(&frame).is_err() {
        return false;
    }
    
    thread::sleep(Duration::from_millis(100));
    
    let mut buffer = vec![0u8; 256];
    match port.read(&mut buffer) {
        Ok(n) if n > 0 => true,
        _ => false,
    }
}

// ==================== API ROUTES ====================

async fn read_all(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    
    let mut values = std::collections::HashMap::new();
    let addr = 6u8;
    
    // Lectures des tensions
    let regs = vec![
        (0x0006, "v1a", |x| format!("{} V", x)),
        (0x0007, "v1b", |x| format!("{} V", x)),
        (0x0008, "v1c", |x| format!("{} V", x)),
        (0x0009, "v2a", |x| format!("{} V", x)),
        (0x000A, "v2b", |x| format!("{} V", x)),
        (0x000B, "v2c", |x| format!("{} V", x)),
        (0x000C, "swVer", |x| format!("{:.2}", x as f32 / 100.0)),
        (0x0015, "cnt1", |x| x.to_string()),
        (0x0016, "cnt2", |x| x.to_string()),
        (0x0017, "runtime", |x| format!("{} h", x)),
    ];
    
    for (reg, key, formatter) in regs {
        if let Some(val) = read_register(&port_name, addr, reg) {
            values.insert(key.to_string(), formatter(val));
        } else {
            values.insert(key.to_string(), "---".to_string());
        }
    }
    
    // État des sources (0x004F)
    if let Some(power) = read_register(&port_name, addr, 0x004F) {
        let decode = |bit: u8| -> String {
            match (power >> bit) & 0x03 {
                0 => "✅ Normal".to_string(),
                1 => "⚠️ Sous-tension".to_string(),
                2 => "⚠️ Surtension".to_string(),
                _ => "❌ Erreur".to_string(),
            }
        };
        values.insert("s1a".to_string(), decode(8));
        values.insert("s1b".to_string(), decode(10));
        values.insert("s1c".to_string(), decode(12));
        values.insert("s2a".to_string(), decode(0));
        values.insert("s2b".to_string(), decode(2));
        values.insert("s2c".to_string(), decode(4));
    } else {
        for k in ["s1a", "s1b", "s1c", "s2a", "s2b", "s2c"] {
            values.insert(k.to_string(), "---".to_string());
        }
    }
    
    // État commutateur (0x0050)
    if let Some(switch) = read_register(&port_name, addr, 0x0050) {
        values.insert("sw1".to_string(), if switch & 0x02 != 0 { "✅ Fermé".to_string() } else { "⭕ Ouvert".to_string() });
        values.insert("sw2".to_string(), if switch & 0x04 != 0 { "✅ Fermé".to_string() } else { "⭕ Ouvert".to_string() });
        values.insert("swMid".to_string(), if switch & 0x08 != 0 { "⚠️ Oui".to_string() } else { "⭕ Non".to_string() });
        values.insert("swMode".to_string(), if switch & 0x01 != 0 { "🤖 Auto".to_string() } else { "👆 Manuel".to_string() });
        values.insert("swRemote".to_string(), if switch & 0x0100 != 0 { "📡 Activé".to_string() } else { "🔒 Désactivé".to_string() });
    } else {
        for k in ["sw1", "sw2", "swMid", "swMode", "swRemote"] {
            values.insert(k.to_string(), "---".to_string());
        }
    }
    
    let success = values.values().any(|v| v != "---");
    HttpResponse::Ok().json(ModbusResponse {
        success,
        values,
        error: if success { None } else { Some("Aucune réponse".to_string()) },
    })
}

async fn remote_on(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2800, 0x0004);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Télécommande activée"
    }))
}

async fn remote_off(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2800, 0x0000);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Télécommande désactivée"
    }))
}

async fn force_double(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2700, 0x00FF);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Forçage double"
    }))
}

async fn force_source1(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2700, 0x0000);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Forçage Source I"
    }))
}

async fn force_source2(data: web::Data<Mutex<AppState>>) -> impl Responder {
    let state = data.lock().unwrap();
    let port_name = state.port_name.lock().unwrap();
    let success = write_register(&port_name, 6, 0x2700, 0x00AA);
    HttpResponse::Ok().json(serde_json::json!({
        "success": success,
        "message": "Forçage Source II"
    }))
}

async fn index() -> impl Responder {
    NamedFile::open_async("index.html").await.unwrap()
}

// ==================== MAIN ====================

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("========================================");
    println!("  CHINT ATS - Serveur Rust");
    println!("  Port: COM5 | 9600 Even | Adresse 6");
    println!("  Ouvrez http://localhost:5000");
    println!("========================================");
    
    let app_state = web::Data::new(Mutex::new(AppState {
        port_name: Mutex::new("COM5".to_string()),
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
    })
    .bind("localhost:5000")?
    .run()
    .await
}
