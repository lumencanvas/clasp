//! CLASP Signal Types Example (Rust)
//!
//! Demonstrates all five signal types:
//! - Param: Stateful values
//! - Event: One-shot triggers
//! - Stream: High-frequency data
//! - Gesture: Phased input
//! - Timeline: Automation
//!
//! Usage:
//!   cargo run --example signal_types

use clasp_client::Clasp;
use std::env;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CLASP Signal Types Example (Rust) ===\n");

    let url = env::var("CLASP_URL").unwrap_or_else(|_| "ws://localhost:7330".to_string());

    let client = Clasp::connect_to(&url).await?;
    println!("Connected to CLASP server");

    // =====================
    // 1. PARAM
    // =====================
    println!("\n--- 1. PARAM (Stateful Values) ---");

    client.subscribe("/mixer/**", |value, address| {
        println!("[PARAM] {} = {:?}", address, value);
    }).await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    client.set("/mixer/master/volume", 0.8).await?;
    client.set("/mixer/master/mute", false).await?;
    client.set("/mixer/channel/1/volume", 0.65).await?;

    // Get cached value
    if let Some(vol) = client.value("/mixer/master/volume") {
        println!("\nCached master volume: {:?}", vol);
    }

    tokio::time::sleep(Duration::from_millis(300)).await;

    // =====================
    // 2. EVENT
    // =====================
    println!("\n--- 2. EVENT (One-Shot Triggers) ---");

    client.subscribe_events("/cue/**", |payload, address| {
        println!("[EVENT] {}: {:?}", address, payload);
    }).await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    client.emit("/cue/go", serde_json::json!({"cue_id": "intro", "fade_time": 2.0})).await?;
    client.emit("/cue/stop", serde_json::json!({"immediate": false})).await?;

    tokio::time::sleep(Duration::from_millis(300)).await;

    // =====================
    // 3. STREAM
    // =====================
    println!("\n--- 3. STREAM (High-Frequency Data) ---");

    let stream_count = Arc::new(AtomicU32::new(0));
    let count_clone = stream_count.clone();

    client.subscribe_stream("/sensor/**", move |value, address| {
        let count = count_clone.fetch_add(1, Ordering::Relaxed) + 1;
        if count % 10 == 0 {
            println!("[STREAM] {} = {:?} (received {})", address, value, count);
        }
    }, Some(30)).await?; // Rate limit to 30 Hz

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stream at 60Hz for 1 second
    println!("Streaming 60 values...");
    for i in 0..60 {
        let t = i as f64 / 60.0;
        client.stream("/sensor/accelerometer", serde_json::json!({
            "x": (t * std::f64::consts::PI * 2.0).sin(),
            "y": (t * std::f64::consts::PI * 2.0).cos(),
            "z": 0.98
        })).await?;
        tokio::time::sleep(Duration::from_millis(16)).await;
    }

    println!("Sent 60 stream values, received {}", stream_count.load(Ordering::Relaxed));

    tokio::time::sleep(Duration::from_millis(300)).await;

    // =====================
    // 4. GESTURE
    // =====================
    println!("\n--- 4. GESTURE (Phased Input) ---");

    client.subscribe_gestures("/input/**", |gesture| {
        let phase = gesture.phase.as_deref().unwrap_or("?");
        let x = gesture.x.unwrap_or(0.0);
        let y = gesture.y.unwrap_or(0.0);
        println!("[GESTURE] {} at ({:.0}, {:.0})", phase.to_uppercase(), x, y);
    }).await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    let gesture_id = format!("drag-{}", std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis());

    client.gesture("/input/mouse", &gesture_id, "start", 100.0, 100.0, None).await?;

    for i in 1..=5 {
        client.gesture(
            "/input/mouse",
            &gesture_id,
            "move",
            100.0 + i as f64 * 20.0,
            100.0 + i as f64 * 10.0,
            None
        ).await?;
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    client.gesture("/input/mouse", &gesture_id, "end", 200.0, 150.0, None).await?;

    tokio::time::sleep(Duration::from_millis(300)).await;

    // =====================
    // 5. TIMELINE
    // =====================
    println!("\n--- 5. TIMELINE (Automation) ---");

    client.subscribe_timelines("/automation/**", |timeline, address| {
        let kf_count = timeline.keyframes.len();
        let duration = timeline.duration;
        println!("[TIMELINE] {}: {} keyframes, duration={}ms", address, kf_count, duration);
    }).await?;

    tokio::time::sleep(Duration::from_millis(100)).await;

    client.timeline("/automation/light/brightness", clasp_core::Timeline {
        duration: 5000,
        loop_: true,
        keyframes: vec![
            clasp_core::Keyframe { time: 0, value: 0.0.into(), easing: "linear".to_string() },
            clasp_core::Keyframe { time: 1000, value: 1.0.into(), easing: "ease-out".to_string() },
            clasp_core::Keyframe { time: 3000, value: 1.0.into(), easing: "linear".to_string() },
            clasp_core::Keyframe { time: 5000, value: 0.0.into(), easing: "ease-in".to_string() },
        ],
    }).await?;

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Summary
    println!("\n=== Signal Type Summary ===");
    println!("| Type     | QoS     | Persists | Use Case                    |");
    println!("|----------|---------|----------|-----------------------------|");
    println!("| Param    | Confirm | Yes      | Faders, settings, state     |");
    println!("| Event    | Confirm | No       | Button press, cue trigger   |");
    println!("| Stream   | Fire    | No       | Sensors, meters (30-60Hz)   |");
    println!("| Gesture  | Fire    | No       | Touch, pen, mouse drag      |");
    println!("| Timeline | Commit  | Yes      | Animation, automation       |");

    drop(client);
    println!("\n=== Signal types demo complete ===");
    Ok(())
}
