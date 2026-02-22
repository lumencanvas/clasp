---
title: "Protocol Connections"
description: "CLASP supports connecting multiple protocols to the central router. Each protocol connection translates bidirectionally between its native protocol and CLASP."
section: protocols
order: 0
---
# Protocol Connections

CLASP supports connecting multiple protocols to the central router. Each protocol connection translates bidirectionally between its native protocol and CLASP.

## Supported Protocols

| Protocol | Direction | Transport | Use Cases |
|----------|-----------|-----------|-----------|
| [OSC](osc.md) | Bidirectional | UDP | Audio software, VJ apps, TouchOSC |
| [MQTT](mqtt.md) | Bidirectional | TCP/TLS | IoT devices, home automation |
| MIDI | Bidirectional | USB/Virtual | DAWs, controllers, synthesizers |
| Art-Net | Bidirectional | UDP | DMX lighting over Ethernet |
| DMX | Output | Serial | Direct DMX via USB interfaces |
| HTTP | Bidirectional | TCP | REST APIs, webhooks |

For detailed configuration of each protocol, see the [Bridge Reference](../reference/bridges/osc.md).

## Protocol Connection Architecture

Each protocol connection implements the `Bridge` trait:

```rust
#[async_trait]
pub trait Bridge: Send + Sync {
    fn config(&self) -> &BridgeConfig;
    async fn start(&mut self) -> Result<mpsc::Receiver<BridgeEvent>>;
    async fn stop(&mut self) -> Result<()>;
    async fn send(&self, message: Message) -> Result<()>;
    fn is_running(&self) -> bool;
    fn namespace(&self) -> &str;
}
```

### BridgeEvent

Protocol connections emit events through a channel:

```rust
pub enum BridgeEvent {
    ToClasp(Message),  // Message received from external protocol
    Connected,              // Connection established successfully
    Disconnected { reason: Option<String> },
    Error(String),
}
```

## Common Configuration

All protocol connections share some common configuration:

```rust
pub struct BridgeConfig {
    pub name: String,        // Human-readable name
    pub protocol: String,    // Protocol identifier
    pub bidirectional: bool, // Whether bridge can send and receive
    // ...
}
```

## Namespace Mapping

Each protocol connection has a namespace that prefixes all addresses:

| Protocol | Default Namespace | Example Address |
|----------|-------------------|-----------------|
| OSC | `/osc` | `/osc/1/fader1` |
| MIDI | `/midi` | `/midi/ch1/note/60` |
| MQTT | `/mqtt` | `/mqtt/sensors/temp` |
| HTTP | `/http` | `/http/api/status` |

## Next Steps

- [OSC Connection](osc.md) - Open Sound Control
- [MQTT Connection](mqtt.md) - IoT messaging
- [How-To: Add MIDI](../how-to/connections/add-midi.md) - MIDI setup
- [How-To: Add Art-Net](../how-to/connections/add-artnet.md) - DMX over Ethernet
- [How-To: Add HTTP](../how-to/connections/add-http.md) - REST API bridge
- [Bridge Reference](../reference/bridges/osc.md) - Complete bridge documentation
