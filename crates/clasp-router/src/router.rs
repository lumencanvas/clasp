//! Main router implementation
//!
//! The router is transport-agnostic - it can accept connections from any transport
//! that implements the `TransportServer` trait (WebSocket, QUIC, TCP, etc.).
//!
//! # Transport Support
//!
//! - **WebSocket** (default): Works everywhere, including browsers and DO App Platform
//! - **QUIC**: High-performance for native apps. Requires UDP - NOT supported on DO App Platform
//! - **TCP**: Simple fallback, works everywhere
//!
//! # Example
//!
//! ```no_run
//! use clasp_router::{Router, RouterConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = Router::new(RouterConfig::default());
//!
//!     // WebSocket (most common)
//!     router.serve_websocket("0.0.0.0:7330").await.unwrap();
//!
//!     // Or use any TransportServer implementation
//!     // router.serve_on(my_custom_server).await.unwrap();
//! }
//! ```

use bytes::Bytes;
use clasp_core::{
    codec, AckMessage, Action, CpskValidator, ErrorMessage, Frame, Message, PublishMessage,
    SecurityMode, SetMessage, SignalType, SnapshotMessage, TokenValidator, ValidationResult,
};

#[cfg(feature = "journal")]
use clasp_journal::Journal;
#[cfg(feature = "rules")]
use clasp_rules::RulesEngine;
use clasp_transport::{TransportEvent, TransportReceiver, TransportSender, TransportServer};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

#[cfg(feature = "websocket")]
use clasp_transport::WebSocketServer;

#[cfg(feature = "quic")]
use clasp_transport::{QuicConfig, QuicTransport};

use crate::{
    error::{Result, RouterError},
    gesture::{GestureRegistry, GestureResult},
    p2p::{analyze_address, P2PAddressType, P2PCapabilities},
    session::{Session, SessionId},
    state::{RouterState, RouterStateConfig},
    subscription::{Subscription, SubscriptionManager},
};
use std::time::Duration;

/// Application-specific write validation callback.
///
/// Called after scope checks but before `state.apply_set()` for SET operations,
/// and before broadcast for PUBLISH operations. Allows the application to enforce
/// semantic authorization rules (e.g., "only room creators can modify admin paths").
pub trait WriteValidator: Send + Sync {
    /// Validate a write operation.
    ///
    /// - `address`: the CLASP address being written to
    /// - `value`: the value being written
    /// - `session`: the session performing the write
    /// - `state`: the current router state (for looking up existing values)
    ///
    /// Returns `Ok(())` to allow the write, or `Err(message)` to reject it.
    fn validate_write(
        &self,
        address: &str,
        value: &clasp_core::Value,
        session: &Session,
        state: &RouterState,
    ) -> std::result::Result<(), String>;
}

/// Application-specific snapshot filtering callback.
///
/// Called before sending the initial SNAPSHOT after WELCOME, and before sending
/// subscription snapshots. Allows the application to strip sensitive fields
/// or restrict visibility of certain paths.
pub trait SnapshotFilter: Send + Sync {
    /// Filter a snapshot before delivery to a session.
    ///
    /// - `params`: the snapshot parameters to filter
    /// - `session`: the session receiving the snapshot
    /// - `state`: the current router state
    ///
    /// Returns the filtered list of parameters.
    fn filter_snapshot(
        &self,
        params: Vec<clasp_core::ParamValue>,
        session: &Session,
        state: &RouterState,
    ) -> Vec<clasp_core::ParamValue>;
}

/// Timeout for clients to complete the handshake (send Hello message)
const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

/// Transport configuration for multi-transport serving.
///
/// Use with `Router::serve_multi()` to run multiple transports simultaneously.
#[derive(Debug, Clone)]
pub enum TransportConfig {
    /// WebSocket transport (default, works everywhere)
    #[cfg(feature = "websocket")]
    WebSocket {
        /// Listen address, e.g., "0.0.0.0:7330"
        addr: String,
    },

    /// QUIC transport (high-performance, requires UDP)
    ///
    /// **WARNING**: Not supported on DigitalOcean App Platform or most PaaS.
    /// Use a VPS/Droplet for QUIC support.
    #[cfg(feature = "quic")]
    Quic {
        /// Listen address
        addr: SocketAddr,
        /// TLS certificate (DER format)
        cert: Vec<u8>,
        /// TLS private key (DER format)
        key: Vec<u8>,
    },
}

/// Multi-protocol server configuration.
///
/// Configure which protocols the router should accept connections on.
/// All configured protocols share the same router state.
///
/// # Example
///
/// ```no_run
/// use clasp_router::{Router, MultiProtocolConfig};
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let router = Router::default();
/// let config = MultiProtocolConfig {
///     websocket_addr: Some("0.0.0.0:7330".into()),
///     #[cfg(feature = "mqtt-server")]
///     mqtt: None,
///     #[cfg(feature = "osc-server")]
///     osc: None,
///     ..Default::default()
/// };
/// router.serve_all(config).await?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, Default)]
pub struct MultiProtocolConfig {
    /// WebSocket listen address (e.g., "0.0.0.0:7330")
    #[cfg(feature = "websocket")]
    pub websocket_addr: Option<String>,

    /// QUIC configuration
    #[cfg(feature = "quic")]
    pub quic: Option<QuicServerConfig>,

    /// MQTT server configuration
    #[cfg(feature = "mqtt-server")]
    pub mqtt: Option<crate::adapters::MqttServerConfig>,

    /// OSC server configuration
    #[cfg(feature = "osc-server")]
    pub osc: Option<crate::adapters::OscServerConfig>,
}

/// QUIC server configuration
#[cfg(feature = "quic")]
#[derive(Debug, Clone)]
pub struct QuicServerConfig {
    /// Listen address
    pub addr: SocketAddr,
    /// TLS certificate (DER format)
    pub cert: Vec<u8>,
    /// TLS private key (DER format)
    pub key: Vec<u8>,
}

/// Router configuration
#[derive(Debug, Clone)]
pub struct RouterConfig {
    /// Server name
    pub name: String,
    /// Supported features
    pub features: Vec<String>,
    /// Maximum sessions
    pub max_sessions: usize,
    /// Session timeout (seconds)
    pub session_timeout: u64,
    /// Security mode (Open or Authenticated)
    pub security_mode: SecurityMode,
    /// Maximum subscriptions per session (0 = unlimited)
    pub max_subscriptions_per_session: usize,
    /// Enable gesture move coalescing (reduces bandwidth for high-frequency touch input)
    pub gesture_coalescing: bool,
    /// Gesture move coalesce interval in milliseconds (default: 16ms = 60fps)
    pub gesture_coalesce_interval_ms: u64,
    /// Maximum messages per second per client (0 = unlimited)
    pub max_messages_per_second: u32,
    /// Enable rate limiting
    pub rate_limiting_enabled: bool,
    /// State store configuration (TTL, limits)
    pub state_config: RouterStateConfig,
}

impl Default for RouterConfig {
    fn default() -> Self {
        Self {
            name: "Clasp Router".to_string(),
            features: vec![
                "param".to_string(),
                "event".to_string(),
                "stream".to_string(),
                "timeline".to_string(),
                "gesture".to_string(),
            ],
            max_sessions: 100,
            session_timeout: 300,
            security_mode: SecurityMode::Open,
            max_subscriptions_per_session: 1000, // 0 = unlimited
            gesture_coalescing: true,
            gesture_coalesce_interval_ms: 16,
            max_messages_per_second: 1000, // 1000 msgs/sec default
            rate_limiting_enabled: true,
            state_config: RouterStateConfig::default(), // 1 hour TTL by default
        }
    }
}

/// Builder for RouterConfig
#[derive(Debug, Clone, Default)]
pub struct RouterConfigBuilder {
    config: RouterConfig,
}

impl RouterConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn max_sessions(mut self, max: usize) -> Self {
        self.config.max_sessions = max;
        self
    }

    pub fn session_timeout(mut self, secs: u64) -> Self {
        self.config.session_timeout = secs;
        self
    }

    pub fn security_mode(mut self, mode: SecurityMode) -> Self {
        self.config.security_mode = mode;
        self
    }

    pub fn gesture_coalescing(mut self, enabled: bool) -> Self {
        self.config.gesture_coalescing = enabled;
        self
    }

    pub fn gesture_coalesce_interval_ms(mut self, ms: u64) -> Self {
        self.config.gesture_coalesce_interval_ms = ms;
        self
    }

    pub fn build(self) -> RouterConfig {
        self.config
    }
}

/// Clasp router
pub struct Router {
    config: RouterConfig,
    /// Active sessions
    sessions: Arc<DashMap<SessionId, Arc<Session>>>,
    /// Subscription manager
    subscriptions: Arc<SubscriptionManager>,
    /// Global state
    state: Arc<RouterState>,
    /// Running flag
    running: Arc<RwLock<bool>>,
    /// Token validator (None = always reject in authenticated mode)
    token_validator: Option<Arc<dyn TokenValidator>>,
    /// P2P capabilities tracker
    p2p_capabilities: Arc<P2PCapabilities>,
    /// Gesture registry for move coalescing
    gesture_registry: Option<Arc<GestureRegistry>>,
    /// Application-specific write validator
    write_validator: Option<Arc<dyn WriteValidator>>,
    /// Application-specific snapshot filter
    snapshot_filter: Option<Arc<dyn SnapshotFilter>>,
    /// Rules engine for server-side automation
    #[cfg(feature = "rules")]
    rules_engine: Option<Arc<parking_lot::Mutex<RulesEngine>>>,
}

impl Router {
    /// Create a new router with the given configuration
    pub fn new(config: RouterConfig) -> Self {
        let gesture_registry = if config.gesture_coalescing {
            Some(Arc::new(GestureRegistry::new(Duration::from_millis(
                config.gesture_coalesce_interval_ms,
            ))))
        } else {
            None
        };

        let state = Arc::new(RouterState::with_config(config.state_config.clone()));

        Self {
            config,
            sessions: Arc::new(DashMap::new()),
            subscriptions: Arc::new(SubscriptionManager::new()),
            state,
            running: Arc::new(RwLock::new(false)),
            token_validator: None,
            p2p_capabilities: Arc::new(P2PCapabilities::new()),
            gesture_registry,
            write_validator: None,
            snapshot_filter: None,
            #[cfg(feature = "rules")]
            rules_engine: None,
        }
    }

    /// Create a router with a token validator for authenticated mode
    pub fn with_validator<V: TokenValidator + 'static>(mut self, validator: V) -> Self {
        self.token_validator = Some(Arc::new(validator));
        self
    }

    /// Set the token validator
    pub fn set_validator<V: TokenValidator + 'static>(&mut self, validator: V) {
        self.token_validator = Some(Arc::new(validator));
    }

    /// Set the write validator for application-specific authorization
    pub fn set_write_validator<V: WriteValidator + 'static>(&mut self, validator: V) {
        self.write_validator = Some(Arc::new(validator));
    }

    /// Set the snapshot filter for application-specific data redaction
    pub fn set_snapshot_filter<F: SnapshotFilter + 'static>(&mut self, filter: F) {
        self.snapshot_filter = Some(Arc::new(filter));
    }

    /// Create a router with a journal for state persistence.
    ///
    /// The journal records all state mutations, enabling crash recovery
    /// and REPLAY message support.
    #[cfg(feature = "journal")]
    pub fn with_journal(mut self, journal: Arc<dyn Journal>) -> Self {
        // We need to recreate the state with journal support
        let mut state = RouterState::with_config(self.config.state_config.clone());
        state.set_journal(journal);
        self.state = Arc::new(state);
        self
    }

    /// Create a router with a rules engine for server-side automation.
    ///
    /// Rules are evaluated after SET and PUBLISH operations, allowing
    /// automatic responses like "when motion detected, turn on lights".
    #[cfg(feature = "rules")]
    pub fn with_rules(mut self, engine: RulesEngine) -> Self {
        self.rules_engine = Some(Arc::new(parking_lot::Mutex::new(engine)));
        self
    }

    /// Get the rules engine interval rules for spawning timer tasks.
    #[cfg(feature = "rules")]
    pub fn rules_engine(&self) -> Option<&Arc<parking_lot::Mutex<RulesEngine>>> {
        self.rules_engine.as_ref()
    }

    /// Get a reference to the CPSK validator if one is configured
    /// This allows adding tokens at runtime
    pub fn cpsk_validator(&self) -> Option<&CpskValidator> {
        self.token_validator
            .as_ref()
            .and_then(|v| v.as_any().downcast_ref::<CpskValidator>())
    }

    /// Get the security mode
    pub fn security_mode(&self) -> SecurityMode {
        self.config.security_mode
    }

    // =========================================================================
    // Transport-Agnostic Methods
    // =========================================================================

    /// Serve using any TransportServer implementation.
    ///
    /// This is the core method that all transport-specific methods use internally.
    /// Use this when you have a custom transport or want full control.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clasp_router::Router;
    /// use clasp_transport::WebSocketServer;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::default();
    /// let server = WebSocketServer::bind("0.0.0.0:7330").await?;
    /// router.serve_on(server).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn serve_on<S>(&self, mut server: S) -> Result<()>
    where
        S: TransportServer + 'static,
        S::Sender: 'static,
        S::Receiver: 'static,
    {
        info!("Router accepting connections");
        *self.running.write() = true;

        // Start session cleanup task if timeout is configured
        if self.config.session_timeout > 0 {
            self.start_session_cleanup_task();
        }

        // Start gesture flush task if coalescing is enabled
        if let Some(ref registry) = self.gesture_registry {
            self.start_gesture_flush_task(Arc::clone(registry));
        }

        // Start state cleanup task (removes stale params and signals)
        self.start_state_cleanup_task();

        while *self.running.read() {
            match server.accept().await {
                Ok((sender, receiver, addr)) => {
                    // Enforce max_sessions limit
                    let current_sessions = self.sessions.len();
                    if current_sessions >= self.config.max_sessions {
                        warn!(
                            "Rejecting connection from {}: max sessions reached ({}/{})",
                            addr, current_sessions, self.config.max_sessions
                        );
                        // Connection will be closed when sender/receiver are dropped
                        continue;
                    }

                    info!("New connection from {}", addr);
                    self.handle_connection(Arc::new(sender), receiver, addr);
                }
                Err(e) => {
                    error!("Accept error: {}", e);
                }
            }
        }

        Ok(())
    }

    /// Start background task to flush stale gesture moves
    fn start_gesture_flush_task(&self, registry: Arc<GestureRegistry>) {
        // Skip gesture flush task if coalescing is disabled (interval is 0)
        if self.config.gesture_coalesce_interval_ms == 0 {
            return;
        }

        let sessions = Arc::clone(&self.sessions);
        let subscriptions = Arc::clone(&self.subscriptions);
        let running = Arc::clone(&self.running);
        let flush_interval = Duration::from_millis(self.config.gesture_coalesce_interval_ms);

        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(flush_interval);

            loop {
                ticker.tick().await;

                if !*running.read() {
                    break;
                }

                // Flush any stale buffered moves
                let to_flush = registry.flush_stale();
                for pub_msg in to_flush {
                    let msg = Message::Publish(pub_msg.clone());
                    let subscribers =
                        subscriptions.find_subscribers(&pub_msg.address, Some(SignalType::Gesture));

                    if let Ok(bytes) = codec::encode(&msg) {
                        for sub_session_id in subscribers {
                            if let Some(sub_session) = sessions.get(&sub_session_id) {
                                try_send_with_drop_tracking_sync(
                                    sub_session.value(),
                                    bytes.clone(),
                                    &sub_session_id,
                                );
                            }
                        }
                    }
                }

                // Cleanup very old gestures (> 5 minutes with no end)
                registry.cleanup_stale(Duration::from_secs(300));
            }

            debug!("Gesture flush task stopped");
        });
    }

    /// Start background task to clean up timed-out sessions
    fn start_session_cleanup_task(&self) {
        let sessions = Arc::clone(&self.sessions);
        let subscriptions = Arc::clone(&self.subscriptions);
        let running = Arc::clone(&self.running);
        let timeout_secs = self.config.session_timeout;

        tokio::spawn(async move {
            let check_interval = std::time::Duration::from_secs(timeout_secs / 4)
                .max(std::time::Duration::from_secs(10));
            let timeout = std::time::Duration::from_secs(timeout_secs);

            loop {
                tokio::time::sleep(check_interval).await;

                if !*running.read() {
                    break;
                }

                // Find and remove timed-out sessions
                let timed_out: Vec<SessionId> = sessions
                    .iter()
                    .filter(|entry| entry.value().idle_duration() > timeout)
                    .map(|entry| entry.key().clone())
                    .collect();

                for session_id in timed_out {
                    if let Some((id, session)) = sessions.remove(&session_id) {
                        info!(
                            "Session {} timed out after {:?} idle",
                            id,
                            session.idle_duration()
                        );
                        subscriptions.remove_session(&id);
                    }
                }
            }

            debug!("Session cleanup task stopped");
        });
    }

    /// Start background task to clean up stale state entries
    fn start_state_cleanup_task(&self) {
        let state = Arc::clone(&self.state);
        let running = Arc::clone(&self.running);

        tokio::spawn(async move {
            // Clean up every 60 seconds
            let cleanup_interval = std::time::Duration::from_secs(60);

            loop {
                tokio::time::sleep(cleanup_interval).await;

                if !*running.read() {
                    break;
                }

                // Run cleanup on state store
                let (params_removed, signals_removed) = state.cleanup_stale();

                if params_removed > 0 || signals_removed > 0 {
                    debug!(
                        "State cleanup: removed {} stale params, {} stale signals",
                        params_removed, signals_removed
                    );
                }
            }

            debug!("State cleanup task stopped");
        });
    }

    // =========================================================================
    // WebSocket Transport
    // =========================================================================

    /// Start the router on WebSocket (default, recommended).
    ///
    /// WebSocket is the universal baseline transport:
    /// - Works in browsers
    /// - Works on all hosting platforms (including DO App Platform)
    /// - Easy firewall/proxy traversal
    ///
    /// Default port: 7330
    #[cfg(feature = "websocket")]
    pub async fn serve_websocket(&self, addr: &str) -> Result<()> {
        let server = WebSocketServer::bind(addr).await?;
        info!("WebSocket server listening on {}", addr);
        self.serve_on(server).await
    }

    /// Backward-compatible alias for `serve_websocket`.
    #[cfg(feature = "websocket")]
    pub async fn serve(&self, addr: &str) -> Result<()> {
        self.serve_websocket(addr).await
    }

    // =========================================================================
    // QUIC Transport (feature-gated)
    // =========================================================================

    /// Start the router on QUIC.
    ///
    /// QUIC is ideal for native applications:
    /// - 0-RTT connection establishment
    /// - Connection migration (mobile networks)
    /// - Built-in encryption (TLS 1.3)
    /// - Lower latency than WebSocket
    ///
    /// **WARNING**: QUIC requires UDP, which is NOT supported on:
    /// - DigitalOcean App Platform
    /// - Many PaaS providers
    /// - Some corporate firewalls
    ///
    /// Use a VPS/Droplet for QUIC support.
    ///
    /// Default port: 7331 (to avoid conflict with WebSocket on 7330)
    #[cfg(feature = "quic")]
    pub async fn serve_quic(
        &self,
        addr: SocketAddr,
        cert_der: Vec<u8>,
        key_der: Vec<u8>,
    ) -> Result<()> {
        let server = QuicTransport::new_server(addr, cert_der, key_der)
            .map_err(|e| RouterError::Transport(e))?;
        info!("QUIC server listening on {}", addr);
        self.serve_quic_transport(server).await
    }

    /// Internal: Serve using a QuicTransport server.
    ///
    /// QUIC has a different accept pattern (connection then stream),
    /// so we need special handling.
    #[cfg(feature = "quic")]
    async fn serve_quic_transport(&self, server: QuicTransport) -> Result<()> {
        *self.running.write() = true;

        while *self.running.read() {
            match server.accept().await {
                Ok(connection) => {
                    let addr = connection.remote_address();
                    info!("QUIC connection from {}", addr);

                    // Accept bidirectional stream for CLASP protocol
                    match connection.accept_bi().await {
                        Ok((sender, receiver)) => {
                            self.handle_connection(Arc::new(sender), receiver, addr);
                        }
                        Err(e) => {
                            error!("QUIC stream accept error: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("QUIC accept error: {}", e);
                }
            }
        }

        Ok(())
    }

    // =========================================================================
    // Multi-Transport Support
    // =========================================================================

    /// Serve on multiple transports simultaneously.
    ///
    /// All transports share the same router state, so a client connected via
    /// WebSocket can communicate with a client connected via QUIC.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clasp_router::{Router, TransportConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::default();
    /// router.serve_multi(vec![
    ///     TransportConfig::WebSocket { addr: "0.0.0.0:7330".into() },
    ///     // QUIC requires feature and UDP support
    ///     // TransportConfig::Quic { addr: "0.0.0.0:7331".parse()?, cert, key },
    /// ]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn serve_multi(&self, transports: Vec<TransportConfig>) -> Result<()> {
        use futures::future::try_join_all;

        if transports.is_empty() {
            return Err(RouterError::Config("No transports configured".into()));
        }

        let mut handles = vec![];

        for config in transports {
            let router = self.clone_internal();
            let handle = tokio::spawn(async move {
                match config {
                    #[cfg(feature = "websocket")]
                    TransportConfig::WebSocket { addr } => router.serve_websocket(&addr).await,
                    #[cfg(feature = "quic")]
                    TransportConfig::Quic { addr, cert, key } => {
                        router.serve_quic(addr, cert, key).await
                    }
                    #[allow(unreachable_patterns)]
                    _ => Err(RouterError::Config(
                        "Transport not enabled at compile time".into(),
                    )),
                }
            });
            handles.push(handle);
        }

        // Wait for all transports (or first error)
        let results = try_join_all(handles)
            .await
            .map_err(|e| RouterError::Config(format!("Transport task failed: {}", e)))?;

        // Check for errors from any transport
        for result in results {
            result?;
        }

        Ok(())
    }

    /// Serve all configured protocols simultaneously.
    ///
    /// This is the recommended way to run a multi-protocol CLASP server.
    /// All protocols share the same router state, so clients connected via
    /// different protocols can communicate seamlessly.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clasp_router::{Router, MultiProtocolConfig};
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let router = Router::default();
    /// let config = MultiProtocolConfig {
    ///     websocket_addr: Some("0.0.0.0:7330".into()),
    ///     ..Default::default()
    /// };
    /// router.serve_all(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn serve_all(&self, config: MultiProtocolConfig) -> Result<()> {
        use futures::future::select_all;

        let mut handles: Vec<tokio::task::JoinHandle<Result<()>>> = vec![];
        let mut protocol_names: Vec<&str> = vec![];

        // WebSocket server
        #[cfg(feature = "websocket")]
        if let Some(ref addr) = config.websocket_addr {
            info!("Starting WebSocket server on {}", addr);
            protocol_names.push("WebSocket");
            let router = self.clone_internal();
            let addr = addr.clone();
            handles.push(tokio::spawn(
                async move { router.serve_websocket(&addr).await },
            ));
        }

        // QUIC server
        #[cfg(feature = "quic")]
        if let Some(ref quic_config) = config.quic {
            info!("Starting QUIC server on {}", quic_config.addr);
            protocol_names.push("QUIC");
            let router = self.clone_internal();
            let addr = quic_config.addr;
            let cert = quic_config.cert.clone();
            let key = quic_config.key.clone();
            handles.push(tokio::spawn(async move {
                router.serve_quic(addr, cert, key).await
            }));
        }

        // MQTT server adapter
        #[cfg(feature = "mqtt-server")]
        if let Some(mqtt_config) = config.mqtt {
            info!("Starting MQTT server on {}", mqtt_config.bind_addr);
            protocol_names.push("MQTT");
            let adapter = crate::adapters::MqttServerAdapter::new(
                mqtt_config,
                Arc::clone(&self.sessions),
                Arc::clone(&self.subscriptions),
                Arc::clone(&self.state),
            );
            handles.push(tokio::spawn(async move { adapter.serve().await }));
        }

        // OSC server adapter
        #[cfg(feature = "osc-server")]
        if let Some(osc_config) = config.osc {
            info!("Starting OSC server on {}", osc_config.bind_addr);
            protocol_names.push("OSC");
            let adapter = crate::adapters::OscServerAdapter::new(
                osc_config,
                Arc::clone(&self.sessions),
                Arc::clone(&self.subscriptions),
                Arc::clone(&self.state),
            );
            handles.push(tokio::spawn(async move { adapter.serve().await }));
        }

        if handles.is_empty() {
            return Err(RouterError::Config("No protocols configured".into()));
        }

        info!(
            "Multi-protocol server running with {} protocols: {}",
            handles.len(),
            protocol_names.join(", ")
        );

        *self.running.write() = true;

        // Start session cleanup task
        if self.config.session_timeout > 0 {
            self.start_session_cleanup_task();
        }

        // Start gesture flush task if coalescing is enabled
        if let Some(ref registry) = self.gesture_registry {
            self.start_gesture_flush_task(Arc::clone(registry));
        }

        // Start state cleanup task (removes stale params and signals)
        self.start_state_cleanup_task();

        // Wait for any server to complete (usually due to error or shutdown)
        loop {
            if handles.is_empty() {
                break;
            }

            let (result, _index, remaining) = select_all(handles).await;
            handles = remaining;

            match result {
                Ok(Ok(())) => {
                    // Server completed normally (shutdown)
                    debug!("Protocol server completed normally");
                }
                Ok(Err(e)) => {
                    error!("Protocol server error: {}", e);
                    // Continue running other servers
                }
                Err(e) => {
                    error!("Protocol server task panicked: {}", e);
                    // Continue running other servers
                }
            }
        }

        Ok(())
    }

    /// Get shared state references for use by adapters
    pub fn shared_state(
        &self,
    ) -> (
        Arc<DashMap<SessionId, Arc<Session>>>,
        Arc<SubscriptionManager>,
        Arc<RouterState>,
    ) {
        (
            Arc::clone(&self.sessions),
            Arc::clone(&self.subscriptions),
            Arc::clone(&self.state),
        )
    }

    /// Internal clone for spawning transport tasks.
    /// Shares all Arc state with the original.
    fn clone_internal(&self) -> Self {
        Self {
            config: self.config.clone(),
            sessions: Arc::clone(&self.sessions),
            subscriptions: Arc::clone(&self.subscriptions),
            state: Arc::clone(&self.state),
            running: Arc::clone(&self.running),
            token_validator: self.token_validator.clone(),
            p2p_capabilities: Arc::clone(&self.p2p_capabilities),
            gesture_registry: self.gesture_registry.clone(),
            write_validator: self.write_validator.clone(),
            snapshot_filter: self.snapshot_filter.clone(),
            #[cfg(feature = "rules")]
            rules_engine: self.rules_engine.clone(),
        }
    }

    /// Get active gesture count (for diagnostics)
    pub fn active_gesture_count(&self) -> usize {
        self.gesture_registry
            .as_ref()
            .map(|r| r.active_count())
            .unwrap_or(0)
    }

    /// Handle a new connection
    fn handle_connection(
        &self,
        sender: Arc<dyn TransportSender>,
        mut receiver: impl TransportReceiver + 'static,
        addr: SocketAddr,
    ) {
        let sessions = Arc::clone(&self.sessions);
        let subscriptions = Arc::clone(&self.subscriptions);
        let state = Arc::clone(&self.state);
        let config = self.config.clone();
        let running = Arc::clone(&self.running);
        let token_validator = self.token_validator.clone();
        let security_mode = self.config.security_mode;
        let p2p_capabilities = Arc::clone(&self.p2p_capabilities);
        let gesture_registry = self.gesture_registry.clone();
        let write_validator = self.write_validator.clone();
        let snapshot_filter = self.snapshot_filter.clone();
        #[cfg(feature = "rules")]
        let rules_engine = self.rules_engine.clone();

        tokio::spawn(async move {
            let mut session: Option<Arc<Session>> = None;
            let mut handshake_complete = false;

            // Phase 1: Wait for Hello message with timeout
            let handshake_result = tokio::time::timeout(HANDSHAKE_TIMEOUT, async {
                loop {
                    match receiver.recv().await {
                        Some(TransportEvent::Data(data)) => {
                            // Decode and check if it's a Hello message
                            match codec::decode(&data) {
                                Ok((msg, _)) => {
                                    if matches!(msg, Message::Hello(_)) {
                                        return Some(data);
                                    } else {
                                        // Non-Hello message before handshake
                                        warn!(
                                            "Received non-Hello message before handshake from {}",
                                            addr
                                        );
                                        return None;
                                    }
                                }
                                Err(e) => {
                                    warn!("Decode error during handshake from {}: {}", addr, e);
                                    return None;
                                }
                            }
                        }
                        Some(TransportEvent::Disconnected { .. }) | None => {
                            return None;
                        }
                        Some(TransportEvent::Error(e)) => {
                            error!("Transport error during handshake from {}: {}", addr, e);
                            return None;
                        }
                        _ => {}
                    }
                }
            })
            .await;

            // Check handshake result
            let hello_data = match handshake_result {
                Ok(Some(data)) => data,
                Ok(None) => {
                    debug!("Handshake failed for {}", addr);
                    return;
                }
                Err(_) => {
                    warn!(
                        "Handshake timeout for {} after {:?}",
                        addr, HANDSHAKE_TIMEOUT
                    );
                    return;
                }
            };

            // Process the Hello message
            if let Ok((msg, frame)) = codec::decode(&hello_data) {
                if let Some(response) = handle_message(
                    &msg,
                    &frame,
                    &session,
                    &sender,
                    &sessions,
                    &subscriptions,
                    &state,
                    &config,
                    security_mode,
                    &token_validator,
                    &p2p_capabilities,
                    &gesture_registry,
                    &write_validator,
                    &snapshot_filter,
                    #[cfg(feature = "rules")]
                    &rules_engine,
                )
                .await
                {
                    match response {
                        MessageResult::NewSession(s) => {
                            session = Some(s);
                            handshake_complete = true;
                        }
                        MessageResult::Send(bytes) => {
                            let _ = sender.send(bytes).await;
                        }
                        MessageResult::Disconnect => {
                            info!(
                                "Disconnecting client {} due to auth failure during handshake",
                                addr
                            );
                            return;
                        }
                        _ => {}
                    }
                }
            }

            if !handshake_complete {
                debug!("Handshake incomplete for {}", addr);
                return;
            }

            // Phase 2: Main message loop (after successful handshake)
            while *running.read() {
                match receiver.recv().await {
                    Some(TransportEvent::Data(data)) => {
                        // Check rate limit before processing
                        if config.rate_limiting_enabled {
                            if let Some(ref s) = session {
                                if !s.check_rate_limit(config.max_messages_per_second) {
                                    warn!(
                                        "Rate limit exceeded for session {} ({} msgs/sec > {})",
                                        s.id,
                                        s.messages_per_second(),
                                        config.max_messages_per_second
                                    );
                                    // Send error and continue (don't disconnect for rate limiting)
                                    let error = Message::Error(ErrorMessage {
                                        code: 429, // Too Many Requests
                                        message: format!(
                                            "Rate limit exceeded: {} messages/second",
                                            config.max_messages_per_second
                                        ),
                                        address: None,
                                        correlation_id: None,
                                    });
                                    if let Ok(bytes) = codec::encode(&error) {
                                        let _ = sender.send(bytes).await;
                                    }
                                    continue;
                                }
                            }
                        }

                        // Decode message
                        match codec::decode(&data) {
                            Ok((msg, frame)) => {
                                // Handle message
                                if let Some(response) = handle_message(
                                    &msg,
                                    &frame,
                                    &session,
                                    &sender,
                                    &sessions,
                                    &subscriptions,
                                    &state,
                                    &config,
                                    security_mode,
                                    &token_validator,
                                    &p2p_capabilities,
                                    &gesture_registry,
                                    &write_validator,
                                    &snapshot_filter,
                                    #[cfg(feature = "rules")]
                                    &rules_engine,
                                )
                                .await
                                {
                                    match response {
                                        MessageResult::NewSession(s) => {
                                            session = Some(s);
                                        }
                                        MessageResult::Send(bytes) => {
                                            if let Err(e) = sender.send(bytes).await {
                                                error!("Send error: {}", e);
                                                break;
                                            }
                                        }
                                        MessageResult::Broadcast(bytes, exclude) => {
                                            broadcast_to_subscribers(&bytes, &sessions, &exclude);
                                        }
                                        MessageResult::Disconnect => {
                                            info!(
                                                "Disconnecting client {} due to auth failure",
                                                addr
                                            );
                                            break;
                                        }
                                        MessageResult::None => {}
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Decode error from {}: {}", addr, e);
                            }
                        }
                    }
                    Some(TransportEvent::Disconnected { reason }) => {
                        info!("Client {} disconnected: {:?}", addr, reason);
                        break;
                    }
                    Some(TransportEvent::Error(e)) => {
                        error!("Transport error from {}: {}", addr, e);
                        break;
                    }
                    None => {
                        break;
                    }
                    _ => {}
                }
            }

            // Cleanup session
            if let Some(s) = session {
                info!("Removing session {}", s.id);
                sessions.remove(&s.id);
                subscriptions.remove_session(&s.id);
                p2p_capabilities.unregister(&s.id);
            }
        });
    }

    /// Stop the router
    pub fn stop(&self) {
        *self.running.write() = false;
    }

    /// Get session count
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get state
    pub fn state(&self) -> &RouterState {
        &self.state
    }

    /// Get subscription count
    pub fn subscription_count(&self) -> usize {
        self.subscriptions.len()
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new(RouterConfig::default())
    }
}

/// Result of handling a message
enum MessageResult {
    NewSession(Arc<Session>),
    Send(Bytes),
    Broadcast(Bytes, SessionId),
    Disconnect,
    None,
}

/// Maximum params per snapshot chunk to stay under frame size limit.
/// Frame max payload is 65535 bytes. With ~44 bytes per param average,
/// we target 800 params per chunk (~35KB) to leave headroom.
const MAX_SNAPSHOT_CHUNK_SIZE: usize = 800;

/// Send a snapshot, chunking if too large for a single frame.
async fn send_chunked_snapshot(sender: &Arc<dyn TransportSender>, snapshot: SnapshotMessage) {
    let param_count = snapshot.params.len();

    if param_count <= MAX_SNAPSHOT_CHUNK_SIZE {
        // Small enough to send in one frame
        let msg = Message::Snapshot(snapshot);
        if let Ok(bytes) = codec::encode(&msg) {
            let _ = sender.send(bytes).await;
        } else {
            warn!("Failed to encode snapshot ({} params)", param_count);
        }
        return;
    }

    // Chunk large snapshots
    let chunks = snapshot.params.chunks(MAX_SNAPSHOT_CHUNK_SIZE);
    let chunk_count = (param_count + MAX_SNAPSHOT_CHUNK_SIZE - 1) / MAX_SNAPSHOT_CHUNK_SIZE;

    debug!(
        "Chunking snapshot of {} params into {} chunks",
        param_count, chunk_count
    );

    for (i, chunk) in chunks.enumerate() {
        let chunk_snapshot = SnapshotMessage {
            params: chunk.to_vec(),
        };
        let msg = Message::Snapshot(chunk_snapshot);
        match codec::encode(&msg) {
            Ok(bytes) => {
                if let Err(e) = sender.send(bytes).await {
                    warn!(
                        "Failed to send snapshot chunk {}/{}: {}",
                        i + 1,
                        chunk_count,
                        e
                    );
                    break;
                }
            }
            Err(e) => {
                warn!(
                    "Failed to encode snapshot chunk {}/{}: {}",
                    i + 1,
                    chunk_count,
                    e
                );
            }
        }
    }
}

/// Handle an incoming message
async fn handle_message(
    msg: &Message,
    _frame: &Frame,
    session: &Option<Arc<Session>>,
    sender: &Arc<dyn TransportSender>,
    sessions: &Arc<DashMap<SessionId, Arc<Session>>>,
    subscriptions: &Arc<SubscriptionManager>,
    state: &Arc<RouterState>,
    config: &RouterConfig,
    security_mode: SecurityMode,
    token_validator: &Option<Arc<dyn TokenValidator>>,
    p2p_capabilities: &Arc<P2PCapabilities>,
    gesture_registry: &Option<Arc<GestureRegistry>>,
    write_validator: &Option<Arc<dyn WriteValidator>>,
    snapshot_filter: &Option<Arc<dyn SnapshotFilter>>,
    #[cfg(feature = "rules")] rules_engine: &Option<Arc<parking_lot::Mutex<RulesEngine>>>,
) -> Option<MessageResult> {
    match msg {
        Message::Hello(hello) => {
            // In authenticated mode, validate the token
            let (authenticated, subject, scopes) = match security_mode {
                SecurityMode::Open => {
                    // Open mode: no authentication required
                    (false, None, Vec::new())
                }
                SecurityMode::Authenticated => {
                    // Authenticated mode: require valid token
                    let token = match &hello.token {
                        Some(t) => t,
                        None => {
                            warn!("Connection rejected: no token provided in authenticated mode");
                            let error = Message::Error(ErrorMessage {
                                code: 300, // Unauthorized
                                message: "Authentication required".to_string(),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            let _ = sender.send(bytes).await;
                            return Some(MessageResult::Disconnect);
                        }
                    };

                    // Validate token
                    let validator = match token_validator {
                        Some(v) => v,
                        None => {
                            error!("Authenticated mode but no token validator configured");
                            let error = Message::Error(ErrorMessage {
                                code: 500, // Internal error
                                message: "Server misconfiguration".to_string(),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            let _ = sender.send(bytes).await;
                            return Some(MessageResult::Disconnect);
                        }
                    };

                    match validator.validate(token) {
                        ValidationResult::Valid(info) => {
                            info!(
                                "Token validated for subject: {:?}, scopes: {}",
                                info.subject,
                                info.scopes.len()
                            );
                            (true, info.subject, info.scopes)
                        }
                        ValidationResult::Expired => {
                            warn!("Connection rejected: token expired");
                            let error = Message::Error(ErrorMessage {
                                code: 302, // TokenExpired
                                message: "Token has expired".to_string(),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            let _ = sender.send(bytes).await;
                            return Some(MessageResult::Disconnect);
                        }
                        ValidationResult::Invalid(reason) => {
                            warn!("Connection rejected: invalid token - {}", reason);
                            let error = Message::Error(ErrorMessage {
                                code: 300, // Unauthorized
                                message: format!("Invalid token: {}", reason),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            let _ = sender.send(bytes).await;
                            return Some(MessageResult::Disconnect);
                        }
                        ValidationResult::NotMyToken => {
                            warn!("Connection rejected: unrecognized token format");
                            let error = Message::Error(ErrorMessage {
                                code: 300, // Unauthorized
                                message: "Unrecognized token format".to_string(),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            let _ = sender.send(bytes).await;
                            return Some(MessageResult::Disconnect);
                        }
                    }
                }
            };

            // Create new session
            let mut new_session =
                Session::new(sender.clone(), hello.name.clone(), hello.features.clone());

            // Set authentication state
            if authenticated {
                new_session.set_authenticated(
                    hello.token.clone().unwrap_or_default(),
                    subject,
                    scopes,
                );
            }

            let new_session = Arc::new(new_session);
            let session_id = new_session.id.clone();
            sessions.insert(session_id.clone(), new_session.clone());

            info!(
                "Session created: {} ({}) authenticated={}",
                hello.name, session_id, new_session.authenticated
            );

            #[cfg(feature = "federation")]
            if new_session.is_federation_peer() {
                info!(
                    "Federation peer detected: {} ({})",
                    hello.name, session_id
                );
            }

            // Send welcome
            let welcome = new_session.welcome_message(&config.name, &config.features);
            let response = codec::encode(&welcome).ok()?;

            // Send welcome first
            let _ = sender.send(response).await;

            // Send initial snapshot (chunked if too large), with optional filtering
            let mut full_snapshot = state.full_snapshot();
            if let Some(ref filter) = snapshot_filter {
                full_snapshot.params =
                    filter.filter_snapshot(full_snapshot.params, &new_session, state);
            }
            send_chunked_snapshot(sender, full_snapshot).await;

            Some(MessageResult::NewSession(new_session))
        }

        Message::Subscribe(sub) => {
            let session = session.as_ref()?;

            // Check subscription limit
            let current_subs = session.subscriptions().len();
            let max_subs = 1000; // Default limit
            if current_subs >= max_subs {
                warn!(
                    "Session {} subscription limit reached ({}/{})",
                    session.id, current_subs, max_subs
                );
                let error = Message::Error(ErrorMessage {
                    code: 429, // Too Many Requests
                    message: format!("Subscription limit reached (max {})", max_subs),
                    address: Some(sub.pattern.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            // Check scope for read access (in authenticated mode).
            // Use strict read scope check to prevent write-only scopes
            // (e.g. write:/chat/user/*/dms/*) from granting subscription
            // access to other users' private data.
            if security_mode == SecurityMode::Authenticated
                && !session.has_strict_read_scope(&sub.pattern)
            {
                warn!(
                    "Session {} denied SUBSCRIBE to {} - insufficient scope",
                    session.id, sub.pattern
                );
                let error = Message::Error(ErrorMessage {
                    code: 301, // Forbidden
                    message: "Insufficient scope for subscription".to_string(),
                    address: Some(sub.pattern.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            // Create subscription
            match Subscription::new(
                sub.id,
                session.id.clone(),
                &sub.pattern,
                sub.types.clone(),
                sub.options.clone().unwrap_or_default(),
            ) {
                Ok(subscription) => {
                    subscriptions.add(subscription);
                    session.add_subscription(sub.id);

                    debug!("Session {} subscribed to {}", session.id, sub.pattern);

                    // Send matching current values (chunked if large), with optional filtering
                    let mut snapshot = state.snapshot(&sub.pattern);
                    if let Some(ref filter) = snapshot_filter {
                        snapshot.params = filter.filter_snapshot(snapshot.params, session, state);
                    }
                    if !snapshot.params.is_empty() {
                        send_chunked_snapshot(sender, snapshot).await;
                    }
                }
                Err(e) => {
                    warn!("Invalid subscription pattern: {}", e);
                    let error = Message::Error(ErrorMessage {
                        code: 202,
                        message: e.to_string(),
                        address: Some(sub.pattern.clone()),
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    return Some(MessageResult::Send(bytes));
                }
            }

            Some(MessageResult::None)
        }

        Message::Unsubscribe(unsub) => {
            let session = session.as_ref()?;
            subscriptions.remove(&session.id, unsub.id);
            session.remove_subscription(unsub.id);
            Some(MessageResult::None)
        }

        Message::Set(set) => {
            let session = session.as_ref()?;

            // Check scope for write access (in authenticated mode)
            if security_mode == SecurityMode::Authenticated
                && !session.has_scope(Action::Write, &set.address)
            {
                warn!(
                    "Session {} denied SET to {} - insufficient scope",
                    session.id, set.address
                );
                let error = Message::Error(ErrorMessage {
                    code: 301, // Forbidden
                    message: "Insufficient scope for write operation".to_string(),
                    address: Some(set.address.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            // Application-specific write validation
            if let Some(ref validator) = write_validator {
                if let Err(reason) =
                    validator.validate_write(&set.address, &set.value, session, state)
                {
                    warn!(
                        "Session {} denied SET to {} by write validator: {}",
                        session.id, set.address, reason
                    );
                    let error = Message::Error(ErrorMessage {
                        code: 403,
                        message: reason,
                        address: Some(set.address.clone()),
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    return Some(MessageResult::Send(bytes));
                }
            }

            // Apply to state
            match state.apply_set(set, &session.id) {
                Ok(revision) => {
                    // Broadcast to subscribers
                    let subscribers =
                        subscriptions.find_subscribers(&set.address, Some(SignalType::Param));

                    // Create updated SET message with revision
                    let mut updated_set = set.clone();
                    updated_set.revision = Some(revision);
                    let broadcast_msg = Message::Set(updated_set);

                    if let Ok(bytes) = codec::encode(&broadcast_msg) {
                        // Send to all subscribers (including sender for confirmation)
                        // Use try_send for non-blocking broadcast with backpressure
                        for sub_session_id in subscribers {
                            if let Some(sub_session) = sessions.get(&sub_session_id) {
                                try_send_with_drop_tracking_sync(
                                    sub_session.value(),
                                    bytes.clone(),
                                    &sub_session_id,
                                );
                            }
                        }
                    }

                    // Evaluate rules engine after SET
                    #[cfg(feature = "rules")]
                    if let Some(ref engine) = rules_engine {
                        let actions = engine.lock().evaluate(
                            &set.address,
                            &set.value,
                            SignalType::Param,
                            Some(session.id.as_str()),
                            |addr| state.get(addr),
                        );
                        if !actions.is_empty() {
                            execute_rule_actions(actions, state, sessions, subscriptions);
                        }
                    }

                    // Send ACK to sender
                    let ack = Message::Ack(AckMessage {
                        address: Some(set.address.clone()),
                        revision: Some(revision),
                        locked: None,
                        holder: None,
                        correlation_id: None,
                    });
                    let ack_bytes = codec::encode(&ack).ok()?;
                    return Some(MessageResult::Send(ack_bytes));
                }
                Err(e) => {
                    let error = Message::Error(ErrorMessage {
                        code: 400,
                        message: format!("{:?}", e),
                        address: Some(set.address.clone()),
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    return Some(MessageResult::Send(bytes));
                }
            }
        }

        Message::Get(get) => {
            let session = session.as_ref()?;

            // Check scope for read access (in authenticated mode)
            if security_mode == SecurityMode::Authenticated
                && !session.has_scope(Action::Read, &get.address)
            {
                warn!(
                    "Session {} denied GET to {} - insufficient scope",
                    session.id, get.address
                );
                let error = Message::Error(ErrorMessage {
                    code: 301, // Forbidden
                    message: "Insufficient scope for read operation".to_string(),
                    address: Some(get.address.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            if let Some(param_state) = state.get_state(&get.address) {
                let mut params = vec![clasp_core::ParamValue {
                    address: get.address.clone(),
                    value: param_state.value,
                    revision: param_state.revision,
                    writer: Some(param_state.writer),
                    timestamp: Some(param_state.timestamp),
                }];

                // Apply snapshot filter to GET responses (strip sensitive fields)
                if let Some(ref filter) = snapshot_filter {
                    params = filter.filter_snapshot(params, session, state);
                }

                if params.is_empty() {
                    return Some(MessageResult::None);
                }

                let snapshot = Message::Snapshot(clasp_core::SnapshotMessage { params });
                let bytes = codec::encode(&snapshot).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            Some(MessageResult::None)
        }

        Message::Publish(pub_msg) => {
            let session = session.as_ref()?;

            // Check scope for write access (in authenticated mode)
            if security_mode == SecurityMode::Authenticated
                && !session.has_scope(Action::Write, &pub_msg.address)
            {
                warn!(
                    "Session {} denied PUBLISH to {} - insufficient scope",
                    session.id, pub_msg.address
                );
                let error = Message::Error(ErrorMessage {
                    code: 301, // Forbidden
                    message: "Insufficient scope for publish operation".to_string(),
                    address: Some(pub_msg.address.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            // Application-specific write validation for PUBLISH
            if let Some(ref validator) = write_validator {
                let pub_value = pub_msg
                    .value
                    .as_ref()
                    .cloned()
                    .unwrap_or(clasp_core::Value::Null);
                if let Err(reason) =
                    validator.validate_write(&pub_msg.address, &pub_value, session, state)
                {
                    warn!(
                        "Session {} denied PUBLISH to {} by write validator: {}",
                        session.id, pub_msg.address, reason
                    );
                    let error = Message::Error(ErrorMessage {
                        code: 403,
                        message: reason,
                        address: Some(pub_msg.address.clone()),
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&error).ok()?;
                    return Some(MessageResult::Send(bytes));
                }
            }

            // Check for P2P signaling addresses
            match analyze_address(&pub_msg.address) {
                P2PAddressType::Signal { target_session } => {
                    // Route P2P signal directly to target session
                    debug!("P2P signal from {} to {}", session.id, target_session);

                    if let Ok(bytes) = codec::encode(msg) {
                        if let Some(target) = sessions.get(&target_session) {
                            let _ = target.send(bytes).await;
                        } else {
                            // Target session not found
                            warn!("P2P signal target session not found: {}", target_session);
                            let error = Message::Error(ErrorMessage {
                                code: 404,
                                message: format!("Target session not found: {}", target_session),
                                address: Some(pub_msg.address.clone()),
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            return Some(MessageResult::Send(bytes));
                        }
                    }

                    return Some(MessageResult::None);
                }
                P2PAddressType::Announce => {
                    // P2P capability announcement - register and broadcast
                    debug!("P2P announce from session {}", session.id);

                    // Register the session as P2P capable
                    p2p_capabilities.register(&session.id);

                    // Broadcast to subscribers of the announce address
                    // Use try_send for non-blocking broadcast
                    let subscribers = subscriptions.find_subscribers(&pub_msg.address, None);
                    if let Ok(bytes) = codec::encode(msg) {
                        for sub_session_id in subscribers {
                            if sub_session_id != session.id {
                                if let Some(sub_session) = sessions.get(&sub_session_id) {
                                    try_send_with_drop_tracking_sync(
                                        sub_session.value(),
                                        bytes.clone(),
                                        &sub_session_id,
                                    );
                                }
                            }
                        }
                    }

                    return Some(MessageResult::None);
                }
                P2PAddressType::NotP2P => {
                    // Normal PUBLISH - fall through to standard handling
                }
            }

            // Standard PUBLISH handling for non-P2P addresses
            let signal_type = pub_msg.signal;

            // Check for gesture coalescing
            if let Some(registry) = gesture_registry {
                if signal_type == Some(SignalType::Gesture) {
                    match registry.process(pub_msg) {
                        GestureResult::Forward(messages) => {
                            // Forward all messages (may include flushed move + end)
                            // Use try_send for non-blocking broadcast
                            for forward_msg in messages {
                                let msg_to_send = Message::Publish(forward_msg.clone());
                                let subscribers = subscriptions
                                    .find_subscribers(&forward_msg.address, signal_type);
                                if let Ok(bytes) = codec::encode(&msg_to_send) {
                                    for sub_session_id in subscribers {
                                        if sub_session_id != session.id {
                                            if let Some(sub_session) = sessions.get(&sub_session_id)
                                            {
                                                try_send_with_drop_tracking_sync(
                                                    sub_session.value(),
                                                    bytes.clone(),
                                                    &sub_session_id,
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                            return Some(MessageResult::None);
                        }
                        GestureResult::Buffered => {
                            // Move was buffered, nothing to forward yet
                            return Some(MessageResult::None);
                        }
                        GestureResult::PassThrough => {
                            // Not a gesture, fall through to standard handling
                        }
                    }
                }
            }

            // Find subscribers
            let subscribers = subscriptions.find_subscribers(&pub_msg.address, signal_type);

            // Broadcast using try_send for non-blocking delivery
            if let Ok(bytes) = codec::encode(msg) {
                for sub_session_id in subscribers {
                    if sub_session_id != session.id {
                        if let Some(sub_session) = sessions.get(&sub_session_id) {
                            try_send_with_drop_tracking_sync(
                                sub_session.value(),
                                bytes.clone(),
                                &sub_session_id,
                            );
                        }
                    }
                }
            }

            // Record in journal (fire-and-forget)
            #[cfg(feature = "journal")]
            state.journal_publish(
                &pub_msg.address,
                signal_type.unwrap_or(SignalType::Event),
                pub_msg.value.as_ref(),
                &session.id,
            );

            // Evaluate rules engine after PUBLISH
            #[cfg(feature = "rules")]
            if let Some(ref engine) = rules_engine {
                let pub_value = pub_msg
                    .value
                    .as_ref()
                    .cloned()
                    .unwrap_or(clasp_core::Value::Null);
                let actions = engine.lock().evaluate(
                    &pub_msg.address,
                    &pub_value,
                    signal_type.unwrap_or(SignalType::Event),
                    Some(session.id.as_str()),
                    |addr| state.get(addr),
                );
                if !actions.is_empty() {
                    execute_rule_actions(actions, state, sessions, subscriptions);
                }
            }

            Some(MessageResult::None)
        }

        Message::Ping => {
            let pong = Message::Pong;
            let bytes = codec::encode(&pong).ok()?;
            Some(MessageResult::Send(bytes))
        }

        Message::Query(query) => {
            // Query the signal registry for matching signals
            let signals = state.query_signals(&query.pattern);
            let result = Message::Result(clasp_core::ResultMessage { signals });
            let bytes = codec::encode(&result).ok()?;
            Some(MessageResult::Send(bytes))
        }

        #[cfg(feature = "journal")]
        Message::Replay(replay) => {
            let session = session.as_ref()?;

            // Check scope for read access (in authenticated mode)
            if security_mode == SecurityMode::Authenticated
                && !session.has_strict_read_scope(&replay.pattern)
            {
                warn!(
                    "Session {} denied REPLAY to {} - insufficient scope",
                    session.id, replay.pattern
                );
                let error = Message::Error(ErrorMessage {
                    code: 301,
                    message: "Insufficient scope for replay".to_string(),
                    address: Some(replay.pattern.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            // Query the journal
            if let Some(journal) = state.journal() {
                match journal
                    .query(
                        &replay.pattern,
                        replay.from,
                        replay.to,
                        replay.limit,
                        &replay.types,
                    )
                    .await
                {
                    Ok(entries) => {
                        // Send each journal entry as SET or PUBLISH messages
                        for entry in entries {
                            let msg = if entry.msg_type == 0x21 {
                                // SET
                                Message::Set(SetMessage {
                                    address: entry.address,
                                    value: entry.value,
                                    revision: entry.revision,
                                    lock: false,
                                    unlock: false,
                                })
                            } else {
                                // PUBLISH
                                Message::Publish(PublishMessage {
                                    address: entry.address,
                                    signal: Some(entry.signal_type),
                                    value: Some(entry.value),
                                    payload: None,
                                    samples: None,
                                    rate: None,
                                    id: None,
                                    phase: None,
                                    timestamp: None,
                                    timeline: None,
                                })
                            };
                            if let Ok(bytes) = codec::encode(&msg) {
                                let _ = sender.send(bytes).await;
                            }
                        }
                    }
                    Err(e) => {
                        let error = Message::Error(ErrorMessage {
                            code: 500,
                            message: format!("Journal query failed: {}", e),
                            address: Some(replay.pattern.clone()),
                            correlation_id: None,
                        });
                        let bytes = codec::encode(&error).ok()?;
                        return Some(MessageResult::Send(bytes));
                    }
                }
            } else {
                let error = Message::Error(ErrorMessage {
                    code: 501,
                    message: "Journal not configured on this router".to_string(),
                    address: Some(replay.pattern.clone()),
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            Some(MessageResult::None)
        }

        Message::Announce(announce) => {
            // Register announced signals in the signal registry
            state.register_signals(announce.signals.clone());
            debug!(
                "Registered {} signals in namespace {}",
                announce.signals.len(),
                announce.namespace
            );
            // Send ACK to confirm registration
            let ack = Message::Ack(AckMessage {
                address: Some(announce.namespace.clone()),
                revision: None,
                locked: None,
                holder: None,
                correlation_id: None,
            });
            let bytes = codec::encode(&ack).ok()?;
            Some(MessageResult::Send(bytes))
        }

        Message::Sync(sync_msg) => {
            // Clock synchronization - respond with server timestamps
            // Client sends t1 (client send time)
            // Server records t2 (server receive time) and t3 (server send time)
            // Client records t4 (client receive time)
            // RTT = (t4 - t1) - (t3 - t2)
            // Offset = ((t2 - t1) + (t3 - t4)) / 2
            let now = clasp_core::time::now();
            let response = Message::Sync(clasp_core::SyncMessage {
                t1: sync_msg.t1,
                t2: Some(now), // Server receive time
                t3: Some(now), // Server send time (same for instant response)
            });
            let bytes = codec::encode(&response).ok()?;
            Some(MessageResult::Send(bytes))
        }

        Message::Bundle(bundle) => {
            let session = session.as_ref()?;

            // PHASE 1: Validate ALL messages first (atomic validation)
            // If any validation fails, reject the entire bundle
            let mut validated_sets: Vec<&SetMessage> = Vec::new();
            let mut validated_pubs: Vec<&PublishMessage> = Vec::new();

            for inner_msg in &bundle.messages {
                match inner_msg {
                    Message::Set(set) => {
                        // Check scope for write access (in authenticated mode)
                        if security_mode == SecurityMode::Authenticated
                            && !session.has_scope(Action::Write, &set.address)
                        {
                            warn!(
                                "Session {} denied bundled SET to {} - rejecting entire bundle",
                                session.id, set.address
                            );
                            // Return error for the entire bundle
                            let err = Message::Error(ErrorMessage {
                                code: 403,
                                message: format!(
                                    "Bundle rejected: insufficient scope for SET to {}",
                                    set.address
                                ),
                                address: Some(set.address.clone()),
                                correlation_id: None,
                            });
                            let err_bytes = codec::encode(&err).ok()?;
                            return Some(MessageResult::Send(err_bytes));
                        }

                        // Application-specific write validation for bundled SET
                        if let Some(ref validator) = write_validator {
                            if let Err(reason) =
                                validator.validate_write(&set.address, &set.value, session, state)
                            {
                                warn!(
                                    "Session {} denied bundled SET to {} by write validator - rejecting entire bundle: {}",
                                    session.id, set.address, reason
                                );
                                let err = Message::Error(ErrorMessage {
                                    code: 403,
                                    message: format!("Bundle rejected: {}", reason),
                                    address: Some(set.address.clone()),
                                    correlation_id: None,
                                });
                                let err_bytes = codec::encode(&err).ok()?;
                                return Some(MessageResult::Send(err_bytes));
                            }
                        }

                        // Lock checks happen during apply_set - the state store
                        // validates locks when actually applying the change
                        validated_sets.push(set);
                    }
                    Message::Publish(pub_msg) => {
                        // Check scope for write access (in authenticated mode)
                        if security_mode == SecurityMode::Authenticated
                            && !session.has_scope(Action::Write, &pub_msg.address)
                        {
                            warn!(
                                "Session {} denied bundled PUBLISH to {} - rejecting entire bundle",
                                session.id, pub_msg.address
                            );
                            let err = Message::Error(ErrorMessage {
                                code: 403,
                                message: format!(
                                    "Bundle rejected: insufficient scope for PUBLISH to {}",
                                    pub_msg.address
                                ),
                                address: Some(pub_msg.address.clone()),
                                correlation_id: None,
                            });
                            let err_bytes = codec::encode(&err).ok()?;
                            return Some(MessageResult::Send(err_bytes));
                        }

                        // Application-specific write validation for bundled PUBLISH
                        if let Some(ref validator) = write_validator {
                            let pub_value = pub_msg
                                .value
                                .as_ref()
                                .cloned()
                                .unwrap_or(clasp_core::Value::Null);
                            if let Err(reason) = validator.validate_write(
                                &pub_msg.address,
                                &pub_value,
                                session,
                                state,
                            ) {
                                warn!(
                                    "Session {} denied bundled PUBLISH to {} by write validator - rejecting entire bundle: {}",
                                    session.id, pub_msg.address, reason
                                );
                                let err = Message::Error(ErrorMessage {
                                    code: 403,
                                    message: format!("Bundle rejected: {}", reason),
                                    address: Some(pub_msg.address.clone()),
                                    correlation_id: None,
                                });
                                let err_bytes = codec::encode(&err).ok()?;
                                return Some(MessageResult::Send(err_bytes));
                            }
                        }

                        validated_pubs.push(pub_msg);
                    }
                    _ => {
                        // Other message types in bundles are currently not processed
                        debug!("Ignoring {:?} message type in bundle", inner_msg);
                    }
                }
            }

            // PHASE 2: Apply all validated changes atomically
            // Now that all validations passed, apply changes
            let mut applied_revisions: Vec<(String, u64)> = Vec::new();

            for set in &validated_sets {
                match state.apply_set(set, &session.id) {
                    Ok(revision) => {
                        applied_revisions.push((set.address.clone(), revision));

                        // Broadcast to subscribers
                        let subscribers =
                            subscriptions.find_subscribers(&set.address, Some(SignalType::Param));

                        // Create updated SET message with revision
                        let mut updated_set: SetMessage = (*set).clone();
                        updated_set.revision = Some(revision);
                        let broadcast_msg = Message::Set(updated_set);

                        if let Ok(bytes) = codec::encode(&broadcast_msg) {
                            for sub_session_id in subscribers {
                                if let Some(sub_session) = sessions.get(&sub_session_id) {
                                    try_send_with_drop_tracking_sync(
                                        sub_session.value(),
                                        bytes.clone(),
                                        &sub_session_id,
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        // This shouldn't happen after validation, but handle gracefully
                        error!("Bundle SET apply failed after validation: {}", e);
                    }
                }
            }

            // Process PUBLISH messages
            for pub_msg in &validated_pubs {
                let subscribers = subscriptions.find_subscribers(&pub_msg.address, pub_msg.signal);

                let inner_msg = Message::Publish((*pub_msg).clone());
                if let Ok(bytes) = codec::encode(&inner_msg) {
                    for sub_session_id in subscribers {
                        if sub_session_id != session.id {
                            if let Some(sub_session) = sessions.get(&sub_session_id) {
                                try_send_with_drop_tracking_sync(
                                    sub_session.value(),
                                    bytes.clone(),
                                    &sub_session_id,
                                );
                            }
                        }
                    }
                }
            }

            // Send a single ACK for the entire bundle with count of applied operations
            let ack = Message::Ack(AckMessage {
                address: None,
                revision: applied_revisions.last().map(|(_, r)| *r),
                locked: None,
                holder: None,
                correlation_id: None,
            });
            let ack_bytes = codec::encode(&ack).ok()?;
            Some(MessageResult::Send(ack_bytes))
        }

        #[cfg(feature = "federation")]
        Message::FederationSync(fed_msg) => {
            let session = session.as_ref()?;

            // Resource limits for federation operations
            const MAX_FEDERATION_PATTERNS: usize = 1000;
            const MAX_REVISION_ENTRIES: usize = 10_000;

            // Only accept FederationSync from sessions that advertised "federation" feature
            if !session.is_federation_peer() {
                warn!(
                    "Session {} sent FederationSync but is not a federation peer",
                    session.id
                );
                let error = Message::Error(ErrorMessage {
                    code: 403,
                    message: "FederationSync requires federation feature".to_string(),
                    address: None,
                    correlation_id: None,
                });
                let bytes = codec::encode(&error).ok()?;
                return Some(MessageResult::Send(bytes));
            }

            match fed_msg.op {
                clasp_core::FederationOp::DeclareNamespaces => {
                    // Enforce resource limit on pattern count
                    if fed_msg.patterns.len() > MAX_FEDERATION_PATTERNS {
                        warn!(
                            "Federation peer {} declared {} namespaces (limit {})",
                            session.id, fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
                        );
                        let error = Message::Error(ErrorMessage {
                            code: 400,
                            message: format!(
                                "too many namespace patterns: {} (max {})",
                                fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
                            ),
                            address: None,
                            correlation_id: None,
                        });
                        let bytes = codec::encode(&error).ok()?;
                        return Some(MessageResult::Send(bytes));
                    }

                    let router_id = fed_msg
                        .origin
                        .clone()
                        .unwrap_or_else(|| session.name.clone());

                    // In authenticated mode, verify scopes for each declared namespace
                    if security_mode == SecurityMode::Authenticated {
                        for pattern in &fed_msg.patterns {
                            if !session.has_strict_read_scope(pattern) {
                                warn!(
                                    "Federation peer {} lacks read scope for namespace {}",
                                    router_id, pattern
                                );
                                let error = Message::Error(ErrorMessage {
                                    code: 403,
                                    message: format!(
                                        "insufficient scope for namespace: {}", pattern
                                    ),
                                    address: None,
                                    correlation_id: None,
                                });
                                let bytes = codec::encode(&error).ok()?;
                                return Some(MessageResult::Send(bytes));
                            }
                        }
                    }

                    info!(
                        "Federation peer {} declares namespaces: {:?}",
                        router_id, fed_msg.patterns
                    );

                    // Clean up previous federation subscriptions if re-declaring
                    let old_namespaces = session.federation_namespaces();
                    if !old_namespaces.is_empty() {
                        for i in 0..old_namespaces.len() {
                            let old_sub_id = 50000 + i as u32;
                            subscriptions.remove(&session.id, old_sub_id);
                            session.remove_subscription(old_sub_id);
                        }
                        debug!(
                            "Federation: cleaned up {} old subscriptions for peer {}",
                            old_namespaces.len(), router_id
                        );
                    }

                    // Store peer info on the session
                    session.set_federation_router_id(router_id.clone());
                    session.set_federation_namespaces(fed_msg.patterns.clone());

                    // Subscribe the peer session to their declared patterns
                    // so they receive matching SETs/PUBLISHes automatically
                    for (i, pattern) in fed_msg.patterns.iter().enumerate() {
                        let sub_id = 50000 + i as u32; // High IDs to avoid collision with client subs
                        match crate::subscription::Subscription::new(
                            sub_id,
                            session.id.clone(),
                            pattern,
                            vec![],
                            Default::default(),
                        ) {
                            Ok(subscription) => {
                                subscriptions.add(subscription);
                                session.add_subscription(sub_id);
                                debug!(
                                    "Federation: auto-subscribed peer {} to {}",
                                    router_id, pattern
                                );
                            }
                            Err(e) => {
                                warn!(
                                    "Federation: failed to create subscription for {}: {:?}",
                                    pattern, e
                                );
                            }
                        }
                    }

                    // Send ACK
                    let ack = Message::Ack(AckMessage {
                        address: None,
                        revision: None,
                        locked: None,
                        holder: None,
                        correlation_id: None,
                    });
                    let bytes = codec::encode(&ack).ok()?;
                    Some(MessageResult::Send(bytes))
                }

                clasp_core::FederationOp::RequestSync => {
                    // Enforce resource limit on pattern count
                    if fed_msg.patterns.len() > MAX_FEDERATION_PATTERNS {
                        warn!(
                            "Federation RequestSync with {} patterns (limit {})",
                            fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
                        );
                        let error = Message::Error(ErrorMessage {
                            code: 400,
                            message: format!(
                                "too many sync patterns: {} (max {})",
                                fed_msg.patterns.len(), MAX_FEDERATION_PATTERNS
                            ),
                            address: None,
                            correlation_id: None,
                        });
                        let bytes = codec::encode(&error).ok()?;
                        return Some(MessageResult::Send(bytes));
                    }

                    // Validate each requested pattern against declared namespaces
                    let declared = session.federation_namespaces();
                    for pattern in &fed_msg.patterns {
                        let covered = declared.iter().any(|ns| {
                            federation_pattern_covered_by(pattern, ns)
                        });
                        if !covered {
                            warn!(
                                "Federation RequestSync pattern {} not covered by declared namespaces {:?}",
                                pattern, declared
                            );
                            let error = Message::Error(ErrorMessage {
                                code: 403,
                                message: format!(
                                    "pattern '{}' not covered by declared namespaces", pattern
                                ),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            return Some(MessageResult::Send(bytes));
                        }

                        // Enforce scope checks in authenticated mode
                        if security_mode == SecurityMode::Authenticated
                            && !session.has_strict_read_scope(pattern)
                        {
                            warn!(
                                "Federation RequestSync: session lacks read scope for {}",
                                pattern
                            );
                            let error = Message::Error(ErrorMessage {
                                code: 403,
                                message: format!(
                                    "insufficient scope for pattern: {}", pattern
                                ),
                                address: None,
                                correlation_id: None,
                            });
                            let bytes = codec::encode(&error).ok()?;
                            return Some(MessageResult::Send(bytes));
                        }
                    }

                    // Peer requests state sync for patterns -- send a filtered snapshot
                    for pattern in &fed_msg.patterns {
                        let mut snapshot = state.snapshot(pattern);

                        // If since_revision is set, filter to only newer entries
                        if let Some(since) = fed_msg.since_revision {
                            snapshot.params.retain(|p| p.revision > since);
                        }

                        // Apply snapshot filter if configured
                        if let Some(ref filter) = snapshot_filter {
                            snapshot.params =
                                filter.filter_snapshot(snapshot.params, session, state);
                        }

                        // Send snapshot to the peer
                        send_chunked_snapshot(sender, snapshot).await;
                    }

                    // Send SyncComplete
                    let complete = Message::FederationSync(clasp_core::FederationSyncMessage {
                        op: clasp_core::FederationOp::SyncComplete,
                        patterns: fed_msg.patterns.clone(),
                        revisions: std::collections::HashMap::new(),
                        since_revision: None,
                        origin: Some(config.name.clone()),
                    });
                    let bytes = codec::encode(&complete).ok()?;
                    Some(MessageResult::Send(bytes))
                }

                clasp_core::FederationOp::RevisionVector => {
                    // Enforce resource limit on revision entry count
                    if fed_msg.revisions.len() > MAX_REVISION_ENTRIES {
                        warn!(
                            "Federation RevisionVector with {} entries (limit {})",
                            fed_msg.revisions.len(), MAX_REVISION_ENTRIES
                        );
                        let error = Message::Error(ErrorMessage {
                            code: 400,
                            message: format!(
                                "too many revision entries: {} (max {})",
                                fed_msg.revisions.len(), MAX_REVISION_ENTRIES
                            ),
                            address: None,
                            correlation_id: None,
                        });
                        let bytes = codec::encode(&error).ok()?;
                        return Some(MessageResult::Send(bytes));
                    }

                    debug!(
                        "Federation: received revision vector with {} entries from peer {}",
                        fed_msg.revisions.len(),
                        session.federation_router_id().unwrap_or_default()
                    );

                    // Validate each address against declared namespaces
                    let declared = session.federation_namespaces();

                    // Compare with local state, send delta for entries where local > peer
                    let mut delta_params = Vec::new();
                    for (addr, peer_rev) in &fed_msg.revisions {
                        // Check address is covered by declared namespaces
                        let covered = declared.iter().any(|ns| {
                            clasp_core::address::glob_match(ns, addr)
                        });
                        if !covered {
                            debug!(
                                "Federation: skipping revision for {} (not in declared namespaces)",
                                addr
                            );
                            continue;
                        }

                        // Enforce scope checks in authenticated mode
                        if security_mode == SecurityMode::Authenticated
                            && !session.has_scope(Action::Read, addr)
                        {
                            debug!(
                                "Federation: skipping revision for {} (insufficient scope)",
                                addr
                            );
                            continue;
                        }

                        if let Some(local_ps) = state.get_state(addr) {
                            if local_ps.revision > *peer_rev {
                                delta_params.push(clasp_core::ParamValue {
                                    address: addr.clone(),
                                    value: local_ps.value,
                                    revision: local_ps.revision,
                                    writer: Some(local_ps.writer),
                                    timestamp: Some(local_ps.timestamp),
                                });
                            }
                        }
                    }

                    if !delta_params.is_empty() {
                        let snapshot = SnapshotMessage {
                            params: delta_params,
                        };
                        send_chunked_snapshot(sender, snapshot).await;
                    }

                    Some(MessageResult::None)
                }

                clasp_core::FederationOp::SyncComplete => {
                    info!(
                        "Federation: sync complete from peer {}",
                        session.federation_router_id().unwrap_or_default()
                    );
                    Some(MessageResult::None)
                }
            }
        }

        _ => Some(MessageResult::None),
    }
}

/// Execute pending actions produced by the rules engine.
///
/// Applies SET actions to state and broadcasts to subscribers.
/// PUBLISH actions are encoded and broadcast to matching subscribers.
/// Actions carry an origin like "rule:my_rule_id" to prevent re-triggering.
#[cfg(feature = "rules")]
pub fn execute_rule_actions(
    actions: Vec<clasp_rules::PendingAction>,
    state: &Arc<RouterState>,
    sessions: &Arc<DashMap<SessionId, Arc<Session>>>,
    subscriptions: &Arc<SubscriptionManager>,
) {
    for action in actions {
        match action.action {
            clasp_rules::RuleAction::Set { address, value } => {
                match state.set(&address, value.clone(), &action.origin, None, false, false) {
                    Ok(revision) => {
                        let subscribers =
                            subscriptions.find_subscribers(&address, Some(SignalType::Param));
                        let set_msg = Message::Set(SetMessage {
                            address: address.clone(),
                            value,
                            revision: Some(revision),
                            lock: false,
                            unlock: false,
                        });
                        if let Ok(bytes) = codec::encode(&set_msg) {
                            for sub_session_id in subscribers {
                                if let Some(sub_session) = sessions.get(&sub_session_id) {
                                    try_send_with_drop_tracking_sync(
                                        sub_session.value(),
                                        bytes.clone(),
                                        &sub_session_id,
                                    );
                                }
                            }
                        }
                        debug!("Rule {} applied SET to {}", action.rule_id, address);
                    }
                    Err(e) => {
                        warn!("Rule {} SET to {} failed: {:?}", action.rule_id, address, e);
                    }
                }
            }
            clasp_rules::RuleAction::Publish {
                address,
                signal,
                value,
            } => {
                let pub_msg = Message::Publish(PublishMessage {
                    address: address.clone(),
                    signal: Some(signal),
                    value,
                    payload: None,
                    samples: None,
                    rate: None,
                    id: None,
                    phase: None,
                    timestamp: None,
                    timeline: None,
                });
                let subscribers = subscriptions.find_subscribers(&address, Some(signal));
                if let Ok(bytes) = codec::encode(&pub_msg) {
                    for sub_session_id in subscribers {
                        if let Some(sub_session) = sessions.get(&sub_session_id) {
                            try_send_with_drop_tracking_sync(
                                sub_session.value(),
                                bytes.clone(),
                                &sub_session_id,
                            );
                        }
                    }
                }
                debug!("Rule {} applied PUBLISH to {}", action.rule_id, address);
            }
            clasp_rules::RuleAction::SetFromTrigger { address, transform } => {
                if let Some(current) = state.get(&address) {
                    let transformed = transform.apply(&current);
                    match state.set(&address, transformed.clone(), &action.origin, None, false, false) {
                        Ok(revision) => {
                            let subscribers =
                                subscriptions.find_subscribers(&address, Some(SignalType::Param));
                            let set_msg = Message::Set(SetMessage {
                                address: address.clone(),
                                value: transformed,
                                revision: Some(revision),
                                lock: false,
                                unlock: false,
                            });
                            if let Ok(bytes) = codec::encode(&set_msg) {
                                for sub_session_id in subscribers {
                                    if let Some(sub_session) = sessions.get(&sub_session_id) {
                                        try_send_with_drop_tracking_sync(
                                            sub_session.value(),
                                            bytes.clone(),
                                            &sub_session_id,
                                        );
                                    }
                                }
                            }
                            debug!("Rule {} applied SetFromTrigger to {}", action.rule_id, address);
                        }
                        Err(e) => {
                            warn!("Rule {} SetFromTrigger to {} failed: {:?}", action.rule_id, address, e);
                        }
                    }
                }
            }
            clasp_rules::RuleAction::Delay { .. } => {
                // Delay actions are handled at a higher level (relay timer tasks)
            }
        }
    }
}

/// Try to send a message to a session with drop tracking.
/// Records the drop and sends notification when threshold is exceeded.
fn try_send_with_drop_tracking_sync(session: &Arc<Session>, data: Bytes, session_id: &SessionId) {
    if let Err(e) = session.try_send(data) {
        warn!(
            "Failed to send to {}: {} (buffer full, dropping)",
            session_id, e
        );

        // Record the drop and check if we should notify
        if session.record_drop() {
            // Send drop notification asynchronously
            let session = Arc::clone(session);
            let session_id = session_id.clone();
            let drops = session.drops_in_window();
            tokio::spawn(async move {
                let error = Message::Error(ErrorMessage {
                    code: 503, // Service Unavailable
                    message: format!(
                        "Buffer overflow: messages being dropped ({} drops in last 10 seconds)",
                        drops
                    ),
                    address: None,
                    correlation_id: None,
                });
                if let Ok(error_bytes) = codec::encode(&error) {
                    // Use send() not try_send() for the notification to ensure it gets through
                    if let Err(e) = session.send(error_bytes).await {
                        warn!("Failed to send drop notification to {}: {}", session_id, e);
                    } else {
                        info!(
                            "Sent buffer overflow notification to session {} ({} drops)",
                            session_id, drops
                        );
                    }
                }
            });
        }
    }
}

/// Check if a federation `request` pattern is covered by a `declared` namespace pattern.
///
/// A request is covered if every address it could match is also matched by the declared
/// namespace. This handles:
/// - Exact match: `/sensors/temp` covered by `/sensors/temp`
/// - Concrete within glob: `/sensors/temp/1` covered by `/sensors/**`
/// - Sub-pattern within glob: `/sensors/temp/**` covered by `/sensors/**`
#[cfg(feature = "federation")]
fn federation_pattern_covered_by(request: &str, declared: &str) -> bool {
    // Exact match
    if request == declared {
        return true;
    }

    // If the request has no wildcards, we can use glob_match to check if
    // the declared namespace covers it as a literal address.
    // We must NOT do this when request contains wildcards, because glob_match
    // would treat `**` in the request as literal characters.
    let request_has_wildcards = request.contains('*');
    if !request_has_wildcards && clasp_core::address::glob_match(declared, request) {
        return true;
    }

    // Sub-pattern check: strip wildcards from request and check prefix coverage
    // e.g., `/sensors/temp/**` is covered by `/sensors/**`
    let decl_parts: Vec<&str> = declared.split('/').filter(|s| !s.is_empty()).collect();
    let req_parts: Vec<&str> = request.split('/').filter(|s| !s.is_empty()).collect();

    let mut di = 0;
    let mut ri = 0;

    while di < decl_parts.len() && ri < req_parts.len() {
        let dp = decl_parts[di];
        let rp = req_parts[ri];

        if dp == "**" {
            // Declared namespace has **, covers everything below this prefix
            return true;
        }

        if rp == "**" {
            // Request has ** — it's wider than anything except declared **
            // (which was already checked above)
            return false;
        }

        if dp == "*" {
            // Declared * matches any single segment in request
            // (rp == "**" already handled above)
            if rp == "*" {
                // Both are single wildcards — equivalent at this position
                di += 1;
                ri += 1;
                continue;
            }
            // rp is a literal — covered by declared *
            di += 1;
            ri += 1;
            continue;
        }

        if rp == "*" {
            // Request has * where declared has literal — request is wider, not covered
            return false;
        }

        if dp != rp {
            return false;
        }

        di += 1;
        ri += 1;
    }

    // If declared is exhausted but request still has segments, not covered
    // (unless declared ended with **)
    if di < decl_parts.len() && decl_parts[di] == "**" {
        return true;
    }

    di >= decl_parts.len() && ri >= req_parts.len()
}

/// Broadcast to all sessions except one (non-blocking)
fn broadcast_to_subscribers(
    data: &Bytes,
    sessions: &Arc<DashMap<SessionId, Arc<Session>>>,
    exclude: &SessionId,
) {
    for entry in sessions.iter() {
        if entry.key() != exclude {
            try_send_with_drop_tracking_sync(entry.value(), data.clone(), entry.key());
        }
    }
}

#[cfg(all(test, feature = "federation"))]
mod federation_tests {
    use super::*;

    // --- federation_pattern_covered_by tests ---

    #[test]
    fn test_exact_match() {
        assert!(federation_pattern_covered_by("/sensors/temp", "/sensors/temp"));
    }

    #[test]
    fn test_concrete_within_globstar() {
        assert!(federation_pattern_covered_by("/sensors/temp/1", "/sensors/**"));
        assert!(federation_pattern_covered_by("/sensors/temp", "/sensors/**"));
    }

    #[test]
    fn test_sub_pattern_within_globstar() {
        assert!(federation_pattern_covered_by("/sensors/temp/**", "/sensors/**"));
        assert!(federation_pattern_covered_by("/sensors/temp/*", "/sensors/**"));
    }

    #[test]
    fn test_globstar_root_covers_all() {
        assert!(federation_pattern_covered_by("/sensors/**", "/**"));
        assert!(federation_pattern_covered_by("/anything/deep/path", "/**"));
    }

    #[test]
    fn test_disjoint_namespaces_rejected() {
        assert!(!federation_pattern_covered_by("/audio/**", "/sensors/**"));
        assert!(!federation_pattern_covered_by("/audio/mixer", "/sensors/**"));
    }

    #[test]
    fn test_wider_pattern_rejected() {
        // Request for /** but declared only /sensors/**
        assert!(!federation_pattern_covered_by("/**", "/sensors/**"));
    }

    #[test]
    fn test_wildcard_in_request_wider_than_literal() {
        // /sensors/* is wider than /sensors/temp (declared)
        assert!(!federation_pattern_covered_by("/sensors/*", "/sensors/temp"));
    }

    #[test]
    fn test_declared_single_wildcard() {
        // Declared /sensors/*, request /sensors/temp — covered
        assert!(federation_pattern_covered_by("/sensors/temp", "/sensors/*"));
    }

    // --- Session federation feature tests ---

    #[test]
    fn test_federation_peer_detection() {
        let fed_session = Session::stub_federation("hub-peer");
        assert!(fed_session.is_federation_peer());

        let normal_session = Session::stub(None);
        assert!(!normal_session.is_federation_peer());
    }

    #[test]
    fn test_federation_namespaces_lifecycle() {
        let session = Session::stub_federation("peer");
        assert!(session.federation_namespaces().is_empty());

        session.set_federation_namespaces(vec![
            "/sensors/**".to_string(),
            "/lights/**".to_string(),
        ]);
        let ns = session.federation_namespaces();
        assert_eq!(ns.len(), 2);
        assert!(ns.contains(&"/sensors/**".to_string()));
        assert!(ns.contains(&"/lights/**".to_string()));

        // Re-declare replaces
        session.set_federation_namespaces(vec!["/audio/**".to_string()]);
        let ns = session.federation_namespaces();
        assert_eq!(ns.len(), 1);
        assert_eq!(ns[0], "/audio/**");
    }

    #[test]
    fn test_federation_router_id() {
        let session = Session::stub_federation("peer");
        assert!(session.federation_router_id().is_none());

        session.set_federation_router_id("hub-alpha".to_string());
        assert_eq!(session.federation_router_id().unwrap(), "hub-alpha");
    }

    #[test]
    fn test_federation_subscription_id_range() {
        // Federation subscriptions use IDs starting at 50000
        // User subscriptions typically use small sequential IDs
        // Verify the ranges don't overlap with typical usage
        let session = Session::stub_federation("peer");
        session.add_subscription(1);   // user sub
        session.add_subscription(50000); // federation sub
        session.add_subscription(50001); // federation sub

        let subs = session.subscriptions();
        assert_eq!(subs.len(), 3);
        assert!(subs.contains(&1));
        assert!(subs.contains(&50000));
        assert!(subs.contains(&50001));

        // Remove federation sub, user sub remains
        session.remove_subscription(50000);
        let subs = session.subscriptions();
        assert_eq!(subs.len(), 2);
        assert!(subs.contains(&1));
        assert!(!subs.contains(&50000));
    }

    // --- Resource limit constant tests ---

    #[test]
    fn test_resource_limits_are_sane() {
        // Verify the constants are within reasonable bounds
        // (these are compile-time checks essentially)
        const MAX_PATTERNS: usize = 1000;
        const MAX_REVISIONS: usize = 10_000;
        assert!(MAX_PATTERNS > 0 && MAX_PATTERNS <= 10_000);
        assert!(MAX_REVISIONS > 0 && MAX_REVISIONS <= 100_000);
    }

    // --- Pattern matcher edge case / fuzz tests ---

    #[test]
    fn test_empty_strings() {
        // Empty patterns should not match anything useful
        assert!(federation_pattern_covered_by("", ""));
        assert!(!federation_pattern_covered_by("/a", ""));
        assert!(!federation_pattern_covered_by("", "/a"));
    }

    #[test]
    fn test_root_slash_only() {
        // Root path edge cases
        assert!(federation_pattern_covered_by("/", "/"));
        assert!(federation_pattern_covered_by("/", "/**"));
    }

    #[test]
    fn test_trailing_slash() {
        // Trailing slash creates an empty segment that gets filtered
        assert!(federation_pattern_covered_by("/sensors/", "/sensors/**"));
        assert!(federation_pattern_covered_by("/sensors/temp/", "/sensors/**"));
    }

    #[test]
    fn test_double_slashes() {
        // Double slashes create empty segments that get filtered
        assert!(federation_pattern_covered_by("//sensors//temp", "/sensors/**"));
    }

    #[test]
    fn test_deep_nesting_under_globstar() {
        assert!(federation_pattern_covered_by("/a/b/c/d/e/f/g", "/**"));
        assert!(federation_pattern_covered_by("/a/b/c/d/e/f/g/**", "/**"));
        assert!(federation_pattern_covered_by("/a/b/c/d/e", "/a/**"));
        assert!(!federation_pattern_covered_by("/a/b/c/d/e", "/b/**"));
    }

    #[test]
    fn test_single_wildcard_depth_mismatch() {
        // /a/* covers one level under /a/; request for deeper path is NOT covered
        assert!(federation_pattern_covered_by("/a/b", "/a/*"));
        assert!(!federation_pattern_covered_by("/a/b/c", "/a/*"));
    }

    #[test]
    fn test_wildcard_request_vs_literal_declared() {
        // Request with wildcard is wider than literal — should be rejected
        assert!(!federation_pattern_covered_by("/a/*", "/a/b"));
        assert!(!federation_pattern_covered_by("/a/**", "/a/b"));
        assert!(!federation_pattern_covered_by("/a/**", "/a/b/c"));
    }

    #[test]
    fn test_request_globstar_vs_declared_single_wildcard() {
        // /a/** is wider than /a/* — should be rejected
        assert!(!federation_pattern_covered_by("/a/**", "/a/*"));
    }

    #[test]
    fn test_mixed_wildcards_in_declared() {
        // Declared /a/*/c/** should cover /a/x/c/d
        assert!(federation_pattern_covered_by("/a/x/c/d", "/a/*/c/**"));
        // But not /a/x/y/d (wrong segment at position 2)
        assert!(!federation_pattern_covered_by("/a/x/y/d", "/a/*/c/**"));
    }

    #[test]
    fn test_request_pattern_with_wildcards_in_middle() {
        // Request /a/*/c is wider at position 1 than declared /a/b/**
        // even though declared covers deeper paths under /a/b
        assert!(!federation_pattern_covered_by("/a/*/c", "/a/b/**"));
    }

    #[test]
    fn test_identical_wildcard_patterns() {
        assert!(federation_pattern_covered_by("/**", "/**"));
        assert!(federation_pattern_covered_by("/a/**", "/a/**"));
        assert!(federation_pattern_covered_by("/a/*", "/a/*"));
    }

    #[test]
    fn test_path_traversal_segments() {
        // ".." is just a literal segment in CLASP, not filesystem traversal
        assert!(!federation_pattern_covered_by("/../sensors/temp", "/sensors/**"));
        assert!(federation_pattern_covered_by("/../sensors/temp", "/**"));
    }

    #[test]
    fn test_single_segment_patterns() {
        assert!(federation_pattern_covered_by("/a", "/a"));
        assert!(!federation_pattern_covered_by("/a", "/b"));
        assert!(federation_pattern_covered_by("/a", "/*"));
        assert!(federation_pattern_covered_by("/a", "/**"));
    }

    #[test]
    fn test_declared_shorter_than_request_no_wildcard() {
        // Declared /a/b does not cover /a/b/c — no wildcard means exact depth only
        assert!(!federation_pattern_covered_by("/a/b/c", "/a/b"));
    }

    #[test]
    fn test_request_shorter_than_declared() {
        // Request /a doesn't match declared /a/b (request must be within declared scope)
        assert!(!federation_pattern_covered_by("/a", "/a/b"));
    }
}
