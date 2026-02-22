//! CLASP Relay Server (Multi-Protocol)
//!
//! A CLASP relay server supporting multiple protocols:
//! - WebSocket (default, port 7330)
//! - QUIC (optional, port 7331)
//! - MQTT (optional, port 1883)
//! - OSC (optional, port 8000)
//!
//! All protocols share the same router state, allowing cross-protocol communication.
//!
//! # Usage
//!
//! ```bash
//! # Default (WebSocket only on port 7330)
//! clasp-relay
//!
//! # WebSocket + MQTT
//! clasp-relay --mqtt-port 1883
//!
//! # With auth enabled
//! clasp-relay --auth-port 7350
//!
//! # With app-specific rules
//! clasp-relay --auth-port 7350 --app-config config/chat.json
//!
//! # All protocols
//! clasp-relay --mqtt-port 1883 --osc-port 8000 --quic-port 7331 --cert cert.pem --key key.pem
//! ```

mod app_config;
mod auth;
mod config;
mod cpsk;
#[cfg(feature = "federation")]
mod federation;
mod health;
#[cfg(feature = "registry")]
mod registry;
mod server;

use anyhow::Result;
use clap::Parser;
use config::RelayConfig;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = config::Cli::parse();

    // Setup logging (uses cli.verbose before conversion)
    let filter = if cli.verbose {
        EnvFilter::new("debug,clasp=trace")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };

    let use_json = std::env::var("LOG_FORMAT")
        .map(|v| v.eq_ignore_ascii_case("json"))
        .unwrap_or(false);

    if use_json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(filter)
            .init();
    } else {
        tracing_subscriber::fmt().with_env_filter(filter).init();
    }

    // Convert CLI args to RelayConfig (loads --app-config if specified)
    let config = RelayConfig::from(cli);

    // Rule-based validators are created in server.rs from the app config.
    // Library consumers can still inject compiled Rust validators via
    // RelayConfig.write_validator / .snapshot_filter (takes precedence).
    server::run(config).await
}
