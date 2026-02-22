//! Federation configuration types

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Federation operating mode
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum FederationMode {
    /// Hub mode: accepts leaf connections, central point of star
    #[default]
    Hub,
    /// Leaf mode: connects to a single hub router
    Leaf {
        /// Hub endpoint URL (e.g., "wss://hub.example.com:7330")
        hub_endpoint: String,
    },
    /// Mesh mode: connects to multiple peer routers
    Mesh {
        /// Peer endpoint URLs
        peers: Vec<String>,
    },
}

/// Configuration for a federation link
#[derive(Debug, Clone)]
pub struct FederationConfig {
    /// Operating mode
    pub mode: FederationMode,
    /// Router identity name (used in origin field for loop prevention)
    pub router_id: String,
    /// Namespace patterns this router owns (e.g., "/site-a/**")
    pub owned_namespaces: Vec<String>,
    /// Token for authenticating to peers (if required)
    pub auth_token: Option<String>,
    /// Reconnect on disconnect
    pub auto_reconnect: bool,
    /// Reconnect delay
    pub reconnect_delay: Duration,
    /// Maximum reconnect attempts (0 = unlimited)
    pub max_reconnect_attempts: u32,
    /// How often to exchange revision vectors for sync verification
    pub sync_interval: Duration,
    /// Client name to advertise in HELLO
    pub client_name: String,
    /// Features to advertise in HELLO
    pub features: Vec<String>,
}

impl Default for FederationConfig {
    fn default() -> Self {
        Self {
            mode: FederationMode::Hub,
            router_id: uuid::Uuid::new_v4().to_string(),
            owned_namespaces: vec!["/**".to_string()],
            auth_token: None,
            auto_reconnect: true,
            reconnect_delay: Duration::from_secs(5),
            max_reconnect_attempts: 0,
            sync_interval: Duration::from_secs(30),
            client_name: "clasp-federation".to_string(),
            features: vec![
                "param".to_string(),
                "event".to_string(),
                "stream".to_string(),
                "federation".to_string(),
            ],
        }
    }
}

/// Information about a connected peer router
#[derive(Debug, Clone)]
pub struct PeerInfo {
    /// Peer's router ID
    pub router_id: String,
    /// Peer's session ID on the local router (if the peer is a virtual session)
    pub session_id: Option<String>,
    /// Namespace patterns the peer owns
    pub namespaces: Vec<String>,
    /// Endpoint URL (for reconnection)
    pub endpoint: Option<String>,
    /// Whether we initiated the connection (outbound) or they connected to us (inbound)
    pub outbound: bool,
    /// Connection state
    pub state: PeerState,
}

/// Peer connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PeerState {
    /// Connecting to peer
    Connecting,
    /// Connected, performing handshake
    Handshaking,
    /// Performing initial state sync
    Syncing,
    /// Fully operational
    Active,
    /// Disconnected, will reconnect
    Disconnected,
    /// Permanently failed
    Failed,
}
