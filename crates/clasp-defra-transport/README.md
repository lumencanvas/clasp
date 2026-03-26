# clasp-defra-transport

Tunnel DefraDB P2P sync traffic over CLASP transports (WebSocket, WebRTC, QUIC).

## Why

DefraDB uses libp2p for P2P networking, which has limited browser support. CLASP has production WebSocket, WebRTC, and WASM clients. This crate bridges the gap, enabling browser-native DefraDB P2P via CLASP's transport layer.

## Tunnel protocol

Messages are serialized as JSON and sent as CLASP binary payloads on the `/defra/sync/` namespace:

| Message | Purpose |
|---------|---------|
| `SyncRequest` | Request sync for a collection |
| `DagBlock` | Transfer an IPLD DAG node |
| `HeadUpdate` | Notify of new document commit |
| `BlockAck` | Acknowledge received blocks |
| `PeerInfo` | Exchange peer capabilities |
| `QueryForward` | Forward GraphQL query to peer |
| `QueryResponse` | Return query results |

## Address namespace

```
/defra/sync/{peer_id}                -- peer sync channel
/defra/sync/{peer_id}/{collection}   -- collection-specific sync
/defra/sync/{peer_id}/blocks         -- block transfer channel
```

## Usage

```rust
use clasp_defra_transport::DefraTunnel;

let tunnel = DefraTunnel::new("http://localhost:9181", "my-peer-id");
tunnel.add_peer("remote-peer", vec!["User".into()]).await;

// Handle incoming messages
let response = tunnel.handle_message("remote-peer", msg).await?;

// Poll for outgoing updates
let updates = tunnel.poll_updates("remote-peer").await?;
```

## License

MIT OR Apache-2.0
