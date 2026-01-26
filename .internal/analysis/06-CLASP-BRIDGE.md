# CLASP-BRIDGE Crate Analysis

## Overview

**Location:** `crates/clasp-bridge`
**Purpose:** Protocol bridges for OSC, MIDI, Art-Net, DMX, sACN, MQTT, WebSocket, Socket.IO, HTTP

## Feature Flags

```toml
default = ["osc", "midi", "artnet", "dmx", "mqtt", "websocket", "http"]
```

---

## Core Traits (traits.rs)

### Bridge Trait

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

```rust
pub enum BridgeEvent {
    ToClasp(Message),
    Connected,
    Disconnected { reason: Option<String> },
    Error(String),
}
```

### BridgeConfig

```rust
pub struct BridgeConfig {
    pub name: String,
    pub protocol: String,
    pub bidirectional: bool,
    pub options: HashMap<String, String>,
}
```

### BridgeError

```rust
pub enum BridgeError {
    ConnectionFailed(String),
    Protocol(String),
    Mapping(String),
    Send(String),
    Receive(String),
    DeviceNotFound(String),
    Io(std::io::Error),
    Other(String),
}
```

---

## OSC Bridge (osc.rs)

### Configuration

```rust
pub struct OscBridgeConfig {
    pub bind_addr: String,            // "0.0.0.0:8000"
    pub remote_addr: Option<String>,  // Optional output address
    pub namespace: String,            // "/osc"
}
```

### Value Conversion

```rust
// OSC → CLASP
OscType::Int(i)     → Value::Int(i as i64)
OscType::Float(f)   → Value::Float(f as f64)
OscType::String(s)  → Value::String(s)
OscType::Blob(b)    → Value::Bytes(b)
OscType::Long(l)    → Value::Int(l)
OscType::Double(d)  → Value::Float(d)
OscType::Bool(b)    → Value::Bool(b)
OscType::Nil        → Value::Null
OscType::Inf        → Value::Float(f64::INFINITY)

// Multiple args → Value::Array
// Single arg → unwrapped Value
// Empty args → Value::Null

// CLASP → OSC
Value::Null      → (empty vec)
Value::Bool(b)   → OscType::Bool(b)
Value::Int(i)    → OscType::Long(i)
Value::Float(f)  → OscType::Double(f)
Value::String(s) → OscType::String(s)
Value::Bytes(b)  → OscType::Blob(b)
Value::Array     → flat mapped
Value::Map       → JSON string
```

---

## MIDI Bridge (midi.rs)

### Configuration

```rust
pub struct MidiBridgeConfig {
    pub input_port: Option<String>,
    pub output_port: Option<String>,
    pub namespace: String,            // "/midi"
    pub device_name: String,          // "default"
}
```

### Address Format

```
/midi/{device}/ch/{channel}/cc/{num}   - Control Change
/midi/{device}/ch/{channel}/note       - Note On/Off
/midi/{device}/ch/{channel}/bend       - Pitch Bend
/midi/{device}/clock                   - MIDI Clock
/midi/{device}/transport               - Start/Stop/Continue
```

### Message Parsing

```rust
// Status bytes
0x80/0x90 (Note Off/On) → Publish("note") with {note, velocity, on}
0xB0 (Control Change)   → Set(f"/cc/{cc}") with value
0xC0 (Program Change)   → Publish("program")
0xE0 (Pitch Bend)       → Set(f"/bend") with 14-bit value
0xF8 (Clock)            → Publish("clock")
0xFA/0xFB/0xFC          → Publish("transport") with {start|continue|stop}
```

---

## Art-Net Bridge (artnet.rs)

### Configuration

```rust
pub struct ArtNetBridgeConfig {
    pub bind_addr: String,            // "0.0.0.0:6454"
    pub remote_addr: Option<String>,
    pub universes: Vec<u16>,          // Filter (empty = all)
    pub namespace: String,            // "/artnet"
}
```

### Address Format

```
/artnet/{universe}/{channel}
Channel: 1-512 (1-indexed)
Value: 0-255 (u8)
```

### Delta Detection

- Tracks DMX state per universe: `HashMap<u16, [u8; 512]>`
- Only sends changed channels to CLASP
- Supports poll() for node discovery

---

## sACN Bridge (sacn.rs)

### Configuration

```rust
pub enum SacnMode { Sender, Receiver, Bidirectional }

pub struct SacnBridgeConfig {
    pub mode: SacnMode,                   // Receiver
    pub universes: Vec<u16>,              // [1]
    pub source_name: String,              // "CLASP sACN Bridge"
    pub priority: u8,                     // 100 (0-200)
    pub bind_address: Option<String>,
    pub multicast: bool,                  // true
    pub unicast_destinations: Vec<String>,
    pub namespace: String,                // "/sacn"
    pub preview: bool,
    pub sync_address: u16,                // 0 = no sync
}
```

### Address Format

```
/sacn/{universe}/{channel}
Channel: 1-512
```

### E1.31 Implementation

- Receiver: Subscribes to universes, sends changes
- Sender: Registers universes, transmits at 44Hz (23ms)
- Priority-based conflict resolution

---

## DMX Bridge (dmx.rs)

### Configuration

```rust
pub enum DmxInterfaceType {
    EnttecPro,
    EnttecOpen,
    Ftdi,
    Virtual,    // Default (logging only)
}

pub struct DmxBridgeConfig {
    pub port: Option<String>,
    pub interface_type: DmxInterfaceType,
    pub universe: u16,                    // 0
    pub namespace: String,                // "/dmx"
    pub refresh_rate: f64,                // 44.0 Hz
}
```

### Address Format

```
/dmx/{universe}/{channel}
```

### ENTTEC Pro Protocol

```
Frame: [0x7E][0x06][len_lo][len_hi][0x00][512 DMX bytes][0xE7]
```

---

## MQTT Bridge (mqtt.rs)

### Configuration

```rust
pub struct MqttBridgeConfig {
    pub broker_host: String,              // "localhost"
    pub broker_port: u16,                 // 1883
    pub client_id: String,                // UUID-based
    pub username: Option<String>,
    pub password: Option<String>,
    pub subscribe_topics: Vec<String>,    // ["#"]
    pub qos: u8,                          // 0 (0, 1, or 2)
    pub keep_alive_secs: u16,             // 60
    pub namespace: String,                // "/mqtt"
}
```

### QoS Translation

```rust
0 → MqttQoS::AtMostOnce
1 → MqttQoS::AtLeastOnce
2 → MqttQoS::ExactlyOnce
```

### Payload Parsing Order

1. Try parse as JSON → convert to Value
2. Try parse as f64 → Value::Float
3. Check for "true"/"false" → Value::Bool
4. Return as string → Value::String
5. Return as bytes → Value::Bytes

### Topic Mapping

```
MQTT: sensors/temp
→ CLASP: /mqtt/sensors/temp
```

---

## WebSocket Bridge (websocket.rs)

### Configuration

```rust
pub enum WsMessageFormat { Json, MsgPack, Raw }
pub enum WsMode { Client, Server }

pub struct WebSocketBridgeConfig {
    pub mode: WsMode,                     // Client
    pub url: String,
    pub path: Option<String>,
    pub format: WsMessageFormat,          // Json
    pub ping_interval_secs: u32,          // 30
    pub auto_reconnect: bool,             // true
    pub reconnect_delay_secs: u32,        // 5
    pub headers: HashMap<String, String>,
    pub namespace: String,                // "/ws"
}
```

### JSON Format

```json
{
    "address": "/signal/path",
    "value": <any>
}
```

### Server Mode

- Multi-client support (tracked with atomic ID)
- Broadcast to all connected clients
- Per-client send channel

---

## Socket.IO Bridge (socketio.rs)

### Configuration

```rust
pub struct SocketIOBridgeConfig {
    pub url: String,                      // "http://localhost:3000"
    pub sio_namespace: String,            // "/"
    pub events: Vec<String>,              // ["message"]
    pub auth: Option<serde_json::Value>,
    pub reconnect: bool,                  // true
    pub namespace: String,                // "/socketio"
}
```

### Address Format

```
/socketio/{event_name}
```

### Payload Conversion

```rust
Payload::Text(values) → Value::Array or single element
Payload::Binary(data) → Value::Bytes
```

---

## HTTP Bridge (http.rs)

### Configuration

```rust
pub enum HttpMethod { GET, POST, PUT, DELETE, PATCH }
pub enum HttpMode { Server, Client }

pub struct EndpointConfig {
    pub path: String,                     // "/api/lights/:id"
    pub method: HttpMethod,
    pub clasp_address: String,
    pub description: Option<String>,
    pub enabled: bool,                    // true
    pub required_scope: Option<String>,
    pub rate_limit: u32,
}

pub struct HttpBridgeConfig {
    pub mode: HttpMode,                   // Server
    pub url: String,
    pub endpoints: Vec<EndpointConfig>,
    pub cors_enabled: bool,               // true
    pub cors_origins: Vec<String>,
    pub base_path: String,                // "/api"
    pub timeout_secs: u32,                // 30
    pub namespace: String,                // "/http"
    pub poll_interval_ms: u64,            // Client polling
    pub poll_endpoints: Vec<String>,
}
```

### Default Endpoints

```
GET  /api/signals               → List all signals
GET  /api/signals/*path         → Get signal value
PUT  /api/signals/*path         → Set signal value
POST /api/signals/*path         → Publish event
```

### Server Built with Axum

- CORS support (configurable)
- Signal caching via `Arc<RwLock<HashMap>>`
- Health check endpoint

---

## Mapping Module (mapping.rs)

### AddressMapping

```rust
pub struct AddressMapping {
    pub from: String,                     // Protocol address pattern
    pub to: String,                       // CLASP address pattern
    pub transform: Option<ValueTransform>,
}
```

### Pattern Matching

- Supports `*` wildcard in both `from` and `to`
- Extracts captured segments
- Replaces `*` in `to` with captured values

### ValueTransform

```rust
pub enum ValueTransform {
    Identity,
    Scale { from_min, from_max, to_min, to_max },
    Clamp { min, max },
    Invert,
    ToInt,
    ToFloat,
    Expression(String),  // evalexpr engine
}
```

---

## Transform Module (transform.rs)

### Transform Types

```rust
pub enum Transform {
    // Basic
    Identity,
    Expression { expr: String },

    // Numeric
    Scale { from_min, from_max, to_min, to_max },
    Clamp { min, max },
    Invert,
    ToInt,
    ToFloat,

    // Lookups & Curves
    Lookup { table: HashMap, default: Option },
    Curve { curve_type: CurveType },
    Quantize { steps: u32 },

    // Smoothing & Control
    DeadZone { threshold: f64 },
    Smooth { factor: f64 },
    RateLimit { max_delta: f64 },
    Threshold { value: f64, mode: ThresholdMode },

    // Operations
    Modulo { divisor: f64 },
    Abs,
    Negate,
    Power { exponent: f64 },
    Log { base: Option<f64> },
    Round { decimals: u32 },

    // Advanced
    Chain { transforms: Vec<Transform> },
    Conditional { condition, if_true, if_false },
    JsonPath { path: String },
    MapType { from_type, to_type },
    Bitwise { operation, operand },
}
```

### CurveType

```rust
pub enum CurveType {
    Linear,
    EaseIn, EaseOut, EaseInOut,
    QuadIn, QuadOut, QuadInOut,
    CubicIn, CubicOut, CubicInOut,
    ExpoIn, ExpoOut, ExpoInOut,
    SineIn, SineOut, SineInOut,
    CircIn, CircOut, CircInOut,
    ElasticIn, ElasticOut,
    BounceOut,
    Bezier { x1, y1, x2, y2 },
}
```

### Condition

```rust
pub enum Condition {
    GreaterThan { value: f64 },
    LessThan { value: f64 },
    Equals { value: f64, tolerance: Option<f64> },
    InRange { min: f64, max: f64 },
    Expression { expr: String },
    And { conditions: Vec<Condition> },
    Or { conditions: Vec<Condition> },
    Not { condition: Box<Condition> },
}
```

### Aggregator

```rust
pub enum Aggregator {
    Average,
    Sum,
    Min,
    Max,
    Latest,
    First,
    Count,
    MovingAverage { window_size: usize },
    RateOfChange,
    StdDev,
}
```

---

## Summary

| Bridge | Protocol | Bidirectional | Key Feature |
|--------|----------|---------------|-------------|
| OSC | UDP | ✓ | Type conversion, array handling |
| MIDI | Native | ✓ | CC, notes, transport, clock |
| Art-Net | UDP | ✓ | Universe filtering, delta detect |
| sACN | UDP | ✓ | E1.31, multicast, 44Hz |
| DMX | Serial | - | ENTTEC Pro, refresh rate |
| MQTT | TCP | ✓ | QoS, topic patterns |
| WebSocket | TCP | ✓ | JSON/MsgPack, multi-client |
| Socket.IO | TCP | ✓ | Event-based, auth |
| HTTP | TCP | ✓ | REST, polling, CORS |

## Architectural Patterns

1. **Async/Await**: All bridges use `#[async_trait]`
2. **Thread-safe State**: `Arc<Mutex<T>>` for shared state
3. **Channel Communication**: `tokio::sync::mpsc` for events
4. **Namespace Isolation**: Each bridge in configurable namespace
5. **Delta Detection**: Art-Net, sACN, HTTP polling
6. **Value Conversion**: Unified to/from CLASP Value
7. **Feature Gating**: Compile-time optional protocols
