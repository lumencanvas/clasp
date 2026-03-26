//! Main tunnel bridging DefraDB HTTP API and CLASP signals.

use std::collections::HashMap;

use clasp_journal_defra::DefraClient;
use tracing::{debug, info, warn};

use crate::error::{Result, TunnelError};
use crate::protocol::TunnelMessage;
use crate::sync::{apply_received_blocks, compute_sync_diff, BlockInfo};

/// State tracked for each known remote peer.
struct PeerState {
    collections: Vec<String>,
    /// Maps collection name to the last CID we synced to that peer.
    last_sync: HashMap<String, String>,
}

/// Tunnels DefraDB sync traffic over CLASP signals.
///
/// On the sending side, it polls DefraDB for new commits and sends
/// them as CLASP binary payloads to the peer's sync channel.
///
/// On the receiving side, it listens for incoming DAG blocks and
/// forwards them to the local DefraDB instance.
pub struct DefraTunnel {
    local_defra: DefraClient,
    local_peer_id: String,
    /// Known remote peers and their collections.
    peers: tokio::sync::RwLock<HashMap<String, PeerState>>,
}

impl DefraTunnel {
    /// Create a new tunnel backed by a DefraDB instance.
    pub fn new(defra_url: &str, peer_id: &str) -> Self {
        Self {
            local_defra: DefraClient::new(defra_url),
            local_peer_id: peer_id.to_string(),
            peers: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Handle an incoming tunnel message from a CLASP peer.
    ///
    /// Returns an optional response message to send back.
    pub async fn handle_message(
        &self,
        from_peer: &str,
        msg: TunnelMessage,
    ) -> Result<Option<TunnelMessage>> {
        match msg {
            TunnelMessage::PeerInfo {
                peer_id,
                collections,
            } => {
                info!(peer = %peer_id, ?collections, "received peer info");
                self.add_peer(&peer_id, collections.clone()).await;

                // Respond with our own peer info
                let our_collections = {
                    let peers = self.peers.read().await;
                    // Gather all unique collections we know about
                    let mut cols: Vec<String> = peers
                        .values()
                        .flat_map(|p| p.collections.iter().cloned())
                        .collect();
                    cols.sort();
                    cols.dedup();
                    cols
                };

                Ok(Some(TunnelMessage::PeerInfo {
                    peer_id: self.local_peer_id.clone(),
                    collections: our_collections,
                }))
            }

            TunnelMessage::SyncRequest {
                collection,
                since_cid,
            } => {
                debug!(peer = %from_peer, %collection, "handling sync request");
                let blocks =
                    compute_sync_diff(&self.local_defra, &collection, since_cid.as_deref()).await?;

                // Send each block as a separate DagBlock message.
                // For now, return the first block; the caller should iterate.
                if let Some(block) = blocks.into_iter().next() {
                    Ok(Some(TunnelMessage::DagBlock {
                        cid: block.cid,
                        data: block.data,
                        links: block.links,
                    }))
                } else {
                    Ok(None)
                }
            }

            TunnelMessage::DagBlock { cid, data, links } => {
                debug!(peer = %from_peer, %cid, "received DAG block");
                let block_info = BlockInfo { cid: cid.clone(), data, links };
                let applied = apply_received_blocks(&self.local_defra, &[block_info]).await?;
                Ok(Some(TunnelMessage::BlockAck {
                    cids: if applied > 0 {
                        vec![cid]
                    } else {
                        vec![]
                    },
                }))
            }

            TunnelMessage::HeadUpdate {
                collection,
                doc_id,
                head_cid,
            } => {
                debug!(peer = %from_peer, %collection, %doc_id, %head_cid, "received head update");
                // Update the peer's last known CID for this collection
                let mut peers = self.peers.write().await;
                if let Some(state) = peers.get_mut(from_peer) {
                    state.last_sync.insert(collection.clone(), head_cid);
                }
                // Respond with a sync request to fetch blocks we might be missing
                Ok(Some(TunnelMessage::SyncRequest {
                    collection,
                    since_cid: None,
                }))
            }

            TunnelMessage::BlockAck { cids } => {
                debug!(peer = %from_peer, count = cids.len(), "received block acknowledgement");
                Ok(None)
            }

            TunnelMessage::QueryForward { query_id, query } => {
                debug!(peer = %from_peer, %query_id, "handling forwarded query");
                match self.handle_query(&query).await {
                    Ok(data) => Ok(Some(TunnelMessage::QueryResponse { query_id, data })),
                    Err(e) => {
                        warn!(error = %e, "query forward failed");
                        Ok(Some(TunnelMessage::QueryResponse {
                            query_id,
                            data: serde_json::json!({"error": e.to_string()}),
                        }))
                    }
                }
            }

            TunnelMessage::QueryResponse { query_id, .. } => {
                debug!(peer = %from_peer, %query_id, "received query response");
                // The caller is responsible for routing this back to the
                // original query issuer.
                Ok(None)
            }
        }
    }

    /// Generate outgoing messages for a peer (call periodically or on change).
    pub async fn poll_updates(&self, target_peer: &str) -> Result<Vec<TunnelMessage>> {
        let peers = self.peers.read().await;
        let state = match peers.get(target_peer) {
            Some(s) => s,
            None => return Ok(Vec::new()),
        };

        let mut messages = Vec::new();

        for collection in &state.collections {
            let last_cid = state.last_sync.get(collection).map(|s| s.as_str());
            let blocks =
                compute_sync_diff(&self.local_defra, collection, last_cid).await?;

            for block in blocks {
                messages.push(TunnelMessage::DagBlock {
                    cid: block.cid,
                    data: block.data,
                    links: block.links,
                });
            }
        }

        Ok(messages)
    }

    /// Register a remote peer with its known collections.
    pub async fn add_peer(&self, peer_id: &str, collections: Vec<String>) {
        let mut peers = self.peers.write().await;
        peers.insert(
            peer_id.to_string(),
            PeerState {
                collections,
                last_sync: HashMap::new(),
            },
        );
        debug!(peer = %peer_id, "registered remote peer");
    }

    /// Remove a remote peer.
    pub async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);
        debug!(peer = %peer_id, "removed remote peer");
    }

    /// List known peer IDs.
    pub async fn peers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers.keys().cloned().collect()
    }

    /// Forward a GraphQL query to the local DefraDB and return the response.
    pub async fn handle_query(&self, query: &str) -> Result<serde_json::Value> {
        self.local_defra
            .graphql(query, None)
            .await
            .map_err(|e| TunnelError::GraphQL(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_tunnel() -> DefraTunnel {
        DefraTunnel::new("http://localhost:19181", "test-peer-local")
    }

    #[tokio::test]
    async fn peer_registration() {
        let tunnel = make_tunnel();

        assert!(tunnel.peers().await.is_empty());

        tunnel
            .add_peer("peer-a", vec!["users".into(), "posts".into()])
            .await;
        tunnel.add_peer("peer-b", vec!["users".into()]).await;

        let mut peers = tunnel.peers().await;
        peers.sort();
        assert_eq!(peers, vec!["peer-a", "peer-b"]);

        tunnel.remove_peer("peer-a").await;
        let peers = tunnel.peers().await;
        assert_eq!(peers, vec!["peer-b"]);

        tunnel.remove_peer("peer-b").await;
        assert!(tunnel.peers().await.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_poll_updates_empty() {
        let tunnel = make_tunnel();
        // No peers registered, should return empty
        let msgs = tunnel.poll_updates("nonexistent").await.unwrap();
        assert!(msgs.is_empty());
    }

    #[tokio::test]
    #[ignore]
    async fn test_handle_peer_info() {
        let tunnel = make_tunnel();

        let msg = TunnelMessage::PeerInfo {
            peer_id: "remote-peer".into(),
            collections: vec!["devices".into()],
        };

        let response = tunnel
            .handle_message("remote-peer", msg)
            .await
            .unwrap();

        assert!(response.is_some());
        if let Some(TunnelMessage::PeerInfo { peer_id, .. }) = response {
            assert_eq!(peer_id, "test-peer-local");
        } else {
            panic!("expected PeerInfo response");
        }

        let peers = tunnel.peers().await;
        assert!(peers.contains(&"remote-peer".to_string()));
    }
}
