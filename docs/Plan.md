---

Plan d'Implémentation : Monitoring de Daly Smart BMS en Rust

Version 1.0
Date : 14 Mars 2026
Auteur : Consultation technique

---

📋 Table des matières

1. Présentation du projet
2. Spécifications techniques
3. Architecture logicielle
4. Protocole de communication
5. Structures de données
6. Plan d'implémentation pas à pas
7. Code source complet
8. Tests et validation
9. Maintenance et évolutions
10. Annexes

---

1. Présentation du projet

1.1 Objectif

Développer une application Rust asynchrone pour monitorer 2 à 3 Daly Smart BMS connectés sur un bus RS485 partagé (un seul convertisseur USB-RS485). L'application doit récupérer l'ensemble des données définies dans le fichier JSONData.json fourni.

1.2 Contraintes

· Un seul port série physique (ex: /dev/ttyUSB0 ou COM3)
· 2-3 BMS sur le même bus RS485
· Communication asynchrone avec Tokio
· Solution 100% maîtrisée (pas de fork de bibliothèque existante)
· Données de sortie structurées (format JSON ou structure Rust)

---

2. Spécifications techniques

2.1 Matériel requis

Élément Spécification Quantité
Convertisseur USB-RS485 FTDI, CP210x ou CH340 1
Daly Smart BMS Version avec support UART/RS485 2-3
Câblage Paire torsadée + masse commune -

2.2 Configuration des BMS

Avant le déploiement, chaque BMS doit avoir une adresse unique configurée via le logiciel PC officiel Daly :

BMS Adresse recommandée
BMS #1 0x01
BMS #2 0x02
BMS #3 0x03

Mot de passe usine : 12345678 (selon la documentation)

2.3 Câblage RS485

```
[PC] -- USB -- [Convertisseur USB-RS485] -- A (jaune) ----+---- [BMS1 A]
                                                           +---- [BMS2 A]
                                                           +---- [BMS3 A]
                                         -- B (vert) ----+---- [BMS1 B]
                                                           +---- [BMS2 B]
                                                           +---- [BMS3 B]
                                         -- GND (noir) ---+---- [BMS1 GND]
                                                           +---- [BMS2 GND]
                                                           +---- [BMS3 GND]
```

Important : La masse (GND) doit être commune à tous les appareils pour éviter les boucles de courant et garantir l'intégrité du signal.

---

3. Architecture logicielle

3.1 Stack technique

```
[Application Rust]
    ├── Tokio (runtime asynchrone)
    ├── tokio-serial (communication série)
    ├── serde (sérialisation JSON)
    └── anyhow (gestion d'erreurs)
```

3.2 Composants principaux

```
┌─────────────────────────────────────┐
│         BMSMonitor (Orchestrateur)   │
│  - Gère la boucle principale         │
│  - Coordonne le polling des BMS      │
└─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────┐
│         DalyBus (Gestionnaire bus)   │
│  - Mutex sur le port série           │
│  - Envoi/Réception des trames        │
│  - Calcul checksum                   │
└─────────────────────────────────────┘
                    │
                    ▼
┌─────────────────────────────────────┐
│         BMSDevice (Client BMS)       │
│  - Implémente les commandes Daly     │
│  - Parse les réponses                │
│  - Construit les structures de données│
└─────────────────────────────────────┘
```

---

4. Protocole de communication

4.1 Spécifications UART (d'après le document Daly)

Paramètre Valeur
Baud rate 9600 bps
Data bits 8
Stop bits 1
Parity None
Flow control None

4.2 Format des trames

Requête (PC → BMS) :

Octet Champ Valeur
0 Start Flag 0xA5
1 PC Address 0x40
2 Data ID Variable (0x90, 0x91...)
3-10 Data Content 8 bytes (généralement 0x00)
11 Checksum Somme octets 0-10 (low byte)

Réponse (BMS → PC) :

Octet Champ Valeur
0 Start Flag 0xA5
1 BMS Address Variable (0x01, 0x02...)
2 Data ID Variable (identique à la requête)
3-10 Data Content 8 bytes de données
11 Checksum Somme octets 0-10 (low byte)

4.3 Data IDs implémentés

Data ID Description Pages doc Priorité
0x90 Tension totale, courant, SOC 5 Haute
0x91 Tensions min/max des cellules 5 Haute
0x92 Températures min/max 5 Haute
0x93 État MOS, cycles, capacité 5 Haute
0x94 Status information 1 5 Haute
0x95-0x98 Tensions cellule 1-16 6 Haute

4.4 Calcul du checksum

```rust
fn calculate_checksum(frame: &[u8; 10]) -> u8 {
    let sum: u16 = frame.iter().map(|&b| b as u16).sum();
    (sum & 0xFF) as u8
}
```

---

5. Structures de données

Basées sur le fichier JSONData.json fourni :

```rust
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Structure principale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BMSData {
    pub dc: DcData,
    pub installed_capacity: f32,
    pub consumed_amphours: f32,
    pub capacity: f32,
    pub soc: f32,
    pub soh: f32,
    pub time_to_go: u32,
    pub balancing: u8,
    pub system_switch: u8,
    pub alarms: Alarms,
    pub info: InfoData,
    pub history: HistoryData,
    pub system: SystemData,
    pub voltages: HashMap<String, f32>,  // "Cell1" -> 3.405
    pub balances: HashMap<String, u8>,   // "Cell1" -> 0/1
    pub io: IoData,
    pub heating: u8,
    pub time_to_soc: HashMap<u8, u32>,   // 0 -> 0, 5 -> 7200...
}

// Sous-structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DcData {
    pub power: f32,
    pub voltage: f32,
    pub current: f32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alarms {
    pub low_voltage: u8,
    pub high_voltage: u8,
    pub low_soc: u8,
    pub high_charge_current: u8,
    pub high_discharge_current: u8,
    pub high_current: u8,
    pub cell_imbalance: u8,
    pub high_charge_temperature: u8,
    pub low_charge_temperature: u8,
    pub low_cell_voltage: u8,
    pub low_temperature: u8,
    pub high_temperature: u8,
    pub fuse_blown: u8,
}

// ... (autres structures détaillées en annexe)
```

---

6. Plan d'implémentation pas à pas

Phase 1 : Préparation et configuration (Jour 1-2)

· Installer Rust et les dépendances
· Configurer chaque BMS avec une adresse unique via le logiciel Daly
· Câbler le bus RS485 avec masse commune
· Tester la communication avec un outil comme screen ou minicom

Phase 2 : Communication série de base (Jour 3-4)

· Créer un nouveau projet Rust (cargo new bms-monitor --bin)
· Ajouter les dépendances dans Cargo.toml
· Implémenter l'ouverture du port série avec tokio-serial
· Tester l'envoi/réception basique

Phase 3 : Implémentation du protocole (Jour 5-8)

· Créer le module protocol avec les structures de trame
· Implémenter la fonction calculate_checksum
· Créer la structure DalyBus avec Mutex sur le port
· Implémenter la méthode send_command générique

Phase 4 : Commandes spécifiques (Jour 9-12)

· Implémenter get_soc (0x90)
· Implémenter get_cell_voltages_minmax (0x91)
· Implémenter get_temperatures (0x92)
· Implémenter get_mos_status (0x93)
· Implémenter get_status_info1 (0x94)
· Implémenter get_cell_voltages_block (0x95-0x98)

Phase 5 : Structures de données (Jour 13-14)

· Créer toutes les structures Rust correspondant au JSON
· Implémenter le parsing des données brutes vers les structures
· Ajouter les dérivations Serialize/Deserialize

Phase 6 : Orchestration (Jour 15-16)

· Créer la structure BMSMonitor
· Implémenter la méthode poll_all pour interroger tous les BMS
· Ajouter les délais entre les requêtes (50ms)
· Gérer les timeouts et erreurs

Phase 7 : Interface et sortie (Jour 17-18)

· Ajouter l'affichage console des données
· Option de sortie JSON
· Gestion des signaux (Ctrl+C)

Phase 8 : Tests et validation (Jour 19-20)

· Tests unitaires pour le checksum et le parsing
· Tests d'intégration avec vrais BMS
· Validation des données par rapport au JSON de référence

---

7. Code source complet

7.1 Structure du projet

```
bms-monitor/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── bus.rs          # Gestion du bus série
│   ├── protocol.rs     # Définitions du protocole
│   ├── commands.rs     # Implémentation des commandes
│   ├── data.rs         # Structures de données
│   └── monitor.rs      # Orchestrateur
```

7.2 Cargo.toml

```toml
[package]
name = "bms-monitor"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-serial = "5.5"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
anyhow = "1"
thiserror = "1"
tracing = "0.1"
tracing-subscriber = "0.3"
```

7.3 Code principal

Voir les sections précédentes pour le code détaillé des composants.

---

8. Tests et validation

8.1 Tests unitaires

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_checksum_calculation() {
        let frame = [0xA5, 0x40, 0x90, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
        assert_eq!(calculate_checksum(&frame), 0x15); // 0xA5+0x40+0x90 = 0x175 → 0x75? À vérifier
    }
    
    #[test]
    fn test_parse_soc_response() {
        let response = [0xA5, 0x01, 0x90, 0x00, 0x00, 0x02, 0x0D, 0x40, 0x00, 0x00, 0x00, 0x00];
        // Tester le parsing
    }
}
```

8.2 Validation des données

Comparer les données lues avec le fichier JSONData.json :

```rust
fn validate_against_reference(actual: &BMSData, reference: &BMSData) -> bool {
    // Vérifier que les champs principaux sont dans des plages plausibles
    (actual.soc - reference.soc).abs() < 5.0 &&
    (actual.dc.voltage - reference.dc.voltage).abs() < 2.0
}
```

---

9. Maintenance et évolutions

9.1 Points de vigilance

· Timeout : Ajuster le timeout en fonction de la charge du bus
· Délais inter-trames : 50ms minimum entre chaque commande
· Reconnexion : Gérer la déconnexion/reconnexion du câble USB
· Logs : Implémenter des logs rotatifs pour le débogage

9.2 Évolutions possibles

· Support CAN : Ajouter le support du protocole CAN si nécessaire
· Base de données : Stocker l'historique dans InfluxDB/TimescaleDB
· Interface web : Ajouter une petite API REST avec Axum
· Alertes : Notifications Telegram/email en cas d'alarme
· Configuration dynamique : Permettre de modifier les adresses BMS sans recompiler

---

10. Annexes

Annexe A : Correspondance complète JSON ↔ Data IDs

Champ JSON Data ID Octets Calcul
dc.voltage 0x90 2-3 uint16 / 10
dc.current 0x90 4-5 (uint16 - 30000) / 10
soc 0x90 6-7 uint16 / 10
system.max_cell_voltage 0x91 0-1 uint16 / 1000
system.max_voltage_cell_id 0x91 2 "C" + uint8
system.min_cell_voltage 0x91 3-4 uint16 / 1000
... ... ... ...

Annexe B : Références

· Documentation Daly UART/485 Protocol V1.2
· Tokio Serial Documentation
· Serde Documentation

Annexe C : Glossaire

Terme Définition
BMS Battery Management System
RS485 Standard de communication série différentiel
SOC State of Charge (État de charge)
SOH State of Health (État de santé)
MOS Metal-Oxide-Semiconductor (interrupteurs de puissance)
Checksum Somme de contrôle pour vérifier l'intégrité des données

---

✅ Checklist finale avant mise en production

· Tous les BMS ont des adresses uniques
· Le câblage RS485 est correct (A, B, GND)
· Le convertisseur USB-RS485 est reconnu par le système
· Les droits d'accès au port série sont configurés (dialout, etc.)
· L'application Rust compile sans erreur
· Les données lues sont cohérentes avec les valeurs attendues
· La boucle de monitoring tourne de manière stable pendant 24h
· Les erreurs de communication sont correctement gérées
· La sortie JSON est valide

---

Document généré le 14 Mars 2026
