# Architecture

This document explains how CLASP is structured and how the components work together.

## Overview

CLASP systems have four types of components:

```
┌─────────────────────────────────────────────────────────────┐
│                    APPLICATIONS                              │
│  Desktop App  │  CLI Tools  │  Your Custom App              │
├─────────────────────────────────────────────────────────────┤
│                     LIBRARIES                                │
│  clasp-client  │  clasp-router  │  clasp-bridge             │
├─────────────────────────────────────────────────────────────┤
│                    TRANSPORT                                 │
│  WebSocket  │  WebRTC  │  QUIC  │  UDP  │  BLE  │  Serial   │
├─────────────────────────────────────────────────────────────┤
│                      CORE                                    │
│  Types  │  Codec  │  Addresses  │  Security                 │
├─────────────────────────────────────────────────────────────┤
│                    EMBEDDED                                  │
│  no_std client (bring your own transport)                   │
└─────────────────────────────────────────────────────────────┘
```

## Component Roles

### Router

The **router** is the central message hub:

- Routes messages between clients based on subscriptions
- Maintains authoritative state for all parameters
- Handles conflict resolution
- Manages subscriptions with wildcard matching
- Provides clock synchronization

```
                    ┌──────────────────┐
                    │     Router       │
                    │                  │
     ┌──────────────┤  State Store     │
     │              │  Pattern Matcher │
     │              │  Clock Source    │
     │              └────────┬─────────┘
     │                       │
     │         ┌─────────────┼─────────────┐
     │         │             │             │
┌────▼────┐ ┌──▼──────┐ ┌────▼────┐ ┌──────▼────┐
│ Client  │ │ Client  │ │ Bridge  │ │  Bridge   │
│  (JS)   │ │ (Python)│ │  (OSC)  │ │  (MIDI)   │
└─────────┘ └─────────┘ └─────────┘ └───────────┘
```

A CLASP system typically has one router (though multiple can be connected for federation).

### Client

**Clients** connect to routers to send and receive messages:

- Connect via any supported transport
- Subscribe to address patterns
- Set parameters and emit events
- Receive state updates
- Can be in any language with a CLASP library

```rust
// Rust client example
let client = ClaspBuilder::new("ws://router:7330")
    .name("My App")
    .connect()
    .await?;

client.set("/my/value", 42.0).await?;
client.subscribe("/other/**", |value, addr| {
    println!("{} = {:?}", addr, value);
}).await?;
```

### Bridge

**Bridges** translate between CLASP and external protocols:

- Connect to the router as a special type of client
- Listen on the external protocol (OSC, MIDI, etc.)
- Translate messages bidirectionally
- Map addresses between protocols

```
┌─────────────┐      ┌──────────────┐      ┌─────────────┐
│  TouchOSC   │ OSC  │  OSC Bridge  │CLASP │   Router    │
│   (iPad)    │ ───► │  (client)    │ ───► │  (server)   │
└─────────────┘      └──────────────┘      └─────────────┘
```

A bridge is just a client that also speaks another protocol.

### Transport

**Transports** carry CLASP frames over the network:

- WebSocket (default, browser-compatible)
- QUIC (native apps, mobile)
- UDP (LAN, embedded)
- WebRTC (P2P, NAT traversal)
- Serial (hardware)
- BLE (IoT)

The protocol is transport-agnostic—the same messages work over any transport.

## Common Patterns

### Pattern 1: Simple Client-Router

The most basic setup:

```
Your App → WebSocket → CLASP Router
```

### Pattern 2: Multiple Clients

Multiple applications sharing state:

```
┌───────────┐     ┌───────────┐     ┌───────────┐
│  App A    │     │  App B    │     │  App C    │
└─────┬─────┘     └─────┬─────┘     └─────┬─────┘
      │                 │                 │
      └─────────────────┼─────────────────┘
                        │
                 ┌──────▼──────┐
                 │   Router    │
                 └─────────────┘
```

### Pattern 3: Embedded Router

Running the router inside your application:

```rust
// Your app contains the router
let router = Router::new(config);

// Other clients connect to you
router.serve_websocket("0.0.0.0:7330").await?;

// Your app also uses the router directly
router.state().set_value("/app/status", Value::String("running"), "server");
```

### Pattern 4: Protocol Bridging

Connecting external protocols:

```
┌──────────────────────────────────────────────────────────────┐
│                      CLASP Router                             │
│                                                               │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐     │
│  │  OSC     │  │  MIDI    │  │  MQTT    │  │ Art-Net  │     │
│  │  Bridge  │  │  Bridge  │  │  Bridge  │  │  Bridge  │     │
│  └────┬─────┘  └────┬─────┘  └────┬─────┘  └────┬─────┘     │
└───────│─────────────│─────────────│─────────────│─────────────┘
        │             │             │             │
        ▼             ▼             ▼             ▼
   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐
   │   OSC   │   │  MIDI   │   │  MQTT   │   │ Art-Net │
   │  Apps   │   │ Devices │   │ Sensors │   │  Nodes  │
   └─────────┘   └─────────┘   └─────────┘   └─────────┘
```

## Data Flow

### Publishing a Value

```
1. Client calls set("/lights/1/brightness", 0.8)
2. Frame encoded and sent over transport
3. Router receives and validates
4. Router updates state store (revision incremented)
5. Router finds matching subscriptions
6. Router sends to all subscribers
7. Router sends ACK to publisher (if QoS requires)
```

### Subscribing

```
1. Client calls subscribe("/lights/**")
2. SUBSCRIBE message sent to router
3. Router registers subscription in pattern matcher
4. Router sends SNAPSHOT of matching current state
5. Future matching SETs are forwarded to subscriber
```

## State Management

The router maintains authoritative state:

```
State Store
├── /lights/1/brightness = 0.8 (rev: 42, writer: session:abc)
├── /lights/1/color = [255, 0, 0] (rev: 17, writer: session:xyz)
├── /lights/2/brightness = 0.5 (rev: 8, writer: session:abc)
└── ...
```

Each parameter has:
- Current value
- Revision number (monotonically increasing)
- Last writer (session ID)
- Timestamp

This enables:
- Late-joiner synchronization (new clients get current state)
- Conflict resolution (revision-based)
- Audit trail (who changed what)

## Scaling

### Single Router (Most Use Cases)

A single router handles thousands of clients and millions of messages per day.

### Router Federation (Large Scale)

For large deployments, routers can be connected:

```
┌───────────────┐     ┌───────────────┐     ┌───────────────┐
│   Router A    │◄───►│   Router B    │◄───►│   Router C    │
│  (Building 1) │     │  (Building 2) │     │  (Cloud)      │
└───────────────┘     └───────────────┘     └───────────────┘
```

Federation is an advanced topic covered separately.

## Crate Organization

The Rust implementation is organized into crates:

| Crate | Role | Dependencies |
|-------|------|--------------|
| `clasp-core` | Types, codec, addresses | None |
| `clasp-transport` | Network transports | clasp-core |
| `clasp-client` | Client library | clasp-core, clasp-transport |
| `clasp-router` | Router implementation | clasp-core, clasp-transport |
| `clasp-bridge` | Protocol bridges | clasp-core |
| `clasp-discovery` | Service discovery | clasp-core |
| `clasp-embedded` | no_std client | None |

## See Also

- [Router vs Client](router-vs-client.md) — Detailed role comparison
- [Transport Agnosticism](transport-agnosticism.md) — Why multiple transports
- [Bridge Architecture](bridge-architecture.md) — How bridges work
