//! Federation manager -- orchestrates federation links
//!
//! The FederationManager owns all outbound federation links and
//! coordinates namespace management, message forwarding, and
//! reconnection logic.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{info, warn};

use crate::config::{FederationConfig, FederationMode, PeerInfo, PeerState};
use crate::link::{FederationLink, LinkEvent};
use crate::namespace::NamespaceManager;

/// Federation manager that coordinates all peer connections.
///
/// The manager maintains a set of federation links and a namespace
/// registry, routing messages to the appropriate peers based on
/// address patterns.
pub struct FederationManager {
    /// Federation configuration
    config: FederationConfig,
    /// Namespace manager (shared with links)
    namespaces: Arc<RwLock<NamespaceManager>>,
    /// Active peer connections (router_id -> peer info)
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Channel for receiving events from links
    event_rx: Option<mpsc::Receiver<LinkEvent>>,
    /// Event sender (cloned to each link)
    event_tx: mpsc::Sender<LinkEvent>,
}

impl FederationManager {
    /// Create a new federation manager
    pub fn new(config: FederationConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(1024);
        let namespaces = NamespaceManager::new(config.owned_namespaces.clone());

        Self {
            config,
            namespaces: Arc::new(RwLock::new(namespaces)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            event_rx: Some(event_rx),
            event_tx,
        }
    }

    /// Get the event sender for creating new links
    pub fn event_sender(&self) -> mpsc::Sender<LinkEvent> {
        self.event_tx.clone()
    }

    /// Take the event receiver (can only be called once).
    ///
    /// The caller should process events in a loop to handle
    /// messages from federation peers.
    pub fn take_event_receiver(&mut self) -> Option<mpsc::Receiver<LinkEvent>> {
        self.event_rx.take()
    }

    /// Create a federation link for an established transport connection.
    ///
    /// The link will perform the CLASP handshake, exchange federation metadata,
    /// and begin relaying messages. Call `link.run(receiver)` to start it.
    pub fn create_link(
        &self,
        sender: Arc<dyn clasp_transport::TransportSender>,
    ) -> FederationLink {
        FederationLink::new(self.config.clone(), sender, self.event_tx.clone())
    }

    /// Process a link event, updating internal state.
    ///
    /// Call this for each event received from `take_event_receiver()`.
    /// Returns the event for further processing by the router.
    pub async fn process_event(&self, event: &LinkEvent) {
        match event {
            LinkEvent::PeerNamespaces {
                router_id,
                patterns,
            } => {
                info!(
                    "Registering peer {} namespaces: {:?}",
                    router_id, patterns
                );
                self.namespaces
                    .write()
                    .await
                    .register_peer(router_id, patterns.clone());

                self.peers.write().await.entry(router_id.clone()).and_modify(|p| {
                    p.namespaces = patterns.clone();
                    p.state = PeerState::Syncing;
                }).or_insert_with(|| PeerInfo {
                    router_id: router_id.clone(),
                    session_id: None,
                    namespaces: patterns.clone(),
                    endpoint: None,
                    outbound: true,
                    state: PeerState::Syncing,
                });

                // Check for conflicts
                let conflicts = self.namespaces.read().await.find_conflicts();
                for (pattern, a, b) in &conflicts {
                    warn!(
                        "Namespace conflict detected: {} between {} and {}",
                        pattern, a, b
                    );
                }
            }

            LinkEvent::Connected { router_id } => {
                info!("Federation peer connected: {}", router_id);
                if let Some(peer) = self.peers.write().await.get_mut(router_id) {
                    peer.state = PeerState::Active;
                }
            }

            LinkEvent::Disconnected { router_id, reason } => {
                info!(
                    "Federation peer disconnected: {} (reason: {:?})",
                    router_id, reason
                );
                self.namespaces.write().await.remove_peer(router_id);
                if let Some(peer) = self.peers.write().await.get_mut(router_id) {
                    peer.state = PeerState::Disconnected;
                }
            }

            LinkEvent::SyncComplete {
                router_id,
                pattern,
                revision,
            } => {
                info!(
                    "Sync complete with peer {} for {} at rev {}",
                    router_id, pattern, revision
                );
            }

            // RemoteSet and RemotePublish are handled by the router, not here
            _ => {}
        }
    }

    /// Check if an address should be forwarded to federation peers
    pub async fn should_forward(&self, address: &str, origin: Option<&str>) -> bool {
        let ns = self.namespaces.read().await;
        !ns.peers_for_address(address, origin).is_empty()
    }

    /// Get peers that should receive a message for the given address
    pub async fn peers_for_address(
        &self,
        address: &str,
        exclude_origin: Option<&str>,
    ) -> Vec<String> {
        self.namespaces
            .read()
            .await
            .peers_for_address(address, exclude_origin)
    }

    /// Get information about a peer
    pub async fn peer_info(&self, router_id: &str) -> Option<PeerInfo> {
        self.peers.read().await.get(router_id).cloned()
    }

    /// Get all active peers
    pub async fn active_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .read()
            .await
            .values()
            .filter(|p| p.state == PeerState::Active)
            .cloned()
            .collect()
    }

    /// Get peer count
    pub async fn peer_count(&self) -> usize {
        self.peers.read().await.len()
    }

    /// Get the federation mode
    pub fn mode(&self) -> &FederationMode {
        &self.config.mode
    }

    /// Get the local router ID
    pub fn router_id(&self) -> &str {
        &self.config.router_id
    }

    /// Get local namespace patterns
    pub fn owned_namespaces(&self) -> &[String] {
        &self.config.owned_namespaces
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> FederationConfig {
        FederationConfig {
            router_id: "test-router".to_string(),
            owned_namespaces: vec!["/local/**".to_string()],
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = FederationManager::new(test_config());
        assert_eq!(manager.router_id(), "test-router");
        assert_eq!(manager.peer_count().await, 0);
    }

    #[tokio::test]
    async fn test_process_peer_namespaces() {
        let manager = FederationManager::new(test_config());

        let event = LinkEvent::PeerNamespaces {
            router_id: "peer-a".to_string(),
            patterns: vec!["/remote/**".to_string()],
        };

        manager.process_event(&event).await;

        assert_eq!(manager.peer_count().await, 1);
        assert!(manager.should_forward("/remote/foo", None).await);
        assert!(!manager.should_forward("/local/foo", None).await);
    }

    #[tokio::test]
    async fn test_process_disconnect() {
        let manager = FederationManager::new(test_config());

        // Connect
        let event = LinkEvent::PeerNamespaces {
            router_id: "peer-a".to_string(),
            patterns: vec!["/remote/**".to_string()],
        };
        manager.process_event(&event).await;
        assert!(manager.should_forward("/remote/foo", None).await);

        // Disconnect
        let event = LinkEvent::Disconnected {
            router_id: "peer-a".to_string(),
            reason: None,
        };
        manager.process_event(&event).await;
        assert!(!manager.should_forward("/remote/foo", None).await);
    }

    #[tokio::test]
    async fn test_origin_exclusion() {
        let manager = FederationManager::new(test_config());

        let event = LinkEvent::PeerNamespaces {
            router_id: "peer-a".to_string(),
            patterns: vec!["/shared/**".to_string()],
        };
        manager.process_event(&event).await;

        // Should forward to peer-a
        let peers = manager.peers_for_address("/shared/foo", None).await;
        assert_eq!(peers.len(), 1);

        // Should not forward back to origin
        let peers = manager
            .peers_for_address("/shared/foo", Some("peer-a"))
            .await;
        assert!(peers.is_empty());
    }
}
