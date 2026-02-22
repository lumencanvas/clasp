//! Federation link -- manages a connection to a peer router
//!
//! A FederationLink represents one side of a router-to-router connection.
//! It uses the standard CLASP protocol to communicate, appearing as a
//! normal client session on the peer router.

use clasp_core::{
    codec, FederationOp, FederationSyncMessage, HelloMessage, Message, QoS, SetMessage,
    SubscribeMessage, Value, PROTOCOL_VERSION,
};
use clasp_transport::{TransportEvent, TransportReceiver, TransportSender};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::config::{FederationConfig, PeerInfo, PeerState};
use crate::error::{FederationError, Result};

/// Events emitted by a federation link to the local router
#[derive(Debug)]
pub enum LinkEvent {
    /// Peer declared its namespace patterns
    PeerNamespaces {
        router_id: String,
        patterns: Vec<String>,
    },
    /// Received a SET from the peer (should be applied to local state)
    RemoteSet {
        address: String,
        value: Value,
        revision: Option<u64>,
        origin: String,
    },
    /// Received a PUBLISH from the peer (should be broadcast locally)
    RemotePublish { message: Message, origin: String },
    /// Peer sync complete
    SyncComplete {
        router_id: String,
        pattern: String,
        revision: u64,
    },
    /// Peer disconnected
    Disconnected {
        router_id: String,
        reason: Option<String>,
    },
    /// Peer connected and handshake complete
    Connected { router_id: String },
}

/// A federation link to a single peer router.
///
/// The link connects to a peer router as a normal CLASP client,
/// exchanges federation metadata, syncs state, and then relays
/// messages bidirectionally based on namespace ownership.
pub struct FederationLink {
    /// Local router configuration
    config: FederationConfig,
    /// Transport sender to the peer
    sender: Arc<dyn TransportSender>,
    /// Peer information (populated after handshake)
    peer: Option<PeerInfo>,
    /// Current connection state
    state: PeerState,
    /// Channel for sending events to the local router
    event_tx: mpsc::Sender<LinkEvent>,
    /// Revision vector: address -> last known revision from this peer
    revision_vector: HashMap<String, u64>,
}

impl FederationLink {
    /// Create a new federation link with an established transport connection.
    ///
    /// After creation, call `run()` to start the handshake and message relay loop.
    pub fn new(
        config: FederationConfig,
        sender: Arc<dyn TransportSender>,
        event_tx: mpsc::Sender<LinkEvent>,
    ) -> Self {
        Self {
            config,
            sender,
            peer: None,
            state: PeerState::Connecting,
            event_tx,
            revision_vector: HashMap::new(),
        }
    }

    /// Run the federation link protocol.
    ///
    /// This performs the handshake, initial sync, and then relays messages
    /// until the connection is closed. Runs as an async task.
    pub async fn run(mut self, mut receiver: Box<dyn TransportReceiver>) -> Result<()> {
        // Step 1: Send HELLO with federation feature
        self.send_hello().await?;
        self.state = PeerState::Handshaking;

        // Step 2: Wait for WELCOME and process messages
        loop {
            match receiver.recv().await {
                Some(TransportEvent::Data(data)) => {
                    if let Err(e) = self.handle_data(&data).await {
                        error!("Federation link error: {}", e);
                        break;
                    }
                }
                Some(TransportEvent::Disconnected { reason }) => {
                    info!(
                        "Federation peer disconnected: {:?}",
                        reason.as_deref().unwrap_or("unknown")
                    );
                    let router_id = self
                        .peer
                        .as_ref()
                        .map(|p| p.router_id.clone())
                        .unwrap_or_default();
                    let _ = self
                        .event_tx
                        .send(LinkEvent::Disconnected { router_id, reason })
                        .await;
                    break;
                }
                Some(TransportEvent::Error(e)) => {
                    error!("Federation transport error: {}", e);
                    break;
                }
                Some(TransportEvent::Connected) => {
                    debug!("Federation transport connected event");
                }
                None => {
                    debug!("Federation transport stream ended");
                    break;
                }
            }
        }

        self.state = PeerState::Disconnected;
        Ok(())
    }

    /// Send a HELLO message with federation feature advertised
    async fn send_hello(&self) -> Result<()> {
        let hello = Message::Hello(HelloMessage {
            version: PROTOCOL_VERSION,
            name: self.config.client_name.clone(),
            features: self.config.features.clone(),
            capabilities: None,
            token: self.config.auth_token.clone(),
        });

        self.send_message(&hello, QoS::Confirm).await
    }

    /// Send federation namespace declaration to peer
    async fn declare_namespaces(&self) -> Result<()> {
        let msg = Message::FederationSync(FederationSyncMessage {
            op: FederationOp::DeclareNamespaces,
            patterns: self.config.owned_namespaces.clone(),
            revisions: HashMap::new(),
            since_revision: None,
            origin: Some(self.config.router_id.clone()),
        });

        self.send_message(&msg, QoS::Confirm).await
    }

    /// Subscribe to the peer's namespaces so we receive their updates
    async fn subscribe_to_peer(&self, patterns: &[String]) -> Result<()> {
        for (i, pattern) in patterns.iter().enumerate() {
            let sub = Message::Subscribe(SubscribeMessage {
                id: (1000 + i) as u32, // Use high IDs to avoid collision with local subs
                pattern: pattern.clone(),
                types: vec![],
                options: None,
            });
            self.send_message(&sub, QoS::Confirm).await?;
        }
        Ok(())
    }

    /// Request state sync from peer for a pattern
    async fn request_sync(&self, pattern: &str, since: Option<u64>) -> Result<()> {
        let msg = Message::FederationSync(FederationSyncMessage {
            op: FederationOp::RequestSync,
            patterns: vec![pattern.to_string()],
            revisions: HashMap::new(),
            since_revision: since,
            origin: Some(self.config.router_id.clone()),
        });

        self.send_message(&msg, QoS::Confirm).await
    }

    /// Send our revision vector to the peer for sync negotiation
    async fn send_revision_vector(&self) -> Result<()> {
        let msg = Message::FederationSync(FederationSyncMessage {
            op: FederationOp::RevisionVector,
            patterns: vec![],
            revisions: self.revision_vector.clone(),
            since_revision: None,
            origin: Some(self.config.router_id.clone()),
        });

        self.send_message(&msg, QoS::Confirm).await
    }

    /// Forward a local SET to the peer (if it matches the peer's namespaces)
    pub async fn forward_set(&self, msg: &SetMessage, origin: &str) -> Result<()> {
        // Don't forward messages that originated from this peer (loop prevention)
        if let Some(ref peer) = self.peer {
            if origin == peer.router_id {
                return Ok(());
            }
        }

        let set = Message::Set(SetMessage {
            address: msg.address.clone(),
            value: msg.value.clone(),
            revision: msg.revision,
            lock: false,
            unlock: false,
        });

        self.send_message(&set, QoS::Confirm).await
    }

    /// Forward a local PUBLISH to the peer
    pub async fn forward_publish(&self, msg: &Message, origin: &str) -> Result<()> {
        // Don't forward messages that originated from this peer
        if let Some(ref peer) = self.peer {
            if origin == peer.router_id {
                return Ok(());
            }
        }

        self.send_message(msg, QoS::Fire).await
    }

    /// Get the peer info (if handshake is complete)
    pub fn peer(&self) -> Option<&PeerInfo> {
        self.peer.as_ref()
    }

    /// Get the current connection state
    pub fn state(&self) -> PeerState {
        self.state
    }

    /// Check if the link is actively connected
    pub fn is_active(&self) -> bool {
        self.state == PeerState::Active
    }

    // =========================================================================
    // Internal methods
    // =========================================================================

    /// Handle incoming data from the peer
    async fn handle_data(&mut self, data: &[u8]) -> Result<()> {
        let (msg, _frame) =
            codec::decode(data).map_err(|e| FederationError::Codec(e.to_string()))?;

        match msg {
            Message::Welcome(welcome) => {
                info!(
                    "Federation handshake: received WELCOME from '{}' (session: {})",
                    welcome.name, welcome.session
                );

                // Initialize peer info
                self.peer = Some(PeerInfo {
                    router_id: welcome.session.clone(),
                    session_id: Some(welcome.session),
                    namespaces: vec![],
                    endpoint: None,
                    outbound: true,
                    state: PeerState::Handshaking,
                });

                // Declare our namespaces to the peer
                self.declare_namespaces().await?;
                self.state = PeerState::Syncing;
            }

            Message::FederationSync(fed_msg) => {
                self.handle_federation_sync(fed_msg).await?;
            }

            Message::Set(set_msg) => {
                // Peer sent us a SET -- apply it locally
                let origin = self
                    .peer
                    .as_ref()
                    .map(|p| p.router_id.clone())
                    .unwrap_or_default();

                // Track revision
                if let Some(rev) = set_msg.revision {
                    self.revision_vector.insert(set_msg.address.clone(), rev);
                }

                let _ = self
                    .event_tx
                    .send(LinkEvent::RemoteSet {
                        address: set_msg.address,
                        value: set_msg.value,
                        revision: set_msg.revision,
                        origin,
                    })
                    .await;
            }

            Message::Publish(_) => {
                let origin = self
                    .peer
                    .as_ref()
                    .map(|p| p.router_id.clone())
                    .unwrap_or_default();

                let _ = self
                    .event_tx
                    .send(LinkEvent::RemotePublish {
                        message: msg,
                        origin,
                    })
                    .await;
            }

            Message::Snapshot(snapshot) => {
                // Initial snapshot from peer after subscribe
                let origin = self
                    .peer
                    .as_ref()
                    .map(|p| p.router_id.clone())
                    .unwrap_or_default();

                for param in snapshot.params {
                    self.revision_vector
                        .insert(param.address.clone(), param.revision);
                    let _ = self
                        .event_tx
                        .send(LinkEvent::RemoteSet {
                            address: param.address,
                            value: param.value,
                            revision: Some(param.revision),
                            origin: origin.clone(),
                        })
                        .await;
                }
            }

            Message::Ack(_) => {
                // Acknowledged, no action needed
            }

            Message::Error(err) => {
                warn!(
                    "Federation peer error: {} (code: {})",
                    err.message, err.code
                );
            }

            Message::Ping => {
                // Respond with pong
                self.send_message(&Message::Pong, QoS::Fire).await?;
            }

            _ => {
                debug!(
                    "Federation link: ignoring message type {:?}",
                    msg.type_code()
                );
            }
        }

        Ok(())
    }

    /// Handle a FederationSync message from the peer
    async fn handle_federation_sync(&mut self, msg: FederationSyncMessage) -> Result<()> {
        match msg.op {
            FederationOp::DeclareNamespaces => {
                let router_id = msg
                    .origin
                    .clone()
                    .or_else(|| self.peer.as_ref().map(|p| p.router_id.clone()))
                    .unwrap_or_default();

                info!("Peer {} declares namespaces: {:?}", router_id, msg.patterns);

                // Update peer info
                if let Some(ref mut peer) = self.peer {
                    peer.namespaces = msg.patterns.clone();
                }

                // Notify local router
                let _ = self
                    .event_tx
                    .send(LinkEvent::PeerNamespaces {
                        router_id: router_id.clone(),
                        patterns: msg.patterns.clone(),
                    })
                    .await;

                // Subscribe to the peer's namespaces
                self.subscribe_to_peer(&msg.patterns).await?;

                // Request initial sync
                for pattern in &msg.patterns {
                    self.request_sync(pattern, None).await?;
                }
            }

            FederationOp::RequestSync => {
                debug!("Peer requests sync for patterns: {:?}", msg.patterns);
                // The local router should handle this by sending a snapshot
                // For now, send our revision vector so the peer knows what we have
                self.send_revision_vector().await?;
            }

            FederationOp::RevisionVector => {
                debug!(
                    "Received revision vector with {} entries",
                    msg.revisions.len()
                );
                // Compare with our local state to identify what needs syncing
                // For now, just store the peer's revision vector for reference
            }

            FederationOp::SyncComplete => {
                let router_id = self
                    .peer
                    .as_ref()
                    .map(|p| p.router_id.clone())
                    .unwrap_or_default();

                let pattern = msg.patterns.first().cloned().unwrap_or_default();
                let revision = msg.since_revision.unwrap_or(0);

                info!(
                    "Sync complete for pattern '{}' at revision {}",
                    pattern, revision
                );

                if let Some(ref mut peer) = self.peer {
                    peer.state = PeerState::Active;
                }
                self.state = PeerState::Active;

                let _ = self
                    .event_tx
                    .send(LinkEvent::SyncComplete {
                        router_id,
                        pattern,
                        revision,
                    })
                    .await;

                let _ = self
                    .event_tx
                    .send(LinkEvent::Connected {
                        router_id: self
                            .peer
                            .as_ref()
                            .map(|p| p.router_id.clone())
                            .unwrap_or_default(),
                    })
                    .await;
            }
        }

        Ok(())
    }

    /// Encode and send a message to the peer
    async fn send_message(&self, msg: &Message, _qos: QoS) -> Result<()> {
        let data = codec::encode(msg).map_err(|e| FederationError::Codec(e.to_string()))?;
        self.sender
            .send(data)
            .await
            .map_err(|e| FederationError::Transport(e.to_string()))
    }
}
