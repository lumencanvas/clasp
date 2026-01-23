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

See `test-suite/src/bin/p2p_connection_tests.rs` for concrete end‑to‑end examples; language‑specific docs map these behaviors into each runtime’s idioms.

