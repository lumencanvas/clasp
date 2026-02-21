//! CLI argument parsing and configuration for the CLASP relay server.
//!
//! All runtime options (ports, auth, TLS, persistence, federation, etc.) are
//! defined as `clap` arguments in [`Cli`].

use clap::Parser;
use std::path::PathBuf;

#[cfg(feature = "rendezvous")]
pub use clasp_discovery::rendezvous::{RendezvousConfig, RendezvousServer};

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
    #[arg(long, default_value = "chat-auth.db")]
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
