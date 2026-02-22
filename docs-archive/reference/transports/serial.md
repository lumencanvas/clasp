---
title: "Serial Transport"
description: "UART/Serial communication for CLASP."
section: reference
order: 3
---
# Serial Transport

UART/Serial communication for CLASP.

## Overview

Serial transport enables CLASP communication over UART/RS-232 connections, commonly used with microcontrollers and embedded systems.

## Features

| Feature | Support |
|---------|---------|
| Bidirectional | Yes |
| Reliable delivery | Application-level |
| Ordered delivery | Yes |
| Browser support | Web Serial API |
| Encryption | No |
| Connection overhead | None |
| Latency | Low |

## Connection

```
serial://port?baud=115200
serial:///dev/ttyUSB0?baud=115200
```

## Configuration

### Rust

```rust
use clasp_transport::serial::{SerialTransport, SerialConfig};

let config = SerialConfig {
    port: "/dev/ttyUSB0".into(),
    baud_rate: 115200,
    data_bits: 8,
    stop_bits: 1,
    parity: Parity::None,
    flow_control: FlowControl::None,
};

let transport = SerialTransport::new(config)?;
```

### Common Baud Rates

| Rate | Use Case |
|------|----------|
| 9600 | Legacy devices |
| 115200 | Common default |
| 250000 | DMX (special) |
| 921600 | High-speed |
| 1000000+ | USB-Serial |

## Frame Delimiting

### SLIP (Serial Line IP)

Default framing protocol:

```
END = 0xC0
ESC = 0xDB
ESC_END = 0xDC
ESC_ESC = 0xDD

Frame: [data with escapes] END
```

### COBS (Consistent Overhead Byte Stuffing)

More efficient for binary data:

```rust
let config = SerialConfig {
    framing: Framing::Cobs,
    ..Default::default()
};
```

### Length-Prefixed

Simple fixed header:

```
[length: 2 bytes] [payload: length bytes]
```

### Line Delimited

For text protocols:

```rust
let config = SerialConfig {
    framing: Framing::LineDelimited('\n'),
    ..Default::default()
};
```

## Embedded Integration

### ESP32 (Arduino)

```cpp
#include <CLASP.h>

CLASPSerial clasp(Serial);

void setup() {
    Serial.begin(115200);
    clasp.begin();
}

void loop() {
    // Send sensor data
    clasp.set("/sensor/temp", readTemp());

    // Process incoming
    clasp.update();
}
```

### Rust (no_std)

```rust
use clasp_embedded::{Client, SerialFramer};

let mut client = Client::new();
let mut framer = SerialFramer::new();

// Send
let frame = client.prepare_set("/sensor/temp", Value::Float(23.5));
uart_send(framer.encode(&frame));

// Receive
while let Some(byte) = uart_recv() {
    if let Some(frame) = framer.decode(byte) {
        handle_message(frame);
    }
}
```

## Web Serial API

Browser serial access (Chrome):

```javascript
// Request port
const port = await navigator.serial.requestPort();
await port.open({ baudRate: 115200 });

// Create CLASP transport
const transport = new SerialTransport(port);
const client = await new ClaspBuilder(transport);

await client.set('/sensor/value', 42);
```

## Message Format

Same CLASP binary format, wrapped in framing:

```
┌─────────────────────────────────────────┐
│ Frame Delimiter / Header                │
├─────────────────────────────────────────┤
│ CLASP Message (encoded)                 │
├─────────────────────────────────────────┤
│ Frame Delimiter / CRC                   │
└─────────────────────────────────────────┘
```

## Flow Control

### Hardware (RTS/CTS)

```rust
let config = SerialConfig {
    flow_control: FlowControl::Hardware,
    ..Default::default()
};
```

### Software (XON/XOFF)

```rust
let config = SerialConfig {
    flow_control: FlowControl::Software,
    ..Default::default()
};
```

## Platform Notes

### Linux

```bash
# List ports
ls /dev/ttyUSB* /dev/ttyACM*

# Permissions
sudo usermod -a -G dialout $USER
```

### macOS

```bash
# List ports
ls /dev/tty.usb*
ls /dev/cu.usb*
```

### Windows

Ports are `COM1`, `COM2`, etc.

## Troubleshooting

### No Communication

1. Check port name and permissions
2. Verify baud rate matches device
3. Check TX/RX wiring (may need swap)
4. Verify ground connection

### Garbled Data

1. Baud rate mismatch
2. Data bits/parity/stop bits wrong
3. Flow control mismatch
4. Noise on line (use shielded cable)

### Buffer Overflow

1. Enable flow control
2. Reduce message rate
3. Increase receive buffer

## See Also

- [Embedded Systems](../../use-cases/embedded-systems.md)
- [clasp-embedded](../api/rust/clasp-embedded.md)
- [BLE Transport](ble.md)
