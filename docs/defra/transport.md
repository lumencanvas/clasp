---
title: Transport Tunnel
description: DefraDB sync over CLASP WebSocket, WebRTC, QUIC, and BLE
order: 6
---

# Transport Tunnel

The `clasp-defra-transport` crate tunnels DefraDB's P2P sync protocol over CLASP transports. DefraDB uses LibP2P, which barely works in browsers and doesn't work on BLE sensors or serial-connected devices. CLASP has production WebSocket, WebRTC, QUIC, BLE, and Serial transports.

This crate bridges the gap.

## Tunnel Protocol

DefraDB sync messages are wrapped as `TunnelMessage` variants that serialize to JSON and flow over any CLASP transport:

| Message | Purpose |
|---------|---------|
| `SyncRequest` | Request sync for a collection since a given CID |
| `DagBlock` | Transfer an IPLD DAG node (CID + binary data + links) |
| `HeadUpdate` | Notify peer of a new document commit |
| `BlockAck` | Acknowledge receipt of blocks |
| `PeerInfo` | Exchange peer capabilities and collection lists |
| `QueryForward` | Forward a GraphQL query to a peer's local DefraDB |
| `QueryResponse` | Return query results from a forwarded query |

## Address Namespace

Sync traffic flows on reserved CLASP addresses:

```
/defra/sync/{peer_id}                -- peer sync channel
/defra/sync/{peer_id}/{collection}   -- collection-specific sync
/defra/sync/{peer_id}/blocks         -- block transfer channel
```

## Usage

```rust
use clasp_defra_transport::DefraTunnel;

let tunnel = DefraTunnel::new("http://localhost:9181", "my-peer-id");

// Register a remote peer
tunnel.add_peer("remote-peer", vec!["SensorData".into()]).await;

// Handle incoming messages from CLASP
if let Some(response) = tunnel.handle_message("remote-peer", msg).await? {
    // Send response back via CLASP
    sender.emit(&addr, response).await?;
}

// Poll for outgoing updates
let updates = tunnel.poll_updates("remote-peer").await?;
```

## What This Enables

A web application running CLASP's WASM client can:
1. Connect to a CLASP router via WebSocket
2. Subscribe to `/defra/sync/**` for sync traffic
3. Exchange `TunnelMessage` payloads with other peers
4. Participate in DefraDB replication without running LibP2P

Same applies to an ESP32 over BLE, an Arduino over Serial, or a mobile app over QUIC.
