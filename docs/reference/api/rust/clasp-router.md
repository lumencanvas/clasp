# clasp-router (Rust)

CLASP router library for building routers and servers.

## Overview

`clasp-router` provides a high-performance router implementation.

```toml
[dependencies]
clasp-router = "3.0"
tokio = { version = "1", features = ["full"] }
```

## Quick Start

```rust
use clasp_router::{Router, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .port(7330)
        .build();

    let router = Router::new(config).await?;
    router.run().await?;

    Ok(())
}
```

## Configuration

### Basic Config

```rust
use clasp_router::Config;

let config = Config::builder()
    .port(7330)
    .bind_address("0.0.0.0")
    .build();
```

### Full Config

```rust
let config = Config::builder()
    // Network
    .port(7330)
    .bind_address("0.0.0.0")

    // TLS
    .tls_cert("/path/to/cert.pem")
    .tls_key("/path/to/key.pem")

    // Limits
    .max_connections(10000)
    .max_message_size(64 * 1024)
    .max_subscriptions_per_client(1000)

    // Security
    .require_auth(true)
    .token_secret("your-secret")

    // Discovery
    .mdns_enabled(true)
    .mdns_name("My Router")

    // Performance
    .worker_threads(8)

    .build();
```

### From File

```rust
let config = Config::from_file("clasp.yaml")?;
```

## Router Operations

### Create and Run

```rust
let router = Router::new(config).await?;

// Run in foreground (blocks)
router.run().await?;

// Or run in background
let handle = tokio::spawn(async move {
    router.run().await
});
```

### Graceful Shutdown

```rust
use tokio::signal;

let router = Router::new(config).await?;
let router_clone = router.clone();

// Run in background
tokio::spawn(async move {
    router_clone.run().await
});

// Wait for shutdown signal
signal::ctrl_c().await?;

// Graceful shutdown
router.shutdown().await?;
```

## Local Client

Get a client connected directly to the router (no network overhead):

```rust
let router = Router::new(config).await?;
let client = router.local_client();

// Use like any other client
client.set("/local/value", 42).await?;

let value = client.get("/local/value").await?;
```

## Direct State Access

Access router state without going through the client interface:

```rust
let router = Router::new(config).await?;

// Read state
let value = router.state().get("/path/to/value").await?;

// Write state
router.state().set("/path/to/value", Value::Int(42)).await?;

// List addresses
let addresses = router.state().list("/sensors/**").await?;

// Delete
router.state().delete("/path/to/value").await?;
```

## Event Hooks

### Connection Events

```rust
let router = Router::new(config).await?;

router.on_client_connect(|client_id, info| {
    println!("Client connected: {} from {}", client_id, info.remote_addr);
});

router.on_client_disconnect(|client_id, reason| {
    println!("Client disconnected: {} - {:?}", client_id, reason);
});
```

### Message Events

```rust
router.on_message(|msg, client_id| {
    // Log all messages
    tracing::debug!("From {}: {:?}", client_id, msg);
});

// Filter by type
router.on_set(|address, value, client_id| {
    println!("{} set {} = {:?}", client_id, address, value);
});
```

## Message Handler

Intercept and modify messages:

```rust
use clasp_router::{Router, Config, MessageHandler, Message};
use async_trait::async_trait;

struct MyHandler;

#[async_trait]
impl MessageHandler for MyHandler {
    async fn handle(&self, msg: &Message, client_id: &str) -> Option<Message> {
        // Return None to pass through unchanged
        // Return Some(msg) to modify
        // Return Some(error) to reject

        if msg.address().unwrap_or("").starts_with("/admin/") {
            // Block admin routes from non-admin clients
            if !is_admin(client_id) {
                return Some(Message::Error(ErrorMessage {
                    id: msg.id(),
                    code: 403,
                    message: "Permission denied".into(),
                }));
            }
        }

        None  // Pass through
    }
}

let config = Config::builder()
    .port(7330)
    .message_handler(Arc::new(MyHandler))
    .build();
```

## Persistence

### Enable State Persistence

```rust
use clasp_router::PersistenceConfig;

let config = Config::builder()
    .port(7330)
    .persistence(PersistenceConfig {
        enabled: true,
        backend: PersistenceBackend::Sqlite,
        path: "/var/lib/clasp/state.db".into(),
        sync_interval: Duration::from_secs(5),
    })
    .build();
```

### Manual State Operations

```rust
// Save state manually
router.save_state().await?;

// Load state
router.load_state().await?;

// Clear persisted state
router.clear_state().await?;
```

## Statistics

```rust
let stats = router.stats();

println!("Connections: {}", stats.connections);
println!("Messages/sec: {}", stats.messages_per_second);
println!("State entries: {}", stats.state_entries);
println!("Subscriptions: {}", stats.subscriptions);
println!("Uptime: {:?}", stats.uptime);
```

## Metrics

### Prometheus Metrics

```rust
let config = Config::builder()
    .port(7330)
    .metrics_enabled(true)
    .metrics_port(9090)
    .build();

// Metrics available at http://localhost:9090/metrics
```

### Custom Metrics

```rust
use clasp_router::metrics;

// Register custom metric
metrics::register_counter("my_custom_counter", "Description");

// Increment
metrics::counter_inc("my_custom_counter");

// Gauge
metrics::register_gauge("my_gauge", "Description");
metrics::gauge_set("my_gauge", 42.0);
```

## Security

### Token Validation

```rust
let config = Config::builder()
    .port(7330)
    .require_auth(true)
    .token_secret("your-256-bit-secret")
    .build();

// Or with public key (RS256)
let config = Config::builder()
    .port(7330)
    .require_auth(true)
    .token_public_key("/path/to/public.pem")
    .build();
```

### Custom Authentication

```rust
use clasp_router::{Authenticator, AuthResult};

struct MyAuthenticator;

#[async_trait]
impl Authenticator for MyAuthenticator {
    async fn authenticate(&self, token: &str) -> AuthResult {
        // Validate token
        if validate_token(token) {
            AuthResult::Ok(Permissions {
                read: vec!["/**".into()],
                write: vec!["/user/**".into()],
                emit: vec![],
                subscribe: vec!["/**".into()],
            })
        } else {
            AuthResult::Denied("Invalid token".into())
        }
    }
}

let config = Config::builder()
    .port(7330)
    .authenticator(Arc::new(MyAuthenticator))
    .build();
```

## Clustering

### Multi-Router Setup

```rust
// Router 1
let config1 = Config::builder()
    .port(7330)
    .cluster_enabled(true)
    .cluster_peers(vec!["192.168.1.101:7330".into()])
    .build();

// Router 2
let config2 = Config::builder()
    .port(7330)
    .cluster_enabled(true)
    .cluster_peers(vec!["192.168.1.100:7330".into()])
    .build();
```

## Error Handling

```rust
use clasp_router::Error;

match router.run().await {
    Ok(()) => println!("Router stopped"),
    Err(Error::BindError(e)) => eprintln!("Failed to bind: {}", e),
    Err(Error::TlsError(e)) => eprintln!("TLS error: {}", e),
    Err(e) => eprintln!("Error: {:?}", e),
}
```

## See Also

- [clasp-core](clasp-core.md) - Core types
- [clasp-client](clasp-client.md) - Client library
- [Embed Router](../../../how-to/advanced/embed-router.md)
- [Start Router](../../../how-to/connections/start-router.md)
