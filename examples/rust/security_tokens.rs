//! CLASP Security & Capability Tokens Example (Rust)
//!
//! Demonstrates authentication and authorization using CPSK tokens.
//!
//! Usage:
//!   # Start router with security
//!   cargo run -p clasp-router-server -- --security token
//!
//!   # Run example
//!   CLASP_TOKEN=cpsk_xxx cargo run --example security_tokens

use clasp_client::{Clasp, ClaspBuilder};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CLASP Security & Tokens Example (Rust) ===");

    let clasp_url = env::var("CLASP_URL").unwrap_or_else(|_| "ws://localhost:7330".to_string());
    println!("Server: {}", clasp_url);

    // Example tokens
    let read_token = env::var("READ_TOKEN").unwrap_or_else(|_| "cpsk_read_only_demo".to_string());
    let lights_token =
        env::var("LIGHTS_TOKEN").unwrap_or_else(|_| "cpsk_lights_control_demo".to_string());
    let admin_token = env::var("ADMIN_TOKEN").unwrap_or_else(|_| "cpsk_admin_demo".to_string());

    // Check if server requires auth
    let test_client = Clasp::connect_to(&clasp_url).await;
    match test_client {
        Ok(c) => {
            println!("\nNote: Server is in OPEN mode (no authentication required)");
            c.disconnect().await;
            return Ok(());
        }
        Err(e) => {
            if e.to_string().contains("AUTH_REQUIRED") {
                println!("\nServer requires authentication - proceeding with demos");
            } else {
                return Err(e.into());
            }
        }
    }

    // =====================
    // Read-Only Demo
    // =====================
    println!("\n=== Read-Only Token Demo ===");
    println!("Scope: read:/**");

    let client = ClaspBuilder::new(&clasp_url)
        .name("Read-Only Client")
        .token(&read_token)
        .build()
        .await?;

    println!("Connected with read-only token");

    // Subscribe works
    client
        .subscribe("/lights/**", |value, address| {
            println!("[READ] {} = {:?}", address, value);
        })
        .await?;
    println!("Subscribed to /lights/** - OK");

    // Try to write
    match client.set("/lights/1/brightness", 0.5).await {
        Ok(_) => println!("Write succeeded - UNEXPECTED!"),
        Err(e) => println!("Write denied as expected: {}", e),
    }

    client.disconnect().await;

    // =====================
    // Scoped Write Demo
    // =====================
    println!("\n=== Scoped Write Token Demo ===");
    println!("Scope: read:/**,write:/lights/**");

    let client = ClaspBuilder::new(&clasp_url)
        .name("Lights Controller")
        .token(&lights_token)
        .build()
        .await?;

    println!("Connected with lights control token");

    // Write to lights (allowed)
    client.set("/lights/living-room/brightness", 0.75).await?;
    println!("Set /lights/living-room/brightness = 0.75 - OK");

    // Try to write outside namespace
    match client.set("/audio/master/volume", 0.8).await {
        Ok(_) => println!("Write to /audio succeeded - UNEXPECTED!"),
        Err(e) => println!("Write to /audio denied as expected: {}", e),
    }

    client.disconnect().await;

    // =====================
    // Admin Demo
    // =====================
    println!("\n=== Admin Token Demo ===");
    println!("Scope: admin:/**");

    let client = ClaspBuilder::new(&clasp_url)
        .name("Admin Client")
        .token(&admin_token)
        .build()
        .await?;

    println!("Connected with admin token");

    // Admin can write everywhere
    client.set("/lights/1/brightness", 1.0).await?;
    client.set("/audio/master/volume", 0.7).await?;
    client.set("/system/config/debug", true).await?;
    println!("Admin writes to all namespaces - OK");

    client.disconnect().await;

    println!("\n=== All security demos complete ===");
    Ok(())
}
