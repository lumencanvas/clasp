---
title: Rust SDK
description: Build CLASP clients with Rust
order: 3
---

# Rust SDK

The `clasp-client` crate provides an async CLASP client built on tokio. It supports the builder pattern, closure-based subscriptions, all five signal types, automatic reconnection, and optional P2P connectivity.

## Installation

Add the crate to your `Cargo.toml`:

```bash
cargo add clasp-client
```

Or manually:

```toml
[dependencies]
clasp-client = "3.5"
tokio = { version = "1", features = ["full"] }
```

## Connecting

### Builder Pattern

```rust
use clasp_client::ClaspBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ClaspBuilder::new("ws://localhost:7330")
        .name("Sensor Hub")
        .reconnect(true)
        .reconnect_interval(3000)
        .features(vec!["state".into(), "events".into()])
        .connect()
        .await?;

    println!("Connected, session: {}", client.session_id());
    Ok(())
}
```

### Quick Connect

For simple cases, skip the builder:

```rust
use clasp_client::Clasp;

let client = Clasp::connect_to("ws://localhost:7330").await?;
```

### Builder Methods

| Method | Description |
|---|---|
| `ClaspBuilder::new(url)` | Create a builder with the router URL |
| `.name(s)` | Client display name |
| `.token(s)` | CPSK auth token |
| `.reconnect(bool)` | Enable auto-reconnect |
| `.reconnect_interval(ms)` | Delay between reconnect attempts (milliseconds) |
| `.features(vec)` | Requested feature set |
| `.connect()` | Connect and return `Result<Clasp>` |

## Setting and Getting State

State is the primary data model in CLASP. Values set at an address persist on the router and are delivered to late joiners.

```rust
use clasp_client::Value;

// Write state
client.set("/lights/brightness", Value::Float(0.8)).await?;
client.set("/lights/color", Value::from_json(r#"{"r":255,"g":100,"b":0}"#)?).await?;

// Read state from the router (async round-trip)
let brightness = client.get("/lights/brightness").await?;
println!("Brightness: {:?}", brightness);

// Read from local cache (no network call)
if let Some(cached) = client.cached("/lights/brightness") {
    println!("Cached: {:?}", cached);
}
```

## Subscriptions

Subscribe to addresses or wildcard patterns with closure callbacks:

```rust
// Exact address
client.subscribe("/lights/brightness", |value, address| {
    println!("Brightness changed: {:?}", value);
}).await?;

// Single-level wildcard
client.subscribe("/sensors/*", |value, address| {
    println!("{}: {:?}", address, value);
}).await?;

// Multi-level wildcard
client.subscribe("/sensors/**", |value, address| {
    println!("Deep sensor {}: {:?}", address, value);
}).await?;

// Unsubscribe
client.unsubscribe("/sensors/*").await?;
```

### Wildcard Patterns

| Pattern | Matches | Example Match |
|---|---|---|
| `/sensors/temperature` | Exact address | `/sensors/temperature` |
| `/sensors/*` | Any single level under `/sensors/` | `/sensors/humidity` |
| `/sensors/**` | Any depth under `/sensors/` | `/sensors/room/1/temp` |

## Signal Types

CLASP defines five signal types. `set()` handles persistent state (Param signals). The other four are for transient signals.

### Events

Fire-and-forget notifications. Not stored as state, not delivered to late joiners.

```rust
client.emit("/alerts/motion-detected", Value::from_json(
    r#"{"zone":"lobby","confidence":0.95}"#
)?).await?;

client.emit("/cues/go", Value::Null).await?;
```

### Streams

High-rate continuous data.

```rust
loop {
    let level = read_audio_level();
    client.stream("/audio/level", Value::Float(level)).await?;
    tokio::time::sleep(Duration::from_millis(16)).await; // ~60Hz
}
```

### Gestures

Phased interaction signals with a lifecycle: `begin`, `update`, `end`.

```rust
client.gesture(
    "/input/fader", "fader-1", "begin",
    Value::from_json(r#"{"value":0.0}"#)?
).await?;

client.gesture(
    "/input/fader", "fader-1", "update",
    Value::from_json(r#"{"value":0.5}"#)?
).await?;

client.gesture(
    "/input/fader", "fader-1", "end",
    Value::Null
).await?;
```

### Timelines

Keyframe-based animations executed by the router.

```rust
client.timeline("/lights/brightness", Value::from_json(r#"[
    {"time": 0, "value": 0.0},
    {"time": 1000, "value": 1.0},
    {"time": 3000, "value": 1.0},
    {"time": 4000, "value": 0.0}
]"#)?).await?;
```

## Bundles

Group multiple operations into a single message. Bundles are delivered atomically.

```rust
use clasp_client::BundleMessage;

client.bundle(vec![
    BundleMessage::Set("/lights/1/brightness".into(), Value::Float(0.8)),
    BundleMessage::Set("/lights/2/brightness".into(), Value::Float(0.6)),
    BundleMessage::Emit("/cues/scene-change".into(), Value::from_json(
        r#"{"scene":"Act 2"}"#
    )?),
]).await?;
```

## Events

Register callbacks for connection lifecycle events:

```rust
client.on_connect(|| {
    println!("Connected");
}).await;

client.on_disconnect(|| {
    println!("Disconnected");
}).await;

client.on_error(|err| {
    eprintln!("Error: {:?}", err);
}).await;
```

## Auth

Pass a CPSK token to authenticate with the router:

```rust
let client = ClaspBuilder::new("ws://localhost:7330")
    .name("Secure Client")
    .token("cpsk_a1b2c3d4e5f6...")
    .connect()
    .await?;
```

The router validates the token during the HELLO handshake. If the token is invalid or lacks required scopes, `connect()` returns an `Err`. See [Auth](../auth/README.md) for token generation and scope configuration.

## Feature Flags

The `clasp-client` crate supports optional features via Cargo feature flags:

```toml
[dependencies]
clasp-client = { version = "3.5", features = ["p2p"] }
```

| Feature | Description |
|---|---|
| `p2p` | Enable peer-to-peer WebRTC connectivity via `p2p_manager()` |

### P2P

When the `p2p` feature is enabled, access the P2P manager for direct peer connections:

```rust
let p2p = client.p2p_manager();
// Use p2p for direct peer-to-peer communication
```

## Error Handling

All async operations return `Result` types. Handle errors with standard Rust patterns:

```rust
use clasp_client::{Clasp, ClaspBuilder, ClaspError};

match ClaspBuilder::new("ws://localhost:7330")
    .name("My App")
    .connect()
    .await
{
    Ok(client) => {
        println!("Connected: {}", client.session_id());
    }
    Err(ClaspError::ConnectionRefused) => {
        eprintln!("Router is not running");
    }
    Err(ClaspError::AuthFailed(reason)) => {
        eprintln!("Auth failed: {}", reason);
    }
    Err(e) => {
        eprintln!("Unexpected error: {:?}", e);
    }
}
```

Connection state can be checked at any time:

```rust
if client.connected() {
    client.set("/status/alive", Value::Bool(true)).await?;
}
```

## Time

Access synchronized server time:

```rust
let server_time = client.time(); // microseconds
```

## Reconnection & Connection Lifecycle

The Rust client supports auto-reconnect via the builder:

```rust
let client = ClaspBuilder::new("ws://localhost:7330")
    .name("Resilient App")
    .reconnect(true)
    .reconnect_interval(3000) // milliseconds
    .connect()
    .await?;

client.on_connect(|| {
    println!("Connected (or reconnected)");
}).await;

client.on_disconnect(|| {
    println!("Disconnected, auto-reconnecting...");
}).await;

client.on_error(|err| {
    eprintln!("Error: {:?}", err);
}).await;
```

On successful reconnect, all subscriptions are re-established and the router sends a fresh SNAPSHOT. Call `client.close()` to disconnect without triggering auto-reconnect.

## Examples

Working examples in `examples/rust/`:

| File | Description |
|------|-------------|
| `basic-client.rs` | Basic connection and value publishing |
| `signal_types.rs` | All five signal types in action |
| `bundles_and_scheduling.rs` | Atomic and scheduled bundles |
| `p2p_webrtc.rs` | Peer-to-peer communication via WebRTC |
| `late_joiner.rs` | Late-joiner state synchronization |
| `security_tokens.rs` | CPSK token authentication |
| `embedded-server.rs` | Embedded CLASP router |

Run any example with:

```bash
cargo run --example basic-client
```

## Next Steps

- [Core Concepts](../concepts/architecture.md) -- understand signals, state, and the router model
- [Protocol Bridges](../protocols/README.md) -- connect CLASP to OSC, MIDI, MQTT, and more
- [Auth](../auth/README.md) -- CPSK tokens and capability delegation
- [P2P & WebRTC](../core/p2p.md) -- direct peer-to-peer connections
- [Embedded SDK](embedded.md) -- run CLASP on microcontrollers
- [JavaScript SDK](javascript.md) -- build CLASP clients with JavaScript
- [Python SDK](python.md) -- build CLASP clients with Python
