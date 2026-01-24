//! CLASP P2P WebRTC Example (Rust)
//!
//! Demonstrates peer-to-peer communication using WebRTC DataChannels.
//!
//! Usage:
//!   # Terminal 1
//!   PEER_ID=peer-a cargo run --example p2p_webrtc
//!
//!   # Terminal 2
//!   PEER_ID=peer-b CONNECT_TO=peer-a cargo run --example p2p_webrtc

use clasp_client::Clasp;
use clasp_transport::p2p::{P2PConfig, P2PManager};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== CLASP P2P WebRTC Example (Rust) ===");

    let peer_id = env::var("PEER_ID").unwrap_or_else(|_| {
        format!(
            "peer-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        )
    });
    let connect_to = env::var("CONNECT_TO").ok();
    let rendezvous_url =
        env::var("RENDEZVOUS_URL").unwrap_or_else(|_| "wss://rendezvous.clasp.to".to_string());

    println!("Peer ID: {}", peer_id);

    // Create P2P manager
    let config = P2PConfig {
        peer_id: peer_id.clone(),
        rendezvous_url,
        ice_servers: vec![
            "stun:stun.l.google.com:19302".to_string(),
            "stun:stun1.l.google.com:19302".to_string(),
        ],
        use_unreliable_channel: true,
        ..Default::default()
    };

    let p2p = Arc::new(P2PManager::new(config).await?);

    // Handle incoming connections
    let p2p_clone = p2p.clone();
    tokio::spawn(async move {
        let mut rx = p2p_clone.connections();
        while let Some(peer) = rx.recv().await {
            println!("[P2P] Peer connected: {}", peer.id());

            // Subscribe to messages
            let peer_id = peer.id().to_string();
            peer.subscribe("/chat/*", move |value, address| {
                println!("[{}] {}: {:?}", peer_id, address, value);
            })
            .await
            .ok();

            let peer_id = peer.id().to_string();
            peer.subscribe("/sensor/accel", move |value, _| {
                if let Some(obj) = value.as_object() {
                    let x = obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let y = obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    let z = obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    println!(
                        "[{}] Accelerometer: x={:.2}, y={:.2}, z={:.2}",
                        peer_id, x, y, z
                    );
                }
            })
            .await
            .ok();
        }
    });

    // Register with rendezvous
    println!("\nRegistering with rendezvous server...");
    p2p.register(
        vec!["demo".to_string(), "webrtc".to_string(), "rust".to_string()],
        serde_json::json!({
            "name": format!("Rust Peer {}", peer_id),
            "capabilities": ["chat", "sensors"]
        }),
    )
    .await?;
    println!("Registered successfully!");

    // Connect to peer if specified
    if let Some(target) = connect_to {
        println!("\nConnecting to peer: {}...", target);

        match p2p.connect(&target).await {
            Ok(peer) => {
                println!("Connected to {}!", peer.id());

                // Send greeting
                peer.set("/chat/greeting", format!("Hello from Rust {}!", peer_id))
                    .await?;

                // Stream sensor data
                let peer = Arc::new(peer);
                let peer_clone = peer.clone();
                tokio::spawn(async move {
                    let mut i = 0u64;
                    loop {
                        let t = i as f64 * 0.1;
                        let _ = peer_clone
                            .stream(
                                "/sensor/accel",
                                serde_json::json!({
                                    "x": t.sin(),
                                    "y": t.cos(),
                                    "z": (t * 0.5).sin()
                                }),
                            )
                            .await;
                        i += 1;
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                });
            }
            Err(e) => {
                println!("Failed to connect: {}", e);
            }
        }
    } else {
        println!("\nWaiting for incoming connections...");
        println!(
            "Run another instance with: CONNECT_TO={} cargo run --example p2p_webrtc",
            peer_id
        );
    }

    // Discover peers
    println!("\nDiscovering peers...");
    let peers = p2p.discover(Some(vec!["demo".to_string()]), 100).await?;
    println!("Found {} peer(s):", peers.len());
    for p in peers {
        println!(
            "  - {}: {}",
            p.id,
            p.metadata
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown")
        );
    }

    // Keep running
    println!("\nPress Ctrl+C to exit");
    tokio::signal::ctrl_c().await?;

    Ok(())
}
