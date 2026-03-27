import serial
import time

PORT = 'COM5'
BAUDRATE = 9600

# Trames à tester (adresse 6)
COMMANDS = {
    "État sources": bytes([0x06, 0x03, 0x00, 0x4F, 0x00, 0x01, 0xB4, 0x6A]),
    "État commutateur": bytes([0x06, 0x03, 0x00, 0x50, 0x00, 0x01, 0x44, 0xBE]),
    "Tension A Source I": bytes([0x06, 0x03, 0x00, 0x06, 0x00, 0x01, 0x25, 0xF4]),
    "Tension B Source I": bytes([0x06, 0x03, 0x00, 0x07, 0x00, 0x01, 0x74, 0x34]),
    "Tension C Source I": bytes([0x06, 0x03, 0x00, 0x08, 0x00, 0x01, 0x35, 0xF4]),
    "Version logicielle": bytes([0x06, 0x03, 0x00, 0x0C, 0x00, 0x01, 0x45, 0xF5]),
    "Adresse Modbus": bytes([0x06, 0x03, 0x01, 0x00, 0x00, 0x01, 0x85, 0xF5]),
    "Parité": bytes([0x06, 0x03, 0x00, 0x0E, 0x00, 0x01, 0xC5, 0xF4]),
}

# Commandes d'écriture (nécessitent télécommande activée d'abord)
WRITE_COMMANDS = {
    "Activer télécommande": bytes([0x06, 0x06, 0x28, 0x00, 0x00, 0x04, 0x49, 0x14]),
    "Désactiver télécommande": bytes([0x06, 0x06, 0x28, 0x00, 0x00, 0x00, 0x48, 0xD4]),
    "Forcer double": bytes([0x06, 0x06, 0x27, 0x00, 0x00, 0xFF, 0x83, 0x91]),
    "Forcer Source I": bytes([0x06, 0x06, 0x27, 0x00, 0x00, 0x00, 0x43, 0xD1]),
    "Forcer Source II": bytes([0x06, 0x06, 0x27, 0x00, 0x00, 0xAA, 0xC3, 0x98]),
}

ser = serial.Serial(PORT, BAUDRATE, bytesize=8, parity='E', stopbits=1, timeout=1)
time.sleep(0.5)

print("=" * 60)
print("TEST DES COMMANDES DE LECTURE")
print("=" * 60)

for name, frame in COMMANDS.items():
    print(f"\n📋 {name}:")
    print(f"   Envoi: {frame.hex().upper()}")
    
    ser.write(frame)
    time.sleep(0.1)
    response = ser.read(256)
    
    if response:
        print(f"   ✅ Réponse: {response.hex().upper()}")
        if len(response) >= 5:
            value = (response[3] << 8) | response[4]
            print(f"   📊 Valeur: {value} (0x{value:04X})")
    else:
        print(f"   ❌ TIMEOUT - Pas de réponse")

print("\n" + "=" * 60)
print("TEST DES COMMANDES D'ÉCRITURE")
print("=" * 60)
print("⚠️  Note: Certaines commandes peuvent ne pas répondre si la télécommande n'est pas activée")
print("=" * 60)

for name, frame in WRITE_COMMANDS.items():
    print(f"\n📋 {name}:")
    print(f"   Envoi: {frame.hex().upper()}")
    
    ser.write(frame)
    time.sleep(0.1)
    response = ser.read(256)
    
    if response:
        print(f"   ✅ Réponse: {response.hex().upper()}")
    else:
        print(f"   ❌ TIMEOUT - Pas de réponse")

ser.close()
print("\n" + "=" * 60)
print("Test terminé")
