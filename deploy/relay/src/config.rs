//! CLI argument parsing and programmatic configuration for the CLASP relay server.
//!
//! Two configuration paths exist:
//! - [`Cli`]: `clap`-based argument parsing for the binary entrypoint.
//! - [`RelayConfig`]: Programmatic configuration for library consumers.
//!
//! The binary converts `Cli` -> `RelayConfig` via `From<Cli>`. Library users
//! construct `RelayConfig` directly (with [`Default`] providing sane defaults).

use clasp_router::{WriteValidator, SnapshotFilter};
use clap::Parser;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[cfg(feature = "rendezvous")]
pub use clasp_discovery::rendezvous::{RendezvousConfig, RendezvousServer};

// ---------------------------------------------------------------------------
// CLI (binary entrypoint)
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "clasp-relay")]
#[command(about = "CLASP Multi-Protocol Relay Server")]
#[command(version)]
pub struct Cli {
    /// WebSocket listen port (default: 7330)
    #[arg(short = 'p', long = "ws-port", alias = "port", default_value = "7330")]
    pub ws_port: u16,

    /// Listen host
    #[arg(long, default_value = "0.0.0.0")]
    pub host: String,

    /// Server name (shown in WELCOME)
    #[arg(short, long, default_value = "CLASP Relay")]
    pub name: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Auth HTTP server port (enables authentication)
    #[arg(long)]
    pub auth_port: Option<u16>,

    /// Auth database path
    #[arg(long, default_value = "relay-auth.db")]
    pub auth_db: String,

    /// QUIC listen port (enables QUIC transport, requires --cert and --key)
    #[arg(long)]
    pub quic_port: Option<u16>,

    /// MQTT listen port (enables MQTT server adapter)
    #[arg(long)]
    pub mqtt_port: Option<u16>,

    /// MQTT namespace prefix (default: /mqtt)
    #[arg(long, default_value = "/mqtt")]
    pub mqtt_namespace: String,

    /// OSC listen port (enables OSC server adapter)
    #[arg(long)]
    pub osc_port: Option<u16>,

    /// OSC namespace prefix (default: /osc)
    #[arg(long, default_value = "/osc")]
    pub osc_namespace: String,

    /// TLS certificate file (PEM format, for QUIC and MQTTS)
    #[arg(long)]
    pub cert: Option<PathBuf>,

    /// TLS private key file (PEM format, for QUIC and MQTTS)
    #[arg(long)]
    pub key: Option<PathBuf>,

    /// Maximum clients (0 = unlimited)
    #[arg(long, default_value = "1000")]
    pub max_sessions: usize,

    /// Session timeout in seconds
    #[arg(long, default_value = "300")]
    pub session_timeout: u64,

    /// Disable WebSocket (use other protocols only)
    #[arg(long)]
    pub no_websocket: bool,

    /// Parameter TTL in seconds (0 = disabled, default: 3600 = 1 hour)
    /// Parameters not updated within this time will be automatically removed.
    #[arg(long, default_value = "3600")]
    pub param_ttl: u64,

    /// Signal TTL in seconds (0 = disabled, default: 3600 = 1 hour)
    /// Signal definitions not accessed within this time will be automatically removed.
    #[arg(long, default_value = "3600")]
    pub signal_ttl: u64,

    /// Disable all TTL expiration (parameters and signals persist indefinitely)
    #[arg(long)]
    pub no_ttl: bool,

    /// Rendezvous server port for WAN discovery (default: same as ws-port, serves /api/v1/*)
    /// Set to 0 to disable rendezvous server.
    #[arg(long, default_value = "7340")]
    pub rendezvous_port: u16,

    /// Rendezvous TTL in seconds (how long device registrations last)
    #[arg(long, default_value = "300")]
    pub rendezvous_ttl: u64,

    /// Path to state snapshot file (enables persistence across restarts)
    #[arg(long)]
    pub persist: Option<PathBuf>,

    /// Snapshot interval in seconds (default: 30)
    #[arg(long, default_value = "30")]
    pub persist_interval: u64,

    /// Allowed CORS origin(s) for the auth API (comma-separated).
    /// If not set, CORS is permissive (development only).
    #[arg(long)]
    pub cors_origin: Option<String>,

    // -- Journal --

    /// SQLite journal path for state persistence and replay
    #[arg(long)]
    pub journal: Option<PathBuf>,

    /// Use in-memory journal (ring buffer, no persistence)
    #[arg(long)]
    pub journal_memory: bool,

    // -- Capability Tokens --

    /// Trust anchor public key file(s) for capability tokens (32-byte Ed25519, repeatable)
    #[arg(long = "trust-anchor")]
    pub trust_anchor: Vec<PathBuf>,

    /// Maximum delegation chain depth for capability tokens (default: 5)
    #[arg(long = "cap-max-depth", default_value = "5")]
    pub cap_max_depth: usize,

    // -- Entity Registry --

    /// SQLite database path for the entity registry
    #[arg(long = "registry-db")]
    pub registry_db: Option<PathBuf>,

    // -- Rules Engine --

    /// JSON file containing rule definitions
    #[arg(long)]
    pub rules: Option<PathBuf>,

    // -- App Config --

    /// JSON file defining scopes, write rules, and snapshot rules for the application.
    /// If not specified, auto-detects from /etc/clasp/ or ./config/ (single JSON file).
    #[arg(long = "app-config")]
    pub app_config: Option<PathBuf>,

    // -- Admin Bootstrap --

    /// Default TTL for CPSK tokens in seconds (0 = no default expiry).
    /// Tokens registered without an explicit expiry will expire after this duration.
    #[arg(long = "token-ttl", default_value = "86400")]
    pub token_ttl: u64,

    /// Admin token file path. If the file exists, reads the token from it.
    /// If not, generates a new admin token and writes it to the file.
    /// The token is registered with admin:/** scope (no expiry).
    #[arg(long = "admin-token")]
    pub admin_token: Option<PathBuf>,

    // -- Federation --

    /// Hub WebSocket URL for federation leaf mode (e.g. ws://hub:7330)
    #[arg(long = "federation-hub")]
    pub federation_hub: Option<String>,

    /// Local router identity for federation
    #[arg(long = "federation-id")]
    pub federation_id: Option<String>,

    /// Namespace pattern(s) owned by this router (repeatable)
    #[arg(long = "federation-namespace")]
    pub federation_namespace: Vec<String>,

    /// Auth token to present to the federation hub
    #[arg(long = "federation-token")]
    pub federation_token: Option<String>,

    // -- Metrics --

    /// Prometheus metrics HTTP port (enables /metrics endpoint).
    /// Requires the `metrics` feature. Default: 9090.
    #[arg(long = "metrics-port")]
    pub metrics_port: Option<u16>,

    // -- Health --

    /// Health check HTTP port (enables /healthz and /readyz endpoints).
    #[arg(long = "health-port")]
    pub health_port: Option<u16>,

    /// Graceful shutdown drain timeout in seconds (default: 30).
    /// After receiving SIGTERM, the server waits this long before force-closing connections.
    #[arg(long = "drain-timeout", default_value = "30")]
    pub drain_timeout: u64,
}

// ---------------------------------------------------------------------------
// RelayConfig (library API)
// ---------------------------------------------------------------------------

/// Programmatic relay configuration for library consumers.
///
/// Construct with [`Default::default()`] for sane defaults, then override
/// individual fields. The binary entrypoint converts from [`Cli`] via
/// `RelayConfig::from(cli)`.
///
/// # Example
///
/// ```rust,no_run
/// use clasp_relay::config::RelayConfig;
///
/// let config = RelayConfig {
///     ws_port: 9000,
///     auth_port: Some(9001),
///     ..Default::default()
/// };
/// ```
pub struct RelayConfig {
    // -- Network --
    pub ws_port: u16,
    pub host: String,
    pub name: String,
    pub auth_port: Option<u16>,
    pub auth_db: String,
    pub health_port: Option<u16>,
    pub no_websocket: bool,

    // -- QUIC --
    pub quic_port: Option<u16>,
    pub cert: Option<PathBuf>,
    pub key: Option<PathBuf>,

    // -- MQTT --
    pub mqtt_port: Option<u16>,
    pub mqtt_namespace: String,

    // -- OSC --
    pub osc_port: Option<u16>,
    pub osc_namespace: String,

    // -- Sessions --
    pub max_sessions: usize,
    pub session_timeout: u64,

    // -- TTL --
    pub no_ttl: bool,
    pub param_ttl: u64,
    pub signal_ttl: u64,

    // -- Rendezvous --
    pub rendezvous_port: u16,
    pub rendezvous_ttl: u64,

    // -- Persistence --
    pub persist: Option<PathBuf>,
    pub persist_interval: u64,

    // -- Auth --
    pub cors_origin: Option<String>,
    pub token_ttl: u64,
    pub admin_token: Option<PathBuf>,

    // -- Journal --
    pub journal: Option<PathBuf>,
    pub journal_memory: bool,

    // -- Capability Tokens --
    pub trust_anchor: Vec<PathBuf>,
    pub cap_max_depth: usize,

    // -- Entity Registry --
    pub registry_db: Option<PathBuf>,

    // -- Rules --
    pub rules: Option<PathBuf>,

    // -- App Config --
    pub app_config: Option<crate::app_config::AppConfig>,

    // -- Federation --
    pub federation_hub: Option<String>,
    pub federation_id: Option<String>,
    pub federation_namespace: Vec<String>,
    pub federation_token: Option<String>,

    // -- Metrics --
    pub metrics_port: Option<u16>,

    // -- Shutdown --
    pub drain_timeout: Duration,

    // -- Injectable validators (library API) --
    /// Application-specific write validator. If `None`, no custom validation
    /// is applied beyond scope checks.
    pub write_validator: Option<Arc<dyn WriteValidator>>,
    /// Application-specific snapshot filter. If `None`, snapshots are
    /// delivered unfiltered.
    pub snapshot_filter: Option<Arc<dyn SnapshotFilter>>,
}

impl Default for RelayConfig {
    fn default() -> Self {
        Self {
            ws_port: 7330,
            host: "0.0.0.0".into(),
            name: "CLASP Relay".into(),
            auth_port: None,
            auth_db: "relay-auth.db".into(),
            health_port: None,
            no_websocket: false,
            quic_port: None,
            cert: None,
            key: None,
            mqtt_port: None,
            mqtt_namespace: "/mqtt".into(),
            osc_port: None,
            osc_namespace: "/osc".into(),
            max_sessions: 1000,
            session_timeout: 300,
            no_ttl: false,
            param_ttl: 3600,
            signal_ttl: 3600,
            rendezvous_port: 7340,
            rendezvous_ttl: 300,
            persist: None,
            persist_interval: 30,
            cors_origin: None,
            token_ttl: 86400,
            admin_token: None,
            journal: None,
            journal_memory: false,
            trust_anchor: Vec::new(),
            cap_max_depth: 5,
            registry_db: None,
            rules: None,
            app_config: None,
            federation_hub: None,
            federation_id: None,
            federation_namespace: Vec::new(),
            federation_token: None,
            metrics_port: None,
            drain_timeout: Duration::from_secs(30),
            write_validator: None,
            snapshot_filter: None,
        }
    }
}

impl From<Cli> for RelayConfig {
    fn from(cli: Cli) -> Self {
        // Resolve app config path: explicit flag or auto-detect from well-known locations
        let app_config_path = cli.app_config.clone().or_else(|| {
            for dir in &["/etc/clasp", "./config"] {
                let dir = std::path::Path::new(dir);
                if dir.is_dir() {
                    if let Ok(entries) = std::fs::read_dir(dir) {
                        let jsons: Vec<_> = entries
                            .filter_map(|e| e.ok())
                            .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
                            .collect();
                        if jsons.len() == 1 {
                            let path = jsons[0].path();
                            tracing::info!("Auto-detected app config: {}", path.display());
                            return Some(path);
                        } else if jsons.len() > 1 {
                            tracing::debug!(
                                "Multiple JSON files in {}, skipping auto-detect (use --app-config)",
                                dir.display()
                            );
                        }
                    }
                }
            }
            None
        });

        let app_config = app_config_path.as_ref().map(|path| {
            let json = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read app config {}: {}", path.display(), e));
            serde_json::from_str::<crate::app_config::AppConfig>(&json)
                .unwrap_or_else(|e| panic!("Failed to parse app config {}: {}", path.display(), e))
        });

        Self {
            ws_port: cli.ws_port,
            host: cli.host,
            name: cli.name,
            auth_port: cli.auth_port,
            auth_db: cli.auth_db,
            health_port: cli.health_port,
            no_websocket: cli.no_websocket,
            quic_port: cli.quic_port,
            cert: cli.cert,
            key: cli.key,
            mqtt_port: cli.mqtt_port,
            mqtt_namespace: cli.mqtt_namespace,
            osc_port: cli.osc_port,
            osc_namespace: cli.osc_namespace,
            max_sessions: cli.max_sessions,
            session_timeout: cli.session_timeout,
            no_ttl: cli.no_ttl,
            param_ttl: cli.param_ttl,
            signal_ttl: cli.signal_ttl,
            rendezvous_port: cli.rendezvous_port,
            rendezvous_ttl: cli.rendezvous_ttl,
            persist: cli.persist,
            persist_interval: cli.persist_interval,
            cors_origin: cli.cors_origin,
            token_ttl: cli.token_ttl,
            admin_token: cli.admin_token,
            journal: cli.journal,
            journal_memory: cli.journal_memory,
            trust_anchor: cli.trust_anchor,
            cap_max_depth: cli.cap_max_depth,
            registry_db: cli.registry_db,
            rules: cli.rules,
            app_config,
            federation_hub: cli.federation_hub,
            federation_id: cli.federation_id,
            federation_namespace: cli.federation_namespace,
            federation_token: cli.federation_token,
            metrics_port: cli.metrics_port,
            drain_timeout: Duration::from_secs(cli.drain_timeout),
            // The binary sets chat-specific validators below in main.rs;
            // library consumers provide their own or leave as None.
            write_validator: None,
            snapshot_filter: None,
        }
    }
}
