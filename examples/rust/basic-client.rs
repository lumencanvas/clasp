//! CLASP Basic Client Example
//!
//! Demonstrates basic usage of the clasp-client crate.
//!
//! Usage:
//!   cargo run --example basic-client
//!
//! Or add to your Cargo.toml:
//!   clasp-client = "3.0"

use anyhow::Result;
use clasp_client::Clasp;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    let server_url = std::env::var("CLASP_URL")
        .unwrap_or_else(|_| "ws://localhost:7330".to_string());

    println!("CLASP Basic Client Example");
    println!("==========================\n");

    // Connect to server using builder pattern
    let client = Clasp::builder(&server_url)
        .with_name("rust-example")
        .with_features(vec!["param", "event", "stream"])
        .connect()
        .await?;

    println!("Connected to {}", server_url);
    println!("Session: {:?}\n", client.session_id());

    // Example 1: Set parameter values
    println!("Setting parameter values...");
    client.set("/example/rust/fader", 0.75).await?;
    client.set("/example/rust/toggle", true).await?;
    client.set("/example/rust/name", "Rust Client").await?;
    println!("  /example/rust/fader = 0.75");
    println!("  /example/rust/toggle = true");
    println!("  /example/rust/name = \"Rust Client\"\n");

    // Example 2: Subscribe to addresses
    println!("Setting up subscriptions...");

    // Subscribe to all example parameters
    let sub_id = client.on("/example/**", |value, address| {
        println!("  [SUB] {} = {:?}", address, value);
    }).await?;
    println!("  Subscribed to /example/** (id: {})", sub_id);

    // Example 3: Emit an event
    println!("\nEmitting events...");
    client.emit("/example/rust/started", serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "pid": std::process::id()
    })).await?;
    println!("  Emitted /example/rust/started");

    // Example 4: Stream high-rate data
    println!("\nStreaming values for 3 seconds...");
    let start = std::time::Instant::now();
    while start.elapsed() < Duration::from_secs(3) {
        let t = start.elapsed().as_secs_f64();
        let sine = (t * 2.0 * std::f64::consts::PI).sin() * 0.5 + 0.5;
        client.stream("/example/rust/sine", sine).await?;
        sleep(Duration::from_millis(33)).await; // ~30 fps
    }
    println!("  Streaming complete");

    // Example 5: Get a value
    println!("\nGetting values...");
    match client.get("/example/rust/fader").await {
        Ok(value) => println!("  /example/rust/fader = {:?}", value),
        Err(e) => println!("  /example/rust/fader: error - {}", e),
    }

    // Example 6: Check cached value
    if let Some(cached) = client.cached("/example/rust/toggle") {
        println!("  /example/rust/toggle (cached) = {:?}", cached);
    }

    // Example 7: Atomic bundle
    println!("\nSending atomic bundle...");
    use clasp_core::{Message, SetMessage};
    client.bundle(vec![
        Message::Set(SetMessage {
            address: "/example/rust/bundle/a".to_string(),
            value: 1.0.into(),
            revision: None,
            lock: false,
            unlock: false,
        }),
        Message::Set(SetMessage {
            address: "/example/rust/bundle/b".to_string(),
            value: 2.0.into(),
            revision: None,
            lock: false,
            unlock: false,
        }),
    ]).await?;
    println!("  Bundle sent");

    // Example 8: Scheduled bundle
    println!("\nSending scheduled bundle (executes in 1 second)...");
    let future_time = client.time() + 1_000_000; // 1 second from now (microseconds)
    client.bundle_at(vec![
        Message::Set(SetMessage {
            address: "/example/rust/scheduled".to_string(),
            value: "delayed!".into(),
            revision: None,
            lock: false,
            unlock: false,
        }),
    ], future_time).await?;
    println!("  Scheduled bundle queued");

    // Wait a bit to see scheduled bundle execute
    sleep(Duration::from_secs(2)).await;

    // Example 9: Unsubscribe
    println!("\nUnsubscribing...");
    client.unsubscribe(sub_id).await?;
    println!("  Unsubscribed from id: {}", sub_id);

    // Clean up
    println!("\nClosing connection...");
    client.close().await;
    println!("Done!");

    Ok(())
}
