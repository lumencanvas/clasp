# clasp-router

Message router and server for CLASP (Creative Low-Latency Application Streaming Protocol).

## Features

- **Message Routing** - Route messages between connected clients
- **Pattern Matching** - Wildcard subscriptions with `*` and `**`
- **State Management** - Parameter state with revision tracking
- **Session Management** - Track client connections and subscriptions
- **Multiple Transports** - WebSocket, QUIC, TCP
- **Protocol Adapters** - Accept MQTT and OSC clients directly (optional features)
- **Rate Limiting** - Configurable per-client message rate limits
- **Gesture Coalescing** - Reduce bandwidth for high-frequency gesture streams

## Installation

```toml
[dependencies]
clasp-router = "3.5"

# Optional: Enable protocol adapters
clasp-router = { version = "3.5", features = ["mqtt-server", "osc-server"] }
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `websocket` | WebSocket transport (default) |
| `quic` | QUIC transport with built-in TLS |
| `tcp` | Raw TCP transport |
| `mqtt-server` | Accept MQTT clients directly |
| `osc-server` | Accept OSC clients via UDP |
| `journal` | State persistence and replay via `clasp-journal` |
| `rules` | Server-side automation via `clasp-rules` |
| `federation` | Accept inbound federation peers |
| `metrics` | Prometheus-compatible instrumentation via `metrics` crate |
| `full` | All features enabled |

## Basic Usage

```rust
use clasp_router::{Router, RouterConfig};
use clasp_core::SecurityMode;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new(RouterConfig {
        name: "My Router".into(),
        max_sessions: 100,
        session_timeout: 60,
        features: vec!["param".into(), "event".into()],
        security_mode: SecurityMode::Open,
        max_subscriptions_per_session: 100,
        gesture_coalescing: true,
        gesture_coalesce_interval_ms: 16,
        max_messages_per_second: 1000,
        rate_limiting_enabled: true,
    });

    // Serve on WebSocket
    router.serve_websocket("0.0.0.0:7330").await?;
    Ok(())
}
```

## Multi-Protocol Server

Serve multiple protocols simultaneously with shared state:

```rust
use clasp_router::{Router, RouterConfig, MultiProtocolConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let router = Router::new(RouterConfig::default());

    let config = MultiProtocolConfig {
        websocket_addr: Some("0.0.0.0:7330".into()),
        #[cfg(feature = "mqtt-server")]
        mqtt: Some(clasp_router::MqttServerConfig {
            bind_addr: "0.0.0.0:1883".into(),
            namespace: "/mqtt".into(),
            ..Default::default()
        }),
        #[cfg(feature = "osc-server")]
        osc: Some(clasp_router::OscServerConfig {
            bind_addr: "0.0.0.0:8000".into(),
            namespace: "/osc".into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    // All protocols share the same router state
    router.serve_all(config).await?;
    Ok(())
}
```

## Protocol Adapters

### MQTT Server Adapter

Accept MQTT clients directly without an external broker:

```rust
use clasp_router::MqttServerConfig;

let mqtt_config = MqttServerConfig {
    bind_addr: "0.0.0.0:1883".into(),
    namespace: "/mqtt".into(),      // MQTT topic "sensors/temp" -> CLASP "/mqtt/sensors/temp"
    require_auth: false,
    max_clients: 100,
    session_timeout_secs: 300,
    ..Default::default()
};
```

MQTT to CLASP mapping:

| MQTT | CLASP |
|------|-------|
| CONNECT | Hello -> Session |
| SUBSCRIBE `sensors/#` | Subscribe `/mqtt/sensors/**` |
| PUBLISH `sensors/temp` | Set `/mqtt/sensors/temp` |
| QoS 0 | Fire-and-forget |
| QoS 1 | With acknowledgment |

### OSC Server Adapter

Accept OSC clients via UDP with automatic session tracking:

```rust
use clasp_router::OscServerConfig;

let osc_config = OscServerConfig {
    bind_addr: "0.0.0.0:8000".into(),
    namespace: "/osc".into(),       // OSC "/synth/volume" -> CLASP "/osc/synth/volume"
    session_timeout_secs: 30,       // Sessions expire after 30s of inactivity
    auto_subscribe: false,
    ..Default::default()
};
```

## Configuration Reference

### RouterConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `name` | String | "CLASP Router" | Server name shown to clients |
| `max_sessions` | usize | 1000 | Maximum concurrent connections |
| `session_timeout` | u64 | 300 | Session timeout in seconds |
| `security_mode` | SecurityMode | Open | Authentication mode |
| `max_subscriptions_per_session` | usize | 1000 | Max subscriptions per client |
| `gesture_coalescing` | bool | true | Enable gesture move coalescing |
| `gesture_coalesce_interval_ms` | u64 | 16 | Coalesce interval (16ms = 60fps) |
| `max_messages_per_second` | u32 | 1000 | Rate limit per client (0 = unlimited) |
| `rate_limiting_enabled` | bool | true | Enable rate limiting |
| `state_config` | RouterStateConfig | Default (1h TTL) | State store configuration |

### State Configuration (TTL)

Parameters and signals can be configured to expire after a time-to-live period:

```rust
use clasp_router::{RouterConfig, RouterStateConfig};
use std::time::Duration;

let config = RouterConfig {
    state_config: RouterStateConfig {
        param_config: clasp_core::state::StateStoreConfig {
            max_params: Some(100_000),
            param_ttl: Some(Duration::from_secs(3600)), // 1 hour
            eviction: clasp_core::state::EvictionStrategy::Lru,
        },
        signal_ttl: Some(Duration::from_secs(3600)), // 1 hour
        max_signals: Some(100_000),
    },
    ..Default::default()
};

// Or use unlimited (no expiration):
let unlimited_config = RouterConfig {
    state_config: RouterStateConfig::unlimited(),
    ..Default::default()
};
```

### Rate Limiting

Rate limiting prevents clients from overwhelming the router:

```rust
let config = RouterConfig {
    rate_limiting_enabled: true,
    max_messages_per_second: 500,  // 500 msg/s per client
    ..Default::default()
};
```

When a client exceeds the rate limit, excess messages are dropped and a warning is logged.

### Buffer Overflow Notifications

When a client's receive buffer fills and messages are dropped, the router sends an ERROR 503 notification after 100 drops within 10 seconds. This helps slow clients detect they're missing messages. Notifications are rate-limited to 1 per 10 seconds per session.

## Journal Integration

Enable state persistence with the `journal` feature. The router records all SET and PUBLISH operations to an append-only journal for crash recovery and replay:

```rust
use clasp_journal::SqliteJournal;
use std::sync::Arc;

let journal = Arc::new(SqliteJournal::new("state.db")?);
let router = Router::new(config).with_journal(journal);
```

On restart, state can be replayed from the journal. Clients can request replay of missed messages using the Replay handler.

## Rules Engine

Enable server-side automation with the `rules` feature. Rules are evaluated after state changes:

```rust
use clasp_rules::{Rule, Trigger, RuleAction, RulesEngine};
```

The router evaluates matching rules on each SET/PUBLISH, executing actions like setting values, publishing events, or copying values with transforms. See [`clasp-rules`](../clasp-rules/) for rule definition syntax.

## Federation

Enable router-to-router state sharing with the `federation` feature. The router accepts inbound federation peers that advertise `"federation"` in their HELLO features.

Federation operations handled by the router:

| Operation | Description |
|-----------|-------------|
| `DeclareNamespaces` | Peer declares owned namespace patterns, router auto-subscribes |
| `RequestSync` | Peer requests state snapshot for a pattern range |
| `RevisionVector` | Peer exchanges revision vectors for delta sync |
| `SyncComplete` | Marks sync as complete |

**Security:** In authenticated mode, federation peers must have scopes covering their declared namespaces. `RequestSync` and `RevisionVector` are validated against declared namespaces (peers cannot request data outside their declared scope). Resource limits prevent exhaustion: max 1,000 patterns per peer, max 10,000 entries per revision vector.

Non-federation sessions that attempt `FederationSync` receive a 403 error.

## Architecture

```
                    ┌─────────────────────────────────────────────────┐
                    │                  CLASP Router                   │
                    │  ┌─────────────────────────────────────────┐    │
                    │  │              Shared State                │    │
                    │  │   sessions | subscriptions | state       │    │
                    │  └──────┬──────────────┬──────────────┬────┘    │
                    │         │              │              │         │
                    │  ┌──────▼──────┐ ┌─────▼─────┐ ┌─────▼─────┐  │
                    │  │   Journal   │ │   Rules   │ │ Federation│  │
                    │  │ (optional)  │ │ (optional)│ │ (optional)│  │
                    │  └─────────────┘ └───────────┘ └─────┬─────┘  │
                    │                                      │         │
                    │        ▲           ▲           ▲      │         │
                    │        │           │           │      ▼         │
                    │  ┌─────┴───┐ ┌─────┴───┐ ┌─────┴───┐ ┌──────┐  │
                    │  │WebSocket│ │  MQTT   │ │   OSC   │ │ Peer │  │
                    │  │ :7330   │ │  :1883  │ │  :8000  │ │Router│  │
                    │  └─────────┘ └─────────┘ └─────────┘ └──────┘  │
                    └─────────────────────────────────────────────────┘
```

All protocol adapters and federation peers share the same router state, enabling cross-protocol and cross-site communication.

## Performance

| Metric | Value |
|--------|-------|
| E2E throughput | 173k msg/s |
| Fanout 100 subs | 175k deliveries/s |
| Events (no state) | 259k msg/s |
| Late-joiner replay | Yes (chunked snapshots) |

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio)
