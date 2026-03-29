//! Journal query subcommands: query, since, latest, snapshot via relay REST API.

use anyhow::{Context, Result};
use colored::Colorize;

/// Query journal entries by address pattern.
pub async fn handle_query(
    relay_url: &str,
    pattern: &str,
    from: Option<u64>,
    to: Option<u64>,
    limit: Option<u32>,
    types: Option<&str>,
    token: Option<&str>,
) -> Result<()> {
    let mut url = format!(
        "{}/api/journal/query?pattern={}",
        relay_url.trim_end_matches('/'),
        percent_encode(pattern),
    );
    if let Some(f) = from {
        url.push_str(&format!("&from={}", f));
    }
    if let Some(t) = to {
        url.push_str(&format!("&to={}", t));
    }
    if let Some(l) = limit {
        url.push_str(&format!("&limit={}", l));
    }
    if let Some(t) = types {
        url.push_str(&format!("&types={}", t));
    }

    let body = api_get(&url, token).await?;
    print_journal_entries(&body);
    Ok(())
}

/// Get journal entries since a sequence number.
pub async fn handle_since(
    relay_url: &str,
    seq: u64,
    limit: Option<u32>,
    token: Option<&str>,
) -> Result<()> {
    let mut url = format!(
        "{}/api/journal/since?seq={}",
        relay_url.trim_end_matches('/'),
        seq,
    );
    if let Some(l) = limit {
        url.push_str(&format!("&limit={}", l));
    }

    let body = api_get(&url, token).await?;
    print_journal_entries(&body);
    Ok(())
}

/// Get the latest journal sequence number.
pub async fn handle_latest(relay_url: &str, token: Option<&str>) -> Result<()> {
    let url = format!("{}/api/journal/latest", relay_url.trim_end_matches('/'));

    let body = api_get(&url, token).await?;

    if let Some(seq) = body.get("seq").and_then(|v| v.as_u64()) {
        println!("{}: {}", "Latest seq".cyan(), seq);
    }
    if let Some(ts) = body.get("timestamp").and_then(|v| v.as_u64()) {
        println!("{}: {}", "Timestamp".cyan(), format_micros(ts));
    }
    if body.get("seq").is_none() && body.get("timestamp").is_none() {
        println!("{}", serde_json::to_string_pretty(&body)?);
    }

    Ok(())
}

/// Load the most recent state snapshot from the journal.
pub async fn handle_snapshot(relay_url: &str, token: Option<&str>) -> Result<()> {
    let url = format!("{}/api/journal/snapshot", relay_url.trim_end_matches('/'));

    let body = api_get(&url, token).await?;
    println!("{}", serde_json::to_string_pretty(&body)?);

    Ok(())
}

// -------------------------------------------------------------------------
// Helpers
// -------------------------------------------------------------------------

async fn api_get(url: &str, token: Option<&str>) -> Result<serde_json::Value> {
    let client = reqwest::Client::new();
    let mut req = client.get(url);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {}", t));
    }

    let resp = req
        .send()
        .await
        .with_context(|| format!("Failed to reach relay at {}", url))?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        anyhow::bail!("GET {} returned {}: {}", url, status, body);
    }

    resp.json()
        .await
        .context("Failed to parse response as JSON")
}

fn print_journal_entries(body: &serde_json::Value) {
    let entries = body
        .as_array()
        .or_else(|| body.get("entries").and_then(|v| v.as_array()));

    match entries {
        Some(arr) if arr.is_empty() => {
            println!("No entries.");
        }
        Some(arr) => {
            for entry in arr {
                print_entry(entry);
            }
        }
        None => {
            // Fall back to raw JSON
            if let Ok(pretty) = serde_json::to_string_pretty(body) {
                println!("{}", pretty);
            }
        }
    }
}

fn print_entry(entry: &serde_json::Value) {
    let seq = entry
        .get("seq")
        .and_then(|v| v.as_u64())
        .map(|v| v.to_string())
        .unwrap_or_else(|| "?".into());

    let ts = entry
        .get("timestamp")
        .and_then(|v| v.as_u64())
        .map(format_micros)
        .unwrap_or_else(|| "?".into());

    let op = entry
        .get("op")
        .or_else(|| entry.get("type"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");

    let address = entry
        .get("address")
        .or_else(|| entry.get("path"))
        .and_then(|v| v.as_str())
        .unwrap_or("?");

    let value_str = entry
        .get("value")
        .map(|v| {
            if let Some(s) = v.as_str() {
                s.to_string()
            } else {
                v.to_string()
            }
        })
        .unwrap_or_default();

    let mut meta_parts: Vec<String> = Vec::new();
    if let Some(rev) = entry.get("rev").and_then(|v| v.as_u64()) {
        meta_parts.push(format!("rev:{}", rev));
    }
    if let Some(by) = entry.get("by").and_then(|v| v.as_str()) {
        meta_parts.push(format!("by:{}", by));
    }

    let meta = if meta_parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", meta_parts.join(", "))
    };

    let value_display = if value_str.is_empty() {
        String::new()
    } else {
        format!(" = {}", value_str)
    };

    println!(
        "[seq:{}] {} {} {}{}{}",
        seq.yellow(),
        ts,
        op.to_uppercase().green(),
        address.cyan(),
        value_display,
        meta,
    );
}

/// Format microsecond timestamp to ISO-8601-ish string.
fn format_micros(us: u64) -> String {
    let secs = us / 1_000_000;
    let subsec_us = us % 1_000_000;

    // Simple UTC breakdown without chrono dependency
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let h = time_of_day / 3600;
    let m = (time_of_day % 3600) / 60;
    let s = time_of_day % 60;

    // Gregorian date from day count (algorithm from Howard Hinnant)
    let z = days_since_epoch as i64 + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 };
    let yr = if mo <= 2 { y + 1 } else { y };

    if subsec_us == 0 {
        format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", yr, mo, d, h, m, s)
    } else {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:06}Z",
            yr, mo, d, h, m, s, subsec_us
        )
    }
}

/// Minimal percent-encoding for URL query parameter values.
fn percent_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'A'..=b'Z'
            | b'a'..=b'z'
            | b'0'..=b'9'
            | b'-'
            | b'_'
            | b'.'
            | b'~'
            | b'/'
            | b'*' => out.push(b as char),
            _ => {
                out.push('%');
                out.push_str(&format!("{:02X}", b));
            }
        }
    }
    out
}
