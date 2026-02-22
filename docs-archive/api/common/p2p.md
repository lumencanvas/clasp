---
title: "P2p"
description: "Documentation page for the CLASP protocol."
section: api
order: 3
---
## Peer‑to‑Peer (P2P) Connections

CLASP supports **true peer‑to‑peer** communication using WebRTC DataChannels, with a CLASP router acting as the **signaling** point only.

### Architecture

1. Each client connects to a router over a normal transport (typically WebSocket).
2. Clients exchange **session IDs** and P2P offers/answers via the router.
3. WebRTC negotiates a DataChannel between peers using ICE/STUN/TURN.
4. Once connected, CLASP frames flow directly over the DataChannel.

The router is **not** a STUN/TURN server; it simply forwards signaling messages.

### Configuration

P2P APIs typically let you:

- Enable P2P on a client (e.g. provide a `P2PConfig` with ICE servers).
- Initiate a P2P connection to another session ID.
- Observe P2P connection state and failure events.
- **Send data directly to peers** via `send_p2p()`.
- **Control routing behavior** via `set_p2p_routing_mode()`.

### Rust Client API (v3.3.0+)

```rust
// Enable P2P when building client
let client = Clasp::builder("ws://localhost:7330")
    .p2p_config(P2PConfig::default())
    .connect()
    .await?;

// Connect to peer
client.connect_to_peer("peer-session-id").await?;

// Send data (reliable or unreliable)
let result = client.send_p2p("peer-session-id", data, true).await?;
// result is SendResult::P2P or SendResult::Relay

// Control routing mode
client.set_p2p_routing_mode(RoutingMode::PreferP2P);  // Default: try P2P, fall back to relay
client.set_p2p_routing_mode(RoutingMode::P2POnly);    // Only P2P, fail if unavailable
client.set_p2p_routing_mode(RoutingMode::ServerOnly); // Always use relay

// Check current mode
let mode = client.p2p_routing_mode();

// Listen for P2P events
client.on_p2p_event(|event| {
    match event {
        P2PEvent::Connected { peer_session_id } => { /* ... */ }
        P2PEvent::Data { peer_session_id, data, reliable } => { /* ... */ }
        P2PEvent::Disconnected { peer_session_id, reason } => { /* ... */ }
        P2PEvent::ConnectionFailed { peer_session_id, reason } => { /* ... */ }
    }
});
```

### Connection Timeout

P2P connections that fail to establish within the configured timeout (default: 30 seconds) will emit a `P2PEvent::ConnectionFailed` event. Configure via `P2PConfig::connection_timeout_secs`.

See `clasp-e2e/src/bin/p2p_connection_tests.rs` for concrete end‑to‑end examples; language‑specific docs map these behaviors into each runtime's idioms.

