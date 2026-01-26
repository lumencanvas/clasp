# CLASP Crate Dependency Map

## Workspace Structure

```toml
[workspace]
resolver = "2"
members = [
    "crates/*",     # 10 library crates
    "tools/*",      # 3 binary crates
    "clasp-e2e",    # E2E test suite
]
```

## Dependency Graph

```
                    ┌─────────────────┐
                    │   clasp-core    │
                    │ (protocol types)│
                    └────────┬────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        ▼                    ▼                    ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│clasp-transport│   │clasp-discovery│   │clasp-embedded │
│ (WS,QUIC,TCP) │   │ (mDNS,HTTP)   │   │   (no_std)    │
└───────┬───────┘   └───────────────┘   └───────────────┘
        │
        ├──────────────────┐
        │                  │
        ▼                  ▼
┌───────────────┐   ┌───────────────┐
│ clasp-router  │   │ clasp-client  │
│(message hub)  │   │ (client API)  │
└───────┬───────┘   └───────┬───────┘
        │                   │
        ▼                   ▼
┌───────────────┐   ┌───────────────┐
│ clasp-bridge  │   │  clasp-wasm   │
│(OSC,MIDI,etc.)│   │(browser WASM) │
└───────────────┘   └───────────────┘

Binary Crates:
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  clasp-cli    │   │ clasp-router  │   │clasp-service  │
│  (commands)   │   │   (server)    │   │ (Electron IPC)│
└───────────────┘   └───────────────┘   └───────────────┘
```

## Crate Details

### clasp-core (lib)
**Purpose:** Protocol types, binary codec, frame format

**Dependencies:**
- `serde` + `serde_json` - Serialization
- `rmp-serde` - MessagePack (v2 compat)
- `thiserror` - Error handling
- `bytes` - Byte buffers
- `glob-match` - Address patterns
- `regex-lite` - Pattern validation

**Feature Flags:**
- `default = ["std"]`
- `std` - Standard library
- `alloc` - Alloc-only (no_std)

**Exports:**
```rust
pub mod address;   // Address, Pattern, glob_match
pub mod codec;     // encode, decode, Frame
pub mod error;     // Error, ErrorCode
pub mod frame;     // FrameFlags, Frame
pub mod p2p;       // P2PSignal, P2PConfig (std only)
pub mod security;  // Scope, TokenValidator (std only)
pub mod state;     // ParamState, StateStore
pub mod time;      // ClockSync, SessionTime, JitterBuffer
pub mod timeline;  // TimelinePlayer (std only)
pub mod types;     // Message, Value, SignalType, QoS
```

---

### clasp-transport (lib)
**Purpose:** Transport abstraction + 7 implementations

**Dependencies:**
- `clasp-core`
- `tokio` - Async runtime
- `tokio-tungstenite` - WebSocket
- `quinn` - QUIC
- `socket2` - TCP/UDP
- `tokio-serial` - Serial ports
- `btleplug` - Bluetooth LE
- `webrtc` - WebRTC
- `web-sys` - WASM WebSocket

**Feature Flags:**
```
default = ["websocket", "tcp", "udp", "quic"]
full = ["websocket", "tcp", "udp", "quic", "serial", "ble", "webrtc"]
wasm = ["wasm-websocket"]
```

**Exports:**
```rust
pub trait Transport;           // Connect abstraction
pub trait TransportSender;     // Send interface
pub trait TransportReceiver;   // Receive interface
pub trait TransportServer;     // Accept connections

pub mod websocket;   // WebSocketTransport, WebSocketServer
pub mod tcp;         // TcpTransport, TcpServer
pub mod udp;         // UdpTransport
pub mod quic;        // QuicTransport, QuicConnection
pub mod serial;      // SerialTransport
pub mod ble;         // BleTransport, BleDevice
pub mod webrtc;      // WebRtcTransport
pub mod wasm_websocket; // WasmWebSocketTransport (WASM only)
```

---

### clasp-router (lib)
**Purpose:** Message routing, sessions, subscriptions

**Dependencies:**
- `clasp-core`
- `clasp-transport`
- `dashmap` - Concurrent maps
- `tokio` - Async runtime
- `mqttbytes` - MQTT codec
- `rosc` - OSC codec

**Feature Flags:**
```
default = []
websocket = ["clasp-transport/websocket"]
quic = ["clasp-transport/quic"]
mqtt-server = ["mqttbytes"]
osc-server = ["rosc"]
```

**Exports:**
```rust
pub struct Router;              // Central message hub
pub struct RouterConfig;        // Configuration
pub struct Session;             // Client connection
pub struct SubscriptionManager; // Pattern matching
pub struct RouterState;         // Parameter storage
pub struct GestureRegistry;     // Move coalescing

pub mod adapters {
    pub mod mqtt_server;  // MqttServerAdapter
    pub mod osc_server;   // OscServerAdapter
}
```

---

### clasp-bridge (lib)
**Purpose:** Protocol bridges for OSC, MIDI, Art-Net, etc.

**Dependencies:**
- `clasp-core`
- `rosc` - OSC protocol
- `midir` - MIDI I/O
- `artnet_protocol` - Art-Net/DMX
- `sacn` - sACN/E1.31
- `rumqttc` - MQTT client
- `tokio-tungstenite` - WebSocket
- `rust_socketio` - Socket.IO
- `axum` - HTTP server
- `evalexpr` - Expression transforms
- `jsonpath_lib` - JSON path queries

**Feature Flags:**
```
default = ["osc", "midi", "artnet", "dmx", "mqtt", "websocket", "http"]
```

**Exports:**
```rust
pub trait Bridge;           // Bridge abstraction
pub enum BridgeEvent;       // ToClasp, Connected, etc.
pub struct BridgeConfig;    // Common config

pub mod osc;        // OscBridge
pub mod midi;       // MidiBridge
pub mod artnet;     // ArtNetBridge
pub mod dmx;        // DmxBridge
pub mod sacn;       // SacnBridge
pub mod mqtt;       // MqttBridge
pub mod websocket;  // WebSocketBridge
pub mod socketio;   // SocketIOBridge
pub mod http;       // HttpBridge

pub mod mapping;    // AddressMapping, ValueTransform
pub mod transform;  // Transform, CurveType, Condition
```

---

### clasp-client (lib)
**Purpose:** Client API with subscriptions and P2P

**Dependencies:**
- `clasp-core`
- `clasp-transport`
- `dashmap` - Concurrent cache
- `parking_lot` - Fast locks
- `tokio` - Async runtime

**Feature Flags:**
```
default = []
p2p = ["clasp-transport/webrtc"]
```

**Exports:**
```rust
pub struct Clasp;           // Main client
pub struct ClaspBuilder;    // Fluent config
pub struct P2PManager;      // WebRTC connections
pub enum P2PEvent;          // P2P events

pub type SubscriptionCallback = Box<dyn Fn(Value, &str) + Send + Sync>;
```

---

### clasp-discovery (lib)
**Purpose:** Device discovery via mDNS, broadcast, HTTP

**Dependencies:**
- `clasp-core`
- `clasp-transport`
- `mdns-sd` - mDNS/Bonjour
- `axum` - HTTP server
- `reqwest` - HTTP client
- `dashmap` - Concurrent storage

**Feature Flags:**
```
default = ["mdns", "broadcast"]
rendezvous = ["axum", "tower-http", "reqwest", "dashmap"]
```

**Exports:**
```rust
pub struct Device;           // Discovered device
pub struct DeviceInfo;       // Device metadata
pub struct Discovery;        // Discovery manager
pub enum DiscoveryEvent;     // Found, Lost, Error

pub mod mdns;       // mDNS discovery + advertisement
pub mod broadcast;  // UDP broadcast discovery
pub mod rendezvous; // HTTP rendezvous server
```

---

### clasp-wasm (lib, cdylib)
**Purpose:** WebAssembly bindings for browsers

**Dependencies:**
- `clasp-core`
- `wasm-bindgen` - JS interop
- `web-sys` - Browser APIs
- `js-sys` - JavaScript types

**Feature Flags:**
```
default = ["console_error_panic_hook"]
p2p = [] // WebRTC support
```

**Exports:**
```rust
#[wasm_bindgen]
pub struct ClaspWasm;        // Browser client
pub struct WasmP2PManager;   // P2P connections (optional)
```

---

### clasp-embedded (lib, no_std)
**Purpose:** Minimal implementation for embedded devices

**Dependencies:**
- None (no_std, no_alloc compatible)

**Feature Flags:**
```
default = []
server = [] // MiniRouter support
```

**Exports:**
```rust
pub enum Value;          // Null, Bool, Int, Float
pub struct Client;       // Embedded client
pub struct StateCache;   // Fixed 32-entry cache

#[cfg(feature = "server")]
pub struct MiniRouter;   // Embedded server (4 clients)
```

---

### clasp-cli (bin)
**Purpose:** Command-line interface

**Dependencies:**
- `clasp-core`
- `clasp-router`
- `clasp-bridge`
- `clap` - CLI parsing
- `tokio` - Runtime
- `rcgen` - Cert generation

**Commands:**
- `server` - Start router
- `bridge` - Start bridge
- `osc`, `mqtt`, `websocket`, `http` - Protocol shortcuts
- `pub` - Publish to address
- `sub` - Subscribe to pattern
- `token` - Token management

---

### clasp-test-utils (lib)
**Purpose:** Test helpers and utilities

**Dependencies:**
- `clasp-core`
- `clasp-client`
- `clasp-router`
- `tokio` - Async runtime
- `parking_lot` - Synchronization

**Exports:**
```rust
pub struct TestRouter;       // RAII test server
pub struct ValueCollector;   // Capture subscriptions
pub fn find_available_port() -> u16;
pub fn wait_for_count(counter, target, timeout);
```

## External Dependencies Summary

### Core Runtime
- `tokio` (1.35) - Async runtime
- `futures` (0.3) - Async utilities

### Serialization
- `serde` (1.0) - Framework
- `serde_json` (1.0) - JSON
- `rmp-serde` (1.1) - MessagePack
- `bytes` (1.5) - Buffers

### Networking
- `tokio-tungstenite` (0.21) - WebSocket
- `quinn` (0.11) - QUIC
- `socket2` (0.5) - TCP/UDP
- `mdns-sd` (0.10) - mDNS

### Protocol Libraries
- `rosc` (0.10) - OSC
- `midir` (0.9) - MIDI
- `artnet_protocol` (0.2) - Art-Net
- `rumqttc` (0.24) - MQTT
- `axum` (0.7) - HTTP server

### Concurrency
- `dashmap` (5.5) - Concurrent maps
- `parking_lot` (0.12) - Fast locks

### Error Handling
- `thiserror` (1.0) - Error derive
- `anyhow` (1.0) - Generic errors

### Testing
- `criterion` (0.5) - Benchmarks
- `tokio-test` (0.4) - Async testing
