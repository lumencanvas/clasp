//! Entity registry subcommands: list, get, and create entities via the
//! relay's REST API.

use anyhow::{Context, Result};
use colored::Colorize;

/// List all entities from the relay's entity registry.
pub async fn handle_list(relay_url: &str) -> Result<()> {
    let url = format!("{}/api/entities", relay_url.trim_end_matches('/'));
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to reach relay at {}", url))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GET {} returned {}: {}", url, status, body);
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse entity list response as JSON")?;

    if let Some(entities) = body.as_array() {
        if entities.is_empty() {
            println!("No entities registered.");
            return Ok(());
        }
        println!("{} {} entity(ies):\n", "CLASP".cyan().bold(), entities.len());
        for entity in entities {
            print_entity(entity);
        }
    } else {
        // Response may be a wrapper object with a field like "entities"
        if let Some(entities) = body.get("entities").and_then(|v| v.as_array()) {
            if entities.is_empty() {
                println!("No entities registered.");
                return Ok(());
            }
            println!("{} {} entity(ies):\n", "CLASP".cyan().bold(), entities.len());
            for entity in entities {
                print_entity(entity);
            }
        } else {
            // Just print the raw JSON
            println!("{}", serde_json::to_string_pretty(&body)?);
        }
    }

    Ok(())
}

/// Get a single entity by ID from the relay's entity registry.
pub async fn handle_get(relay_url: &str, entity_id: &str) -> Result<()> {
    let url = format!(
        "{}/api/entities/{}",
        relay_url.trim_end_matches('/'),
        entity_id
    );
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .send()
        .await
        .with_context(|| format!("Failed to reach relay at {}", url))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GET {} returned {}: {}", url, status, body);
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse entity response as JSON")?;

    println!("{}", serde_json::to_string_pretty(&body)?);

    Ok(())
}

/// Create a new entity in the relay's entity registry.
pub async fn handle_create(relay_url: &str, name: &str, entity_type: &str) -> Result<()> {
    let url = format!("{}/api/entities", relay_url.trim_end_matches('/'));
    let payload = serde_json::json!({
        "name": name,
        "type": entity_type,
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .with_context(|| format!("Failed to reach relay at {}", url))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("POST {} returned {}: {}", url, status, body);
    }

    let body: serde_json::Value = resp
        .json()
        .await
        .context("Failed to parse create response as JSON")?;

    println!("{} Entity created", "OK".green().bold());
    println!("{}", serde_json::to_string_pretty(&body)?);

    Ok(())
}

fn print_entity(entity: &serde_json::Value) {
    let id = entity
        .get("id")
        .or_else(|| entity.get("entity_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("(unknown)");
    let name = entity
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("(unnamed)");
    let etype = entity
        .get("type")
        .or_else(|| entity.get("entity_type"))
        .and_then(|v| v.as_str())
        .unwrap_or("(unknown)");

    println!("  {} {}", id.yellow(), name);
    println!("    Type: {}", etype);

    if let Some(status) = entity.get("status").and_then(|v| v.as_str()) {
        println!("    Status: {}", status);
    }
    println!();
}
