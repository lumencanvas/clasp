# clasp-client (Rust)

CLASP client library for Rust applications.

## Overview

`clasp-client` provides an async client for connecting to CLASP routers.

```toml
[dependencies]
clasp-client = "3.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use clasp_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::connect("ws://localhost:7330").await?;

    // Set a value
    client.set("/sensors/temp", 23.5).await?;

    // Get a value
    let value = client.get("/sensors/temp").await?;
    println!("Temperature: {:?}", value);

    // Subscribe to changes
    client.on("/sensors/**", |value, address| async move {
        println!("{}: {:?}", address, value);
    }).await;

    Ok(())
}
```

## Client Builder

### Basic Connection

```rust
use clasp_client::Client;

let client = Client::connect("ws://localhost:7330").await?;
```

### With Options

```rust
use clasp_client::Client;
use std::time::Duration;

let client = Client::builder("ws://localhost:7330")
    .name("my-rust-client")
    .timeout(Duration::from_secs(10))
    .connect()
    .await?;
```

### With Authentication

```rust
let client = Client::builder("wss://router.example.com:7330")
    .token("eyJhbGciOi...")
    .connect()
    .await?;
```

### With TLS

```rust
use clasp_client::{Client, TlsConfig};

let tls = TlsConfig::builder()
    .add_root_certificate(cert)
    .build()?;

let client = Client::builder("wss://localhost:7330")
    .tls_config(tls)
    .connect()
    .await?;
```

### With Auto-Reconnect

```rust
let client = Client::builder("ws://localhost:7330")
    .auto_reconnect(true)
    .reconnect_interval(Duration::from_secs(1))
    .max_reconnect_attempts(10)
    .connect()
    .await?;
```

## Core Operations

### Set

```rust
// Set with automatic type conversion
client.set("/path/to/value", 42).await?;
client.set("/path/to/value", "hello").await?;
client.set("/path/to/value", true).await?;

// Set with explicit Value
use clasp_core::Value;
client.set("/path/to/value", Value::Float(3.14)).await?;

// Set with struct (requires serde feature)
#[derive(Serialize)]
struct Data { x: f64, y: f64 }
client.set("/position", Data { x: 1.0, y: 2.0 }).await?;
```

### Get

```rust
// Get raw value
let value: Value = client.get("/path/to/value").await?;

// Get with type conversion
let temp: f64 = client.get_as("/sensors/temp").await?;
let name: String = client.get_as("/device/name").await?;

// Get with default
let value = client.get_or("/config/timeout", 30).await;
```

### Emit (Events)

```rust
// Emit event
client.emit("/events/button_pressed", json!({ "button": 1 })).await?;

// Emit without payload
client.emit("/events/ping", Value::Null).await?;
```

### Delete

```rust
client.delete("/path/to/value").await?;
```

### List

```rust
// List all addresses matching pattern
let addresses = client.list("/sensors/**").await?;
for addr in addresses {
    println!("{}", addr);
}
```

## Subscriptions

### Subscribe to Pattern

```rust
// Subscribe with closure
client.on("/sensors/**", |value, address| async move {
    println!("{}: {:?}", address, value);
}).await;

// Subscribe with function
async fn handle_sensor(value: Value, address: String) {
    println!("{}: {:?}", address, value);
}

client.on("/sensors/**", handle_sensor).await;
```

### Subscription Options

```rust
use clasp_client::SubscribeOptions;

client.on_with_options(
    "/sensors/**",
    |value, address| async move { /* ... */ },
    SubscribeOptions {
        max_rate: Some(30.0),      // Max 30 updates/sec
        debounce: Some(100),       // 100ms debounce
        include_initial: true,     // Receive current value immediately
    }
).await;
```

### Unsubscribe

```rust
// Store subscription handle
let handle = client.on("/sensors/**", handler).await;

// Unsubscribe when done
handle.unsubscribe().await;
```

### One-Time Subscription

```rust
// Wait for single value
let value = client.once("/events/ready").await?;
```

## Bundles

### Atomic Operations

```rust
use clasp_client::BundleBuilder;

client.bundle()
    .set("/lights/1", 255)
    .set("/lights/2", 128)
    .set("/lights/3", 64)
    .emit("/cue/fired", json!({ "cue": 1 }))
    .execute()
    .await?;
```

### Scheduled Bundle

```rust
use std::time::{SystemTime, UNIX_EPOCH};

let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_millis() as u64 + 5000;  // 5 seconds from now

client.bundle()
    .set("/lights/1", 255)
    .set("/lights/2", 255)
    .at_time(timestamp)
    .execute()
    .await?;
```

## QoS Levels

```rust
use clasp_core::QoS;

// Fire and forget (default)
client.set("/data", value).await?;

// With confirmation
client.set_with_qos("/data", value, QoS::Confirm).await?;

// Exactly-once delivery
client.set_with_qos("/critical", value, QoS::Commit).await?;
```

## Streams

For high-rate continuous data:

```rust
use tokio::time::{interval, Duration};

// Create stream
let stream = client.stream("/audio/level").await?;

// Send at high rate
let mut ticker = interval(Duration::from_millis(10));
loop {
    ticker.tick().await;
    let level = get_audio_level();
    stream.send(level).await?;
}
```

## Gestures

For phased interactions:

```rust
// Begin gesture
let gesture = client.gesture_begin("/draw/stroke", json!({
    "x": 100, "y": 100
})).await?;

// Update during gesture
gesture.update(json!({ "x": 150, "y": 120 })).await?;
gesture.update(json!({ "x": 200, "y": 150 })).await?;

// End gesture
gesture.end(json!({ "x": 250, "y": 180 })).await?;
```

## Connection Events

```rust
client.on_connected(|| {
    println!("Connected to router");
});

client.on_disconnected(|reason| {
    println!("Disconnected: {:?}", reason);
});

client.on_error(|error| {
    eprintln!("Error: {:?}", error);
});
```

## Connection State

```rust
// Check if connected
if client.is_connected() {
    // ...
}

// Wait for connection
client.wait_connected().await?;

// Ping router
let latency = client.ping().await?;
println!("Latency: {:?}", latency);
```

## Clock Synchronization

```rust
// Sync clock with router
client.sync_clock().await?;

// Get synchronized time
let synced_time = client.synced_time()?;
```

## Locks

```rust
// Acquire lock
let lock = client.lock("/exclusive/resource").await?;

// Use the locked resource
client.set("/exclusive/resource", value).await?;

// Release lock
lock.release().await?;

// Or use RAII guard
{
    let _lock = client.lock("/exclusive/resource").await?;
    // Lock released when _lock goes out of scope
}
```

## Graceful Shutdown

```rust
// Disconnect gracefully
client.disconnect().await?;

// Or with drop
drop(client);
```

## Error Handling

```rust
use clasp_client::Error;

match client.get("/path").await {
    Ok(value) => println!("{:?}", value),
    Err(Error::NotFound(addr)) => println!("Not found: {}", addr),
    Err(Error::PermissionDenied) => println!("Access denied"),
    Err(Error::Timeout) => println!("Request timed out"),
    Err(e) => println!("Error: {:?}", e),
}
```

## See Also

- [clasp-core](clasp-core.md) - Core types
- [clasp-router](clasp-router.md) - Router library
- [Connect Client](../../../how-to/connections/connect-client.md)
