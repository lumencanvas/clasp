//! P2P WebRTC types and signaling primitives
//!
//! This module provides core types for WebRTC peer-to-peer connections:
//! - Signaling messages (offers, answers, ICE candidates)
//! - P2P configuration
//! - Reserved namespaces for P2P signaling

use serde::{Deserialize, Serialize};

/// Reserved P2P namespace prefix
pub const P2P_NAMESPACE: &str = "/clasp/p2p";

/// Address for P2P signaling to a specific session
/// Format: /clasp/p2p/signal/{target_session_id}
pub const P2P_SIGNAL_PREFIX: &str = "/clasp/p2p/signal/";

/// Address for P2P capability announcements (broadcast)
pub const P2P_ANNOUNCE: &str = "/clasp/p2p/announce";

/// Default connection timeout in seconds
pub const DEFAULT_CONNECTION_TIMEOUT_SECS: u64 = 30;

/// Default maximum connection retries
pub const DEFAULT_MAX_RETRIES: u32 = 3;

/// P2P signaling message types
///
/// These messages are sent via PUBLISH to `/clasp/p2p/signal/{target_session_id}`
/// and relayed by the router to the target peer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum P2PSignal {
    /// SDP offer to initiate connection
    Offer {
        /// Session ID of the sender
        from: String,
        /// SDP offer string
        sdp: String,
        /// Correlation ID for matching offer/answer
        correlation_id: String,
    },
    /// SDP answer in response to offer
    Answer {
        /// Session ID of the sender
        from: String,
        /// SDP answer string
        sdp: String,
        /// Correlation ID matching the offer
        correlation_id: String,
    },
    /// ICE candidate for NAT traversal
    IceCandidate {
        /// Session ID of the sender
        from: String,
        /// ICE candidate JSON string
        candidate: String,
        /// Correlation ID for the connection
        correlation_id: String,
    },
    /// P2P connection established notification
    Connected {
        /// Session ID of the sender
        from: String,
        /// Correlation ID for the connection
        correlation_id: String,
    },
    /// P2P connection closed notification
    Disconnected {
        /// Session ID of the sender
        from: String,
        /// Correlation ID for the connection
        correlation_id: String,
        /// Reason for disconnection
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<String>,
    },
}

impl P2PSignal {
    /// Get the sender's session ID
    pub fn from_session(&self) -> &str {
        match self {
            P2PSignal::Offer { from, .. } => from,
            P2PSignal::Answer { from, .. } => from,
            P2PSignal::IceCandidate { from, .. } => from,
            P2PSignal::Connected { from, .. } => from,
            P2PSignal::Disconnected { from, .. } => from,
        }
    }

    /// Get the correlation ID
    pub fn correlation_id(&self) -> &str {
        match self {
            P2PSignal::Offer { correlation_id, .. } => correlation_id,
            P2PSignal::Answer { correlation_id, .. } => correlation_id,
            P2PSignal::IceCandidate { correlation_id, .. } => correlation_id,
            P2PSignal::Connected { correlation_id, .. } => correlation_id,
            P2PSignal::Disconnected { correlation_id, .. } => correlation_id,
        }
    }
}

/// P2P announce message for capability advertisement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct P2PAnnounce {
    /// Session ID of the announcing peer
    pub session_id: String,
    /// Whether this peer supports P2P connections
    pub p2p_capable: bool,
    /// Supported features (e.g., "webrtc", "reliable", "unreliable")
    #[serde(default)]
    pub features: Vec<String>,
}

/// P2P connection configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    /// ICE servers for NAT traversal (STUN/TURN URLs)
    pub ice_servers: Vec<String>,
    /// Optional TURN servers for symmetric NAT traversal
    pub turn_servers: Vec<TurnServer>,
    /// Connection timeout duration
    pub connection_timeout_secs: u64,
    /// Maximum retry attempts for failed connections
    pub max_retries: u32,
    /// Whether to automatically fall back to server relay on P2P failure
    pub auto_fallback: bool,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            ice_servers: vec![
                "stun:stun.l.google.com:19302".to_string(),
                "stun:stun1.l.google.com:19302".to_string(),
            ],
            turn_servers: Vec::new(),
            connection_timeout_secs: DEFAULT_CONNECTION_TIMEOUT_SECS,
            max_retries: DEFAULT_MAX_RETRIES,
            auto_fallback: true,
        }
    }
}

/// TURN server configuration
#[derive(Debug, Clone)]
pub struct TurnServer {
    /// TURN server URL
    pub url: String,
    /// Username for authentication
    pub username: String,
    /// Credential for authentication
    pub credential: String,
}

/// P2P connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P2PConnectionState {
    /// Initial state, not connected
    Disconnected,
    /// Signaling in progress
    Connecting,
    /// ICE candidates being exchanged
    GatheringCandidates,
    /// WebRTC connection established
    Connected,
    /// Connection failed
    Failed,
    /// Connection closed
    Closed,
}

/// Routing mode for message delivery
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RoutingMode {
    /// Only use server relay (no P2P)
    ServerOnly,
    /// Only use P2P (fail if unavailable)
    P2POnly,
    /// Prefer P2P, fall back to server
    #[default]
    PreferP2P,
}

/// Check if an address is in the P2P namespace
pub fn is_p2p_address(address: &str) -> bool {
    address.starts_with(P2P_NAMESPACE)
}

/// Check if an address is a P2P signal address
pub fn is_p2p_signal_address(address: &str) -> bool {
    address.starts_with(P2P_SIGNAL_PREFIX)
}

/// Extract the target session ID from a P2P signal address
///
/// Returns None if the address is not a valid P2P signal address
pub fn extract_target_session(address: &str) -> Option<&str> {
    if address.starts_with(P2P_SIGNAL_PREFIX) {
        let target = &address[P2P_SIGNAL_PREFIX.len()..];
        if !target.is_empty() && !target.contains('/') {
            return Some(target);
        }
    }
    None
}

/// Create a P2P signal address for a target session
pub fn signal_address(target_session_id: &str) -> String {
    format!("{}{}", P2P_SIGNAL_PREFIX, target_session_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_signal_serialization() {
        let offer = P2PSignal::Offer {
            from: "session-123".to_string(),
            sdp: "v=0\r\n...".to_string(),
            correlation_id: "conn-456".to_string(),
        };

        let json = serde_json::to_string(&offer).unwrap();
        assert!(json.contains("\"type\":\"offer\""));
        assert!(json.contains("\"from\":\"session-123\""));

        let parsed: P2PSignal = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, offer);
    }

    #[test]
    fn test_is_p2p_address() {
        assert!(is_p2p_address("/clasp/p2p/signal/abc"));
        assert!(is_p2p_address("/clasp/p2p/announce"));
        assert!(!is_p2p_address("/lumen/scene/0/opacity"));
        assert!(!is_p2p_address("/clasp/other"));
    }

    #[test]
    fn test_extract_target_session() {
        assert_eq!(
            extract_target_session("/clasp/p2p/signal/session-123"),
            Some("session-123")
        );
        assert_eq!(extract_target_session("/clasp/p2p/signal/"), None);
        assert_eq!(extract_target_session("/clasp/p2p/signal/a/b"), None);
        assert_eq!(extract_target_session("/other/path"), None);
    }

    #[test]
    fn test_signal_address() {
        assert_eq!(
            signal_address("session-123"),
            "/clasp/p2p/signal/session-123"
        );
    }

    #[test]
    fn test_p2p_announce_serialization() {
        let announce = P2PAnnounce {
            session_id: "session-123".to_string(),
            p2p_capable: true,
            features: vec!["webrtc".to_string(), "reliable".to_string()],
        };

        let json = serde_json::to_string(&announce).unwrap();
        let parsed: P2PAnnounce = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, announce);
    }
}
