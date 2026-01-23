# Embed Router

Embed a CLASP router directly into your application.

## Overview

Instead of running a separate router process, you can embed the router in your application. This is useful for:

- Single-binary deployments
- Custom routing logic
- Tight integration with application state
- Reduced operational complexity

## Rust

### Basic Embedded Router

```rust
use clasp_router::{Router, Config};
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure router
    let config = Config::builder()
        .port(7330)
        .build();

    // Create and run router
    let router = Router::new(config).await?;

    // Router runs in background
    let router_handle = tokio::spawn(async move {
        router.run().await
    });

    // Your application logic here
    println!("Application running with embedded router");

    // Wait for router (or handle shutdown)
    router_handle.await??;
    Ok(())
}
```

### Router with Local Client

```rust
use clasp_router::{Router, Config};
use clasp_client::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::builder()
        .port(7330)
        .build();

    let router = Router::new(config).await?;

    // Get local client (no network overhead)
    let client = router.local_client();

    // Start router in background
    tokio::spawn(async move {
        router.run().await
    });

    // Use client directly
    client.set("/app/status", "running").await?;

    client.on("/control/**", |value, address| async move {
        println!("Received: {} = {:?}", address, value);
    }).await;

    Ok(())
}
```

### Custom Message Handling

```rust
use clasp_router::{Router, Config, Message, MessageHandler};

struct MyHandler {
    // Your application state
    counter: AtomicU64,
}

impl MessageHandler for MyHandler {
    async fn handle(&self, msg: &Message) -> Option<Message> {
        // Intercept and modify messages
        if msg.address.starts_with("/metrics/") {
            self.counter.fetch_add(1, Ordering::SeqCst);
        }

        // Return None to pass through, Some to modify/block
        None
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let handler = Arc::new(MyHandler {
        counter: AtomicU64::new(0),
    });

    let config = Config::builder()
        .port(7330)
        .message_handler(handler.clone())
        .build();

    let router = Router::new(config).await?;
    router.run().await
}
```

## JavaScript (Node.js)

### Basic Embedded Router

```javascript
const { Router, Client } = require('@clasp-to/router');

async function main() {
  // Create router
  const router = new Router({
    port: 7330
  });

  await router.start();
  console.log('Router started on port 7330');

  // Get local client
  const client = router.localClient();

  // Use like any other client
  await client.set('/app/status', 'running');

  client.on('/control/**', (value, address) => {
    console.log(`${address} = ${value}`);
  });

  // Your application logic
  setInterval(async () => {
    await client.set('/app/uptime', process.uptime());
  }, 1000);
}

main();
```

### Express Integration

```javascript
const express = require('express');
const { Router } = require('@clasp-to/router');

async function main() {
  const app = express();

  // Create embedded router
  const router = new Router({ port: 7330 });
  await router.start();

  const clasp = router.localClient();

  // HTTP endpoints that use CLASP
  app.get('/api/state/:address', async (req, res) => {
    const value = await clasp.get('/' + req.params.address);
    res.json({ value });
  });

  app.post('/api/state/:address', express.json(), async (req, res) => {
    await clasp.set('/' + req.params.address, req.body.value);
    res.json({ success: true });
  });

  // WebSocket upgrade handled by router
  app.listen(3000, () => {
    console.log('HTTP on 3000, CLASP on 7330');
  });
}

main();
```

## Configuration Options

### Memory Limits

```rust
let config = Config::builder()
    .port(7330)
    .max_connections(1000)
    .max_message_size(64 * 1024)  // 64KB
    .max_subscriptions_per_client(100)
    .build();
```

### State Persistence

```rust
let config = Config::builder()
    .port(7330)
    .persistence(PersistenceConfig {
        enabled: true,
        path: "/var/lib/clasp/state.db",
        sync_interval: Duration::from_secs(5),
    })
    .build();
```

### Security

```rust
let config = Config::builder()
    .port(7330)
    .tls_cert("/path/to/cert.pem")
    .tls_key("/path/to/key.pem")
    .require_auth(true)
    .token_secret("your-secret")
    .build();
```

## Accessing Router State

### Direct State Access

```rust
let router = Router::new(config).await?;

// Read state directly (no network)
let value = router.state().get("/sensors/temp").await?;

// Write state directly
router.state().set("/app/config", config_value).await?;

// List addresses
let addresses = router.state().list("/sensors/**").await?;
```

### Subscribe to Internal Events

```rust
router.on_client_connect(|client_id| {
    println!("Client connected: {}", client_id);
});

router.on_client_disconnect(|client_id| {
    println!("Client disconnected: {}", client_id);
});

router.on_message(|msg| {
    // Log all messages
    log::debug!("{}: {:?}", msg.address, msg.value);
});
```

## Graceful Shutdown

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    let router = Router::new(config).await?;

    let router_clone = router.clone();
    tokio::spawn(async move {
        router_clone.run().await
    });

    // Wait for shutdown signal
    signal::ctrl_c().await?;

    // Graceful shutdown
    router.shutdown().await?;

    Ok(())
}
```

```javascript
const router = new Router({ port: 7330 });
await router.start();

process.on('SIGTERM', async () => {
  console.log('Shutting down...');
  await router.stop();
  process.exit(0);
});
```

## Multi-Router Setup

Run multiple routers in one process:

```rust
let main_router = Router::new(Config::builder()
    .port(7330)
    .build()).await?;

let secondary_router = Router::new(Config::builder()
    .port(7331)
    .build()).await?;

// Bridge between routers
let main_client = main_router.local_client();
let secondary_client = secondary_router.local_client();

main_client.on("/forward/**", move |value, address| {
    let addr = address.replace("/forward", "");
    secondary_client.set(&addr, value)
}).await;
```

## Performance Considerations

### Local Client Optimization

Local clients bypass network stack:

```rust
// Network client: serialize → network → deserialize → process
let network_client = Client::connect("ws://localhost:7330").await?;

// Local client: direct memory access
let local_client = router.local_client();
// ~10x faster for local operations
```

### Shared State

```rust
// Share state between router and application
let shared_state = Arc::new(RwLock::new(AppState::default()));

let state_clone = shared_state.clone();
router.on_message(move |msg| {
    if msg.address.starts_with("/app/") {
        let mut state = state_clone.write().unwrap();
        state.update(&msg);
    }
});
```

## Next Steps

- [Performance Tuning](performance-tuning.md)
- [Custom Bridge](custom-bridge.md)
- [Router Reference](../../reference/api/rust/clasp-router.md)
