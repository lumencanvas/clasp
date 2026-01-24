//! CLASP Bundles & Scheduling Example (Rust)
//!
//! Demonstrates atomic bundles and scheduled execution.
//!
//! Usage:
//!   cargo run --example bundles_and_scheduling

use clasp_client::Clasp;
use clasp_core::{Message, BundleMessage, SetMessage, PublishMessage};
use std::env;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CLASP Bundles & Scheduling Example (Rust) ===\n");

    let url = env::var("CLASP_URL").unwrap_or_else(|_| "ws://localhost:7330".to_string());

    let client = Clasp::connect_to(&url).await?;
    println!("Connected to CLASP server");

    // Subscribe to see all changes
    client.subscribe("/**", |value, address| {
        let time = chrono::Local::now().format("%H:%M:%S%.3f");
        println!("[{}] {} = {:?}", time, address, value);
    }).await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // =====================
    // 1. Atomic Bundle
    // =====================
    println!("\n--- 1. Atomic Bundle ---");
    println!("Setting multiple values atomically...");

    client.bundle(vec![
        Message::Set(SetMessage {
            address: "/scene/active".to_string(),
            value: "sunset".into(),
            revision: None,
        }),
        Message::Set(SetMessage {
            address: "/lights/1/brightness".to_string(),
            value: 0.8.into(),
            revision: None,
        }),
        Message::Set(SetMessage {
            address: "/lights/2/brightness".to_string(),
            value: 0.6.into(),
            revision: None,
        }),
    ], None).await?;

    println!("Atomic bundle sent!");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // =====================
    // 2. Scheduled Bundle
    // =====================
    println!("\n--- 2. Scheduled Bundle ---");

    let server_time = client.time();
    let execute_at = server_time + 2_000_000; // 2 seconds from now
    println!("Scheduling bundle for 2 seconds from now...");

    client.bundle(vec![
        Message::Set(SetMessage {
            address: "/scheduled/counter".to_string(),
            value: 1.into(),
            revision: None,
        }),
    ], Some(execute_at)).await?;

    println!("Scheduled bundle sent! Waiting for execution...");
    tokio::time::sleep(Duration::from_millis(2500)).await;

    // =====================
    // 3. Animation Sequence
    // =====================
    println!("\n--- 3. Chained Scheduled Bundles (Animation) ---");
    println!("Creating a 5-step fade animation...");

    let animation_start = client.time() + 500_000;
    let step_duration = 200_000;

    for i in 0..=5 {
        let brightness = i as f64 / 5.0;
        let execute_time = animation_start + (i as i64 * step_duration);

        client.bundle(vec![
            Message::Set(SetMessage {
                address: "/animation/brightness".to_string(),
                value: brightness.into(),
                revision: None,
            }),
            Message::Set(SetMessage {
                address: "/animation/step".to_string(),
                value: (i as i64).into(),
                revision: None,
            }),
        ], Some(execute_time)).await?;
    }

    println!("Animation scheduled! Watching...");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // =====================
    // 4. Mixed Bundle
    // =====================
    println!("\n--- 4. Mixed Bundle with Events and Params ---");

    client.bundle(vec![
        Message::Set(SetMessage {
            address: "/cue/current".to_string(),
            value: "intro".into(),
            revision: None,
        }),
        Message::Publish(PublishMessage {
            address: "/cue/started".to_string(),
            signal: clasp_core::SignalType::Event,
            payload: Some(serde_json::json!({"name": "intro"})),
            value: None,
            timestamp: None,
        }),
    ], None).await?;

    println!("Mixed bundle sent!");
    tokio::time::sleep(Duration::from_millis(500)).await;

    drop(client);
    println!("\n=== Bundle demos complete ===");
    Ok(())
}
