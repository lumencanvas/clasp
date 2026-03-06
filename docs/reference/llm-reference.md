---
title: LLM Reference
description: Comprehensive CLASP reference for LLM-assisted development
order: 11
---

# CLASP LLM Reference

Dense, structured reference for LLM code generation. Covers every API surface, wire format detail, and configuration option across all SDKs. Use this page to generate correct CLASP code without consulting multiple docs.

## System Overview

CLASP (Control Linkage and Signal Protocol) is a real-time signal routing protocol. Architecture: hub-spoke with a central **router** that clients connect to via WebSocket, QUIC, or other transports. The router manages state, routes signals, enforces auth, runs a rules engine, and handles federation.

- **Version**: 4.1.0
- **Rust crates**: 16 (see Crate Map below)
- **SDKs**: JavaScript (`@clasp-to/core`), Python (`clasp-to`), Rust (`clasp-client`), Embedded (`clasp-embedded`)
- **Default port**: 7330 (WebSocket), 7331 (QUIC/server), 7350 (auth HTTP)
- **Time unit**: microseconds (server-synchronized)

## Quick Start

### JavaScript

```javascript
import { ClaspBuilder } from '@clasp-to/core';

const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .connect();

client.set('/lights/brightness', 0.8);
client.subscribe('/lights/**', (address, value) => {
  console.log(address, value);
});
```

### Rust

```rust
use clasp_client::Clasp;

let client = Clasp::connect_to("ws://localhost:7330").await?;
client.set("/lights/brightness", Value::Float(0.8)).await?;
client.subscribe("/lights/**", |value, address| {
    println!("{}: {:?}", address, value);
}).await?;
```

### Python

```python
from clasp import Clasp

client = Clasp('ws://localhost:7330', name='My App')
await client.connect()
client.set('/lights/brightness', 0.8)

@client.on('/lights/**')
async def on_light(address, value):
    print(address, value)
```

## Signal Types

| Type | Method | Storage | QoS | Late-Joiner | Use Case |
|------|--------|---------|-----|-------------|----------|
| Param | `set`/`get`/`cached` | Yes | Confirm (1) | Yes (snapshot) | Fader position, config |
| Event | `emit` | No | Confirm (1) | No | Button press, cue trigger |
| Stream | `stream` | No | Fire (0) | No | Motion capture, audio level |
| Gesture | `gesture` | Phase only | Fire (0) | No | Touch, pen, multi-touch |
| Timeline | `timeline` | Full | Commit (2) | Yes | Lighting cue, animation |

### Param

```javascript
// JS
client.set('/lights/brightness', 0.8);
const val = await client.get('/lights/brightness');
const cached = client.cached('/lights/brightness');
```

```rust
// Rust
client.set("/lights/brightness", Value::Float(0.8)).await?;
let val = client.get("/lights/brightness").await?;
let cached = client.cached("/lights/brightness");
```

Conflict resolution strategies: LWW (last-writer-wins, default), Max, Min, Lock, Merge.

### Event

```javascript
client.emit('/alerts/motion', { zone: 'lobby', confidence: 0.95 });
```

```rust
client.emit("/alerts/motion", Value::from_json(r#"{"zone":"lobby"}"#)?).await?;
```

### Stream

```javascript
setInterval(() => client.stream('/audio/level', readLevel()), 16); // ~60Hz
```

```rust
client.stream("/audio/level", Value::Float(level)).await?;
```

### Gesture

Phases: `begin`, `update`, `end` (JS/Rust) or `start`, `move`, `end`, `cancel` (wire level).

```javascript
client.gesture('/input/fader', 'fader-1', 'begin', { value: 0.0 });
client.gesture('/input/fader', 'fader-1', 'update', { value: 0.5 });
client.gesture('/input/fader', 'fader-1', 'end');
```

```rust
client.gesture("/input/fader", "fader-1", "begin", Value::from_json(r#"{"value":0.0}"#)?).await?;
```

Router coalesces gestures: 240Hz input becomes ~24 delivered updates/sec.

### Timeline

```javascript
client.timeline('/lights/brightness', [
  { time: 0, value: 0.0 },
  { time: 1000, value: 1.0, easing: 'ease-in' },
  { time: 3000, value: 1.0 },
  { time: 4000, value: 0.0, easing: 'ease-out' }
], { loop: false });
```

Easing types: `linear`, `ease-in`, `ease-out`, `ease-in-out`, `step`, `cubic-bezier`.

## Address System

### Path Format

- Forward-slash separated: `/lights/brightness`, `/sensors/room/1/temp`
- Must start with `/`
- Case-sensitive
- No trailing slash

### Wildcards

| Pattern | Matches |
|---------|---------|
| `/sensors/temperature` | Exact match only |
| `/sensors/*` | Any single level: `/sensors/humidity`, `/sensors/temp` |
| `/sensors/**` | Any depth: `/sensors/room/1/temp`, `/sensors/a/b/c` |

### Namespace Conventions

| Prefix | Protocol |
|--------|----------|
| `/osc/` | OSC bridge |
| `/midi/` | MIDI bridge |
| `/mqtt/` | MQTT bridge |
| `/artnet/` | Art-Net bridge |
| `/dmx/` | DMX bridge |
| `/sacn/` | sACN bridge |
| `/_e2e/` | E2E encryption (internal) |

### Auth Scope Format

`action:pattern` where action is `read`, `write`, or `admin`.

```
read:/**           # read all paths
write:/lights/**   # write anything under /lights/
read:/sensors/*    # read one level under /sensors/
admin:/**          # full access
```

Hierarchy: `admin` > `write` > `read`. `write` implies `read` on same paths.

## Connection Patterns

### JavaScript

```javascript
import { ClaspBuilder, Clasp } from '@clasp-to/core';

// Builder (recommended)
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('My App')
  .withToken('cpsk_...')
  .withReconnect(true)
  .withReconnectInterval(5000)
  .withFeatures(['lighting', 'audio'])
  .connect();

// Static builder
const client2 = await Clasp.builder('ws://localhost:7330')
  .withName('My App')
  .connect();
```

Builder methods: `withName(s)`, `withToken(s)`, `withReconnect(bool)`, `withReconnectInterval(ms)`, `withFeatures(string[])`, `connect()`.

Aliases: `name()`, `token()`, `reconnect()`, `reconnectInterval()`, `features()`.

### Python

```python
from clasp import Clasp

# Builder
client = await Clasp.builder('ws://localhost:7330') \
    .with_name('My App') \
    .with_token('cpsk_...') \
    .with_reconnect(True) \
    .with_features(['lighting']) \
    .connect()

# Direct constructor
client = Clasp('ws://localhost:7330',
    name='My App',
    token='cpsk_...',
    reconnect=True,
    reconnect_interval=5000)
await client.connect()
```

### Rust

```rust
use clasp_client::{Clasp, ClaspBuilder};

// Builder
let client = ClaspBuilder::new("ws://localhost:7330")
    .name("My App")
    .token("cpsk_...")
    .reconnect(true)
    .reconnect_interval(3000)
    .features(vec!["lighting".into()])
    .connect()
    .await?;

// Quick connect
let client = Clasp::connect_to("ws://localhost:7330").await?;
```

## State Operations

| Operation | JS | Python | Rust |
|-----------|----|----|------|
| Set | `client.set(addr, val)` | `client.set(addr, val)` | `client.set(addr, val).await?` |
| Get | `await client.get(addr)` | `await client.get(addr)` | `client.get(addr).await?` |
| Cached | `client.cached(addr)` | `client.cached(addr)` | `client.cached(addr)` |

JS `set` is fire-and-forget (no await). Python `get` accepts `timeout=5.0`. Rust `cached` returns `Option<Value>`.

## Subscriptions

### JavaScript

```javascript
// Returns unsubscribe function
const unsub = client.subscribe('/sensors/**', (address, value, meta) => {
  console.log(address, value, meta.timestamp);
}, { maxRate: 30, epsilon: 0.01 });

// Events
const unsub2 = client.on('/alerts/**', (address, payload, meta) => {
  console.log('Alert:', address, payload);
});

unsub(); // unsubscribe
```

### Python

```python
@client.on('/sensors/**', max_rate=30, epsilon=0.01)
async def on_sensor(address, value):
    print(address, value)

# Or functional
unsub = client.subscribe('/sensors/**', callback)
unsub()
```

### Rust

```rust
client.subscribe("/sensors/**", |value, address| {
    println!("{}: {:?}", address, value);
}).await?;

client.unsubscribe("/sensors/**").await?;
```

### SubscribeOptions

| Option | Type | Description |
|--------|------|-------------|
| `maxRate` | number | Max callbacks per second (client-side throttle) |
| `epsilon` | number | Min numeric change to trigger callback |

## Bundles

Atomic message groups. All operations succeed or none. Always QoS 2 (Commit).

### JavaScript

```javascript
// Atomic
client.bundle([
  { set: ['/lights/r', 255] },
  { set: ['/lights/g', 0] },
  { emit: ['/cues/go', { scene: 'finale' }] }
]);

// Scheduled (5 seconds from now)
client.bundle([
  { set: ['/lights/brightness', 1.0] }
], { at: client.time() + 5_000_000 });
```

### Rust

```rust
use clasp_client::BundleMessage;

client.bundle(vec![
    BundleMessage::Set("/lights/r".into(), Value::Int(255)),
    BundleMessage::Emit("/cues/go".into(), Value::from_json(r#"{"scene":"finale"}"#)?),
]).await?;
```

## Protocol Bridges

| Bridge | External Address | CLASP Address | Signal Mapping | CLI |
|--------|-----------------|---------------|----------------|-----|
| OSC | `/synth/volume` | `/osc/synth/volume` | message -> Param | `clasp osc --port 9000` |
| MIDI | Ch 1, CC 74 | `/midi/ch/1/cc/74` | CC -> Param, Note -> Event | `clasp midi` |
| MQTT | `sensors/temp` | `/mqtt/sensors/temp` | retained -> Param, non-retained -> Event | `clasp mqtt --host localhost` |
| Art-Net | Universe 1, Ch 1 | `/artnet/1/1` | DMX frame -> Stream | `clasp artnet` |
| DMX | Channel 100 | `/dmx/100` | channel -> Param | `clasp dmx` |
| sACN | Universe 2, Ch 50 | `/sacn/2/50` | DMX frame -> Stream | `clasp sacn` |
| HTTP | `PUT /api/lights/on` | `/lights/on` | GET->Get, PUT->Set, POST->Event | `clasp http --port 8080` |
| WebSocket | JSON message | mapped address | JSON -> Param/Event | `clasp websocket` |
| Socket.IO | event name | mapped address | event -> Event | via bridge config |

Namespace prefix is configurable (e.g., `--osc-namespace /osc`).

## Auth System

### Token Types

| Type | Prefix | Signing | Use Case |
|------|--------|---------|----------|
| CPSK | `cpsk_` | Pre-shared key | Register/login/guest |
| Capability | `cap_` | Ed25519 | Delegatable, offline-verifiable |
| Entity | `ent_` | Ed25519 | Device/service identity |

### CPSK Flow

```bash
# Register
curl -X POST http://localhost:7350/auth/register \
  -d '{"username":"alice","password":"secret","scopes":["read:/**","write:/app/alice/**"]}'
# Returns: { "token": "cpsk_..." }

# Login
curl -X POST http://localhost:7350/auth/login \
  -d '{"username":"alice","password":"secret"}'

# Guest (limited scopes)
curl -X POST http://localhost:7350/auth/guest \
  -d '{"scopes":["read:/**"]}'
```

Connect with token:

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withToken('cpsk_a1b2c3...')
  .connect();
```

### Capability Tokens

Ed25519-signed, UCAN-style, delegatable.

```bash
# Generate keypair
clasp key generate -o signing.key

# Create root token
clasp token cap create -k signing.key -s "write:/lights/**" -e 30d

# Delegate subset
clasp token cap delegate $PARENT -k child.key -s "write:/lights/room1/**" -e 7d

# Verify
clasp token cap verify $TOKEN --trust-anchor signing.key
```

## E2E Encryption

Router never sees plaintext. Clients encrypt/decrypt locally.

### JavaScript

```javascript
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto';

const crypto = new CryptoClient(clasp, {
  identityId: 'device-1',
  store: new MemoryKeyStore(),
});
const session = crypto.session('/myapp/signals', {
  rotationInterval: 3600000,
});
await session.start();
await session.enableEncryption();
```

### Rust

```rust
use clasp_crypto::{E2ESession, E2ESessionConfig, MemoryKeyStore};

let store = Arc::new(MemoryKeyStore::new());
let mut session = E2ESession::new(E2ESessionConfig {
    identity_id: "device-1".into(),
    base_path: "/myapp/signals".into(),
    store,
    on_key_change: None,
    password_hash: None,
    rotation_interval: Some(Duration::from_secs(3600)),
    on_rotation: None,
    max_announcement_age: None,
});
session.start().await?;
session.enable_encryption().await?;
```

### Cryptography

- **Key agreement**: ECDH P-256
- **Key derivation**: HKDF-SHA256
- **Symmetric**: AES-256-GCM
- **Fingerprinting**: SHA-256 (TOFU model)

### Key Exchange Protocol

1. Alice publishes ECDH pubkey to `/_e2e/pubkey/alice`
2. Bob publishes pubkey to `/_e2e/pubkey/bob`
3. Both derive shared secret via ECDH
4. Alice encrypts group key with shared secret, EMITs to `/_e2e/keyex/bob`
5. Bob decrypts group key

### Envelope Format

```json
{ "_e2e": 1, "ct": "<base64 ciphertext>", "iv": "<base64 12-byte IV>", "v": 1 }
```

### Key Storage Backends

| Backend | Platform | Persistence |
|---------|----------|-------------|
| `MemoryKeyStore` | Any | None (testing) |
| `IndexedDBKeyStore` | Browser | IndexedDB |
| `FileSystemKeyStore` | Rust (`fs-store` feature) | JSON files |

## Router Configuration

### RouterConfig Key Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | String | `"CLASP Router"` | Router display name |
| `features` | Vec<String> | `[]` | Feature tags |
| `max_sessions` | usize | `256` | Max concurrent clients |
| `session_timeout` | u64 | `30` | Inactivity timeout (seconds) |
| `security_mode` | SecurityMode | `Open` | `Open` or `Authenticated` |
| `max_subscriptions_per_session` | usize | `0` | Per-session sub limit (0 = unlimited) |
| `gesture_coalescing` | bool | `false` | Coalesce rapid gesture updates |
| `gesture_coalesce_interval_ms` | u64 | `16` | Min interval between coalesced dispatches |
| `max_messages_per_second` | u32 | `0` | Per-session rate limit (0 = unlimited) |
| `rate_limiting_enabled` | bool | `false` | Enable rate limiting |

### SecurityMode

```rust
pub enum SecurityMode {
    Open,           // no auth required
    Authenticated,  // CPSK token required
}
```

### Embedded Router (Rust)

```rust
use clasp_router::{Router, RouterConfigBuilder, SecurityMode};

let router = RouterConfigBuilder::new()
    .name("My Router")
    .max_sessions(128)
    .security_mode(SecurityMode::Authenticated)
    .build();

router.serve_websocket("0.0.0.0:7330").await?;
```

## Server Features

### Rules Engine

Enable: `clasp-relay --rules ./rules.json`

```json
{
  "rules": [{
    "id": "temp-alert",
    "name": "High Temperature Alert",
    "enabled": true,
    "trigger": { "type": "on_threshold", "address": "/sensors/temp", "above": 30.0 },
    "conditions": [
      { "address": "/system/alerts-enabled", "op": "eq", "value": true }
    ],
    "actions": [
      { "type": "set", "address": "/hvac/fan", "value": true },
      { "type": "publish", "address": "/alerts/high-temp", "value": { "msg": "Temperature exceeded 30C" } }
    ],
    "cooldown": 60
  }]
}
```

**Trigger types**: `on_change` (pattern match), `on_threshold` (above/below), `on_event` (event pattern), `on_interval` (seconds).

**Condition operators**: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`.

**Action types**: `set` (fixed value), `publish` (emit event), `set_from_trigger` (copy with transform), `delay` (pause ms).

**Transforms**: `identity`, `scale` (factor + offset), `clamp` (min/max), `threshold` (above/below output), `invert` (1.0 - input).

### Persistence

```bash
clasp-relay --journal /data/state.db
```

- Snapshots + append-only journal (SQLite)
- Clients can query history: `REPLAY` queries
- `MemoryJournal` for testing, `SqliteJournal` for production

### Federation

Hub-leaf topology for multi-site scaling.

```bash
# Hub
clasp-relay --federation-hub --federation-port 7340

# Leaf
clasp-relay --federation-leaf --federation-hub ws://hub:7340
```

State synchronization, conflict resolution, and link management between routers.

### Discovery

- **mDNS**: `_clasp._tcp.local` service, zero-config LAN
- **UDP Broadcast**: port 7331 fallback
- **Rendezvous**: central registry for WAN discovery

```javascript
import { discover } from '@clasp-to/core';
const devices = await discover({ timeout: 3000 });
```

## CLI Commands

| Command | Description | Key Flags |
|---------|-------------|-----------|
| `clasp server` | Start router | `-p` protocol, `-P` port, `-b` bind |
| `clasp pub <ADDR> <VAL>` | Publish value | `-s` server URL |
| `clasp sub [PATTERN]` | Subscribe | `-s` server URL |
| `clasp osc` | OSC bridge | `-p` port (default 9000) |
| `clasp midi` | MIDI bridge | |
| `clasp mqtt` | MQTT bridge | `-H` host, `-p` port |
| `clasp artnet` | Art-Net bridge | |
| `clasp dmx` | DMX bridge | |
| `clasp sacn` | sACN bridge | |
| `clasp http` | HTTP bridge | `-b` bind, `--base-path` |
| `clasp websocket` | WebSocket bridge | `-m` mode (server/client) |
| `clasp bridge` | Generic bridge | `-b` type, `-o` key=value opts |
| `clasp key generate` | Ed25519 keypair | `-o` output file |
| `clasp key show <PATH>` | Show public key | `--format` hex/did |
| `clasp token create` | CPSK token | `-s` scopes, `-e` expires |
| `clasp token list` | List tokens | `--show-expired` |
| `clasp token revoke <TOKEN>` | Revoke token | |
| `clasp token cap create` | Capability token | `-k` key, `-s` scopes |
| `clasp token cap delegate` | Delegate cap | `-k` key, `-s` scopes |
| `clasp token cap verify` | Verify cap | `--trust-anchor` key |
| `clasp token entity keygen` | Entity keypair | `-t` type (device/user/service/router) |
| `clasp token entity mint` | Entity token | `-k` key |
| `clasp info` | Version info | |

Global flags: `-c` config file, `-l` log level, `--json-logs`.

## Transports

| Transport | Crate Feature | Protocol | Use Case |
|-----------|--------------|----------|----------|
| WebSocket | `websocket` (default) | TCP + HTTP upgrade | Standard, browser-compatible |
| QUIC | `quic` | UDP + TLS 1.3 | Low latency, multiplexed |
| TCP | `tcp` | Raw TCP | Simple, reliable |
| UDP | `udp` | Raw UDP | Lowest latency, no guarantees |
| Serial | `serial` | USB/UART | Microcontrollers, hardware |
| BLE | `ble` | Bluetooth LE | Mobile, IoT |
| WebRTC | `p2p` (on clasp-client) | ICE/DTLS/SCTP | Peer-to-peer, NAT traversal |

## Wire Protocol

### Frame Format

```
Byte 0: Magic (0x53 = 'S')
Byte 1: Version (0x01)
Byte 2: Flags / Message Type
Byte 3: Payload Length (low byte)
Bytes 4+: Payload
```

Header: 4 bytes. Max payload: 1024 bytes default (configurable).

### Message Type Codes

| Code | Name | Description |
|------|------|-------------|
| `0x01` | HELLO | Client handshake (name, features, token) |
| `0x02` | WELCOME | Server response (session ID, config) |
| `0x10` | SNAPSHOT | Full state dump for late joiners |
| `0x21` | SET | Set state at address |
| `0x22` | GET | Request state |
| `0x23` | VALUE | Response to GET |
| `0x31` | SUBSCRIBE | Subscribe to pattern |
| `0x32` | UNSUBSCRIBE | Remove subscription |
| `0x41` | EMIT | Fire-and-forget event |
| `0x42` | STREAM | High-rate continuous data |
| `0x43` | GESTURE | Phased interaction signal |
| `0x44` | TIMELINE | Keyframe animation |
| `0x50` | BUNDLE | Atomic message group |

### Value Type Codes

| Code | Type | Size |
|------|------|------|
| `0x00` | Null | 0 bytes |
| `0x01` | Bool | 1 byte |
| `0x04` | I32 | 4 bytes (big-endian) |
| `0x05` | I64 | 8 bytes (big-endian) |
| `0x06` | F32 | 4 bytes (big-endian) |
| `0x07` | F64 | 8 bytes (big-endian) |
| `0x08` | String | length-prefixed |
| `0x09` | Bytes | length-prefixed |
| `0x0A` | Array | nested values |
| `0x0B` | Map | key-value pairs |

### Connection Handshake

1. Client sends `HELLO` (name, features, optional token)
2. Server validates token (if auth enabled)
3. Server sends `WELCOME` (session ID, router config)
4. Server sends `SNAPSHOT` (current state for all subscribed addresses)
5. Normal message exchange begins

## Rust Crates Map

| Crate | Layer | Purpose | Key Exports | Features |
|-------|-------|---------|-------------|----------|
| `clasp-core` | Foundation | Types, codec, addressing | `Message`, `Value`, `SignalType`, `Address`, `ParamState`, `ConflictStrategy` | -- |
| `clasp-transport` | Networking | Network transports | `WebSocketTransport`, `QuicTransport`, `UdpTransport`, `SerialTransport`, `BleTransport` | `websocket`, `quic`, `udp`, `serial`, `ble` |
| `clasp-client` | Application | Async client | `Clasp`, `ClaspBuilder` | `p2p` |
| `clasp-router` | Application | Router impl | `Router`, `RouterConfig`, `RouterState` | -- |
| `clasp-bridge` | Networking | Protocol bridges | `Bridge`, `BridgeConfig`, `AddressMapping` | `osc`, `midi`, `artnet`, `dmx`, `sacn`, `mqtt`, `http`, `websocket` |
| `clasp-discovery` | Networking | Service discovery | `DiscoveryConfig`, `DiscoveryEvent` | -- |
| `clasp-embedded` | Standalone | no_std MCU client | `Client`, `Value`, `MiniRouter` | -- |
| `clasp-caps` | Extension | Ed25519 capability tokens | `CapabilityToken`, `CapabilityValidator` | -- |
| `clasp-registry` | Extension | Entity registry | `Entity`, `EntityStore`, `EntityValidator` | `sqlite` |
| `clasp-rules` | Extension | Rules engine | `Rule`, `RulesEngine`, `Trigger`, `RuleAction` | -- |
| `clasp-journal` | Extension | State persistence | `Journal`, `SqliteJournal`, `MemoryJournal` | `sqlite` |
| `clasp-federation` | Extension | Multi-router federation | `FederationManager`, `FederationConfig`, `FederationLink` | -- |
| `clasp-crypto` | Extension | E2E encryption | `E2ESession`, `CryptoClient`, `MemoryKeyStore`, `FileSystemKeyStore` | `client`, `fs-store` |
| `clasp-wasm` | Standalone | WebAssembly bindings | WASM client | `p2p` |

Typical `Cargo.toml` for a full application:

```toml
[dependencies]
clasp-router = "4.1"
clasp-bridge = { version = "4.1", features = ["osc", "midi", "artnet"] }
clasp-journal = { version = "4.1", features = ["sqlite"] }
clasp-rules = "4.1"
clasp-caps = "4.1"
clasp-crypto = { version = "4.1", features = ["client"] }
```

## Common Patterns

### Late-Joiner Sync

Clients receive a SNAPSHOT on connect containing all current param values. No extra code needed -- `subscribe` callbacks fire with cached state immediately.

```javascript
// Late joiner automatically gets current brightness
client.subscribe('/lights/brightness', (addr, val) => {
  updateUI(val); // fires immediately with cached value, then on changes
});
```

### Bridge Chaining

External protocol -> CLASP -> web UI:

```bash
# Terminal 1: Start router
clasp server

# Terminal 2: Bridge OSC into CLASP
clasp osc --port 9000 --namespace /osc

# Terminal 3: JS client subscribes to bridged signals
```

```javascript
client.subscribe('/osc/**', (addr, val) => {
  console.log('OSC signal:', addr, val);
});
```

### E2E Encrypted Group Room

```javascript
import { CryptoClient, MemoryKeyStore } from '@clasp-to/crypto';

const crypto = new CryptoClient(clasp, {
  identityId: 'alice',
  store: new MemoryKeyStore(),
});

const room = crypto.session('/rooms/private', {
  rotationInterval: 3600000,
  passwordHash: 'sha256-of-shared-password',
});
await room.start();
await room.enableEncryption();

// All set/emit within /rooms/private/** are now encrypted
```

### Scheduled Cue Execution

```javascript
const cueTime = client.time() + 10_000_000; // 10 seconds from now

client.bundle([
  { set: ['/lights/scene', 'dramatic'] },
  { set: ['/audio/track', 'thunder.wav'] },
  { emit: ['/cues/go', { name: 'storm' }] }
], { at: cueTime });
```

### Reconnection with State Recovery

```javascript
const client = await new ClaspBuilder('ws://localhost:7330')
  .withName('Resilient App')
  .withReconnect(true)
  .withReconnectInterval(3000)
  .connect();

client.onReconnect(() => {
  console.log('Reconnected -- subscriptions auto-restored, fresh snapshot received');
});
```

```rust
let client = ClaspBuilder::new("ws://localhost:7330")
    .name("Resilient App")
    .reconnect(true)
    .reconnect_interval(3000)
    .connect()
    .await?;
```

On reconnect, all subscriptions are re-established and the router sends a fresh SNAPSHOT.
