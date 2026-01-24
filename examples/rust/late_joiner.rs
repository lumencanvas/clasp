//! CLASP Late Joiner Synchronization Example (Rust)
//!
//! Demonstrates how late-joining clients receive current state.
//!
//! Usage:
//!   cargo run --example late_joiner

use clasp_client::Clasp;
use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CLASP Late Joiner Synchronization Example (Rust) ===\n");

    let url = env::var("CLASP_URL").unwrap_or_else(|_| "ws://localhost:7330".to_string());

    // =====================
    // Setup: Initialize state
    // =====================
    println!("--- Setup: Initializing State ---");

    let initializer = Clasp::connect_to(&url).await?;

    println!("Setting up initial state...\n");

    initializer.set("/lights/living-room/brightness", 0.8).await?;
    initializer.set("/lights/living-room/color", serde_json::json!({"r": 255, "g": 240, "b": 220})).await?;
    initializer.set("/lights/kitchen/brightness", 1.0).await?;
    initializer.set("/lights/bedroom/brightness", 0.3).await?;

    initializer.set("/audio/master/volume", 0.65).await?;
    initializer.set("/audio/master/mute", false).await?;

    initializer.set("/scene/active", "evening").await?;

    println!("Initial state created with 7 params");
    println!("Disconnecting initializer...\n");
    drop(initializer);

    tokio::time::sleep(Duration::from_millis(500)).await;

    // =====================
    // 1. Late Joiner
    // =====================
    println!("--- 1. Late Joiner with Full Wildcard ---");

    let late_joiner = Clasp::connect_to(&url).await?;

    println!("New client connected. Subscribing to /**...\n");

    let snapshot_count = Arc::new(AtomicU32::new(0));
    let count_clone = snapshot_count.clone();

    late_joiner.subscribe("/**", move |value, address| {
        count_clone.fetch_add(1, Ordering::Relaxed);
        println!("  [SNAPSHOT] {} = {:?}", address, value);
    }).await?;

    tokio::time::sleep(Duration::from_millis(300)).await;
    println!("\nReceived {} params in snapshot", snapshot_count.load(Ordering::Relaxed));

    drop(late_joiner);

    // =====================
    // 2. Selective Subscription
    // =====================
    println!("\n--- 2. Selective Subscription ---");

    let lights_only = Clasp::connect_to(&url).await?;

    println!("Subscribing to /lights/** only...\n");

    let lights_count = Arc::new(AtomicU32::new(0));
    let count_clone = lights_count.clone();

    lights_only.subscribe("/lights/**", move |value, address| {
        count_clone.fetch_add(1, Ordering::Relaxed);
        println!("  [SNAPSHOT] {} = {:?}", address, value);
    }).await?;

    tokio::time::sleep(Duration::from_millis(200)).await;
    println!("\nReceived {} light params (audio/scene excluded)", lights_count.load(Ordering::Relaxed));

    drop(lights_only);

    // =====================
    // OSC vs CLASP comparison
    // =====================
    println!("\n--- OSC vs CLASP Comparison ---");

    println!("\nWith OSC (stateless):");
    println!("  - Client connects");
    println!("  - Client waits for someone to send updates...");
    println!("  - No initial state available");

    println!("\nWith CLASP (stateful):");
    println!("  - Client connects");
    println!("  - Subscribes to patterns");
    println!("  - IMMEDIATELY receives ALL current values");

    // Demonstrate instant sync
    let demo = Clasp::connect_to(&url).await?;

    let start_time = Instant::now();
    let received = Arc::new(AtomicU32::new(0));
    let received_clone = received.clone();
    let first_received = Arc::new(std::sync::Mutex::new(None));
    let first_clone = first_received.clone();
    let start_clone = start_time.clone();

    demo.subscribe("/**", move |_value, _address| {
        let count = received_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count == 1 {
            let mut first = first_clone.lock().unwrap();
            if first.is_none() {
                *first = Some(start_clone.elapsed());
            }
        }
    }).await?;

    tokio::time::sleep(Duration::from_millis(300)).await;

    if let Some(elapsed) = *first_received.lock().unwrap() {
        println!("\n  First param received in {:.1}ms after connect!", elapsed.as_secs_f64() * 1000.0);
    }
    println!("  Total: {} params synced instantly", received.load(Ordering::Relaxed));

    drop(demo);

    println!("\n=== Late joiner demo complete ===");
    println!("\nKey takeaway: Unlike OSC, CLASP clients get current state immediately.");

    Ok(())
}
