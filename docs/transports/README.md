---
title: Transports
description: Reference for every transport CLASP supports
order: 1
---

# Transport Reference

This section covers each transport in detail -- configuration, code examples, performance characteristics, and troubleshooting. For the conceptual overview of how transports fit into CLASP, see [Core Concepts: Transports](../core/transports.md).

## Quick Recap

A **transport** is the wire (or radio wave) that carries CLASP binary frames between two endpoints. Every transport uses the exact same frame format. Swap the transport, nothing else changes.

The relay server speaks **WebSocket** (always) and optionally **QUIC**. Every other transport reaches the relay through an intermediate CLASP client process, or is used for direct peer-to-peer / embedded scenarios.

## At a Glance

| Transport | Protocol | Reliable | Ordered | Browser | Default Feature |
|-----------|----------|----------|---------|---------|-----------------|
| [WebSocket](websocket.md) | TCP | Yes | Yes | Yes | Yes |
| [QUIC](quic.md) | UDP | Yes | Per-stream | HTTP/3 only | Yes |
| [TCP](tcp.md) | TCP | Yes | Yes | No | Yes |
| [UDP](udp.md) | UDP | No | No | No | Yes |
| [Serial](serial.md) | UART | App-level | Yes | Web Serial | No |
| [BLE](ble.md) | Bluetooth | Configurable | Yes | Web Bluetooth | No |
| [WebRTC](webrtc.md) | UDP (DTLS) | Configurable | Configurable | Yes | No |

## How They Reach the Relay

```
DIRECT (built into relay):
  Browser ──[WebSocket]──> clasp-relay :7330
  Mobile  ──[QUIC]──────> clasp-relay :7331

VIA INTERMEDIATE (client process bridges two transports):
  ESP32 ──[BLE]──> Laptop ──[WebSocket]──> clasp-relay
  Arduino ──[Serial]──> RPi ──[WebSocket]──> clasp-relay

EMBED clasp-router (your own binary accepts connections):
  Server A ──[TCP]──> Your binary (embeds clasp-router)
  Sensors ──[UDP]──> Your binary (embeds clasp-router)

PEER-TO-PEER (no relay needed):
  Browser A ──[WebRTC DataChannel]──> Browser B
```

## The Shared Trait

Every transport in the `clasp-transport` crate implements the same trait interface. This is what makes them interchangeable:

```rust
pub trait TransportSender: Send + Sync {
    async fn send(&self, data: Bytes) -> Result<()>;
    fn try_send(&self, data: Bytes) -> Result<()>;
    fn is_connected(&self) -> bool;
    async fn close(&self) -> Result<()>;
}

pub trait TransportReceiver: Send {
    async fn recv(&mut self) -> Option<TransportEvent>;
}
```

`TransportEvent` is one of:
- `Data(Bytes)` -- a CLASP binary frame arrived
- `Connected` -- connection established
- `Disconnected { reason }` -- connection lost
- `Error(String)` -- transport-level error

Server-side transports (accepting incoming connections) also implement:

```rust
pub trait TransportServer: Send + Sync {
    async fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver, SocketAddr)>;
}
```

## Cargo Feature Flags

The `clasp-transport` crate gates each transport behind a feature flag:

```toml
# Default features (included automatically)
default = ["websocket", "tcp", "udp", "quic"]

# Optional features (must be explicitly enabled)
# serial, ble, webrtc

# Enable everything
full = ["websocket", "tcp", "udp", "quic", "serial", "ble", "webrtc"]
```

```bash
# Enable just serial and BLE
cargo build --features serial,ble

# Enable everything
cargo build --features full
```

Note: `wasm-websocket` is a separate feature for browser/WASM targets that uses the browser's native WebSocket API instead of `tokio-tungstenite`.

## Choosing a Transport

**Start with WebSocket.** It works everywhere -- browsers, Node.js, Python, Rust, behind proxies, through firewalls. It's the default for the relay and every SDK.

Switch to something else when you have a specific need:

- **QUIC** -- you need fast reconnects on mobile (connection migration, 0-RTT) or high-throughput with independent streams
- **TCP** -- you're embedding `clasp-router` in your own server binary for LAN use
- **UDP** -- you need sub-millisecond latency and can tolerate occasional message loss (sensors, real-time control)
- **Serial** -- you have a microcontroller or hardware device connected over USB/UART
- **BLE** -- you have battery-powered wireless devices (wearables, sensors, controllers)
- **WebRTC** -- you want direct browser-to-browser communication without routing through a server
