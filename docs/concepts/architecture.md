---
title: Architecture
description: System architecture, crate layers, and message flow
order: 2
---

# Architecture

CLASP is organized as a set of Rust crates that layer from protocol primitives up to a full production server. Each layer depends only on the layers below it, so you can use as much or as little of the stack as your application requires -- from the zero-dependency core types in an embedded device to a fully-featured relay with federation, persistence, and authentication.

This page explains how the pieces fit together, how messages flow through the system, and where the extension points are.

## Component Roles

A CLASP deployment has four kinds of components:

**Router** -- The central hub. It routes signals between clients, manages the state store, enforces authentication and authorization, runs the rules engine, and handles federation with other routers. A single router process can serve thousands of concurrent clients. Most deployments have exactly one router (or one per site in federated setups).

**Client** -- Any application that connects to a router. Clients send signals, subscribe to address patterns, and receive state. Client libraries exist for JavaScript (browser and Node.js), Python, and Rust. A client does not need to know about other clients or their protocols.

**Bridge** -- A specialized client that translates between CLASP and an external protocol. The OSC bridge converts OSC messages to CLASP signals and vice versa. The MIDI bridge does the same for MIDI. Bridges are bidirectional: external devices can both read and write CLASP state through their native protocol. Available bridges: OSC, MIDI, Art-Net, DMX, sACN, MQTT, HTTP, WebSocket.

**Transport** -- The wire-level carrier for CLASP frames. Transports are interchangeable: a client on WebSocket and a client on QUIC can communicate through the same router. Available transports: WebSocket, QUIC, UDP, Serial, BLE. See [Transports](../core/transports.md).

## Crate Layers

The Rust implementation is split into crates with explicit dependency boundaries:

```
┌───────────────────────────────────────────────────┐
│                  Applications                      │
│    (your code, chat app, lighting controller)      │
├────────────┬────────────┬─────────────────────────┤
│   Client   │   Router   │   Bridge                │
│   clasp-   │   clasp-   │   clasp-bridge          │
│   client   │   router   │   (osc,midi,mqtt,...)   │
├────────────┴────────────┴─────────────────────────┤
│  Extensions                                        │
│  clasp-caps    (capability tokens, Ed25519)        │
│  clasp-registry (entity identity, REST API)        │
│  clasp-rules   (reactive automation)               │
│  clasp-journal (persistence, REPLAY)               │
│  clasp-federation (multi-site sync)                │
├─────────────────────────┬─────────────────────────┤
│  Transport              │  Discovery               │
│  clasp-transport        │  clasp-discovery         │
│  (ws, quic, udp,        │  (mDNS, broadcast,       │
│   serial, ble)          │   rendezvous)             │
├─────────────────────────┴─────────────────────────┤
│  Core: clasp-core                                  │
│  Types, codec, addressing, state, signals          │
│  Zero external dependencies                        │
├───────────────────────────────────────────────────┤
│  Embedded: clasp-embedded                          │
│  no_std compatible, ~3KB binary, Serial/UDP only   │
└───────────────────────────────────────────────────┘
```

**clasp-core** is the foundation. It defines the five signal types, the binary codec, the address scheme, and the state primitives (values, revisions, conflict resolution). It has zero external dependencies, which is what allows `clasp-embedded` to compile for bare-metal targets.

**clasp-transport** provides async transport implementations. Each transport exposes the same `Transport` trait, so higher layers do not care whether frames arrive over WebSocket or Serial.

**clasp-discovery** handles finding routers on the network. mDNS for local networks, UDP broadcast for simple LANs, and rendezvous for NAT traversal.

**clasp-client** is the async Rust client. It uses a builder pattern for configuration, supports subscriptions with pattern matching, and manages reconnection. The JavaScript and Python clients are separate packages that implement the same protocol.

**clasp-router** is the routing engine. It contains the state store, the pattern matcher for subscriptions, session management, and clock synchronization. This is a library crate -- you can embed it in your own Rust application.

**clasp-bridge** contains protocol translators. Each bridge is a module that maps between an external protocol's concepts and CLASP signals. The OSC bridge maps OSC addresses to CLASP addresses and OSC arguments to CLASP values. The MIDI bridge maps channels and CCs to Param signals.

The **extension crates** add opt-in features to the router: capability tokens, entity registry, reactive rules, journal persistence, and federation. Each extension is a separate crate so the router can be compiled with only the features needed.

**clasp-embedded** is a standalone crate for microcontrollers. It implements the CLASP protocol with `no_std` and `no_alloc` support, compiling to approximately 3KB. It supports Serial and UDP transports only.

## Message Flow

Here is what happens when a client calls `set("/lighting/zone-1/brightness", 0.8)`:

```
Client                    Transport         Router
  │                          │                 │
  │  1. Encode SET message   │                 │
  │  (address + value +      │                 │
  │   signal type + rev)     │                 │
  │ ─────────────────────────>                 │
  │                          │  2. Send frame  │
  │                          │ ───────────────>│
  │                          │                 │
  │                          │      3. Decode message
  │                          │      4. Auth: ValidatorChain
  │                          │         checks session scopes
  │                          │      5. App Config: write rules
  │                          │         validate operation
  │                          │      6. State Store: update value,
  │                          │         increment revision,
  │                          │         record writer
  │                          │      7. Journal: append entry
  │                          │      8. Rules Engine: evaluate
  │                          │         matching triggers
  │                          │      9. Pattern Matcher: find
  │                          │         matching subscriptions
  │                          │     10. Forward to subscribers
  │                          │         + federation peers
  │                          │                 │
  │                          │  11. Send ACK   │
  │                          │ <───────────────│
  │ <─────────────────────────                 │
  │       ACK received       │                 │
```

Steps 4-10 happen synchronously within the router. If any step fails (auth denied, write rule rejected), the router sends an error response instead of an ACK and the state is not modified.

Steps 5, 7, and 8 are conditional -- they only execute if the corresponding feature is enabled (app config, journal, rules). Without those features, the message flow is: decode, auth, state update, pattern match, forward, ACK.

## Router vs Relay

These two terms appear throughout the documentation, and the distinction matters:

**`clasp-router`** is a Rust library crate. It contains the routing engine, state store, and session management. You use it by adding `clasp-router` as a dependency in your `Cargo.toml` and configuring it via the `RouterConfig` struct. This is the right choice when you are building a custom Rust application that needs CLASP routing embedded in it.

**`clasp-relay`** is a standalone binary in `deploy/relay/`. It wraps `clasp-router` with everything needed for production deployment:

- **CLI** (clap) -- command-line flags for all configuration
- **Auth HTTP server** -- REST endpoints for CPSK register/login/guest
- **App config loader** -- reads declarative write rules and visibility from JSON (`--app-config`)
- **Docker support** -- Dockerfile, health checks, graceful shutdown
- **Feature flag system** -- enable/disable extensions via CLI flags

Most users should deploy the relay. Embed the router directly only if you are building a custom Rust application and need fine-grained control over the routing engine.

## Feature Flag Matrix

The relay exposes extension features through CLI flags. Each feature corresponds to a crate and adds specific capabilities:

| Feature | Crate | Relay Flag | What It Adds |
|---|---|---|---|
| Auth | clasp-core (CpskValidator) | `--auth-port` | CPSK register/login/guest endpoints |
| Capabilities | clasp-caps | `--trust-anchor` | Ed25519 delegatable tokens, UCAN chains |
| Registry | clasp-registry | `--registry-db` | Persistent entity identity, REST API |
| Journal | clasp-journal | `--journal` | State persistence, REPLAY queries |
| Rules | clasp-rules | `--rules` | Reactive automation triggers |
| Federation | clasp-federation | `--federation-hub` | Multi-site state sync, hub-leaf topology |
| App Config | relay module | `--app-config` | Declarative write rules, snapshot visibility |

A minimal relay with no flags runs as a plain router: signals in, signals out, state management, no auth. Each flag adds a layer. A production deployment typically uses `--auth-port`, `--app-config`, and `--journal` at minimum.

## Scaling

A single relay handles thousands of concurrent clients. The state store is in-memory with an append-only journal for persistence, so reads and writes are fast. Pattern matching uses a trie structure for efficient subscription lookup.

For larger deployments, CLASP uses federation rather than horizontal scaling of a single router. The hub-leaf topology works as follows:

- Each **site** runs its own relay (a leaf).
- A **hub** relay connects to all leaves and aggregates state.
- Each leaf owns a namespace prefix (e.g., `/site-a/**`). Only that leaf can write to its namespace.
- State changes propagate from leaf to hub to other leaves automatically.
- Clients connect to their local leaf for low latency. Cross-site reads go through the hub.

This design avoids the complexity of distributed consensus. Each leaf is authoritative for its namespace, so there are no cross-site write conflicts. The hub is a relay, not a database -- it forwards state, it does not arbitrate.

## Next Steps

- [Why CLASP](./why-clasp.md) -- The problems CLASP solves and how it compares to alternatives.
- [Security Model](./security-model.md) -- Token types, scope enforcement, and threat model.
- [Router Configuration](../reference/router-config.md) -- Reference for all router configuration options.
