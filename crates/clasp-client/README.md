# clasp-client

Async client library for CLASP (Creative Low-Latency Application Streaming Protocol).

## Usage

```rust
use clasp_client::{Clasp, ClaspBuilder};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect using builder
    let client = ClaspBuilder::new("ws://localhost:7330")
        .name("My App")
        .connect()
        .await?;

    // Set a parameter
    client.set("/lights/front/brightness", 0.75.into()).await?;

    // Get a parameter
    let value = client.get("/lights/front/brightness").await?;
    println!("Brightness: {:?}", value);

    // Subscribe to changes
    let _unsub = client.subscribe("/lights/*", |value, addr| {
        println!("{} = {:?}", addr, value);
    }).await?;

    // Close connection
    client.close().await?;
    Ok(())
}
```

## Features

- Async/await API with Tokio
- WebSocket transport with automatic reconnection
- Time synchronization with server
- Pattern-based subscriptions with wildcards
- P2P WebRTC connections with data transfer (requires `p2p` feature)

## P2P Example

```rust
use clasp_client::{Clasp, RoutingMode, SendResult};
use clasp_core::P2PConfig;
use bytes::Bytes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Connect with P2P enabled
    let client = Clasp::builder("ws://localhost:7330")
        .name("P2P App")
        .p2p_config(P2PConfig::default())
        .connect()
        .await?;

    // Listen for P2P events
    client.on_p2p_event(|event| {
        match event {
            clasp_client::P2PEvent::Connected { peer_session_id } => {
                println!("Connected to peer: {}", peer_session_id);
            }
            clasp_client::P2PEvent::Data { peer_session_id, data, reliable } => {
                println!("Received {} bytes from {} (reliable={})",
                    data.len(), peer_session_id, reliable);
            }
            clasp_client::P2PEvent::Disconnected { peer_session_id, .. } => {
                println!("Disconnected from peer: {}", peer_session_id);
            }
            _ => {}
        }
    });

    // Connect to another peer by session ID
    client.connect_to_peer("other-session-id").await?;

    // Send data via P2P
    let result = client.send_p2p("other-session-id", Bytes::from("hello"), true).await?;
    match result {
        SendResult::P2P => println!("Sent via direct P2P"),
        SendResult::Relay => println!("Sent via server relay"),
    }

    // Control routing mode
    client.set_p2p_routing_mode(RoutingMode::PreferP2P);  // Try P2P first, fall back to relay
    client.set_p2p_routing_mode(RoutingMode::P2POnly);    // Only use P2P, fail if unavailable
    client.set_p2p_routing_mode(RoutingMode::ServerOnly); // Never use P2P, always relay

    Ok(())
}
```

## Documentation

Visit **[clasp.to](https://clasp.to)** for full documentation.

## License

Licensed under either of Apache License, Version 2.0 or MIT license at your option.

---

Maintained by [LumenCanvas](https://lumencanvas.studio) | 2026
