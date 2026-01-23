# BLE Transport

Bluetooth Low Energy communication for CLASP.

## Overview

BLE transport enables CLASP communication with battery-powered devices and peripherals, ideal for wearables, sensors, and mobile applications.

## Features

| Feature | Support |
|---------|---------|
| Bidirectional | Yes |
| Reliable delivery | Configurable |
| Ordered delivery | Yes |
| Browser support | Web Bluetooth API |
| Encryption | BLE Security |
| Range | ~10-100m |
| Power | Very Low |

## GATT Profile

CLASP uses a custom GATT service:

```
Service UUID:     0000FFF0-0000-1000-8000-00805F9B34FB
Characteristic:   0000FFF1-0000-1000-8000-00805F9B34FB
  - Write, Notify
  - MTU: Up to 512 bytes
```

## Central (Client)

### Rust

```rust
use clasp_transport::ble::{BleTransport, BleConfig};

let config = BleConfig {
    device_name: Some("CLASP Sensor".into()),
    service_uuid: CLASP_SERVICE_UUID,
    characteristic_uuid: CLASP_CHAR_UUID,
};

let transport = BleTransport::new(config).await?;
transport.scan_and_connect().await?;
```

### Web Bluetooth

```javascript
// Request device
const device = await navigator.bluetooth.requestDevice({
    filters: [{ services: [CLASP_SERVICE_UUID] }]
});

// Connect
const server = await device.gatt.connect();
const service = await server.getPrimaryService(CLASP_SERVICE_UUID);
const characteristic = await service.getCharacteristic(CLASP_CHAR_UUID);

// Create transport
const transport = new BleTransport(characteristic);
const client = await Clasp.connect(transport);
```

### iOS/Android

```javascript
// React Native with react-native-ble-plx
import { BleManager } from 'react-native-ble-plx';

const manager = new BleManager();

manager.startDeviceScan([CLASP_SERVICE_UUID], null, (error, device) => {
    if (device.name === 'CLASP Sensor') {
        connectToDevice(device);
    }
});
```

## Peripheral (Server)

### ESP32

```cpp
#include <BLEDevice.h>
#include <CLASP.h>

BLEServer *server;
BLECharacteristic *characteristic;
CLASPBle clasp;

void setup() {
    BLEDevice::init("CLASP Sensor");
    server = BLEDevice::createServer();

    BLEService *service = server->createService(CLASP_SERVICE_UUID);
    characteristic = service->createCharacteristic(
        CLASP_CHAR_UUID,
        BLECharacteristic::PROPERTY_WRITE |
        BLECharacteristic::PROPERTY_NOTIFY
    );

    service->start();
    server->getAdvertising()->start();

    clasp.begin(characteristic);
}

void loop() {
    clasp.set("/sensor/temp", readTemp());
    clasp.update();
    delay(1000);
}
```

### Rust (embedded)

```rust
use clasp_embedded::ble::BlePeripheral;

let peripheral = BlePeripheral::new(BleConfig {
    name: "CLASP Sensor",
    service_uuid: CLASP_SERVICE_UUID,
    characteristic_uuid: CLASP_CHAR_UUID,
});

peripheral.start_advertising()?;

loop {
    if let Some(client) = peripheral.accept() {
        handle_client(client);
    }
}
```

## Message Fragmentation

BLE has limited MTU (20-512 bytes). CLASP handles fragmentation:

```
Large Message (> MTU):
  [Fragment 1: header + partial data]
  [Fragment 2: continuation]
  [Fragment 3: final fragment]
```

## Connection Parameters

Optimize for use case:

### Low Latency

```rust
let config = BleConfig {
    connection_interval_min: 7.5,  // ms
    connection_interval_max: 15.0,
    slave_latency: 0,
    ..Default::default()
};
```

### Low Power

```rust
let config = BleConfig {
    connection_interval_min: 100.0,  // ms
    connection_interval_max: 200.0,
    slave_latency: 4,
    ..Default::default()
};
```

## Security

### Pairing

```rust
let config = BleConfig {
    security: BleSecurity {
        bonding: true,
        mitm_protection: true,
        secure_connections: true,
    },
    ..Default::default()
};
```

### Encryption

BLE 4.2+ supports AES-CCM encryption automatically after pairing.

## Multiple Connections

Central can connect to multiple peripherals:

```rust
let sensor1 = BleTransport::connect("Sensor 1").await?;
let sensor2 = BleTransport::connect("Sensor 2").await?;

// Each has independent CLASP client
let client1 = Client::new(sensor1);
let client2 = Client::new(sensor2);
```

## Power Consumption

| Mode | Current |
|------|---------|
| Advertising | ~10-20 mA |
| Connected (idle) | ~1-5 mA |
| Transmitting | ~10-30 mA |
| Sleep | ~1-10 µA |

## Troubleshooting

### Discovery Fails

1. Check device is advertising
2. Verify service UUID
3. Check Bluetooth permissions
4. Try power cycling Bluetooth

### Connection Drops

1. Reduce connection interval
2. Check for interference
3. Verify power supply
4. Stay within range

### Slow Transfer

1. Negotiate larger MTU
2. Reduce connection interval
3. Use notifications instead of indications

## See Also

- [Embedded Systems](../../use-cases/embedded-systems.md)
- [Serial Transport](serial.md)
- [clasp-embedded](../api/rust/clasp-embedded.md)
