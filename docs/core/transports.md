---
title: Transports
description: How CLASP frames move between devices -- the pipe, not the payload
order: 5
---

# Transports

A transport is the pipe that carries CLASP binary frames between two endpoints. It has nothing to do with *what* you're sending (signals, state, events) -- only *how* the bytes get from A to B.

Think of it like shipping a package. The package contents (your CLASP signals) are always the same regardless of whether the delivery truck drives, flies, or takes a boat. The transport is the delivery method.

## Transports vs Bridges

This is the most important distinction to understand:

**A transport** carries native CLASP frames. Both sides speak CLASP. No translation happens. WebSocket, QUIC, BLE, Serial -- these are all transports.

**A bridge** translates between CLASP and a foreign protocol. One side speaks OSC or MIDI or MQTT, the other side speaks CLASP. The bridge sits in the middle and converts. See [Protocol Bridges](../protocols/README.md).

```
TRANSPORT (no translation):
  CLASP Client ──[CLASP frames over WebSocket]──> CLASP Router

BRIDGE (translation):
  OSC Device ──[OSC messages]──> Bridge ──[CLASP frames]──> CLASP Router
```

## Where Transports Run

This is the second key thing to understand: **transports are a client-side choice, not a server configuration**.

The relay server listens on two transports: **WebSocket** (always, port 7330) and optionally **QUIC** (with `--quic-port`). That's it. The relay does not listen on BLE, Serial, UDP, or raw TCP.

So how do a BLE sensor or a serial-connected Arduino participate? Through an intermediate process:

```
                                              The relay only
                                              speaks WS + QUIC
                                                    |
ESP32 ──[BLE]──> Laptop running    ──[WebSocket]──> clasp-relay
                 CLASP client                       :7330
                 with BLE transport

Arduino ──[Serial/USB]──> Raspberry Pi ──[WebSocket]──> clasp-relay
                          running                       :7330
                          CLASP client
```

The laptop scans for BLE devices, connects to the ESP32 over BLE, receives CLASP frames, and forwards them to the relay over WebSocket. The laptop is acting as a transparent bridge between two transports. This is different from a protocol bridge -- no translation happens. The ESP32 is sending real CLASP binary frames over BLE; the laptop just shuttles them to the relay.

Similarly, an Arduino sends CLASP frames over USB serial to a Raspberry Pi, which forwards them to the relay over WebSocket.

The ESP32 and Arduino don't know or care that the relay uses WebSocket. They just send CLASP frames over whatever transport they support. The intermediate machine handles the hop.

## Available Transports

| Transport | Best For | Relay Support | Feature Flag |
|-----------|----------|---------------|--------------|
| [WebSocket](../transports/websocket.md) | Web apps, general use | Direct (default) | `websocket` (default) |
| [QUIC](../transports/quic.md) | Mobile, high-perf native | Direct (`--quic-port`) | `quic` (default) |
| [TCP](../transports/tcp.md) | Server-to-server, LAN | Embed `clasp-router` | `tcp` (default) |
| [UDP](../transports/udp.md) | Discovery, fire-and-forget | Embed `clasp-router` | `udp` (default) |
| [Serial](../transports/serial.md) | Hardware, microcontrollers | Via intermediate | `serial` |
| [BLE](../transports/ble.md) | Wireless controllers, battery | Via intermediate | `ble` |
| [WebRTC](../transports/webrtc.md) | P2P, browser-to-browser | Not needed (P2P) | `webrtc` |

**"Direct"** means the relay binary has built-in support. **"Via intermediate"** means a CLASP client process on a host machine bridges between the transport and the relay. **"Embed `clasp-router`"** means you use the `clasp-router` Rust crate directly in your own binary to accept TCP/UDP connections.

Default Cargo features include `websocket`, `tcp`, `udp`, and `quic`. Serial, BLE, and WebRTC require explicit feature flags:

```bash
cargo build --features serial,ble
# or enable everything:
cargo build --features full
```

## How a Transport Works Internally

Every transport implements the same Rust trait. This is what makes CLASP transport-agnostic -- swap the pipe, keep the protocol:

```rust
// Every transport produces a sender and receiver
let (sender, receiver) = SomeTransport::connect("address").await?;

// Send CLASP binary frames
sender.send(clasp_frame_bytes).await?;

// Receive CLASP binary frames
if let Some(TransportEvent::Data(bytes)) = receiver.recv().await {
    // These bytes are the same CLASP binary format
    // regardless of which transport delivered them
}
```

The full trait hierarchy lives in `clasp-transport::traits`:

```rust
pub trait TransportSender: Send + Sync {
    async fn send(&self, data: Bytes) -> Result<()>;
    fn try_send(&self, data: Bytes) -> Result<()>;  // non-blocking
    fn is_connected(&self) -> bool;
    async fn close(&self) -> Result<()>;
}

pub trait TransportReceiver: Send {
    async fn recv(&mut self) -> Option<TransportEvent>;
}
```

`TransportEvent` is one of: `Data(Bytes)`, `Connected`, `Disconnected { reason }`, or `Error(String)`.

Servers (things that accept incoming connections) implement `TransportServer`:

```rust
pub trait TransportServer: Send + Sync {
    async fn accept(&mut self) -> Result<(Self::Sender, Self::Receiver, SocketAddr)>;
}
```

## Mixed-Transport Systems

A single CLASP deployment often uses multiple transports at once. All clients see the same shared state regardless of how they connect:

```
Browser (WSS) ───────────────────┐
                                 │
Mobile App (QUIC) ──────────────┼── clasp-relay ── State Store
                                 │
Laptop (WS, bridging for:) ─────┘
  |-- ESP32 via BLE
  |-- Arduino via Serial
  |-- Sensor hub via UDP
```

Every client in this diagram can `set()`, `subscribe()`, and `emit()` on the same address space. The browser can read sensor data from the Arduino. The mobile app can control lights connected to the ESP32. The transport is invisible to the application layer.

## Choosing a Transport

| Need | Use |
|------|-----|
| Broadest compatibility, works everywhere | WebSocket |
| Fast reconnects on mobile, network switching | QUIC |
| Lowest possible latency on LAN | UDP |
| Direct microcontroller connection | Serial |
| Wireless battery-powered device | BLE |
| Browser-to-browser without a server | WebRTC |
| Server-to-server on trusted LAN | TCP |

When in doubt, use WebSocket. It works in browsers, behind proxies, through firewalls, and is the default for both the relay and all SDKs.

## Next Steps

- [Transport reference pages](../transports/) -- detailed docs for each transport
- [Protocol Bridges](../protocols/README.md) -- for connecting non-CLASP devices (OSC, MIDI, etc.)
- [P2P & WebRTC](p2p.md) -- direct peer-to-peer connections bypassing the relay
- [Signals](signals.md) -- the signal types carried by transports
