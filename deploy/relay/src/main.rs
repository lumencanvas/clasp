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
//! # All protocols
//! clasp-relay --mqtt-port 1883 --osc-port 8000 --quic-port 7331 --cert cert.pem --key key.pem
//! ```

mod auth;

use anyhow::Result;
use clap::Parser;
use clasp_core::security::{CpskValidator, TokenValidator, ValidationResult};
use clasp_core::SecurityMode;
use clasp_router::{MultiProtocolConfig, Router, RouterConfig, RouterStateConfig};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

/// Wrapper to share a CpskValidator between the router and auth module.
/// Both hold Arc<CpskValidator> pointing to the same instance.
struct SharedValidator(Arc<CpskValidator>);

impl TokenValidator for SharedValidator {
    fn validate(&self, token: &str) -> ValidationResult {
        self.0.validate(token)
    }
    fn name(&self) -> &str {
        self.0.name()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(feature = "rendezvous")]
use clasp_discovery::rendezvous::{RendezvousConfig, RendezvousServer};

#[derive(Parser)]
#[command(name = "clasp-relay")]
#[command(about = "CLASP Multi-Protocol Relay Server")]
#[command(version)]
struct Cli {
    /// WebSocket listen port (default: 7330)
    #[arg(short = 'p', long = "ws-port", alias = "port", default_value = "7330")]
    ws_port: u16,

    /// Listen host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// Server name (shown in WELCOME)
    #[arg(short, long, default_value = "CLASP Relay")]
    name: String,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Auth HTTP server port (enables authentication)
    #[arg(long)]
    auth_port: Option<u16>,

    /// Auth database path
    #[arg(long, default_value = "chat-auth.db")]
    auth_db: String,

    /// QUIC listen port (enables QUIC transport, requires --cert and --key)
    #[arg(long)]
    quic_port: Option<u16>,

    /// MQTT listen port (enables MQTT server adapter)
    #[arg(long)]
    mqtt_port: Option<u16>,

    /// MQTT namespace prefix (default: /mqtt)
    #[arg(long, default_value = "/mqtt")]
    mqtt_namespace: String,

    /// OSC listen port (enables OSC server adapter)
    #[arg(long)]
    osc_port: Option<u16>,

    /// OSC namespace prefix (default: /osc)
    #[arg(long, default_value = "/osc")]
    osc_namespace: String,

    /// TLS certificate file (PEM format, for QUIC and MQTTS)
    #[arg(long)]
    cert: Option<PathBuf>,

    /// TLS private key file (PEM format, for QUIC and MQTTS)
    #[arg(long)]
    key: Option<PathBuf>,

    /// Maximum clients (0 = unlimited)
    #[arg(long, default_value = "1000")]
    max_sessions: usize,

    /// Session timeout in seconds
    #[arg(long, default_value = "300")]
    session_timeout: u64,

    /// Disable WebSocket (use other protocols only)
    #[arg(long)]
    no_websocket: bool,

    /// Parameter TTL in seconds (0 = disabled, default: 3600 = 1 hour)
    /// Parameters not updated within this time will be automatically removed.
    #[arg(long, default_value = "3600")]
    param_ttl: u64,

    /// Signal TTL in seconds (0 = disabled, default: 3600 = 1 hour)
    /// Signal definitions not accessed within this time will be automatically removed.
    #[arg(long, default_value = "3600")]
    signal_ttl: u64,

    /// Disable all TTL expiration (parameters and signals persist indefinitely)
    #[arg(long)]
    no_ttl: bool,

    /// Rendezvous server port for WAN discovery (default: same as ws-port, serves /api/v1/*)
    /// Set to 0 to disable rendezvous server.
    #[arg(long, default_value = "7340")]
    rendezvous_port: u16,

    /// Rendezvous TTL in seconds (how long device registrations last)
    #[arg(long, default_value = "300")]
    rendezvous_ttl: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let filter = if cli.verbose {
        EnvFilter::new("debug,clasp=trace")
    } else {
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"))
    };
    tracing_subscriber::fmt().with_env_filter(filter).init();

    tracing::info!("╔══════════════════════════════════════════════════════════════╗");
    tracing::info!("║           CLASP Multi-Protocol Relay Server                  ║");
    tracing::info!("╚══════════════════════════════════════════════════════════════╝");

    // Create state store configuration based on CLI flags
    let state_config = if cli.no_ttl {
        tracing::info!("TTL disabled: parameters and signals persist indefinitely");
        RouterStateConfig::unlimited()
    } else {
        let param_ttl = if cli.param_ttl > 0 {
            Some(Duration::from_secs(cli.param_ttl))
        } else {
            None
        };
        let signal_ttl = if cli.signal_ttl > 0 {
            Some(Duration::from_secs(cli.signal_ttl))
        } else {
            None
        };
        tracing::info!(
            "TTL enabled: param_ttl={:?}, signal_ttl={:?}",
            param_ttl,
            signal_ttl
        );
        RouterStateConfig {
            param_config: clasp_core::state::StateStoreConfig {
                max_params: Some(100_000),
                param_ttl,
                eviction: clasp_core::state::EvictionStrategy::Lru,
            },
            signal_ttl,
            max_signals: Some(100_000),
        }
    };

    // Determine security mode based on auth
    let auth_enabled = cli.auth_port.is_some();
    let security_mode = if auth_enabled {
        SecurityMode::Authenticated
    } else {
        SecurityMode::Open
    };

    // Create router configuration
    let config = RouterConfig {
        name: cli.name.clone(),
        security_mode,
        max_sessions: cli.max_sessions,
        session_timeout: cli.session_timeout,
        features: vec![
            "param".to_string(),
            "event".to_string(),
            "stream".to_string(),
            "timeline".to_string(),
            "gesture".to_string(),
        ],
        max_subscriptions_per_session: 100,
        gesture_coalescing: true,
        gesture_coalesce_interval_ms: 16,
        max_messages_per_second: if auth_enabled { 30 } else { 0 },
        rate_limiting_enabled: auth_enabled,
        state_config,
    };

    let mut router = Router::new(config);

    // Create shared validator and start auth HTTP server if enabled
    if let Some(auth_port) = cli.auth_port {
        let validator = Arc::new(CpskValidator::new());
        router.set_validator(SharedValidator(Arc::clone(&validator)));

        let auth_state = Arc::new(
            auth::AuthState::new(&cli.auth_db, validator)
                .expect("Failed to initialize auth database"),
        );
        let auth_app = auth::auth_router(auth_state);
        let auth_addr: SocketAddr = format!("{}:{}", cli.host, auth_port).parse()?;
        tracing::info!("Auth HTTP: http://{}", auth_addr);

        let listener = tokio::net::TcpListener::bind(auth_addr).await?;
        tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, auth_app).await {
                tracing::error!("Auth server error: {}", e);
            }
        });
    }

    // Build multi-protocol configuration
    let mut protocols = Vec::new();

    // WebSocket (default)
    #[cfg(feature = "websocket")]
    let websocket_addr = if !cli.no_websocket {
        let addr = format!("{}:{}", cli.host, cli.ws_port);
        tracing::info!("WebSocket: ws://{}", addr);
        protocols.push("WebSocket");
        Some(addr)
    } else {
        None
    };

    #[cfg(not(feature = "websocket"))]
    let websocket_addr: Option<String> = None;

    // QUIC
    #[cfg(feature = "quic")]
    let quic_config = if let Some(quic_port) = cli.quic_port {
        let cert_path = cli
            .cert
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--cert required for QUIC"))?;
        let key_path = cli
            .key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("--key required for QUIC"))?;

        // Load certificate and key
        let cert_pem = std::fs::read(cert_path)?;
        let key_pem = std::fs::read(key_path)?;

        // Parse PEM to DER
        let cert_der = rustls_pemfile::certs(&mut cert_pem.as_slice())
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No certificate found in PEM file"))?
            .to_vec();

        let key_der = rustls_pemfile::private_key(&mut key_pem.as_slice())?
            .ok_or_else(|| anyhow::anyhow!("No private key found in PEM file"))?
            .secret_der()
            .to_vec();

        let addr: SocketAddr = format!("{}:{}", cli.host, quic_port).parse()?;
        tracing::info!("QUIC: {}", addr);
        protocols.push("QUIC");

        Some(clasp_router::QuicServerConfig {
            addr,
            cert: cert_der,
            key: key_der,
        })
    } else {
        None
    };

    #[cfg(not(feature = "quic"))]
    let _quic_config: Option<()> = None;

    // MQTT
    #[cfg(feature = "mqtt-server")]
    let mqtt_config = if let Some(mqtt_port) = cli.mqtt_port {
        let addr = format!("{}:{}", cli.host, mqtt_port);
        tracing::info!("MQTT: mqtt://{} (namespace: {})", addr, cli.mqtt_namespace);
        protocols.push("MQTT");

        Some(clasp_router::MqttServerConfig {
            bind_addr: addr,
            namespace: cli.mqtt_namespace.clone(),
            require_auth: false,
            tls: None,
            max_clients: cli.max_sessions,
            session_timeout_secs: cli.session_timeout,
        })
    } else {
        None
    };

    #[cfg(not(feature = "mqtt-server"))]
    let _mqtt_config: Option<()> = None;

    // OSC
    #[cfg(feature = "osc-server")]
    let osc_config = if let Some(osc_port) = cli.osc_port {
        let addr = format!("{}:{}", cli.host, osc_port);
        tracing::info!("OSC: udp://{} (namespace: {})", addr, cli.osc_namespace);
        protocols.push("OSC");

        Some(clasp_router::OscServerConfig {
            bind_addr: addr,
            namespace: cli.osc_namespace.clone(),
            session_timeout_secs: 30,
            auto_subscribe: false,
        })
    } else {
        None
    };

    #[cfg(not(feature = "osc-server"))]
    let _osc_config: Option<()> = None;

    if protocols.is_empty() {
        anyhow::bail!("No protocols enabled. Enable at least one of: WebSocket, QUIC, MQTT, OSC");
    }

    tracing::info!("Server name: {}", cli.name);
    tracing::info!("Protocols: {}", protocols.join(", "));
    tracing::info!(
        "Max sessions: {}, Timeout: {}s",
        cli.max_sessions,
        cli.session_timeout
    );
    tracing::info!("Security: {:?}", if auth_enabled { "Authenticated" } else { "Open" });
    if cli.no_ttl {
        tracing::info!("TTL: disabled (unlimited parameter lifetime)");
    } else {
        tracing::info!("TTL: param={}s, signal={}s", cli.param_ttl, cli.signal_ttl);
    }
    tracing::info!("────────────────────────────────────────────────────────────────");

    // Create multi-protocol config
    let multi_config = MultiProtocolConfig {
        #[cfg(feature = "websocket")]
        websocket_addr,
        #[cfg(feature = "quic")]
        quic: quic_config,
        #[cfg(feature = "mqtt-server")]
        mqtt: mqtt_config,
        #[cfg(feature = "osc-server")]
        osc: osc_config,
    };

    tracing::info!("Router initialized, accepting connections...");

    // Start rendezvous server if enabled
    #[cfg(feature = "rendezvous")]
    if cli.rendezvous_port > 0 {
        let rendezvous_addr = format!("{}:{}", cli.host, cli.rendezvous_port);
        tracing::info!(
            "Rendezvous: http://{} (TTL: {}s)",
            rendezvous_addr,
            cli.rendezvous_ttl
        );

        let rendezvous_config = RendezvousConfig {
            ttl: cli.rendezvous_ttl,
            ..Default::default()
        };
        let rendezvous = RendezvousServer::new(rendezvous_config);

        // Spawn rendezvous server in background
        let rendezvous_addr_clone = rendezvous_addr.clone();
        tokio::spawn(async move {
            if let Err(e) = rendezvous.serve(&rendezvous_addr_clone).await {
                tracing::error!("Rendezvous server error: {}", e);
            }
        });
    }

    // Serve all protocols
    router.serve_all(multi_config).await?;

    Ok(())
}
