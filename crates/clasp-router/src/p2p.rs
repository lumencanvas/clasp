//! P2P signaling support for the router
//!
//! This module handles P2P signal routing between clients:
//! - Detects P2P signaling addresses
//! - Routes signals to target sessions
//! - Tracks P2P-capable sessions

use clasp_core::{extract_target_session, is_p2p_address, P2P_ANNOUNCE};
use dashmap::DashSet;

/// Tracks P2P capabilities of connected sessions
#[derive(Debug, Default)]
pub struct P2PCapabilities {
    /// Set of session IDs that support P2P
    p2p_capable: DashSet<String>,
}

impl P2PCapabilities {
    /// Create a new P2P capabilities tracker
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a session as P2P capable
    pub fn register(&self, session_id: &str) {
        self.p2p_capable.insert(session_id.to_string());
    }

    /// Unregister a session's P2P capability (on disconnect)
    pub fn unregister(&self, session_id: &str) {
        self.p2p_capable.remove(session_id);
    }

    /// Check if a session is P2P capable
    pub fn is_capable(&self, session_id: &str) -> bool {
        self.p2p_capable.contains(session_id)
    }

    /// Get all P2P capable session IDs
    pub fn all_capable(&self) -> Vec<String> {
        self.p2p_capable.iter().map(|s| s.clone()).collect()
    }

    /// Get count of P2P capable sessions
    pub fn count(&self) -> usize {
        self.p2p_capable.len()
    }
}

/// Result of analyzing a P2P address
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum P2PAddressType {
    /// Not a P2P address
    NotP2P,
    /// P2P signal to a specific session
    Signal { target_session: String },
    /// P2P capability announcement (broadcast)
    Announce,
}

/// Analyze a PUBLISH address to determine P2P routing
pub fn analyze_address(address: &str) -> P2PAddressType {
    if !is_p2p_address(address) {
        return P2PAddressType::NotP2P;
    }

    if address == P2P_ANNOUNCE {
        return P2PAddressType::Announce;
    }

    if let Some(target) = extract_target_session(address) {
        return P2PAddressType::Signal {
            target_session: target.to_string(),
        };
    }

    P2PAddressType::NotP2P
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_p2p_capabilities() {
        let caps = P2PCapabilities::new();

        assert!(!caps.is_capable("session-1"));
        assert_eq!(caps.count(), 0);

        caps.register("session-1");
        caps.register("session-2");

        assert!(caps.is_capable("session-1"));
        assert!(caps.is_capable("session-2"));
        assert!(!caps.is_capable("session-3"));
        assert_eq!(caps.count(), 2);

        caps.unregister("session-1");
        assert!(!caps.is_capable("session-1"));
        assert_eq!(caps.count(), 1);
    }

    #[test]
    fn test_analyze_address() {
        assert_eq!(
            analyze_address("/lumen/scene/0/opacity"),
            P2PAddressType::NotP2P
        );

        assert_eq!(
            analyze_address("/clasp/p2p/announce"),
            P2PAddressType::Announce
        );

        assert_eq!(
            analyze_address("/clasp/p2p/signal/session-123"),
            P2PAddressType::Signal {
                target_session: "session-123".to_string()
            }
        );

        // Invalid P2P addresses
        assert_eq!(analyze_address("/clasp/p2p/other"), P2PAddressType::NotP2P);
        assert_eq!(
            analyze_address("/clasp/p2p/signal/"),
            P2PAddressType::NotP2P
        );
    }
}
