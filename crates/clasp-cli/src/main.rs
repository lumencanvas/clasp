//! CLASP CLI - Command-line interface for CLASP protocol servers and bridges
//!
//! Start protocol servers, bridges, and manage CLASP signals from the command line.

mod server;
mod tokens;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use ed25519_dalek::SigningKey;
use std::path::PathBuf;
use tokens::{create_token, default_token_file, format_timestamp, TokenStore};
use tokio::sync::mpsc;
use tracing::{info, warn};
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// CLASP - Creative Low-Latency Application Streaming Protocol
#[derive(Parser)]
#[command(name = "clasp")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, global = true, default_value = "info")]
    log_level: String,

    /// Output logs as JSON
    #[arg(long, global = true)]
    json_logs: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start a CLASP server
    Server {
        /// Protocol to serve (quic, tcp, websocket)
        #[arg(short, long, default_value = "quic")]
        protocol: String,

        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,

        /// Port number
        #[arg(short = 'P', long, default_value = "7331")]
        port: u16,
    },

    /// Start a protocol bridge
    Bridge {
        /// Bridge type (osc, midi, artnet, mqtt, websocket, http)
        #[arg(short, long)]
        bridge_type: String,

        /// Configuration options (key=value pairs)
        #[arg(short, long)]
        opt: Vec<String>,
    },

    /// Start an OSC server
    Osc {
        /// UDP port to listen on
        #[arg(short, long, default_value = "9000")]
        port: u16,

        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0")]
        bind: String,
    },

    /// Start an MQTT broker connection
    Mqtt {
        /// MQTT broker host
        #[arg(short = 'H', long, default_value = "localhost")]
        host: String,

        /// MQTT broker port
        #[arg(short, long, default_value = "1883")]
        port: u16,

        /// Client ID
        #[arg(short, long)]
        client_id: Option<String>,

        /// Topics to subscribe (supports wildcards)
        #[arg(short, long, default_value = "#")]
        topic: Vec<String>,
    },

    /// Start a WebSocket server or client
    Websocket {
        /// Mode: server or client
        #[arg(short, long, default_value = "server")]
        mode: String,

        /// URL (ws://... for client) or bind address for server
        #[arg(short, long, default_value = "0.0.0.0:8080")]
        url: String,
    },

    /// Start an HTTP REST API server
    Http {
        /// Bind address
        #[arg(short, long, default_value = "0.0.0.0:3000")]
        bind: String,

        /// Base path for API endpoints
        #[arg(short, long, default_value = "/api")]
        base_path: String,

        /// Enable CORS
        #[arg(long, default_value = "true")]
        cors: bool,
    },

    /// Publish a value to a CLASP address
    Pub {
        /// CLASP server URL
        #[arg(short, long, default_value = "quic://localhost:7331")]
        server: String,

        /// Signal address
        address: String,

        /// Value to publish (JSON format)
        value: String,
    },

    /// Subscribe to signals
    Sub {
        /// CLASP server URL
        #[arg(short, long, default_value = "quic://localhost:7331")]
        server: String,

        /// Address pattern to subscribe to
        #[arg(default_value = "/**")]
        pattern: String,
    },

    /// Show version and system info
    Info,

    /// Manage Ed25519 keypairs for capability and entity tokens
    Key {
        #[command(subcommand)]
        action: KeyAction,
    },

    /// Manage authentication tokens
    Token {
        /// Token file path (default: ~/.config/clasp/tokens.json)
        #[arg(long)]
        file: Option<String>,

        #[command(subcommand)]
        action: TokenAction,
    },
}

/// Key management actions
#[derive(Subcommand)]
enum KeyAction {
    /// Generate a new Ed25519 keypair
    Generate {
        /// Output file path (hex-encoded signing key)
        #[arg(short, long)]
        out: Option<PathBuf>,
    },

    /// Show the public key for a signing key file
    Show {
        /// Path to signing key file
        path: PathBuf,

        /// Output format: hex (default) or did
        #[arg(long, default_value = "hex")]
        format: String,
    },
}

/// Token management actions
#[derive(Subcommand)]
enum TokenAction {
    /// Create a new CPSK token
    Create {
        /// Scopes (comma-separated, e.g., "read:/**,write:/lights/**")
        #[arg(short, long)]
        scopes: String,

        /// Expiration (e.g., "7d", "24h", "30m")
        #[arg(short, long)]
        expires: Option<String>,

        /// Subject/description for the token
        #[arg(long)]
        subject: Option<String>,
    },

    /// List all CPSK tokens
    List {
        /// Show expired tokens
        #[arg(long)]
        show_expired: bool,
    },

    /// Show details of a specific CPSK token
    Show {
        /// Token string or prefix
        token: String,
    },

    /// Revoke a CPSK token
    Revoke {
        /// Token string or prefix
        token: String,
    },

    /// Remove all expired tokens
    Prune,

    /// Capability token operations (delegatable Ed25519 tokens)
    #[cfg(feature = "caps")]
    Cap {
        #[command(subcommand)]
        action: CapAction,
    },

    /// Entity token operations (device/user/service identity tokens)
    #[cfg(feature = "registry")]
    Entity {
        #[command(subcommand)]
        action: EntityAction,
    },
}

/// Capability token actions
#[cfg(feature = "caps")]
#[derive(Subcommand)]
enum CapAction {
    /// Create a new root capability token
    Create {
        /// Path to signing key file
        #[arg(short, long)]
        key: PathBuf,

        /// Scopes (comma-separated, e.g., "admin:/**")
        #[arg(short, long)]
        scopes: String,

        /// Expiration (e.g., "30d", "24h")
        #[arg(short, long, default_value = "30d")]
        expires: String,

        /// Audience public key (hex, optional -- omit for bearer token)
        #[arg(long)]
        audience: Option<String>,
    },

    /// Delegate (attenuate) a capability token to create a child
    Delegate {
        /// Parent token string (cap_...)
        parent: String,

        /// Path to child signing key file
        #[arg(short, long)]
        key: PathBuf,

        /// Child scopes (must be subset of parent)
        #[arg(short, long)]
        scopes: String,

        /// Expiration (e.g., "7d") -- clamped to parent's expiry
        #[arg(short, long, default_value = "7d")]
        expires: String,

        /// Audience public key (hex, optional)
        #[arg(long)]
        audience: Option<String>,
    },

    /// Inspect a capability token (decode without verification)
    Inspect {
        /// Token string (cap_...)
        token: String,
    },

    /// Verify a capability token against a trust anchor
    Verify {
        /// Token string (cap_...)
        token: String,

        /// Trust anchor public key file (or hex string)
        #[arg(long)]
        trust_anchor: String,

        /// Maximum chain depth (default: 5)
        #[arg(long, default_value = "5")]
        max_depth: usize,
    },
}

/// Entity token actions
#[cfg(feature = "registry")]
#[derive(Subcommand)]
enum EntityAction {
    /// Generate a new entity keypair (does NOT register in the registry --
    /// use the registry API to register the public key after generation)
    Keygen {
        /// Output key file path
        #[arg(short, long)]
        out: Option<PathBuf>,

        /// Entity name (informational, printed to stderr)
        #[arg(short, long)]
        name: Option<String>,

        /// Entity type (device, user, service, router)
        #[arg(short = 't', long, default_value = "device")]
        entity_type: String,
    },

    /// Mint an entity token from a keypair
    Mint {
        /// Path to signing key file
        #[arg(short, long)]
        key: PathBuf,
    },

    /// Inspect an entity token (decode without full verification)
    Inspect {
        /// Token string (ent_...)
        token: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    setup_logging(&cli.log_level, cli.json_logs)?;

    // Handle Ctrl+C
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let shutdown_tx_clone = shutdown_tx.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        info!("Received shutdown signal");
        let _ = shutdown_tx_clone.send(()).await;
    });

    match cli.command {
        Commands::Server {
            protocol,
            bind,
            port,
        } => {
            server::run_server(&protocol, &bind, port, &mut shutdown_rx).await?;
        }

        Commands::Bridge { bridge_type, opt } => {
            run_bridge(&bridge_type, opt, &mut shutdown_rx).await?;
        }

        Commands::Osc { port, bind } => {
            println!(
                "{} Starting OSC server on {}:{}",
                "CLASP".cyan().bold(),
                bind,
                port
            );
            run_osc_server(&bind, port, &mut shutdown_rx).await?;
        }

        Commands::Mqtt {
            host,
            port,
            client_id,
            topic,
        } => {
            println!(
                "{} Connecting to MQTT broker at {}:{}",
                "CLASP".cyan().bold(),
                host,
                port
            );
            run_mqtt_bridge(&host, port, client_id, topic, &mut shutdown_rx).await?;
        }

        Commands::Websocket { mode, url } => {
            println!(
                "{} Starting WebSocket {} on {}",
                "CLASP".cyan().bold(),
                mode,
                url
            );
            run_websocket(&mode, &url, &mut shutdown_rx).await?;
        }

        Commands::Http {
            bind,
            base_path,
            cors,
        } => {
            println!(
                "{} Starting HTTP server on {} (base: {})",
                "CLASP".cyan().bold(),
                bind,
                base_path
            );
            run_http_server(&bind, &base_path, cors, &mut shutdown_rx).await?;
        }

        Commands::Pub {
            server,
            address,
            value,
        } => {
            println!(
                "{} Publishing to {} -> {}",
                "CLASP".cyan().bold(),
                address.yellow(),
                value
            );
            publish_value(&server, &address, &value).await?;
        }

        Commands::Sub { server, pattern } => {
            println!(
                "{} Subscribing to {} on {}",
                "CLASP".cyan().bold(),
                pattern.yellow(),
                server
            );
            subscribe_pattern(&server, &pattern, &mut shutdown_rx).await?;
        }

        Commands::Info => {
            print_info();
        }

        Commands::Key { action } => {
            handle_key_command(action)?;
        }

        Commands::Token { file, action } => {
            let token_path = file.map(PathBuf::from).unwrap_or_else(default_token_file);

            match action {
                TokenAction::Create {
                    scopes,
                    expires,
                    subject,
                } => {
                    let record = create_token(&scopes, expires.as_deref(), subject.as_deref())?;

                    // Load existing store, add token, save
                    let mut store = TokenStore::load(&token_path)?;
                    let token = record.token.clone();
                    store.add(record);
                    store.save(&token_path)?;

                    println!("{}", token);
                    eprintln!(
                        "{} Token saved to: {}",
                        "OK".green().bold(),
                        token_path.display()
                    );
                }

                TokenAction::List { show_expired } => {
                    let store = TokenStore::load(&token_path)?;

                    if store.is_empty() {
                        println!("No tokens found in {}", token_path.display());
                        return Ok(());
                    }

                    println!(
                        "{} Tokens in {}:\n",
                        "CLASP".cyan().bold(),
                        token_path.display()
                    );

                    for record in store.list() {
                        let is_expired = record.is_expired();
                        if !show_expired && is_expired {
                            continue;
                        }

                        let status = if is_expired {
                            " [EXPIRED]".red().to_string()
                        } else {
                            "".to_string()
                        };

                        // Show truncated token for security
                        let display_token = if record.token.len() > 20 {
                            format!(
                                "{}...{}",
                                &record.token[..12],
                                &record.token[record.token.len() - 4..]
                            )
                        } else {
                            record.token.clone()
                        };

                        println!("  {}{}", display_token.yellow(), status);

                        if let Some(ref subject) = record.subject {
                            println!("    Subject: {}", subject);
                        }

                        println!("    Scopes: {}", record.scopes.join(", "));

                        if let Some(expires_at) = record.expires_at {
                            println!("    Expires: {}", format_timestamp(expires_at));
                        } else {
                            println!("    Expires: never");
                        }

                        println!();
                    }
                }

                TokenAction::Show { token } => {
                    let store = TokenStore::load(&token_path)?;

                    // Find token by exact match or prefix
                    let record = store
                        .list()
                        .find(|r| r.token == token || r.token.starts_with(&token))
                        .context("Token not found")?;

                    println!("{}: {}", "Token".cyan(), record.token);
                    if let Some(ref subject) = record.subject {
                        println!("{}: {}", "Subject".cyan(), subject);
                    }
                    println!("{}:", "Scopes".cyan());
                    for scope in &record.scopes {
                        println!("  - {}", scope);
                    }
                    if let Some(expires_at) = record.expires_at {
                        print!("{}: {}", "Expires".cyan(), format_timestamp(expires_at));
                        if record.is_expired() {
                            println!(" {}", "[EXPIRED]".red());
                        } else {
                            println!();
                        }
                    } else {
                        println!("{}: never", "Expires".cyan());
                    }
                    println!(
                        "{}: {} seconds ago",
                        "Created".cyan(),
                        std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs()
                            - record.created_at
                    );
                }

                TokenAction::Revoke { token } => {
                    let mut store = TokenStore::load(&token_path)?;

                    // Find token by exact match or prefix
                    let full_token = store
                        .list()
                        .find(|r| r.token == token || r.token.starts_with(&token))
                        .map(|r| r.token.clone())
                        .context("Token not found")?;

                    store.remove(&full_token);
                    store.save(&token_path)?;

                    println!("{} Revoked: {}", "OK".green().bold(), full_token);
                }

                TokenAction::Prune => {
                    let mut store = TokenStore::load(&token_path)?;
                    let count = store.prune_expired();
                    store.save(&token_path)?;

                    println!("{} Removed {} expired token(s)", "OK".green().bold(), count);
                }

                #[cfg(feature = "caps")]
                TokenAction::Cap { action } => {
                    handle_cap_command(action)?;
                }

                #[cfg(feature = "registry")]
                TokenAction::Entity { action } => {
                    handle_entity_command(action)?;
                }
            }
        }
    }

    Ok(())
}

fn setup_logging(level: &str, json: bool) -> Result<()> {
    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(level))
        .context("Failed to parse log level")?;

    if json {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_target(false).compact())
            .init();
    }

    Ok(())
}

async fn run_bridge(
    bridge_type: &str,
    opts: Vec<String>,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    println!(
        "{} Starting {} bridge",
        "CLASP".cyan().bold(),
        bridge_type.green()
    );

    // Parse options into a map
    let _options: std::collections::HashMap<String, String> = opts
        .iter()
        .filter_map(|opt| {
            let parts: Vec<&str> = opt.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    match bridge_type {
        "osc" => {
            println!("  Use 'clasp osc' for OSC-specific options");
        }
        "mqtt" => {
            println!("  Use 'clasp mqtt' for MQTT-specific options");
        }
        "websocket" | "ws" => {
            println!("  Use 'clasp websocket' for WebSocket-specific options");
        }
        "http" => {
            println!("  Use 'clasp http' for HTTP-specific options");
        }
        _ => {
            println!("{}", format!("Unknown bridge type: {}", bridge_type).red());
            return Ok(());
        }
    }

    // Wait for shutdown
    shutdown_rx.recv().await;
    println!("{}", "Bridge stopped".yellow());

    Ok(())
}

async fn run_osc_server(bind: &str, port: u16, shutdown_rx: &mut mpsc::Receiver<()>) -> Result<()> {
    use clasp_bridge::{Bridge, OscBridge, OscBridgeConfig};

    let config = OscBridgeConfig {
        bind_addr: format!("{}:{}", bind, port),
        ..Default::default()
    };

    let mut bridge = OscBridge::new(config);
    let mut event_rx = bridge.start().await?;

    println!("{} OSC server listening", "OK".green().bold());

    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    println!("{} {:?}", "OSC".cyan(), event);
                }
            }
            _ = shutdown_rx.recv() => {
                bridge.stop().await?;
                break;
            }
        }
    }

    Ok(())
}

async fn run_mqtt_bridge(
    host: &str,
    port: u16,
    client_id: Option<String>,
    topics: Vec<String>,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    use clasp_bridge::{Bridge, MqttBridge, MqttBridgeConfig};

    let config = MqttBridgeConfig {
        broker_host: host.to_string(),
        broker_port: port,
        client_id: client_id.unwrap_or_else(|| format!("clasp-cli-{}", std::process::id())),
        subscribe_topics: topics,
        ..Default::default()
    };

    let mut bridge = MqttBridge::new(config);
    let mut event_rx = bridge.start().await?;

    println!("{} MQTT bridge connected", "OK".green().bold());

    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    println!("{} {:?}", "MQTT".cyan(), event);
                }
            }
            _ = shutdown_rx.recv() => {
                bridge.stop().await?;
                break;
            }
        }
    }

    Ok(())
}

async fn run_websocket(mode: &str, url: &str, shutdown_rx: &mut mpsc::Receiver<()>) -> Result<()> {
    use clasp_bridge::{Bridge, WebSocketBridge, WebSocketBridgeConfig, WsMode};

    let ws_mode = match mode {
        "server" => WsMode::Server,
        "client" => WsMode::Client,
        _ => {
            println!("{}", "Mode must be 'server' or 'client'".red());
            return Ok(());
        }
    };

    let config = WebSocketBridgeConfig {
        mode: ws_mode,
        url: url.to_string(),
        ..Default::default()
    };

    let mut bridge = WebSocketBridge::new(config);
    let mut event_rx = bridge.start().await?;

    println!("{} WebSocket {} started", "OK".green().bold(), mode);

    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    println!("{} {:?}", "WS".cyan(), event);
                }
            }
            _ = shutdown_rx.recv() => {
                bridge.stop().await?;
                break;
            }
        }
    }

    Ok(())
}

async fn run_http_server(
    bind: &str,
    base_path: &str,
    cors: bool,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    use clasp_bridge::{Bridge, HttpBridge, HttpBridgeConfig, HttpMode};

    let config = HttpBridgeConfig {
        mode: HttpMode::Server,
        url: bind.to_string(),
        base_path: base_path.to_string(),
        cors_enabled: cors,
        ..Default::default()
    };

    let mut bridge = HttpBridge::new(config);
    let mut event_rx = bridge.start().await?;

    println!("{} HTTP server started", "OK".green().bold());
    println!("  Endpoints:");
    println!("    GET  {}/signals       - List all signals", base_path);
    println!("    GET  {}/*path         - Get signal value", base_path);
    println!("    PUT  {}/*path         - Set signal value", base_path);
    println!("    POST {}/*path         - Publish event", base_path);
    println!("    GET  {}/health        - Health check", base_path);

    loop {
        tokio::select! {
            event = event_rx.recv() => {
                if let Some(event) = event {
                    println!("{} {:?}", "HTTP".cyan(), event);
                }
            }
            _ = shutdown_rx.recv() => {
                bridge.stop().await?;
                break;
            }
        }
    }

    Ok(())
}

async fn publish_value(_server: &str, address: &str, value: &str) -> Result<()> {
    // Parse value as JSON
    let parsed: serde_json::Value = serde_json::from_str(value)
        .or_else(|_| Ok::<_, serde_json::Error>(serde_json::Value::String(value.to_string())))?;

    println!(
        "{} Published {} = {}",
        "OK".green().bold(),
        address.yellow(),
        serde_json::to_string_pretty(&parsed)?
    );

    // TODO: Connect to CLASP server and publish
    warn!("Server connection not yet implemented");

    Ok(())
}

async fn subscribe_pattern(
    _server: &str,
    pattern: &str,
    shutdown_rx: &mut mpsc::Receiver<()>,
) -> Result<()> {
    println!(
        "{} Subscribed to pattern: {}",
        "OK".green().bold(),
        pattern.yellow()
    );

    // TODO: Connect to CLASP server and subscribe
    warn!("Server connection not yet implemented - press Ctrl+C to exit");

    shutdown_rx.recv().await;

    Ok(())
}

// =========================================================================
// Key management
// =========================================================================

fn load_signing_key(path: &std::path::Path) -> Result<SigningKey> {
    let hex_str = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read key file: {}", path.display()))?;
    let hex_str = hex_str.trim();
    anyhow::ensure!(hex_str.len() == 64, "Key file must contain 64 hex characters (32-byte Ed25519 signing key)");
    let bytes = hex_decode(hex_str)?;
    let key_bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid key length"))?;
    Ok(SigningKey::from_bytes(&key_bytes))
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hex_decode(s: &str) -> Result<Vec<u8>> {
    anyhow::ensure!(s.len() % 2 == 0, "Odd-length hex string");
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).context("Invalid hex character"))
        .collect()
}

/// Write a file with restrictive permissions (0o600 on Unix) atomically,
/// avoiding the TOCTOU window of write() + set_permissions().
fn write_secret_file(path: &std::path::Path, data: &[u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Write;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(path)?;
        file.write_all(data)?;
    }
    #[cfg(not(unix))]
    {
        std::fs::write(path, data)?;
    }
    Ok(())
}

fn handle_key_command(action: KeyAction) -> Result<()> {
    match action {
        KeyAction::Generate { out } => {
            let signing_key = SigningKey::generate(&mut rand::rngs::OsRng);
            let hex_key = hex_encode(&signing_key.to_bytes());
            let pub_hex = hex_encode(signing_key.verifying_key().as_bytes());

            if let Some(ref path) = out {
                write_secret_file(path, hex_key.as_bytes())
                    .with_context(|| format!("Failed to write key file: {}", path.display()))?;

                eprintln!("{} Key saved to: {}", "OK".green().bold(), path.display());
                eprintln!("{}: {}", "Public key".cyan(), pub_hex);
            } else {
                // Print signing key to stdout (for piping)
                println!("{}", hex_key);
                eprintln!("{}: {}", "Public key".cyan(), pub_hex);
            }
        }

        KeyAction::Show { path, format } => {
            let signing_key = load_signing_key(&path)?;
            let pub_bytes = signing_key.verifying_key().to_bytes();

            match format.as_str() {
                "did" => {
                    // did:key multicodec prefix for Ed25519: 0xed01
                    let mut multicodec = vec![0xed, 0x01];
                    multicodec.extend_from_slice(&pub_bytes);
                    let encoded = bs58::encode(&multicodec).into_string();
                    println!("did:key:z{}", encoded);
                }
                _ => {
                    println!("{}", hex_encode(&pub_bytes));
                }
            }
        }
    }

    Ok(())
}

// =========================================================================
// Capability token commands
// =========================================================================

#[cfg(feature = "caps")]
fn parse_expiry_to_timestamp(expires: &str) -> Result<u64> {
    let duration = clasp_core::security::parse_duration(expires)
        .context("Failed to parse expiration")?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    Ok(now + duration.as_secs())
}

#[cfg(feature = "caps")]
fn handle_cap_command(action: CapAction) -> Result<()> {
    use clasp_caps::CapabilityToken;

    match action {
        CapAction::Create { key, scopes, expires, audience } => {
            let signing_key = load_signing_key(&key)?;
            let scope_list: Vec<String> = scopes.split(',').map(|s| s.trim().to_string()).collect();
            let expires_at = parse_expiry_to_timestamp(&expires)?;

            let audience_bytes = if let Some(ref aud) = audience {
                Some(hex_decode(aud)?)
            } else {
                None
            };

            let token = CapabilityToken::create_root(
                &signing_key,
                scope_list,
                expires_at,
                audience_bytes,
            )?;

            let encoded = token.encode()?;
            println!("{}", encoded);
            eprintln!("{} Root capability token created", "OK".green().bold());
            eprintln!("  {}: {}", "Issuer".cyan(), hex_encode(&token.issuer));
            eprintln!("  {}: {}", "Scopes".cyan(), token.scopes.join(", "));
            eprintln!("  {}: {}", "Expires".cyan(), format_timestamp(token.expires_at));
        }

        CapAction::Delegate { parent, key, scopes, expires, audience } => {
            let parent_token = CapabilityToken::decode(&parent)
                .context("Failed to decode parent token")?;

            let child_key = load_signing_key(&key)?;
            let scope_list: Vec<String> = scopes.split(',').map(|s| s.trim().to_string()).collect();
            let expires_at = parse_expiry_to_timestamp(&expires)?;

            let audience_bytes = if let Some(ref aud) = audience {
                Some(hex_decode(aud)?)
            } else {
                None
            };

            let child = parent_token.delegate(
                &child_key,
                scope_list,
                expires_at,
                audience_bytes,
            )?;

            let encoded = child.encode()?;
            println!("{}", encoded);
            eprintln!("{} Delegated capability token created", "OK".green().bold());
            eprintln!("  {}: {}", "Issuer".cyan(), hex_encode(&child.issuer));
            eprintln!("  {}: {}", "Chain depth".cyan(), child.chain_depth());
            eprintln!("  {}: {}", "Scopes".cyan(), child.scopes.join(", "));
            eprintln!("  {}: {}", "Expires".cyan(), format_timestamp(child.expires_at));
        }

        CapAction::Inspect { token } => {
            let cap = CapabilityToken::decode(&token)
                .context("Failed to decode token")?;

            println!("{}: v{}", "Version".cyan(), cap.version);
            println!("{}: {}", "Issuer".cyan(), hex_encode(&cap.issuer));
            if let Some(ref aud) = cap.audience {
                println!("{}: {}", "Audience".cyan(), hex_encode(aud));
            }
            println!("{}: {}", "Scopes".cyan(), cap.scopes.join(", "));
            println!("{}: {} ({})", "Expires".cyan(), cap.expires_at, format_timestamp(cap.expires_at));
            println!("{}: {}", "Nonce".cyan(), cap.nonce);
            println!("{}: {}", "Chain depth".cyan(), cap.chain_depth());

            if !cap.proofs.is_empty() {
                println!("\n{}:", "Delegation chain".cyan());
                for (i, proof) in cap.proofs.iter().enumerate() {
                    println!("  [{}] issuer: {}", i, hex_encode(&proof.issuer));
                    println!("       scopes: {}", proof.scopes.join(", "));
                }
            }

            // Verify signature
            match cap.verify_signature() {
                Ok(()) => println!("\n{} Signature valid", "OK".green().bold()),
                Err(e) => println!("\n{} Signature invalid: {}", "FAIL".red().bold(), e),
            }

            if cap.is_expired() {
                println!("{}", "WARNING: Token is expired".yellow());
            }
        }

        CapAction::Verify { token, trust_anchor, max_depth } => {
            use clasp_caps::CapabilityValidator;
            use clasp_core::security::TokenValidator;

            // Load trust anchor (file or hex string)
            let anchor_bytes = if std::path::Path::new(&trust_anchor).exists() {
                let signing_key = load_signing_key(std::path::Path::new(&trust_anchor))?;
                signing_key.verifying_key().to_bytes().to_vec()
            } else {
                hex_decode(&trust_anchor)?
            };

            let validator = CapabilityValidator::new(vec![anchor_bytes], max_depth);

            match validator.validate(&token) {
                clasp_core::security::ValidationResult::Valid(info) => {
                    println!("{} Token is valid", "OK".green().bold());
                    println!("  {}: {:?}", "Scopes".cyan(),
                        info.scopes.iter().map(|s| s.as_str().to_string()).collect::<Vec<_>>());
                    if let Some(ref subject) = info.subject {
                        println!("  {}: {}", "Subject".cyan(), subject);
                    }
                    if let Some(depth) = info.metadata.get("chain_depth") {
                        println!("  {}: {}", "Chain depth".cyan(), depth);
                    }
                }
                clasp_core::security::ValidationResult::Expired => {
                    println!("{} Token is expired", "FAIL".red().bold());
                    std::process::exit(1);
                }
                clasp_core::security::ValidationResult::Invalid(reason) => {
                    println!("{} Token is invalid: {}", "FAIL".red().bold(), reason);
                    std::process::exit(1);
                }
                clasp_core::security::ValidationResult::NotMyToken => {
                    println!("{} Not a capability token (missing cap_ prefix)", "FAIL".red().bold());
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

// =========================================================================
// Entity token commands
// =========================================================================

#[cfg(feature = "registry")]
fn handle_entity_command(action: EntityAction) -> Result<()> {
    use clasp_registry::EntityKeypair;

    match action {
        EntityAction::Keygen { out, name, entity_type } => {
            let keypair = EntityKeypair::generate()
                .map_err(|e| anyhow::anyhow!("Failed to generate keypair: {}", e))?;

            let hex_key = hex_encode(&keypair.signing_key.to_bytes());
            let pub_hex = hex_encode(keypair.public_key_bytes());
            let entity_id = keypair.entity_id.as_str().to_string();

            if let Some(ref path) = out {
                write_secret_file(path, hex_key.as_bytes())
                    .with_context(|| format!("Failed to write key file: {}", path.display()))?;

                eprintln!("{} Entity keypair saved to: {}", "OK".green().bold(), path.display());
            } else {
                println!("{}", hex_key);
            }

            eprintln!("  {}: {}", "Entity ID".cyan(), entity_id);
            eprintln!("  {}: {}", "Public key".cyan(), pub_hex);
            eprintln!("  {}: {}", "Type".cyan(), entity_type);
            if let Some(ref n) = name {
                eprintln!("  {}: {}", "Name".cyan(), n);
            }
        }

        EntityAction::Mint { key } => {
            let signing_key = load_signing_key(&key)?;
            let keypair = EntityKeypair::from_signing_key(signing_key)
                .map_err(|e| anyhow::anyhow!("Failed to create entity keypair: {}", e))?;

            let token = clasp_registry::generate_token(&keypair)
                .map_err(|e| anyhow::anyhow!("Failed to generate token: {}", e))?;

            println!("{}", token);
            eprintln!("{} Entity token minted", "OK".green().bold());
            eprintln!("  {}: {}", "Entity ID".cyan(), keypair.entity_id);
        }

        EntityAction::Inspect { token } => {
            let payload = clasp_registry::parse_token(&token)
                .map_err(|e| anyhow::anyhow!("Failed to parse token: {}", e))?;

            println!("{}: {}", "Entity ID".cyan(), payload.entity_id);
            println!("{}: {}", "Timestamp".cyan(), payload.timestamp);
            println!("{}: {} bytes", "Signature".cyan(), payload.signature.len());

            // Show human-readable time
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if payload.timestamp <= now {
                let age = now - payload.timestamp;
                if age < 60 {
                    println!("{}: {} seconds ago", "Created".cyan(), age);
                } else if age < 3600 {
                    println!("{}: {} minutes ago", "Created".cyan(), age / 60);
                } else if age < 86400 {
                    println!("{}: {} hours ago", "Created".cyan(), age / 3600);
                } else {
                    println!("{}: {} days ago", "Created".cyan(), age / 86400);
                }
            }
        }
    }

    Ok(())
}

fn print_info() {
    println!(
        "{}",
        "CLASP - Creative Low-Latency Application Streaming Protocol"
            .cyan()
            .bold()
    );
    println!();
    println!("Version:    {}", env!("CARGO_PKG_VERSION"));
    println!("Platform:   {}", std::env::consts::OS);
    println!("Arch:       {}", std::env::consts::ARCH);
    println!();
    println!("{}", "Supported Protocols:".green());
    println!("  - CLASP/QUIC (native, low-latency)");
    println!("  - OSC (Open Sound Control)");
    println!("  - MIDI (Musical Instrument Digital Interface)");
    println!("  - Art-Net (Ethernet DMX)");
    println!("  - MQTT (IoT messaging)");
    println!("  - WebSocket (bidirectional web)");
    println!("  - HTTP/REST (request-response API)");
    println!();
    println!("{}", "Examples:".green());
    println!("  clasp osc --port 9000            # Start OSC server");
    println!("  clasp mqtt --host broker.local   # Connect to MQTT broker");
    println!("  clasp http --bind 0.0.0.0:3000   # Start HTTP REST API");
    println!("  clasp websocket --mode server    # Start WebSocket server");
}
