---
title: Router Config
description: RouterConfig reference for embedding CLASP in Rust applications
order: 7
---

# Router Config

Reference for the `RouterConfig` struct and `Router` type used when embedding a CLASP router directly in a Rust application. For the relay server CLI (which wraps the router with configuration files and auth), see [Relay CLI Reference](relay-cli.md).

## RouterConfig Fields

| Field                            | Type           | Default            | Description                                                       |
|----------------------------------|----------------|--------------------|-------------------------------------------------------------------|
| `name`                           | `String`       | `"CLASP Router"`   | Human-readable router name, advertised during discovery           |
| `features`                       | `Vec<String>`  | `[]`               | Feature tags advertised to clients (e.g., `["lighting", "audio"]`)|
| `max_sessions`                   | `usize`        | `256`              | Maximum concurrent client sessions                                |
| `session_timeout`                | `u64`          | `30`               | Seconds of inactivity before a session is closed                  |
| `security_mode`                  | `SecurityMode` | `Open`             | Authentication mode: `Open` or `Authenticated`                    |
| `max_subscriptions_per_session`  | `usize`        | `0`                | Maximum subscriptions per session. `0` means unlimited.           |
| `gesture_coalescing`             | `bool`         | `false`            | Whether to coalesce rapid gesture updates into fewer dispatches   |
| `gesture_coalesce_interval_ms`   | `u64`          | `16`               | Minimum interval in ms between coalesced gesture dispatches       |
| `max_messages_per_second`        | `u32`          | `0`                | Per-session rate limit. `0` means unlimited.                      |
| `rate_limiting_enabled`          | `bool`         | `false`            | Whether per-session rate limiting is enforced                     |

## RouterConfigBuilder

The builder pattern is the recommended way to construct a `RouterConfig`. Obtain a builder from `Router::builder()`.

```rust
use clasp_router::{Router, SecurityMode};

let router = Router::builder()
    .name("Stage Controller")
    .max_sessions(50)
    .session_timeout(60)
    .security_mode(SecurityMode::Authenticated)
    .max_subscriptions_per_session(100)
    .gesture_coalescing(true)
    .gesture_coalesce_interval_ms(8)
    .max_messages_per_second(1000)
    .rate_limiting_enabled(true)
    .build();
```

### Builder Methods

| Method                              | Parameter          | Description                                     |
|-------------------------------------|--------------------|-------------------------------------------------|
| `name()`                            | `impl Into<String>`| Set the router name                             |
| `max_sessions()`                    | `usize`            | Set maximum concurrent sessions                 |
| `session_timeout()`                 | `u64`              | Set session timeout in seconds                  |
| `security_mode()`                   | `SecurityMode`     | Set authentication mode                         |
| `max_subscriptions_per_session()`   | `usize`            | Set per-session subscription limit              |
| `gesture_coalescing()`              | `bool`             | Enable or disable gesture coalescing            |
| `gesture_coalesce_interval_ms()`    | `u64`              | Set coalescing interval in milliseconds         |
| `max_messages_per_second()`         | `u32`              | Set per-session message rate limit              |
| `rate_limiting_enabled()`           | `bool`             | Enable or disable rate limiting                 |
| `build()`                           | --                 | Consume the builder, return a `Router`          |

All builder methods return `&mut Self` for chaining, except `build()` which consumes the builder and returns a `Router` instance.

## SecurityMode

```rust
pub enum SecurityMode {
    Open,
    Authenticated,
}
```

| Variant         | Description                                                                                   |
|-----------------|-----------------------------------------------------------------------------------------------|
| `Open`          | No authentication required. Any client may connect and access all addresses.                  |
| `Authenticated` | Clients must present a valid CPSK capability token on connect. The token's scopes restrict which addresses can be read or written. Use `set_write_validator()` to enforce custom write policies. |

## Router Methods

### Construction

| Method                  | Signature                                       | Description                             |
|-------------------------|--------------------------------------------------|-----------------------------------------|
| `Router::new(config)`   | `fn new(config: RouterConfig) -> Router`         | Create a router from a config struct    |
| `Router::builder()`     | `fn builder() -> RouterConfigBuilder`            | Create a builder with defaults          |

### Serving

| Method              | Signature                                                                    | Description                                    |
|---------------------|------------------------------------------------------------------------------|------------------------------------------------|
| `serve_websocket()` | `async fn serve_websocket(&self, addr: &str) -> Result<()>`                 | Listen for WebSocket connections on `addr`      |
| `serve_quic()`      | `async fn serve_quic(&self, addr: &str, cert: &Path, key: &Path) -> Result<()>` | Listen for QUIC connections with TLS        |
| `serve_on()`        | `async fn serve_on(&self, server: impl Transport) -> Result<()>`             | Listen on a custom transport implementation    |
| `serve_multi()`     | `async fn serve_multi(&self, configs: Vec<ServeConfig>) -> Result<()>`       | Listen on multiple transports concurrently     |

### State and Extensibility

| Method                  | Signature                                                                       | Description                                                    |
|-------------------------|---------------------------------------------------------------------------------|----------------------------------------------------------------|
| `state()`               | `fn state(&self) -> &RouterState`                                               | Access the router's parameter state store                      |
| `set_write_validator()` | `fn set_write_validator(&self, f: impl Fn(&Session, &str, &Value) -> bool)`     | Set a callback that approves or rejects writes by address      |
| `set_snapshot_filter()`  | `fn set_snapshot_filter(&self, f: impl Fn(&Session, &str, &Value) -> bool)`     | Filter which parameters are included in snapshots for a session|
| `set_rules_engine()`    | `fn set_rules_engine(&self, rules: RulesEngine)`                                | Attach a rules engine for reactive automation                  |
| `set_journal()`         | `fn set_journal(&self, journal: impl Journal)`                                  | Attach a journal for state persistence                         |

## Example

A minimal embedded router with WebSocket transport:

```rust
use clasp_router::{Router, SecurityMode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::builder()
        .name("My Router")
        .max_sessions(100)
        .security_mode(SecurityMode::Open)
        .build();

    router.serve_websocket("0.0.0.0:7330").await?;
    Ok(())
}
```

An authenticated router with rate limiting, a write validator, and persistence:

```rust
use clasp_router::{Router, SecurityMode};
use clasp_journal::SqliteJournal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::builder()
        .name("Production Router")
        .max_sessions(500)
        .session_timeout(120)
        .security_mode(SecurityMode::Authenticated)
        .rate_limiting_enabled(true)
        .max_messages_per_second(500)
        .build();

    // Only allow writes under the client's own namespace
    router.set_write_validator(|session, address, _value| {
        address.starts_with(&format!("/clients/{}/", session.id()))
    });

    // Persist state to SQLite
    let journal = SqliteJournal::open("state.db").await?;
    router.set_journal(journal);

    router.serve_websocket("0.0.0.0:7330").await?;
    Ok(())
}
```

## Next Steps

- [Relay CLI Reference](relay-cli.md) -- the `clasp-relay` binary that wraps this config with CLI flags and JSON configuration
- [Rules Schema](rules-schema.md) -- JSON schema for the rules engine
- [Architecture](../concepts/architecture.md) -- how the router fits into the CLASP system
