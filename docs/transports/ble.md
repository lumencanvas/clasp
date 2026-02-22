---
title: BLE
description: Bluetooth Low Energy transport for wireless battery-powered devices
order: 7
---

# BLE Transport

BLE (Bluetooth Low Energy) transport carries CLASP frames over Bluetooth wireless connections. It's designed for battery-powered devices -- wearables, wireless sensors, handheld controllers -- where power consumption matters more than throughput.

## How BLE Devices Reach the Relay

BLE is a short-range wireless protocol (~10-100m). It doesn't speak IP, so it can't connect to the relay directly. Instead, a nearby machine acts as an intermediate:

```
                                          The relay only
                                          speaks WS + QUIC
                                                |
ESP32 ──[BLE]──> Laptop running  ──[WebSocket]──> clasp-relay
                 CLASP client                     :7330
                 with BLE transport

Wearable ──[BLE]──> Phone app  ──[WebSocket]──> clasp-relay
                    with CLASP                   :7330
```

The laptop (or phone) scans for BLE devices advertising the CLASP GATT service, connects, and acts as a transparent relay. The ESP32 sends real CLASP binary frames over BLE -- no translation happens. The intermediate machine just shuttles bytes between BLE and WebSocket.

## Feature Flag

BLE is not included in default features:

```bash
cargo build --features ble
```

## GATT Service

CLASP uses a custom GATT service with a single characteristic for bidirectional communication:

```
Service UUID:         0000FFF0-0000-1000-8000-00805F9B34FB
Characteristic UUID:  0000FFF1-0000-1000-8000-00805F9B34FB
  Properties: Write, Notify
  MTU: up to 512 bytes
```

The client writes CLASP frames to the characteristic. The peripheral notifies the client when it has frames to send back.

## Rust Transport API

### Central (Scanning / Connecting)

This is the code that runs on the laptop or phone -- the machine that scans for BLE devices and connects to them:

```rust
use clasp_transport::ble::{BleTransport, BleConfig};

let config = BleConfig {
    device_name: Some("CLASP Sensor".into()),
    ..Default::default()
};

let transport = BleTransport::new(config).await?;
let (sender, receiver) = transport.scan_and_connect().await?;

// Same trait as every other transport
sender.send(clasp_frame).await?;
if let Some(TransportEvent::Data(bytes)) = receiver.recv().await {
    // handle frame from the BLE peripheral
}
```

### Multiple Devices

A central can connect to multiple BLE peripherals simultaneously:

```rust
let sensor1 = BleTransport::connect("Sensor 1").await?;
let sensor2 = BleTransport::connect("Sensor 2").await?;
// Each connection has independent sender/receiver pairs
```

## Peripheral (ESP32 / Embedded)

This is the code that runs on the battery-powered device -- it advertises the CLASP service and waits for a central to connect:

### ESP32 (Arduino)

```cpp
#include <BLEDevice.h>
#include <CLASP.h>

CLASPBle clasp;

void setup() {
    BLEDevice::init("CLASP Sensor");
    clasp.begin();  // sets up GATT service + advertising
}

void loop() {
    clasp.set("/sensor/temp", readTemp());
    clasp.update();
    delay(1000);
}
```

### Web Bluetooth (Browser)

Chrome can connect to BLE devices directly:

```javascript
const device = await navigator.bluetooth.requestDevice({
    filters: [{ services: ['0000fff0-0000-1000-8000-00805f9b34fb'] }]
})

const server = await device.gatt.connect()
const service = await server.getPrimaryService('0000fff0-0000-1000-8000-00805f9b34fb')
const char = await service.getCharacteristic('0000fff1-0000-1000-8000-00805f9b34fb')

const transport = new BleTransport(char)
const client = await new ClaspBuilder(transport)
```

## Message Fragmentation

BLE has a limited MTU (20-512 bytes depending on negotiation). CLASP frames larger than the MTU are automatically fragmented and reassembled by the BLE transport:

```
Large CLASP frame (> MTU):
  [Fragment 1: header + partial data]
  [Fragment 2: continuation]
  [Fragment 3: final fragment]
```

This is handled transparently. You send complete CLASP frames and receive complete CLASP frames.

## Power Consumption

| Mode | Typical Current |
|------|-----------------|
| Advertising (waiting for connection) | 10-20 mA |
| Connected, idle | 1-5 mA |
| Transmitting | 10-30 mA |
| Deep sleep | 1-10 uA |

To optimize battery life, increase the connection interval and enable slave latency:

```rust
// Low-power configuration
let config = BleConfig {
    connection_interval_min_ms: 100.0,
    connection_interval_max_ms: 200.0,
    slave_latency: 4,  // skip 4 connection events if no data
    ..Default::default()
};
```

For low-latency control (at the cost of battery):

```rust
// Low-latency configuration
let config = BleConfig {
    connection_interval_min_ms: 7.5,
    connection_interval_max_ms: 15.0,
    slave_latency: 0,
    ..Default::default()
};
```

## Troubleshooting

**Device not found during scan** -- Check that the peripheral is advertising. Verify the service UUID matches. Check Bluetooth permissions on the scanning device. Try power cycling Bluetooth.

**Connection drops frequently** -- Stay within range (~10-100m depending on environment). Reduce the connection interval. Check for 2.4GHz interference (WiFi, microwaves). Verify power supply is stable on the peripheral.

**Slow data transfer** -- Negotiate a larger MTU (reduces fragmentation overhead). Reduce the connection interval. Use notifications instead of indications (notifications don't wait for acknowledgment).

**Can't connect from Linux** -- Install `bluez` and ensure the `bluetooth` service is running. Your user may need to be in the `bluetooth` group.

## See Also

- [Serial Transport](serial.md) -- wired alternative for microcontrollers
- [Core Concepts: Transports](../core/transports.md) -- how transports fit into CLASP
