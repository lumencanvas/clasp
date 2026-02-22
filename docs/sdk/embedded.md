---
title: Embedded SDK
description: Run CLASP on ESP32, RP2040, and other no_std targets
order: 4
---

# Embedded SDK

The `clasp-embedded` crate provides a `no_std` CLASP client for microcontrollers. It fits within a ~3KB memory budget, encodes and decodes CLASP binary frames directly, and includes a `MiniRouter` for on-device signal routing between local peripherals.

## Installation

Add the crate with default features disabled:

```toml
[dependencies]
clasp-embedded = { version = "3.5", default-features = false }
```

Or via the command line:

```bash
cargo add clasp-embedded --no-default-features
```

## Supported Targets

The crate works on any `no_std` target with a WebSocket transport layer. Tested platforms:

| Target | Chip | Notes |
|---|---|---|
| ESP32 | Xtensa LX6 / RISC-V | Via `esp-wifi` or `embassy-net` |
| RP2040 | ARM Cortex-M0+ | Via `embassy-rp` or `cyw43` WiFi |
| STM32 | ARM Cortex-M4/M7 | Via `embassy-stm32` with Ethernet or WiFi |

Any platform that provides a byte-level socket can use `clasp-embedded` by writing frames to the transport manually.

## Basic Usage

Create a client, prepare a frame, and send it over your WebSocket connection:

```rust
#![no_std]

use clasp_embedded::{Client, ValueType};

// Create a CLASP client
let mut client = Client::new();

// Prepare a SET frame for an address
let mut buf = [0u8; 128];
let len = client.prepare_set(
    &mut buf,
    "/sensors/temperature",
    ValueType::F32,
    &25.5f32.to_be_bytes(),
);

// Send buf[..len] over your WebSocket connection
websocket.send(&buf[..len]).unwrap();
```

### Reading Incoming Frames

Decode incoming data from the router:

```rust
use clasp_embedded::decode_header;

let data = websocket.recv(&mut buf).unwrap();

if let Some((flags, payload_len)) = decode_header(&buf[..data]) {
    let payload = &buf[HEADER_SIZE..HEADER_SIZE + payload_len];
    // Process the payload based on flags/message type
}
```

## Frame Format

All CLASP frames share a compact binary header:

```
Byte 0: Magic (0x53 = 'S')
Byte 1: Version (0x01)
Byte 2: Flags / Message Type
Byte 3: Payload Length (low byte)
Bytes 4+: Payload
```

| Field | Size | Value |
|---|---|---|
| Magic | 1 byte | `0x53` |
| Version | 1 byte | `0x01` |
| Flags | 1 byte | Message type code |
| Payload length | 1 byte | 0--255 (extended framing for larger payloads) |

Header size is 4 bytes. Maximum payload is 1024 bytes by default (configurable).

### Message Type Codes

| Code | Name | Description |
|---|---|---|
| `0x01` | HELLO | Connection handshake |
| `0x02` | WELCOME | Handshake response |
| `0x21` | SET | Set state at an address |
| `0x22` | GET | Request state at an address |
| `0x23` | VALUE | Response to GET |
| `0x31` | SUBSCRIBE | Subscribe to a pattern |
| `0x32` | UNSUBSCRIBE | Remove a subscription |
| `0x41` | EMIT | Fire-and-forget event |
| `0x42` | STREAM | High-rate continuous data |
| `0x43` | GESTURE | Phased interaction signal |
| `0x44` | TIMELINE | Keyframe animation |
| `0x50` | BUNDLE | Atomic message group |

## Value Types

Values are tagged with a single-byte type code:

| Code | Type | Size | Rust Type |
|---|---|---|---|
| `0x00` | Null | 0 bytes | `()` |
| `0x01` | Bool | 1 byte | `bool` |
| `0x04` | I32 | 4 bytes | `i32` |
| `0x05` | I64 | 8 bytes | `i64` |
| `0x06` | F32 | 4 bytes | `f32` |
| `0x07` | F64 | 8 bytes | `f64` |
| `0x08` | String | variable | `&str` |
| `0x09` | Bytes | variable | `&[u8]` |
| `0x0A` | Array | variable | nested values |
| `0x0B` | Map | variable | key-value pairs |

Integer and float values are big-endian encoded. Strings and bytes are length-prefixed.

## MiniRouter

`MiniRouter` provides on-device signal routing for sensor nodes that need to process signals locally without a network connection. Use it to route between peripherals on the same microcontroller.

```rust
use clasp_embedded::MiniRouter;

let mut router = MiniRouter::new();

// Register a local handler
router.on("/sensors/*", |address, value| {
    // React to local sensor changes
    if address == "/sensors/temperature" {
        adjust_fan(value);
    }
});

// Route a local set operation -- fires matching handlers
router.set("/sensors/temperature", ValueType::F32, &25.5f32.to_be_bytes());
```

`MiniRouter` operates entirely in memory with no heap allocation. It is useful for sensor fusion, local control loops, or pre-filtering data before sending it to a remote router.

## Memory Budget

The crate is designed for constrained environments:

| Component | RAM Usage |
|---|---|
| `Client` struct | ~256 bytes |
| Frame buffer (default) | 1028 bytes (header + payload) |
| `MiniRouter` (4 subscriptions) | ~512 bytes |
| **Total minimum** | **~1.8 KB** |

### Configuration

Adjust the maximum payload size at compile time:

```rust
use clasp_embedded::Client;

// Default: MAX_PAYLOAD = 1024
// For tighter memory constraints:
let mut buf = [0u8; 64]; // smaller frame buffer
let len = client.prepare_set(&mut buf, "/s/t", ValueType::F32, &val);
```

The frame buffer size is controlled by the caller. Use smaller buffers for shorter addresses and smaller payloads. The only hard constraint is that the buffer must fit the 4-byte header plus the encoded address, type tag, and value.

## Full Example: ESP32 Temperature Sensor

```rust
#![no_std]
#![no_main]

use clasp_embedded::{Client, ValueType, HEADER_SIZE};

#[entry]
fn main() -> ! {
    // Hardware setup (platform-specific)
    let mut wifi = init_wifi();
    let mut ws = wifi.connect_ws("ws://192.168.1.100:7330").unwrap();

    let mut client = Client::new();
    let mut buf = [0u8; 128];

    // Send HELLO
    let len = client.prepare_hello(&mut buf, "ESP32 Sensor");
    ws.send(&buf[..len]).unwrap();

    // Wait for WELCOME
    let n = ws.recv(&mut buf).unwrap();
    // Parse welcome, extract session ID, etc.

    loop {
        let temperature = read_temperature_sensor();

        let len = client.prepare_set(
            &mut buf,
            "/sensors/temperature",
            ValueType::F32,
            &temperature.to_be_bytes(),
        );
        ws.send(&buf[..len]).unwrap();

        delay_ms(1000);
    }
}
```

## Next Steps

- [Rust SDK](rust.md) -- full-featured async client for `std` environments
- [Core Concepts](../concepts/architecture.md) -- understand signals, state, and the router model
- [Protocol Bridges](../protocols/README.md) -- connect CLASP to OSC, MIDI, MQTT, and more
- [Deployment](../deployment/relay.md) -- run CLASP in production
