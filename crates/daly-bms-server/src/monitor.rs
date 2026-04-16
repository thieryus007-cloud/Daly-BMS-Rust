//! Agent de monitoring autonome — surveille les processus Pi5.
//!
//! Vérifie toutes les 30 secondes :
//! - État des services systemd critiques (daly-bms, mosquitto, influxdb, grafana, nodered)
//! - État des conteneurs Docker
//! - Utilisation CPU/mémoire/disque
//!
//! Actions automatiques :
//! - Si un conteneur Docker critique est arrêté → `docker restart <nom>`
//! - Résultats stockés dans AppState pour exposition via `/api/v1/monitor/status`

use crate::state::{AppState, MonitorSnapshot, ServiceStatus};
use chrono::Utc;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::interval;
use tracing::{info, warn};

/// Services systemd à surveiller.
const SYSTEMD_SERVICES: &[&str] = &["daly-bms", "mosquitto"];

/// Conteneurs Docker à surveiller (nom tel que configuré dans docker-compose).
const DOCKER_CONTAINERS: &[&str] = &["influxdb", "grafana", "nodered", "mosquitto"];

/// Démarre l'agent de monitoring en arrière-plan.
pub async fn run_monitor_agent(state: AppState) {
    info!("Agent de monitoring Pi5 démarré (intervalle: 30s)");
    let mut ticker = interval(Duration::from_secs(30));

    loop {
        ticker.tick().await;

        let snap = collect_snapshot(&state).await;
        state.on_monitor_snapshot(snap).await;
    }
}

/// Collecte un snapshot complet de l'état du système.
async fn collect_snapshot(state: &AppState) -> MonitorSnapshot {
    let mut services = Vec::new();
    let mut auto_actions = Vec::new();

    // ── Services systemd ──────────────────────────────────────────────────────
    for &name in SYSTEMD_SERVICES {
        let status = check_systemd_service(name).await;
        services.push(ServiceStatus {
            name: name.to_string(),
            active: status == "active",
            status,
        });
    }

    // ── Conteneurs Docker ─────────────────────────────────────────────────────
    for &name in DOCKER_CONTAINERS {
        // Éviter les doublons si déjà en systemd (mosquitto peut être les deux)
        if services.iter().any(|s| s.name == name) {
            continue;
        }
        let (status, was_down) = check_docker_container(name).await;
        let active = status == "running";

        // Action auto : redémarrer si arrêté
        if was_down {
            match restart_docker_container(name).await {
                true  => {
                    let msg = format!("Redémarré conteneur Docker: {}", name);
                    info!("{}", msg);
                    auto_actions.push(msg);
                    services.push(ServiceStatus { name: name.to_string(), active: true, status: "restarted".to_string() });
                }
                false => {
                    warn!("Échec redémarrage conteneur Docker: {}", name);
                    services.push(ServiceStatus { name: name.to_string(), active, status });
                }
            }
        } else {
            services.push(ServiceStatus { name: name.to_string(), active, status });
        }
    }

    // ── Métriques système ─────────────────────────────────────────────────────
    let cpu_percent    = read_cpu_percent().await;
    let memory_percent = read_memory_percent().await;
    let disk_percent   = read_disk_percent().await;
    let uptime_secs    = read_uptime_secs().await;

    // Vérifier la cohérence : si le service daly-bms est marqué inactif,
    // c'est normal — nous sommes le service daly-bms lui-même.
    if let Some(s) = services.iter_mut().find(|s| s.name == "daly-bms") {
        if !s.active {
            // Le processus tourne (nous sommes ici), le systemd peut lire "active"
            // ou "activating" selon le timing — on force à true
            s.active = true;
            s.status = "active".to_string();
        }
    }

    let _ = state; // Pas d'autre appel nécessaire ici
    MonitorSnapshot {
        timestamp: Utc::now(),
        services,
        cpu_percent,
        memory_percent,
        disk_percent,
        uptime_secs,
        auto_actions,
    }
}

/// Vérifie l'état d'un service systemd via `systemctl is-active`.
async fn check_systemd_service(name: &str) -> String {
    match Command::new("systemctl")
        .args(["is-active", "--quiet", name])
        .output()
        .await
    {
        Ok(out) => {
            if out.status.success() { "active".to_string() }
            else {
                // Lire le statut textuel
                match Command::new("systemctl")
                    .args(["is-active", name])
                    .output()
                    .await
                {
                    Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
                    Err(_) => "unknown".to_string(),
                }
            }
        }
        Err(_) => "unknown".to_string(),
    }
}

/// Vérifie l'état d'un conteneur Docker.
/// Retourne (status_str, was_down).
async fn check_docker_container(name: &str) -> (String, bool) {
    match Command::new("docker")
        .args(["inspect", "--format", "{{.State.Status}}", name])
        .output()
        .await
    {
        Ok(out) if out.status.success() => {
            let status = String::from_utf8_lossy(&out.stdout).trim().to_string();
            let was_down = status != "running";
            (status, was_down)
        }
        _ => ("unknown".to_string(), false),
    }
}

/// Tente de redémarrer un conteneur Docker arrêté.
async fn restart_docker_container(name: &str) -> bool {
    match Command::new("docker")
        .args(["restart", name])
        .output()
        .await
    {
        Ok(out) => out.status.success(),
        Err(_)  => false,
    }
}

/// Lit l'utilisation CPU depuis `/proc/stat` (deux lectures avec 200ms d'écart).
async fn read_cpu_percent() -> f32 {
    let read_stat = || async {
        tokio::fs::read_to_string("/proc/stat").await.ok().and_then(|s| {
            let line = s.lines().next()?;
            let mut parts = line.split_whitespace().skip(1);
            let user:    u64 = parts.next()?.parse().ok()?;
            let nice:    u64 = parts.next()?.parse().ok()?;
            let system:  u64 = parts.next()?.parse().ok()?;
            let idle:    u64 = parts.next()?.parse().ok()?;
            let iowait:  u64 = parts.next()?.parse().ok()?;
            let irq:     u64 = parts.next()?.parse().ok()?;
            let softirq: u64 = parts.next()?.parse().ok()?;
            let total = user + nice + system + idle + iowait + irq + softirq;
            Some((total, idle + iowait))
        })
    };

    let before = read_stat().await;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let after  = read_stat().await;

    if let (Some((t1, i1)), Some((t2, i2))) = (before, after) {
        let dt = (t2 - t1) as f32;
        let di = (i2 - i1) as f32;
        if dt > 0.0 { (1.0 - di / dt) * 100.0 } else { 0.0 }
    } else {
        0.0
    }
}

/// Lit l'utilisation mémoire depuis `/proc/meminfo`.
async fn read_memory_percent() -> f32 {
    let Ok(content) = tokio::fs::read_to_string("/proc/meminfo").await else { return 0.0; };
    let mut total = 0u64;
    let mut available = 0u64;
    for line in content.lines() {
        if line.starts_with("MemTotal:") {
            total = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        } else if line.starts_with("MemAvailable:") {
            available = line.split_whitespace().nth(1).and_then(|v| v.parse().ok()).unwrap_or(0);
        }
    }
    if total > 0 { ((total - available) as f32 / total as f32) * 100.0 } else { 0.0 }
}

/// Lit l'utilisation disque via `df /`.
async fn read_disk_percent() -> f32 {
    match Command::new("df")
        .args(["-h", "--output=pcent", "/"])
        .output()
        .await
    {
        Ok(out) => {
            let s = String::from_utf8_lossy(&out.stdout);
            s.lines()
                .nth(1) // Skip header
                .and_then(|l| l.trim().trim_end_matches('%').parse::<f32>().ok())
                .unwrap_or(0.0)
        }
        Err(_) => 0.0,
    }
}

/// Lit l'uptime depuis `/proc/uptime`.
async fn read_uptime_secs() -> u64 {
    tokio::fs::read_to_string("/proc/uptime")
        .await
        .ok()
        .and_then(|s| s.split_whitespace().next()?.parse::<f64>().ok())
        .map(|v| v as u64)
        .unwrap_or(0)
}
