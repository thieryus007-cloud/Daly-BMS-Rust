//! Boucle de polling Modbus RTU pour l'ATS CHINT NXZB/NXZBN.
//!
//! ## Protocole
//!
//! - FC=03 (Read Holding Registers) pour la lecture
//! - FC=06 (Write Single Register) pour les commandes
//! - Parité : EVEN (8E1) — bus RS485 SÉPARÉ du bus BMS (8N1)
//!
//! ## Stratégie de lecture groupée
//!
//! Pour minimiser les transactions RS485 :
//! - Bloc A : 0x0006, count=13 → v1a..max1 + sw_ver + freq + parity
//! - Bloc B : 0x0010, count=3  → (réservés) + max2
//! - Bloc C : 0x0015, count=3  → cnt1, cnt2, runtime
//! - Bloc D : 0x004F, count=2  → pwr_status, sw_status
//! - Bloc E : 0x0100, count=2  → modbus_addr, modbus_baud
//! - Bloc F (MN): 0x2065, count=9 → uv1..operation_mode

use super::types::{
    ActiveSource, AtsCommand, AtsSnapshot, FaultCode, OperationMode, PhaseStatus,
};
use crate::config::AtsConfig;
use chrono::Local;
use rs485_bus::{modbus_rtu, SharedBus};
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, info, warn};

// =============================================================================
// Longueur de réponse FC06 (echo de la requête)
// =============================================================================

const FC06_RESPONSE_LEN: usize = 8;

// =============================================================================
// Validation réponse FC06
// =============================================================================

fn validate_fc06_response(addr: u8, response: &[u8]) -> anyhow::Result<()> {
    if response.len() < FC06_RESPONSE_LEN {
        anyhow::bail!(
            "Réponse FC06 trop courte: {} octets (attendu {})",
            response.len(),
            FC06_RESPONSE_LEN
        );
    }
    if response[0] != addr {
        anyhow::bail!(
            "FC06 adresse inattendue: attendu {:#04x}, reçu {:#04x}",
            addr,
            response[0]
        );
    }
    if response[1] == 0x86 {
        let exc = response.get(2).copied().unwrap_or(0);
        anyhow::bail!("FC06 exception Modbus: code {:#04x}", exc);
    }
    if response[1] != 0x06 {
        anyhow::bail!("FC06 function code inattendu: {:#04x}", response[1]);
    }
    let crc_calc = modbus_rtu::crc16(&response[..6]);
    let crc_recv =
        (response[6] as u16) | ((response[7] as u16) << 8);
    if crc_recv != crc_calc {
        anyhow::bail!(
            "FC06 CRC invalide: reçu {:#06x}, calculé {:#06x}",
            crc_recv,
            crc_calc
        );
    }
    Ok(())
}

// =============================================================================
// Polling principal
// =============================================================================

/// Lance la boucle de polling ATS sur son bus RS485 dédié.
///
/// # Paramètres
/// - `bus`         : bus RS485 dédié à l'ATS (parité Even)
/// - `cfg`         : configuration de l'ATS
/// - `on_snapshot` : callback appelé pour chaque snapshot valide
pub async fn run_ats_poll_loop<F>(
    bus: Arc<SharedBus>,
    cfg: AtsConfig,
    mut on_snapshot: F,
)
where
    F: FnMut(AtsSnapshot) + Send + 'static,
{
    let addr = cfg.address;
    let poll_interval = Duration::from_millis(cfg.poll_interval_ms);

    info!(
        addr = format!("{:#04x}", addr),
        name = %cfg.name,
        "ATS CHINT polling démarré (bus RS485 unifié)"
    );

    // Détection du modèle au démarrage
    let model = detect_model(&bus, addr).await;
    info!(addr = format!("{:#04x}", addr), model = %model, "Modèle ATS détecté");

    let mut consecutive_errors: u32 = 0;

    loop {
        match poll_ats(&bus, addr, &cfg.name, &model).await {
            Ok(snap) => {
                debug!(
                    addr       = format!("{:#04x}", addr),
                    source     = %snap.active_source.label(),
                    v1a        = snap.v1a,
                    v2a        = snap.v2a,
                    "ATS snapshot OK"
                );
                consecutive_errors = 0;
                on_snapshot(snap);
            }
            Err(e) => {
                consecutive_errors += 1;
                if consecutive_errors == 1 || consecutive_errors % 10 == 0 {
                    warn!(
                        addr   = format!("{:#04x}", addr),
                        errors = consecutive_errors,
                        "ATS erreur lecture : {:#}",
                        e
                    );
                }
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

// =============================================================================
// Détection du modèle
// =============================================================================

/// Détecte le modèle ATS (MN ou BN) en tentant de lire le registre 0x2065.
/// Si la lecture réussit → MN, sinon → BN.
async fn detect_model(bus: &SharedBus, addr: u8) -> String {
    let req = modbus_rtu::build_fc03(addr, 0x2065, 1);
    let resp_len = modbus_rtu::response_len(1);
    match bus.transact(&req, resp_len).await {
        Ok(resp) if modbus_rtu::parse_read_response(addr, 0x03, &resp).is_ok() => {
            "MN".to_string()
        }
        _ => "BN".to_string(),
    }
}

// =============================================================================
// Lecture d'un snapshot complet
// =============================================================================

async fn poll_ats(
    bus:   &SharedBus,
    addr:  u8,
    name:  &str,
    model: &str,
) -> anyhow::Result<AtsSnapshot> {
    // ── Bloc A : 0x0006, count=13 ─────────────────────────────────────────
    // v1a(6), v1b(7), v1c(8), v2a(9), v2b(A), v2c(B),
    // sw_ver(C), freq(D), parity(E), max1(F), r10, r11, max2(12)
    let req_a = modbus_rtu::build_fc03(addr, 0x0006, 13);
    let resp_a = bus
        .transact(&req_a, modbus_rtu::response_len(13))
        .await
        .map_err(|e| anyhow::anyhow!("ATS bloc A: {}", e))?;
    let regs_a = modbus_rtu::parse_read_response(addr, 0x03, &resp_a)
        .map_err(|e| anyhow::anyhow!("ATS parse bloc A: {}", e))?;
    if regs_a.len() < 13 {
        anyhow::bail!("ATS bloc A trop court ({} registres)", regs_a.len());
    }

    // ── Bloc C : 0x0015, count=3 ──────────────────────────────────────────
    let req_c = modbus_rtu::build_fc03(addr, 0x0015, 3);
    let resp_c = bus
        .transact(&req_c, modbus_rtu::response_len(3))
        .await
        .map_err(|e| anyhow::anyhow!("ATS bloc C: {}", e))?;
    let regs_c = modbus_rtu::parse_read_response(addr, 0x03, &resp_c)
        .map_err(|e| anyhow::anyhow!("ATS parse bloc C: {}", e))?;
    if regs_c.len() < 3 {
        anyhow::bail!("ATS bloc C trop court ({} registres)", regs_c.len());
    }

    // ── Bloc D : 0x004F, count=2 ──────────────────────────────────────────
    let req_d = modbus_rtu::build_fc03(addr, 0x004F, 2);
    let resp_d = bus
        .transact(&req_d, modbus_rtu::response_len(2))
        .await
        .map_err(|e| anyhow::anyhow!("ATS bloc D: {}", e))?;
    let regs_d = modbus_rtu::parse_read_response(addr, 0x03, &resp_d)
        .map_err(|e| anyhow::anyhow!("ATS parse bloc D: {}", e))?;
    if regs_d.len() < 2 {
        anyhow::bail!("ATS bloc D trop court ({} registres)", regs_d.len());
    }

    // ── Bloc E : 0x0100, count=2 ──────────────────────────────────────────
    let (modbus_addr, modbus_baud_code) =
        match read_bloc_e(bus, addr).await {
            Ok((a, b)) => (Some(a), Some(b)),
            Err(_)     => (None, None),
        };

    // ── Décodage Bloc A ────────────────────────────────────────────────────
    let v1a        = regs_a[0] as f32;
    let v1b        = regs_a[1] as f32;
    let v1c        = regs_a[2] as f32;
    let v2a        = regs_a[3] as f32;
    let v2b        = regs_a[4] as f32;
    let v2c        = regs_a[5] as f32;
    let sw_version = regs_a[6] as f32 / 100.0;
    let freq_raw   = regs_a[7];
    let max1_v     = regs_a[9];
    // regs_a[10..11] = réservés
    let max2_v     = regs_a[12];

    let (freq1_hz, freq2_hz) = if model == "MN" {
        (
            Some(((freq_raw >> 8) & 0xFF) as u8),
            Some((freq_raw & 0xFF) as u8),
        )
    } else {
        (None, None)
    };

    // ── Décodage Bloc C ────────────────────────────────────────────────────
    let cnt1      = regs_c[0];
    let cnt2      = regs_c[1];
    let runtime_h = regs_c[2];

    // ── Décodage Bloc D ────────────────────────────────────────────────────
    // Registre 0x004F — statut tension
    // Bits [15:8] = source 1 : [15:14]=C, [13:12]=B, [11:10]=A... non !
    // Selon le code Qwen vérifié sur matériel réel :
    //   bits [15:8] (source 1) : bit 8-9=A, 10-11=B, 12-13=C
    //   bits [7:0]  (source 2) : bit 0-1=A, 2-3=B, 4-5=C
    let pwr = regs_d[0];
    let s1a = PhaseStatus::from_bits(((pwr >> 8) & 0x03) as u8);
    let s1b = PhaseStatus::from_bits(((pwr >> 10) & 0x03) as u8);
    let s1c = PhaseStatus::from_bits(((pwr >> 12) & 0x03) as u8);
    let s2a = PhaseStatus::from_bits((pwr & 0x03) as u8);
    let s2b = PhaseStatus::from_bits(((pwr >> 2) & 0x03) as u8);
    let s2c = PhaseStatus::from_bits(((pwr >> 4) & 0x03) as u8);

    // Registre 0x0050 — statut commutation
    // Bit 3  = SW1 (Onduleur) : 0=fermé, 1=ouvert
    // Bit 4  = SW2 (Réseau)   : 0=fermé, 1=ouvert
    // Bit 1  = Mode           : 0=Manuel, 1=Auto
    // Bit 8  = Télécommande   : 0=Off, 1=On
    // Bits 5-7 = Code défaut
    // Registre 0x0050 — statut commutation
    // bit3=1 → SW1 ouvert (Onduleur côté), bit4=1 → SW2 ouvert (Réseau côté)
    // middle_off = bit3=0 ET bit4=0 → position centrale (double déclenché)
    // SW1 fermé  = bit3=0 ET bit4=1 (SW2 ouvert, SW1 fermé)
    // SW2 fermé  = bit4=0 ET bit3=1 (SW1 ouvert, SW2 fermé)
    // Référence : code Qwen validé sur matériel réel CHINT NXZB.
    let sw = regs_d[1];
    let sw1_raw    = (sw & 0x0008) != 0;
    let sw2_raw    = (sw & 0x0010) != 0;
    let middle_off = !sw1_raw && !sw2_raw;
    let sw1_closed = !middle_off && !sw1_raw;
    let sw2_closed = !middle_off && !sw2_raw;

    let sw_mode    = (sw & 0x0001) != 0; // bit0=1 → Auto
    let remote     = (sw & 0x0100) != 0; // bit8=1 → télécommande active
    let fault      = FaultCode::from_u8(((sw >> 5) & 0x07) as u8);

    let active_source = if sw1_closed {
        ActiveSource::Source1
    } else if sw2_closed {
        ActiveSource::Source2
    } else {
        ActiveSource::Neutral
    };

    // ── Bloc F (MN uniquement) ─────────────────────────────────────────────
    let (operation_mode, uv1, uv2, ov1, ov2, t1_s, t2_s, t3_s, t4_s) =
        if model == "MN" {
            match read_bloc_f(bus, addr).await {
                Ok(regs_f) => (
                    Some(OperationMode::from_u16(regs_f[8])),
                    Some(regs_f[0]),
                    Some(regs_f[1]),
                    Some(regs_f[2]),
                    Some(regs_f[3]),
                    Some(regs_f[4]),
                    Some(regs_f[5]),
                    Some(regs_f[6]),
                    Some(regs_f[7]),
                ),
                Err(_) => (None, None, None, None, None, None, None, None, None),
            }
        } else {
            (None, None, None, None, None, None, None, None, None)
        };

    Ok(AtsSnapshot {
        address: addr,
        name: name.to_string(),
        model: model.to_string(),
        timestamp: Local::now(),
        v1a, v1b, v1c, v2a, v2b, v2c,
        max1_v, max2_v,
        s1a, s1b, s1c, s2a, s2b, s2c,
        sw_mode,
        sw1_closed,
        sw2_closed,
        middle_off,
        remote,
        fault,
        active_source,
        cnt1, cnt2, runtime_h,
        sw_version,
        freq1_hz, freq2_hz,
        operation_mode,
        uv1, uv2, ov1, ov2,
        t1_s, t2_s, t3_s, t4_s,
        modbus_addr, modbus_baud_code,
    })
}

// =============================================================================
// Lectures auxiliaires
// =============================================================================

async fn read_bloc_e(bus: &SharedBus, addr: u8) -> anyhow::Result<(u16, u16)> {
    let req = modbus_rtu::build_fc03(addr, 0x0100, 2);
    let resp = bus.transact(&req, modbus_rtu::response_len(2)).await?;
    let regs = modbus_rtu::parse_read_response(addr, 0x03, &resp)?;
    if regs.len() < 2 {
        anyhow::bail!("bloc E trop court");
    }
    Ok((regs[0], regs[1]))
}

async fn read_bloc_f(bus: &SharedBus, addr: u8) -> anyhow::Result<Vec<u16>> {
    // 0x2065..0x206D = 9 registres
    let req = modbus_rtu::build_fc03(addr, 0x2065, 9);
    let resp = bus.transact(&req, modbus_rtu::response_len(9)).await?;
    let regs = modbus_rtu::parse_read_response(addr, 0x03, &resp)?;
    if regs.len() < 9 {
        anyhow::bail!("bloc F trop court ({} registres)", regs.len());
    }
    Ok(regs)
}

// =============================================================================
// Exécution d'une commande FC=06
// =============================================================================

/// Exécute une commande d'écriture sur l'ATS (FC=06).
///
/// Retourne Ok(()) si la commande a été acceptée par l'ATS.
pub async fn execute_ats_command(
    bus:  &SharedBus,
    addr: u8,
    cmd:  AtsCommand,
) -> anyhow::Result<()> {
    let (reg, value) = cmd.register_value();
    let frame = modbus_rtu::build_fc06(addr, reg, value);

    let response = bus
        .transact(&frame, FC06_RESPONSE_LEN)
        .await
        .map_err(|e| anyhow::anyhow!("ATS FC06 TX erreur: {}", e))?;

    validate_fc06_response(addr, &response)?;

    info!(
        addr  = format!("{:#04x}", addr),
        cmd   = cmd.label(),
        reg   = format!("{:#06x}", reg),
        value = format!("{:#06x}", value),
        "ATS commande exécutée avec succès"
    );

    Ok(())
}
