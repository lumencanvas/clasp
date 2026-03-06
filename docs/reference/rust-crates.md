---
title: Rust Crates
description: Overview of all CLASP Rust crates
order: 10
---

# Rust Crates

CLASP is organized as a collection of Rust crates, each with a specific responsibility. This page provides an overview of every crate, its purpose, key exports, dependencies, and feature flags.

## Crate Map

| Crate              | Purpose                  | Key Exports                                          | Dependencies                       | Feature Flags                                            |
|---------------------|--------------------------|------------------------------------------------------|-------------------------------------|----------------------------------------------------------|
| `clasp-core`        | Types, codec, addressing, state | `Message`, `Value`, `SignalType`, `Address`, `ParamState`, `ConflictStrategy` | None                              | --                                                       |
| `clasp-transport`   | Network transports       | `WebSocketTransport`, `QuicTransport`, `UdpTransport`, `SerialTransport`, `BleTransport` | `clasp-core`                     | `websocket` (default), `quic`, `udp`, `serial`, `ble`   |
| `clasp-client`      | Async client library     | `Clasp`, `ClaspBuilder`                              | `clasp-core`, `clasp-transport`    | `p2p`                                                    |
| `clasp-router`      | Router implementation    | `Router`, `RouterConfig`, `RouterState`              | `clasp-core`, `clasp-transport`    | --                                                       |
| `clasp-bridge`      | Protocol bridges         | `Bridge`, `BridgeConfig`, `AddressMapping`           | `clasp-core`                       | `osc`, `midi`, `artnet`, `dmx`, `sacn`, `mqtt`, `http`, `websocket` |
| `clasp-discovery`   | Service discovery        | `DiscoveryConfig`, `DiscoveryEvent`                  | `clasp-core`                       | --                                                       |
| `clasp-embedded`    | no_std MCU client        | `Client`, `Value`, `MiniRouter`                      | None                               | --                                                       |
| `clasp-caps`        | Ed25519 capability tokens| `CapabilityToken`, `CapabilityValidator`             | `clasp-core`                       | --                                                       |
| `clasp-registry`    | Entity registry          | `Entity`, `EntityStore`, `EntityValidator`           | `clasp-core`                       | `sqlite`                                                 |
| `clasp-rules`       | Rules engine             | `Rule`, `RulesEngine`, `Trigger`, `RuleAction`       | `clasp-core`                       | --                                                       |
| `clasp-journal`     | State persistence        | `Journal`, `SqliteJournal`, `MemoryJournal`          | `clasp-core`                       | `sqlite`                                                 |
| `clasp-federation`  | Multi-router federation  | `FederationManager`, `FederationConfig`, `FederationLink` | `clasp-core`                  | --                                                       |
| `clasp-crypto`      | E2E encryption           | `E2ESession`, `CryptoClient`, `MemoryKeyStore`, `FileSystemKeyStore` | `clasp-core`           | `client`, `fs-store`                                     |
| `clasp-wasm`        | WebAssembly bindings     | WASM client                                              | `clasp-core`, `clasp-client`  | `p2p`                                                    |

## Layer Diagram

The crates are organized in dependency layers. Higher layers depend on lower layers, but never the reverse.

```
                        Extensions
    +-----------+-----------+---------+-----------+--------------+
    | clasp-caps| clasp-    | clasp-  | clasp-    | clasp-       | clasp-  |
    |           | registry  | rules   | journal   | federation   | crypto  |
    +-----------+-----------+---------+-----------+--------------+---------+

                        Application
                +-------------+-------------+
                | clasp-client| clasp-router|
                +-------------+-------------+

                        Networking
         +---------------+-----------+-----------------+
         | clasp-        | clasp-    | clasp-          |
         | transport     | bridge    | discovery       |
         +---------------+-----------+-----------------+

                        Foundation
                    +----------------+
                    |   clasp-core   |
                    +----------------+

                        Standalone
                +----------------+------------+
                | clasp-embedded | clasp-wasm |
                +----------------+------------+
```

**Foundation**: `clasp-core` defines the wire protocol, value types, addressing scheme, and state primitives. All other crates (except `clasp-embedded`) depend on it.

**Standalone**: `clasp-embedded` has zero dependencies and targets `no_std` environments (microcontrollers, bare-metal). It implements a minimal subset of the CLASP protocol.

**Networking**: Transport, bridge, and discovery crates handle communication. They depend only on `clasp-core`.

**Application**: The client and router crates build on core and transport to provide the main developer-facing APIs.

**Extensions**: Caps, registry, rules, journal, and federation are optional modules that plug into the router or operate alongside it.

## Crate Details

### clasp-core

The foundation crate. Defines the binary wire format, value encoding/decoding, address parsing, parameter state with conflict resolution, and all message types.

```toml
[dependencies]
clasp-core = "4.1"
```

Use this crate directly when building custom tooling that operates at the protocol level or when implementing a new transport.

### clasp-transport

Provides transport implementations that carry CLASP frames over the network. Enable only the transports you need via feature flags to minimize dependencies.

```toml
[dependencies]
clasp-transport = { version = "4.1", features = ["websocket", "quic"] }
```

### clasp-client

The async client library for connecting to a CLASP router. This is the primary crate for most application developers.

```toml
[dependencies]
clasp-client = "4.1"
```

Enable `p2p` for direct peer-to-peer connections via WebRTC:

```toml
[dependencies]
clasp-client = { version = "4.1", features = ["p2p"] }
```

### clasp-router

Embed a CLASP router in your application. See [Router Config](router-config.md) for the full `RouterConfig` reference.

```toml
[dependencies]
clasp-router = "4.1"
```

### clasp-bridge

Protocol bridges translate between CLASP and external protocols (OSC, MIDI, Art-Net, etc.). Enable only the bridges you need.

```toml
[dependencies]
clasp-bridge = { version = "4.1", features = ["osc", "midi"] }
```

### clasp-discovery

Automatic service discovery using mDNS, UDP broadcast, and rendezvous servers.

```toml
[dependencies]
clasp-discovery = "4.1"
```

### clasp-embedded

A `no_std`, zero-dependency client for microcontrollers. Implements a minimal CLASP subset with a small memory footprint.

```toml
[dependencies]
clasp-embedded = "4.1"
```

### clasp-caps

Ed25519-based capability tokens for authentication and authorization. Tokens encode scopes as `action:pattern` pairs.

```toml
[dependencies]
clasp-caps = "4.1"
```

### clasp-registry

An entity registry backed by SQLite for tracking devices, users, or other domain objects.

```toml
[dependencies]
clasp-registry = { version = "4.1", features = ["sqlite"] }
```

### clasp-rules

A reactive rules engine that evaluates triggers, conditions, and actions against the router's state. See [Rules Schema](rules-schema.md) for the JSON format.

```toml
[dependencies]
clasp-rules = "4.1"
```

### clasp-journal

State persistence with pluggable backends. `MemoryJournal` for testing, `SqliteJournal` for production.

```toml
[dependencies]
clasp-journal = { version = "4.1", features = ["sqlite"] }
```

### clasp-federation

Multi-router federation for scaling CLASP across multiple nodes. Handles state synchronization, conflict resolution, and link management between routers.

```toml
[dependencies]
clasp-federation = "4.1"
```

### clasp-crypto

Client-side E2E encryption using ECDH P-256 key exchange and AES-256-GCM. The router never sees plaintext. Enable `client` for the `CryptoClient` wrapper over a `Clasp` instance, and `fs-store` for `FileSystemKeyStore`.

```toml
[dependencies]
clasp-crypto = { version = "4.1", features = ["client"] }
```

### clasp-wasm

WebAssembly bindings for using CLASP in the browser or any WASM runtime. Wraps `clasp-client` for JS interop via `wasm-bindgen`.

```toml
[dependencies]
clasp-wasm = "4.1"
```

Enable `p2p` for WebRTC peer-to-peer connections in the browser:

```toml
[dependencies]
clasp-wasm = { version = "4.1", features = ["p2p"] }
```

## Using Crates

Most users need only one crate:

| Use Case                        | Crate           | Command                                |
|---------------------------------|-----------------|----------------------------------------|
| Build a client application      | `clasp-client`  | `cargo add clasp-client`               |
| Embed a router in your app      | `clasp-router`  | `cargo add clasp-router`               |
| Work at the protocol level      | `clasp-core`    | `cargo add clasp-core`                 |
| Target a microcontroller        | `clasp-embedded`| `cargo add clasp-embedded`             |
| Bridge to OSC, MIDI, etc.       | `clasp-bridge`  | `cargo add clasp-bridge --features osc`|
| E2E encryption                  | `clasp-crypto`  | `cargo add clasp-crypto --features client` |
| WASM browser client             | `clasp-wasm`    | `cargo add clasp-wasm`                 |

For a full application with an embedded router, bridges, and persistence:

```toml
[dependencies]
clasp-router = "4.1"
clasp-bridge = { version = "4.1", features = ["osc", "midi", "artnet"] }
clasp-journal = { version = "4.1", features = ["sqlite"] }
clasp-rules = "4.1"
clasp-caps = "4.1"
clasp-crypto = { version = "4.1", features = ["client"] }
```

## Next Steps

- [Router Config](router-config.md) -- `RouterConfig` struct reference for embedding routers
- [JavaScript API](js-api.md) -- JavaScript/TypeScript client reference
- [Python API](python-api.md) -- Python client reference
- [Architecture](../concepts/architecture.md) -- how the crates fit together
