# CLASP Architecture Overview

## Core Philosophy

CLASP (Creative Low-Latency Application Streaming Protocol) is designed for real-time creative applications requiring:
- Sub-millisecond latency for local networks
- Reliable state synchronization across clients
- Protocol bridging between creative tools
- Flexible transport options (WebSocket to BLE)

## System Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         APPLICATIONS                             │
├───────────────────┬───────────────────┬─────────────────────────┤
│   CLASP Bridge    │    CLASP Site     │    User Applications    │
│   (Electron)      │    (Vue.js)       │    (Custom)             │
└───────────────────┴───────────────────┴─────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      LANGUAGE BINDINGS                           │
├───────────────────┬───────────────────┬─────────────────────────┤
│   JavaScript      │     Python        │       Rust              │
│   (@clasp-to/core)│   (clasp-to)      │   (clasp-client)        │
└───────────────────┴───────────────────┴─────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                        CLASP CLIENT                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐│
│  │ Subscription│ │  Parameter  │ │      P2P Manager            ││
│  │  Manager    │ │   Cache     │ │  (WebRTC Data Channels)     ││
│  └─────────────┘ └─────────────┘ └─────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                       CLASP TRANSPORT                            │
├─────────┬─────────┬────────┬────────┬────────┬────────┬────────┤
│WebSocket│  QUIC   │  TCP   │  UDP   │ Serial │  BLE   │ WebRTC │
└─────────┴─────────┴────────┴────────┴────────┴────────┴────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                        CLASP ROUTER                              │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐│
│  │  Session    │ │Subscription │ │    Gesture                  ││
│  │  Manager    │ │   Index     │ │    Coalescing               ││
│  └─────────────┘ └─────────────┘ └─────────────────────────────┘│
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────────────────────┐│
│  │  State      │ │    P2P      │ │   Protocol Adapters         ││
│  │  Store      │ │ Capabilities│ │   (MQTT, OSC servers)       ││
│  └─────────────┘ └─────────────┘ └─────────────────────────────┘│
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                       CLASP BRIDGE                               │
├─────────┬─────────┬────────┬────────┬────────┬────────┬────────┤
│   OSC   │  MIDI   │Art-Net │  sACN  │  MQTT  │  HTTP  │Socket.IO│
└─────────┴─────────┴────────┴────────┴────────┴────────┴────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                    EXTERNAL PROTOCOLS                            │
│   TouchOSC, VDMX, Resolume, Ableton, lighting consoles, etc.    │
└─────────────────────────────────────────────────────────────────┘
```

## Crate Dependencies

```
clasp-core ─────────────────────────────────────────────────┐
     │                                                       │
     ├──► clasp-transport ◄─────────────────────────────────┤
     │          │                                            │
     │          ├──► clasp-router ◄──────────────────────────┤
     │          │         │                                  │
     │          │         └──► clasp-bridge                  │
     │          │                                            │
     │          └──► clasp-client                            │
     │                    │                                  │
     │                    └──► clasp-wasm                    │
     │                                                       │
     ├──► clasp-discovery                                    │
     │                                                       │
     └──► clasp-embedded (no_std)                            │
```

## Message Flow

### Publishing a Parameter
```
1. Client calls set("/lights/room/1", 0.75)
2. ClaspClient encodes SetMessage with revision check
3. Transport sends binary frame over WebSocket
4. Router validates, applies to StateStore
5. Router increments revision
6. Router broadcasts to matching subscribers
7. Subscribers receive SET with new value
8. Client receives ACK with new revision
```

### Gesture Input (Touch)
```
1. Touch begins → PUBLISH with phase: Start
2. Touch moves → PUBLISH with phase: Move (buffered)
3. Router coalesces MOVE events at 60fps
4. Touch ends → Flush buffer + PUBLISH with phase: End
5. Subscribers receive Start, coalesced Moves, End
```

### P2P Connection
```
1. Client A announces P2P capability
2. Client B wants to connect to A
3. B sends P2PSignal::Offer to A via Router
4. A sends P2PSignal::Answer back via Router
5. ICE candidates exchanged via Router
6. WebRTC DataChannel established
7. Direct client-to-client communication
```

## Key Patterns

### 1. Builder Pattern
Used extensively for configuration:
```rust
let client = ClaspBuilder::new(url)
    .name("My App")
    .features(vec!["param", "event"])
    .reconnect(true)
    .connect()
    .await?;
```

### 2. Trait Objects for Extensibility
```rust
pub trait Bridge: Send + Sync {
    fn start(&mut self) -> Result<mpsc::Receiver<BridgeEvent>>;
    fn stop(&mut self) -> Result<()>;
    fn send(&self, message: Message) -> Result<()>;
}
```

### 3. DashMap for Concurrent Access
Lock-free concurrent hash maps for sessions, subscriptions, state:
```rust
sessions: Arc<DashMap<SessionId, Arc<Session>>>
subscriptions: Arc<DashMap<(SessionId, u32), Subscription>>
```

### 4. Event-Driven Architecture
All bridges and transports use mpsc channels:
```rust
pub enum BridgeEvent {
    ToClasp(Message),
    Connected,
    Disconnected { reason: Option<String> },
    Error(String),
}
```

### 5. Feature Gating
Conditional compilation for optional features:
```rust
#[cfg(feature = "quic")]
pub mod quic;

#[cfg(feature = "p2p")]
pub mod p2p;
```

## Protocol Versioning

### Binary Frame Format (v3)
```
Byte 0:     Magic (0x53 = 'S')
Byte 1:     Flags [QoS:2][TS:1][Enc:1][Cmp:1][Ver:3]
Bytes 2-3:  Payload length (u16 big-endian)
[Bytes 4-11: Timestamp (optional, u64)]
Payload:    Binary-encoded message
```

### Message Types
```
HELLO = 0x01      WELCOME = 0x02     ANNOUNCE = 0x03
SUBSCRIBE = 0x10  UNSUBSCRIBE = 0x11
PUBLISH = 0x20    SET = 0x21         GET = 0x22     SNAPSHOT = 0x23
BUNDLE = 0x30
SYNC = 0x40       PING = 0x41        PONG = 0x42
ACK = 0x50        ERROR = 0x51
QUERY = 0x60      RESULT = 0x61
```

## Quality of Service

| Level | Name | Semantics | Use Case |
|-------|------|-----------|----------|
| 0 | Fire | Best effort, no ack | Streams, high-rate data |
| 1 | Confirm | At-least-once, ACK required | Parameters, events |
| 2 | Commit | Exactly-once, ordered | Bundles, transactions |

## Security Architecture

### Authentication Flow
```
1. Client connects with token in HELLO
2. Router validates via TokenValidator chain
3. CpskValidator checks prefix "cpsk_"
4. Token info extracted: scopes, expiration
5. Session marked as authenticated
6. All subsequent messages checked against scopes
```

### Scope-Based Authorization
```
Scope format: "action:pattern"
Examples:
  - "read:/lights/**"     → Can subscribe/get under /lights/
  - "write:/controls/*"   → Can set/publish to /controls/{one-level}
  - "admin:/**"           → Full access everywhere
```

## Deployment Topologies

### 1. Star (Central Router)
```
     Client A ─────┐
     Client B ─────┼──► Router ◄──── Bridge (OSC)
     Client C ─────┘        │
                           └──── Bridge (MQTT)
```

### 2. Mesh (P2P with Fallback)
```
     Client A ◄─────► Client B
         │               │
         └───► Router ◄──┘
              (fallback)
```

### 3. Federated (Multi-Router)
```
Router 1 ◄────────► Router 2
    │                   │
 Clients             Clients
```

## Performance Characteristics

### Latency Targets
- Local (same machine): < 100µs
- LAN (Ethernet): < 1ms
- WAN (Internet): < 100ms

### Throughput Targets
- Encoding: 50,000+ msg/s
- Decoding: 50,000+ msg/s
- Router forwarding: 10,000+ msg/s

### Memory Budget (Embedded)
- Client: ~2-3 KB
- MiniRouter: ~4 KB
- State cache: ~2 KB (32 entries)
