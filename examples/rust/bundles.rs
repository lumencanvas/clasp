//! CLASP Bundles & Scheduling Example (Rust)
//!
//! Demonstrates atomic bundles and scheduled execution.
//!
//! Usage:
//!   cargo run --example bundles

use clasp_client::Clasp;
use clasp_core::{BundleMessage, Message, SetMessage};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CLASP Bundles & Scheduling Example (Rust) ===\n");

    let clasp_url =
        std::env::var("CLASP_URL").unwrap_or_else(|_| "ws://localhost:7330".to_string());

    let client = Clasp::connect_to(&clasp_url).await?;
    println!("Connected to CLASP server");

    // Subscribe to see changes
    client
        .subscribe("/**", |value, address| {
            let time = chrono::Local::now().format("%H:%M:%S%.3f");
            println!("[{}] {} = {:?}", time, address, value);
        })
        .await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // =====================
    // 1. Atomic Bundle
    // =====================
    println!("\n--- 1. Atomic Bundle ---");
    println!("Setting multiple values atomically...");

    client
        .bundle(vec![
            Message::Set(SetMessage {
                address: "/scene/active".into(),
                value: "sunset".into(),
                ..Default::default()
            }),
            Message::Set(SetMessage {
                address: "/lights/1/brightness".into(),
                value: 0.8.into(),
                ..Default::default()
            }),
            Message::Set(SetMessage {
                address: "/lights/2/brightness".into(),
                value: 0.6.into(),
                ..Default::default()
            }),
        ])
        .await?;

    println!("Atomic bundle sent!");
    tokio::time::sleep(Duration::from_millis(500)).await;

    // =====================
    // 2. Scheduled Bundle
    // =====================
    println!("\n--- 2. Scheduled Bundle ---");

    let server_time = client.time();
    let execute_at = server_time + 2_000_000; // 2 seconds
    println!("Scheduling bundle for 2 seconds from now...");

    client
        .bundle_at(
            vec![
                Message::Set(SetMessage {
                    address: "/scheduled/counter".into(),
                    value: 1.into(),
                    ..Default::default()
                }),
                Message::Set(SetMessage {
                    address: "/scheduled/triggered".into(),
                    value: true.into(),
                    ..Default::default()
                }),
            ],
            execute_at,
        )
        .await?;

    println!("Scheduled bundle sent! Waiting...");
    tokio::time::sleep(Duration::from_millis(2500)).await;

    // =====================
    // 3. Animation
    // =====================
    println!("\n--- 3. Chained Animation ---");
    println!("Creating 5-step fade...");

    let start = client.time() + 500_000;
    let step = 200_000u64;

    for i in 0..=5 {
        let brightness = i as f64 / 5.0;
        let at = start + (i as u64 * step);

        client
            .bundle_at(
                vec![Message::Set(SetMessage {
                    address: "/animation/brightness".into(),
                    value: brightness.into(),
                    ..Default::default()
                })],
                at,
            )
            .await?;
    }

    println!("Animation scheduled!");
    tokio::time::sleep(Duration::from_millis(2000)).await;

    client.disconnect().await;
    println!("\n=== Bundle demos complete ===");
    Ok(())
}
