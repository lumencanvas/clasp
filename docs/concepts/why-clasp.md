---
title: Why CLASP
description: Why CLASP exists and how it compares to alternatives
order: 1
---

# Why CLASP

CLASP exists because real-time systems suffer from protocol fragmentation, and existing protocols each lack features that modern applications need. A lighting designer using DMX, a musician sending OSC, and a web developer using WebSocket are all solving the same fundamental problem -- moving signals between devices in real time -- but their tools cannot talk to each other without custom glue code.

CLASP is a unified signal protocol that bridges these worlds. It provides a shared state model, semantic signal types, security, and server-side logic so that engineers can build real-time systems without reinventing infrastructure.

## The Problem

Real-time and creative systems rely on a patchwork of protocols, each designed for a specific domain:

| Domain | Protocols |
|---|---|
| Lighting | DMX-512, Art-Net, sACN |
| Audio / Creative | OSC, MIDI |
| IoT | MQTT, CoAP |
| Web | WebSocket, HTTP, SSE |

This creates an N-squared integration problem. Every new device or service needs a custom adapter for every protocol it talks to. A lighting console that speaks Art-Net cannot coordinate with a web dashboard over WebSocket unless someone writes a bespoke bridge between the two. Add OSC for audio control, MQTT for IoT sensors, and MIDI for hardware -- and the number of custom adapters grows quadratically.

Beyond connectivity, these protocols disagree on fundamentals. Some have state, some do not. Some have security, most do not. None of them share a namespace scheme, a conflict resolution strategy, or a timing model. The result is that every production team, every installation, every product builds its own ad-hoc middleware layer and maintains it forever.

## What CLASP Provides

### 1. Protocol Bridge

One integration per protocol, not N-squared pairs. The OSC bridge handles all OSC devices. The MQTT bridge handles all MQTT devices. Devices on different protocols communicate through the CLASP router without knowing anything about each other's wire format.

A MIDI controller can set a parameter that an Art-Net fixture reads -- through two bridges and one router, with no custom code.

### 2. Semantic Signals

CLASP defines five signal types instead of generic byte messages. The router handles each type differently:

- **Param** -- Stateful value. The router stores the current value and delivers it to late joiners. Conflict resolution applies.
- **Event** -- Fire-and-forget trigger. No state retained. Delivered to current subscribers only.
- **Stream** -- High-rate continuous data. Optimized for throughput, optional lossy delivery on congestion.
- **Gesture** -- Phased input with begin/update/end lifecycle. The router tracks active gestures and can cancel orphans.
- **Timeline** -- Automation curve. Keyframes with interpolation, scheduled execution via synchronized clocks.

This eliminates an entire class of application-layer code. The router already knows that a Param should be stored and a Stream should be dropped under pressure -- your application does not need to implement that logic.

### 3. State Management

Every Param has a current value that the router maintains. This solves three problems that plague stateless protocols like OSC:

- **Late joiners** get a snapshot of all current state when they connect. No need to poll or wait for the next update.
- **Conflict resolution** is built in. When two clients write the same Param simultaneously, the router resolves the conflict using a configurable strategy: last-writer-wins, max, min, lock, or merge.
- **Revisions** track every change. Clients can detect missed updates and request replay from the journal.

### 4. Security

Three token types cover different deployment scenarios:

- **CPSK** (`cpsk_`) -- Pre-shared key authentication. Register, login, and guest flows. Simple, suitable for user-facing applications.
- **Capability** (`cap_`) -- Ed25519-signed, delegatable tokens following the UCAN model. A root token can issue narrower tokens offline. Suitable for IoT devices and third-party access.
- **Entity** (`ent_`) -- Registry-backed persistent identity for devices and services. Status lifecycle (active, inactive, revoked) managed through a REST API.

All three token types enforce scoped permissions on every operation. A token with `read:/lighting/**` can subscribe to lighting data but cannot write to it, and cannot see anything outside `/lighting/`.

### 5. Server-Side Logic

Three features that eliminate the need for custom server code:

- **Rules engine** -- Reactive automation. "When `/sensors/motion` becomes true, set `/lighting/zone-1/brightness` to 100." Evaluated server-side on every state change.
- **App config** -- Declarative write rules and visibility controls. Define who can write where, which clients can see which data, and how snapshots are transformed before delivery. Loaded from a configuration file, no code required.
- **Journal** -- Persistence and REPLAY queries. Every state change is recorded. Clients can query historical values. State survives router restarts.

### 6. Federation

Multi-site deployment with hub-leaf topology. Each site runs its own relay. A hub relay aggregates state from leaf relays, enforcing namespace ownership to prevent conflicts. State changes propagate automatically across sites.

This enables scenarios like a touring show with a local relay at each venue and a cloud hub for centralized monitoring, or a museum installation with independent galleries that share visitor data.

### 7. Transport Flexibility

CLASP runs over multiple transports with the same API:

- **WebSocket** -- Browser-compatible, works through proxies and firewalls.
- **QUIC** -- Low-latency, multiplexed, connection migration for native clients.
- **UDP** -- Minimal overhead for embedded devices and local networks.
- **Serial** -- Direct hardware connection for microcontrollers.
- **BLE** -- Wireless IoT devices without Wi-Fi infrastructure.

A client written against the CLASP API works identically regardless of transport. The transport is a deployment decision, not an application decision.

## Why Not Just Use...

| Protocol | Limitation CLASP Addresses |
|---|---|
| OSC | No state management, no standard namespaces, UDP only (no reliability), no security model, no persistence, no conflict resolution |
| MQTT | No semantic signal types, no timing primitives, no conflict resolution, no protocol bridging, separate broker ecosystem |
| Custom WebSocket | Every team invents their own message format, no interop between projects, no bridges to hardware protocols, no discovery |
| MIDI 2.0 | Hardware-focused transport, not designed for network distribution, limited data types, no pub/sub, no state management |
| gRPC / REST | Request-response only, no pub/sub, no real-time streaming, no device bridging, no state synchronization |

CLASP is not a replacement for these protocols. OSC devices still speak OSC, MQTT sensors still speak MQTT. CLASP bridges them into a unified system where they can share state, coordinate timing, and operate under a single security model.

## Who Benefits

- **Live performance** (VJ, lighting, audio) -- Bridge OSC, MIDI, and DMX through one system. Shared state means the lighting console and the VJ software always agree on the current scene.
- **Installation art** -- Federated multi-site deployments, offline operation with journal persistence, embedded sensor nodes on ESP32 or RP2040 with the 3KB client.
- **Home automation** -- IoT bridge (MQTT), rules engine for automation logic, mobile web apps via WebSocket. No cloud dependency required.
- **Collaborative tools** -- Real-time state sync with conflict resolution, per-user auth and data isolation, late-joiner snapshots.
- **Embedded systems** -- The `clasp-embedded` crate compiles to approximately 3KB with `no_std` support. Runs on bare-metal microcontrollers with Serial or UDP transport.

## Next Steps

- [Architecture](./architecture.md) -- How the crates fit together and how messages flow through the system.
- [Security Model](./security-model.md) -- Token types, scope enforcement, and threat model.
- [First Connection](../getting-started/first-connection.md) -- Connect a client to a router in five minutes.
