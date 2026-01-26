# CLASP Codebase Master Analysis Index

**Generated:** 2026-01-25
**Protocol Version:** 3.1.0
**Analysis Depth:** AST-level, comprehensive

## Quick Navigation

| Document | Description |
|----------|-------------|
| [01-ARCHITECTURE](./01-ARCHITECTURE.md) | High-level architecture and design patterns |
| [02-CRATE-DEPENDENCY-MAP](./02-CRATE-DEPENDENCY-MAP.md) | Crate relationships and dependency graph |
| [03-CLASP-CORE](./03-CLASP-CORE.md) | Protocol types, codec, frame format |
| [04-CLASP-ROUTER](./04-CLASP-ROUTER.md) | Message routing, sessions, subscriptions |
| [05-CLASP-TRANSPORT](./05-CLASP-TRANSPORT.md) | Transport abstractions and implementations |
| [06-CLASP-BRIDGE](./06-CLASP-BRIDGE.md) | Protocol bridges (OSC, MIDI, Art-Net, etc.) |
| [07-CLASP-CLIENT](./07-CLASP-CLIENT.md) | Client library and P2P capabilities |
| [08-CLASP-DISCOVERY](./08-CLASP-DISCOVERY.md) | mDNS, broadcast, rendezvous discovery |
| [09-AUXILIARY-CRATES](./09-AUXILIARY-CRATES.md) | WASM, embedded, CLI, test-utils |
| [10-E2E-TEST-SUITE](./10-E2E-TEST-SUITE.md) | Test infrastructure and coverage |
| [11-LANGUAGE-BINDINGS](./11-LANGUAGE-BINDINGS.md) | JavaScript/TypeScript and Python bindings |
| [12-APPLICATIONS](./12-APPLICATIONS.md) | Bridge desktop app and marketing site |
| [13-TESTING-STRATEGY](./13-TESTING-STRATEGY.md) | Testing patterns and coverage analysis |
| [14-SECURITY-MODEL](./14-SECURITY-MODEL.md) | Authentication, authorization, tokens |
| [15-FUNCTION-CLASS-MAP](./15-FUNCTION-CLASS-MAP.md) | Complete type and function reference |

---

## Codebase Statistics

### Repository Structure
```
clasp/
├── crates/           # 10 Rust library crates
├── tools/            # 3 Binary tool crates
├── clasp-e2e/        # End-to-end test suite (40 binaries)
├── bindings/         # Language bindings (JS, Python)
├── apps/             # Applications (Bridge, Site)
├── deploy/           # Deployment configs (Docker)
├── docs/             # Documentation (Markdown)
├── examples/         # Usage examples
└── .internal/        # Internal analysis and archives
```

### Code Metrics
- **Rust Source Files:** ~150 files
- **Lines of Rust:** ~40,000 (excluding tests)
- **Test Coverage:** ~2,800+ test assertions
- **JavaScript/TypeScript:** ~5,000 lines
- **Python:** ~1,500 lines
- **Vue Components:** ~25 components

### Crate Breakdown

| Crate | Type | Lines | Purpose |
|-------|------|-------|---------|
| clasp-core | lib | 3,500 | Protocol types, codec, frame format |
| clasp-router | lib | 2,800 | Message routing, sessions, state |
| clasp-transport | lib | 3,200 | Transport abstractions (WS, QUIC, etc.) |
| clasp-bridge | lib | 4,000 | Protocol bridges (OSC, MIDI, etc.) |
| clasp-client | lib | 2,100 | Client API, subscriptions, P2P |
| clasp-discovery | lib | 1,500 | Discovery (mDNS, broadcast, HTTP) |
| clasp-wasm | lib | 800 | WebAssembly bindings |
| clasp-embedded | lib | 600 | no_std embedded support |
| clasp-cli | bin | 700 | Command-line interface |
| clasp-test-utils | lib | 400 | Test helpers |

---

## Architecture Overview

### Core Protocol Flow
```
Client ←→ Transport ←→ Router ←→ Bridge ←→ External Protocol
           ↑                        ↑
        WebSocket              OSC/MIDI/DMX
         QUIC                   Art-Net
         TCP/UDP                MQTT
         WebRTC                 HTTP
```

### Signal Types
1. **Param** - Stateful parameters with revision tracking
2. **Event** - Ephemeral one-shot events
3. **Stream** - High-rate continuous data
4. **Gesture** - Multi-touch input with phases
5. **Timeline** - Keyframe-based automation

### Transport Options
- **WebSocket** - Primary transport, browser-compatible
- **QUIC** - High-performance, UDP-based
- **TCP** - Simple TCP sockets
- **UDP** - Connectionless datagrams
- **Serial** - USB/Serial devices
- **BLE** - Bluetooth Low Energy
- **WebRTC** - Peer-to-peer data channels

### Protocol Bridges
- OSC (Open Sound Control)
- MIDI (Musical Instrument Digital Interface)
- Art-Net (DMX over Ethernet)
- sACN/E1.31 (Streaming ACN)
- DMX (Direct serial)
- MQTT (IoT messaging)
- WebSocket (JSON/Binary)
- Socket.IO (Real-time)
- HTTP (REST API)

---

## Key Design Decisions

### 1. Binary Encoding (v3)
- 54% smaller than MessagePack
- 5x faster encoding/decoding
- Backward-compatible with v2 MessagePack

### 2. Gesture Coalescing
- Buffers MOVE events at 60fps
- Reduces network traffic for touch input
- Preserves START/END/CANCEL immediately

### 3. State Synchronization
- Last-Write-Wins (LWW) conflict resolution
- Optimistic locking with revisions
- Late-joiner state snapshots

### 4. Security Model
- Capability Pre-Shared Keys (CPSK)
- Scope-based authorization (read/write patterns)
- Token expiration and revocation

### 5. Discovery Mechanisms
- mDNS/Bonjour for LAN
- UDP broadcast fallback
- HTTP rendezvous for WAN

---

## Testing Infrastructure

### Test Categories
- **Unit Tests:** Per-crate module tests
- **Integration Tests:** Cross-crate functionality
- **E2E Tests:** Full protocol validation
- **Compliance Tests:** Spec conformance
- **Load Tests:** Performance benchmarks
- **Stress Tests:** Resilience validation
- **Chaos Tests:** Failure scenarios

### Performance Targets
- Encoding: 50k+ msg/s
- Decoding: 50k+ msg/s
- Roundtrip: 20k+ msg/s
- Latency (local): p99 < 1ms

---

## Vestigial Code Location

All archived planning documents and internal notes are in:
```
.internal/archive/   # 44 markdown planning documents
```

These are historical artifacts from development and should not be considered current documentation.

---

## Navigation Tips

1. **New to CLASP?** Start with [01-ARCHITECTURE](./01-ARCHITECTURE.md)
2. **Implementing a client?** See [07-CLASP-CLIENT](./07-CLASP-CLIENT.md)
3. **Adding a bridge?** Check [06-CLASP-BRIDGE](./06-CLASP-BRIDGE.md)
4. **Running tests?** Read [10-E2E-TEST-SUITE](./10-E2E-TEST-SUITE.md)
5. **Security questions?** See [14-SECURITY-MODEL](./14-SECURITY-MODEL.md)
