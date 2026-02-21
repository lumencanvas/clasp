//! Server orchestration: router setup, transport configuration, and main run loop.

use crate::config::Cli;
use crate::cpsk::{write_secret_file, SharedValidator};

use anyhow::{Context, Result};
use clasp_core::security::{CpskValidator, ValidatorChain};
use clasp_core::types::SnapshotMessage;
use clasp_core::SecurityMode;
use clasp_router::{MultiProtocolConfig, Router, RouterConfig, RouterState, RouterStateConfig};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::signal;

/// Main server entry point. Call after CLI parsing and tracing initialization.
pub async fn run(cli: Cli) -> Result<()> {
    tracing::info!("╔══════════════════════════════════════════════════════════════╗");
    tracing::info!("║           CLASP Multi-Protocol Relay Server                  ║");
    tracing::info!("╚══════════════════════════════════════════════════════════════╝");

    // Start Prometheus metrics exporter if configured
    #[cfg(feature = "metrics")]
    if let Some(metrics_port) = cli.metrics_port {
        let metrics_addr: SocketAddr = format!("{}:{}", cli.host, metrics_port)
            .parse()
            .context("Invalid metrics address")?;
        let builder = metrics_exporter_prometheus::PrometheusBuilder::new();
        builder
            .with_http_listener(metrics_addr)
            .install()
            .context("Failed to install Prometheus metrics exporter")?;
        tracing::info!("Metrics: http://{}/metrics", metrics_addr);
    }

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
        features: {
            let mut f = vec![
                "param".to_string(),
                "event".to_string(),
                "stream".to_string(),
                "timeline".to_string(),
                "gesture".to_string(),
            ];
            #[cfg(feature = "federation")]
            f.push("federation".to_string());
            f
        },
        max_subscriptions_per_session: 100,
        gesture_coalescing: true,
        gesture_coalesce_interval_ms: 16,
        max_messages_per_second: if auth_enabled { 30 } else { 0 },
        rate_limiting_enabled: auth_enabled,
        state_config,
    };

    let mut router = Router::new(config);

    // Wire journal if configured
    #[cfg(feature = "journal")]
    {
        if let Some(ref path) = cli.journal {
            let journal = std::sync::Arc::new(
                clasp_journal::SqliteJournal::new(
                    path.to_str().expect("journal path must be valid UTF-8"),
                )
                .expect("Failed to open journal database"),
            );
            router = router.with_journal(journal);
            tracing::info!("Journal: {}", path.display());
        } else if cli.journal_memory {
            let journal = std::sync::Arc::new(
                clasp_journal::SqliteJournal::in_memory()
                    .expect("Failed to create in-memory journal"),
            );
            router = router.with_journal(journal);
            tracing::info!("Journal: in-memory");
        }
    }

    // Recover state from journal if available (after journal is wired, before serving)
    #[cfg(feature = "journal")]
    if cli.journal.is_some() || cli.journal_memory {
        match router.state().recover_from_journal().await {
            Ok(count) => {
                if count > 0 {
                    tracing::info!("Recovered {} params from journal", count);
                }
            }
            Err(e) => tracing::warn!("Journal recovery failed: {}", e),
        }
    }

    // Wire rules engine if configured
    #[cfg(feature = "rules")]
    let mut interval_rules: Vec<(String, u64)> = Vec::new();
    #[cfg(feature = "rules")]
    if let Some(ref rules_path) = cli.rules {
        let json = std::fs::read_to_string(rules_path)
            .with_context(|| format!("Failed to read rules file {}", rules_path.display()))?;
        let rules: Vec<clasp_rules::Rule> = serde_json::from_str(&json)
            .with_context(|| format!("Failed to parse rules JSON from {}", rules_path.display()))?;
        let mut engine = clasp_rules::RulesEngine::new();
        for rule in &rules {
            if let clasp_rules::Trigger::OnInterval { seconds } = &rule.trigger {
                interval_rules.push((rule.id.clone(), *seconds));
            }
            engine
                .add_rule(rule.clone())
                .with_context(|| format!("Failed to add rule '{}'", rule.id))?;
        }
        tracing::info!("Rules engine: {} rule(s) from {}", rules.len(), rules_path.display());
        router = router.with_rules(engine);

        if !interval_rules.is_empty() {
            tracing::info!(
                "Rules: {} interval trigger(s) registered",
                interval_rules.len()
            );
        }
    }

    // Set up chat-specific write validation and snapshot filtering
    if auth_enabled {
        router.set_write_validator(crate::validator::ChatWriteValidator);
        router.set_snapshot_filter(crate::validator::ChatSnapshotFilter);
        tracing::info!("Chat write validator and snapshot filter enabled");
    }

    // Restore state from disk if --persist is set and file exists
    if let Some(ref persist_path) = cli.persist {
        if persist_path.exists() {
            match std::fs::read_to_string(persist_path) {
                Ok(json) => match serde_json::from_str::<SnapshotMessage>(&json) {
                    Ok(snapshot) => {
                        let count = snapshot.params.len();
                        let writer = "restore".to_string();
                        for pv in snapshot.params {
                            let _ = router.state().set(
                                &pv.address,
                                pv.value,
                                &writer,
                                Some(pv.revision),
                                false,
                                false,
                            );
                        }
                        tracing::info!("Restored {} params from {}", count, persist_path.display());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse snapshot {}: {}", persist_path.display(), e);
                    }
                },
                Err(e) => {
                    tracing::warn!("Failed to read snapshot {}: {}", persist_path.display(), e);
                }
            }
        } else {
            tracing::info!("No existing snapshot at {}, starting fresh", persist_path.display());
        }
    }

    // Create shared validator and start auth HTTP server if enabled
    #[cfg(feature = "registry")]
    let mut entity_store: Option<Arc<dyn clasp_registry::EntityStore>> = None;

    if let Some(auth_port) = cli.auth_port {
        let cpsk_validator = Arc::new(if cli.token_ttl > 0 {
            tracing::info!("CPSK token default TTL: {}s", cli.token_ttl);
            CpskValidator::with_default_ttl(Duration::from_secs(cli.token_ttl))
        } else {
            CpskValidator::new()
        });

        // Bootstrap admin token if --admin-token is set
        if let Some(ref admin_token_path) = cli.admin_token {
            let token = if admin_token_path.exists() {
                // Read existing token
                let t = std::fs::read_to_string(admin_token_path)
                    .with_context(|| format!("Failed to read admin token file {}", admin_token_path.display()))?
                    .trim()
                    .to_string();
                tracing::info!("Admin token loaded from {}", admin_token_path.display());
                t
            } else {
                // Generate new token and write to file
                let t = CpskValidator::generate_token();
                if let Some(parent) = admin_token_path.parent() {
                    std::fs::create_dir_all(parent)
                        .with_context(|| format!("Failed to create directory for admin token: {}", parent.display()))?;
                }
                // Write with restrictive permissions atomically to avoid TOCTOU
                write_secret_file(admin_token_path, t.as_bytes())
                    .with_context(|| format!("Failed to write admin token to {}", admin_token_path.display()))?;
                tracing::info!("Admin token generated and saved to {}", admin_token_path.display());
                t
            };

            // Register admin token with admin:/** scope, no expiry
            use clasp_core::security::{Scope, TokenInfo};
            let scopes = vec![Scope::new(clasp_core::security::Action::Admin, "/**")
                .expect("valid admin scope")];
            let info = TokenInfo::new(token.clone(), scopes).with_subject("admin-bootstrap".to_string());
            cpsk_validator.register(token, info);
            tracing::info!("Admin bootstrap token registered with admin:/** scope");
        }

        let mut chain = ValidatorChain::new();
        chain.add(SharedValidator(Arc::clone(&cpsk_validator)));

        // Add capability token validator if trust anchors provided
        #[cfg(feature = "caps")]
        if !cli.trust_anchor.is_empty() {
            let anchors: Vec<Vec<u8>> = {
                let mut result = Vec::new();
                for p in &cli.trust_anchor {
                    // Trust anchor files contain hex-encoded signing keys (same format
                    // as `clasp key generate --out`). Read, hex-decode, derive public key.
                    let contents = std::fs::read_to_string(p)
                        .with_context(|| format!("Failed to read trust anchor file {}", p.display()))?;
                    let hex_str = contents.trim();
                    if hex_str.len() == 64 {
                        // 64 hex chars = 32-byte signing key -> derive public key
                        let key_bytes: Vec<u8> = (0..hex_str.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16)
                                .map_err(|_| anyhow::anyhow!("Invalid hex in trust anchor file {}", p.display())))
                            .collect::<anyhow::Result<Vec<u8>>>()?;
                        let key_array: [u8; 32] = key_bytes.try_into()
                            .map_err(|_| anyhow::anyhow!("Invalid key length in trust anchor file {}", p.display()))?;
                        let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_array);
                        result.push(signing_key.verifying_key().to_bytes().to_vec());
                    } else if hex_str.len() == 32 {
                        // Raw 32-byte binary public key file
                        let bytes = std::fs::read(p)
                            .with_context(|| format!("Failed to read trust anchor file {}", p.display()))?;
                        result.push(bytes);
                    } else {
                        anyhow::bail!(
                            "Trust anchor file {} has unexpected size: expected 64 hex chars (signing key) or 32 raw bytes (public key), got {} chars",
                            p.display(), hex_str.len()
                        );
                    }
                }
                result
            };
            chain.add(clasp_caps::CapabilityValidator::new(anchors, cli.cap_max_depth));
            tracing::info!(
                "Capability tokens: {} trust anchor(s), max depth {}",
                cli.trust_anchor.len(),
                cli.cap_max_depth
            );
        }

        // Add entity registry validator if configured
        #[cfg(feature = "registry")]
        if let Some(ref db_path) = cli.registry_db {
            let store: Arc<dyn clasp_registry::EntityStore> = Arc::new(
                clasp_registry::SqliteEntityStore::open(
                    db_path
                        .to_str()
                        .expect("registry-db path must be valid UTF-8"),
                )
                .expect("Failed to open entity registry database"),
            );
            chain.add(clasp_registry::EntityValidator::new(Arc::clone(&store)));
            entity_store = Some(store);
            tracing::info!("Entity registry: {}", db_path.display());
        }

        router.set_validator(chain);

        let auth_state = Arc::new(
            crate::auth::AuthState::new(&cli.auth_db, Arc::clone(&cpsk_validator))
                .expect("Failed to initialize auth database"),
        );
        #[allow(unused_mut)]
        let mut auth_app = crate::auth::auth_router(auth_state, cli.cors_origin.as_deref());

        // Mount entity registry REST routes if configured
        #[cfg(feature = "registry")]
        if let Some(ref store) = entity_store {
            // Collect trust anchor public keys as hex strings for /api/trust-anchors
            #[cfg(feature = "caps")]
            let trust_anchor_hexes: Vec<String> = cli
                .trust_anchor
                .iter()
                .filter_map(|p| {
                    let contents = std::fs::read_to_string(p).ok()?;
                    let hex_str = contents.trim();
                    if hex_str.len() == 64 {
                        // Hex-encoded signing key -> derive public key hex
                        let key_bytes: Vec<u8> = (0..hex_str.len())
                            .step_by(2)
                            .map(|i| u8::from_str_radix(&hex_str[i..i + 2], 16).ok())
                            .collect::<Option<Vec<u8>>>()?;
                        let key_array: [u8; 32] = key_bytes.try_into().ok()?;
                        let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_array);
                        let pub_bytes = signing_key.verifying_key().to_bytes();
                        Some(pub_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>())
                    } else {
                        // Raw binary -> hex-encode directly
                        let bytes = std::fs::read(p).ok()?;
                        Some(bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>())
                    }
                })
                .collect();
            #[cfg(not(feature = "caps"))]
            let trust_anchor_hexes: Vec<String> = Vec::new();

            let reg_state = Arc::new(
                crate::registry::RegistryState::new(
                    Arc::clone(store),
                    Arc::clone(&cpsk_validator),
                )
                .with_trust_anchors(trust_anchor_hexes, cli.cap_max_depth)
            );
            auth_app = auth_app.merge(crate::registry::registry_router(reg_state));
            tracing::info!("Entity REST API mounted at /api/entities (admin auth required)");
            tracing::info!("Trust anchors API mounted at /api/trust-anchors (public)");
        }

        let auth_addr: SocketAddr = format!("{}:{}", cli.host, auth_port).parse()?;
        tracing::info!("Auth HTTP: http://{}", auth_addr);

        let listener = tokio::net::TcpListener::bind(auth_addr).await?;
        tokio::spawn(async move {
            if let Err(e) = axum::serve(
                listener,
                auth_app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
            )
            .await
            {
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

    // Log persistence config
    if let Some(ref persist_path) = cli.persist {
        tracing::info!(
            "Persistence: {} (interval: {}s)",
            persist_path.display(),
            cli.persist_interval
        );
    } else {
        tracing::info!("Persistence: disabled (use --persist <path> to enable)");
    }

    tracing::info!("Router initialized, accepting connections...");

    // Start rendezvous server if enabled
    #[cfg(feature = "rendezvous")]
    if cli.rendezvous_port > 0 {
        use crate::config::{RendezvousConfig, RendezvousServer};

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

    // Get shared state refs for persistence, rules, and federation tasks
    let (sessions_arc, subscriptions_arc, state_arc) = router.shared_state();

    // Spawn interval rule timer tasks
    #[cfg(feature = "rules")]
    if !interval_rules.is_empty() {
        if let Some(rules_engine) = router.rules_engine().cloned() {
            for (rule_id, seconds) in interval_rules {
                let engine = Arc::clone(&rules_engine);
                let state = Arc::clone(&state_arc);
                let sessions = Arc::clone(&sessions_arc);
                let subs = Arc::clone(&subscriptions_arc);
                tokio::spawn(async move {
                    let mut interval = tokio::time::interval(Duration::from_secs(seconds));
                    loop {
                        interval.tick().await;
                        let actions = engine.lock().evaluate_interval(
                            &rule_id,
                            |addr| state.get(addr),
                        );
                        if !actions.is_empty() {
                            clasp_router::execute_rule_actions(actions, &state, &sessions, &subs);
                        }
                    }
                });
            }
        }
    }

    // Spawn background persistence task if --persist is set
    if let Some(ref path) = cli.persist {
        let bg_state = Arc::clone(&state_arc);
        let bg_path = path.clone();
        let bg_interval = cli.persist_interval;
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(bg_interval));
            interval.tick().await; // skip first immediate tick
            loop {
                interval.tick().await;
                write_snapshot(&bg_state, &bg_path);
            }
        });
    }

    // Start federation leaf if configured
    #[cfg(feature = "federation")]
    if let Some(ref hub_url) = cli.federation_hub {
        let fed_config = clasp_federation::FederationConfig {
            mode: clasp_federation::FederationMode::Leaf {
                hub_endpoint: hub_url.clone(),
            },
            router_id: cli
                .federation_id
                .clone()
                .unwrap_or_else(|| {
                    // Generate a unique ID from CPSK token generator
                    let token = CpskValidator::generate_token();
                    format!("relay-{}", &token[5..21])
                }),
            owned_namespaces: if cli.federation_namespace.is_empty() {
                vec!["/**".to_string()]
            } else {
                cli.federation_namespace.clone()
            },
            auth_token: cli.federation_token.clone(),
            ..Default::default()
        };
        let fed_state = Arc::clone(&state_arc);
        let fed_sessions = Arc::clone(&sessions_arc);
        let fed_subs = Arc::clone(&subscriptions_arc);
        tracing::info!(
            "Federation: leaf mode, hub={}, id={}, namespaces={:?}",
            hub_url,
            fed_config.router_id,
            fed_config.owned_namespaces
        );
        tokio::spawn(async move {
            crate::federation::run_federation_leaf(fed_config, fed_state, fed_sessions, fed_subs).await;
        });
    }

    // Start health check server if configured
    let health_state = Arc::new(crate::health::HealthState::new());
    if let Some(health_port) = cli.health_port {
        let health_addr: SocketAddr = format!("{}:{}", cli.host, health_port)
            .parse()
            .context("Invalid health check address")?;
        let hs = Arc::clone(&health_state);
        tokio::spawn(async move {
            crate::health::start_health_server(health_addr, hs).await;
        });
    }

    // Mark as ready
    health_state.ready.store(true, std::sync::atomic::Ordering::Relaxed);

    // Run serve_all alongside a shutdown signal listener
    let shutdown_state = Arc::clone(&state_arc);
    let persist_path_shutdown = cli.persist.clone();
    let drain_timeout = Duration::from_secs(cli.drain_timeout);

    tokio::select! {
        result = router.serve_all(multi_config) => {
            result?;
        }
        _ = shutdown_signal() => {
            tracing::info!("Shutdown signal received, starting graceful drain...");

            // Mark as not ready so load balancers stop sending traffic
            health_state.ready.store(false, std::sync::atomic::Ordering::Relaxed);

            // Wait for in-flight messages to drain
            tracing::info!("Draining connections (timeout: {:?})...", drain_timeout);
            tokio::time::sleep(drain_timeout.min(Duration::from_secs(5))).await;

            // Write final snapshot
            if let Some(ref path) = persist_path_shutdown {
                tracing::info!("Writing final snapshot before exit...");
                write_snapshot(&shutdown_state, path);
            }

            tracing::info!("Shutdown complete");
        }
    }

    Ok(())
}

/// Write a state snapshot atomically (write to .tmp then rename).
fn write_snapshot(state: &RouterState, path: &std::path::Path) {
    let snapshot = state.full_snapshot();
    let count = snapshot.params.len();
    match serde_json::to_string(&snapshot) {
        Ok(json) => {
            let tmp_path = path.with_extension("json.tmp");
            if let Err(e) = std::fs::write(&tmp_path, &json) {
                tracing::error!("Failed to write snapshot tmp: {}", e);
                return;
            }
            if let Err(e) = std::fs::rename(&tmp_path, path) {
                tracing::error!("Failed to rename snapshot: {}", e);
                return;
            }
            tracing::debug!("Snapshot: {} params, {} bytes", count, json.len());
        }
        Err(e) => {
            tracing::error!("Failed to serialize snapshot: {}", e);
        }
    }
}

/// Wait for SIGINT or SIGTERM.
async fn shutdown_signal() {
    let ctrl_c = signal::ctrl_c();
    #[cfg(unix)]
    {
        let mut sigterm = signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to register SIGTERM handler");
        tokio::select! {
            _ = ctrl_c => {}
            _ = sigterm.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        ctrl_c.await.ok();
    }
}
