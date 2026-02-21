# clasp-federation

Router-to-router state sharing for multi-site CLASP deployments.

## Features

- **Hub/Leaf Topology** - Central hub with connecting leaf routers
- **Namespace Ownership** - Each router declares owned address patterns
- **State Synchronization** - Initial sync via snapshots, steady-state via forwarding
- **Revision Vectors** - Track remote state versions for consistency
- **Loop Prevention** - Origin-based forwarding guards prevent message loops
- **Auto-Reconnect** - Configurable reconnect with backoff
- **Resource Limits** - Bounded pattern counts and revision entries

## Installation

```toml
[dependencies]
clasp-federation = "3.5"
```

## Usage

### Leaf Configuration

```rust
use clasp_federation::{FederationConfig, FederationMode, FederationManager};

let config = FederationConfig {
    mode: FederationMode::Leaf {
        hub_endpoint: "ws://hub.example.com:7330".to_string(),
    },
    router_id: "site-a-router".to_string(),
    owned_namespaces: vec!["/site-a/**".to_string()],
    auth_token: Some("cap_...".to_string()),
    auto_reconnect: true,
    ..Default::default()
};

let mut manager = FederationManager::new(config);
```

### Hub Configuration

Hub mode accepts inbound federation peers. Enable via the `federation` feature flag on `clasp-router`:

```rust
use clasp_router::{Router, RouterConfig};

let router = Router::new(RouterConfig {
    features: vec!["param".into(), "event".into(), "federation".into()],
    ..Default::default()
});
```

Inbound peers are auto-detected when they advertise `"federation"` in their HELLO features. The router handles `DeclareNamespaces`, `RequestSync`, `RevisionVector`, and `SyncComplete` operations.

### CLI Usage

```bash
# Start as hub (accepts inbound federation peers)
clasp server --port 7330 --features federation

# Start as leaf (connects to hub)
clasp server --port 7331 \
    --federation-mode leaf \
    --federation-hub ws://hub:7330 \
    --federation-namespaces "/site-a/**"
```

### Working with the Manager

```rust
// Create federation links from established transport connections
let link = manager.create_link(transport_sender);

// Process events from federation links
let mut rx = manager.take_event_receiver().unwrap();
while let Some(event) = rx.recv().await {
    manager.process_event(&event).await;
}

// Query peer state
let peers = manager.active_peers().await;
let count = manager.peer_count().await;
let info = manager.peer_info("site-b-router").await;

// Check if an address should be forwarded to federation
let should_fwd = manager.should_forward("/site-b/lights/1", None).await;
let targets = manager.peers_for_address("/site-b/lights/1", None).await;
```

## Architecture

```
  Site A (Leaf)                    Hub                     Site B (Leaf)
 ┌──────────────┐           ┌──────────────┐           ┌──────────────┐
 │ CLASP Router │           │ CLASP Router │           │ CLASP Router │
 │              │◄─────────►│              │◄─────────►│              │
 │ /site-a/**   │  WS/QUIC  │   (hub mode) │  WS/QUIC  │ /site-b/**   │
 └──────┬───────┘           └──────┬───────┘           └──────┬───────┘
        │                          │                          │
   Local Clients              Hub Clients               Local Clients
```

**Message flow:**

1. Client at Site A sets `/site-a/lights/1/brightness`
2. Hub receives the SET via federation link
3. Hub forwards to Site B (whose subscription covers `/site-a/**`)
4. Site B delivers to local subscribers

**Loop prevention:** Every forwarded message carries an `origin` field set to the source router's `router_id`. Peers never forward a message back to its origin.

## Handshake Sequence

```
  Leaf                              Hub
   │                                 │
   │──── HELLO (features: [federation]) ───►│
   │                                 │
   │◄─── WELCOME ───────────────────│
   │                                 │
   │──── DeclareNamespaces ─────────►│
   │     ["/site-a/**"]              │
   │                                 │
   │◄─── Subscribe to /site-a/** ───│
   │                                 │
   │──── RequestSync("/site-a/**") ──►│
   │                                 │
   │◄─── Snapshot (current state) ──│
   │◄─── SyncComplete ─────────────│
   │                                 │
   │◄──── Steady-state forwarding ──►│
```

## Configuration Reference

### FederationConfig

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `mode` | `FederationMode` | `Hub` | Operating mode |
| `router_id` | `String` | UUID v4 | Unique router identity |
| `owned_namespaces` | `Vec<String>` | `["/**"]` | Namespace patterns this router owns |
| `auth_token` | `Option<String>` | `None` | Authentication token for peers |
| `auto_reconnect` | `bool` | `true` | Reconnect on disconnect |
| `reconnect_delay` | `Duration` | 5s | Delay between reconnect attempts |
| `max_reconnect_attempts` | `u32` | `0` (unlimited) | Max reconnect attempts |
| `sync_interval` | `Duration` | 30s | Revision vector exchange interval |
| `client_name` | `String` | `"clasp-federation"` | Client name in HELLO |
| `features` | `Vec<String>` | `["param","event","stream","federation"]` | Advertised features |

### FederationMode

| Variant | Fields | Description |
|---------|--------|-------------|
| `Hub` | -- | Accept inbound federation peers |
| `Leaf` | `hub_endpoint: String` | Connect to a hub |
| `Mesh` | `peers: Vec<String>` | Peer-to-peer (not yet implemented) |

### PeerState

| State | Description |
|-------|-------------|
| `Connecting` | Transport connection in progress |
| `Handshaking` | Connected, performing HELLO/WELCOME handshake |
| `Syncing` | Performing initial state sync |
| `Active` | Fully operational, forwarding messages |
| `Disconnected` | Disconnected, will auto-reconnect if enabled |
| `Failed` | Permanently failed |

### Resource Limits (Router-Enforced)

| Limit | Value | Description |
|-------|-------|-------------|
| `MAX_FEDERATION_PATTERNS` | 1,000 | Max namespace patterns per peer |
| `MAX_REVISION_ENTRIES` | 10,000 | Max entries in a revision vector |

These limits are enforced by the hub router to prevent resource exhaustion from federation peers.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio)
