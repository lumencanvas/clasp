//! Tunnel protocol for wrapping DefraDB sync messages in CLASP binary payloads.

use serde::{Deserialize, Serialize};

use crate::error::{Result, TunnelError};

/// Message types for the DefraDB-over-CLASP tunnel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TunnelMessage {
    /// Request to sync a collection.
    SyncRequest {
        collection: String,
        since_cid: Option<String>,
    },

    /// A DAG block (IPLD node) being transferred.
    DagBlock {
        cid: String,
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
        links: Vec<String>,
    },

    /// Head update notification (new commit on a document).
    HeadUpdate {
        collection: String,
        doc_id: String,
        head_cid: String,
    },

    /// Acknowledge receipt of blocks.
    BlockAck { cids: Vec<String> },

    /// Peer info exchange.
    PeerInfo {
        peer_id: String,
        collections: Vec<String>,
    },

    /// GraphQL query forwarded to a peer's DefraDB.
    QueryForward { query_id: String, query: String },

    /// GraphQL response from peer.
    QueryResponse {
        query_id: String,
        data: serde_json::Value,
    },
}

impl TunnelMessage {
    /// Encode to a JSON byte vector for transport over CLASP.
    pub fn encode(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(|e| TunnelError::Encode(e.to_string()))
    }

    /// Decode from a byte slice received over CLASP.
    pub fn decode(data: &[u8]) -> Result<Self> {
        serde_json::from_slice(data).map_err(|e| TunnelError::Decode(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tunnel_message_encode_decode() {
        let variants: Vec<TunnelMessage> = vec![
            TunnelMessage::SyncRequest {
                collection: "users".into(),
                since_cid: Some("bafy123".into()),
            },
            TunnelMessage::DagBlock {
                cid: "bafy456".into(),
                data: vec![0xDE, 0xAD, 0xBE, 0xEF],
                links: vec!["bafy789".into()],
            },
            TunnelMessage::HeadUpdate {
                collection: "posts".into(),
                doc_id: "doc1".into(),
                head_cid: "bafyhead".into(),
            },
            TunnelMessage::BlockAck {
                cids: vec!["bafy1".into(), "bafy2".into()],
            },
            TunnelMessage::PeerInfo {
                peer_id: "peer-abc".into(),
                collections: vec!["users".into(), "posts".into()],
            },
            TunnelMessage::QueryForward {
                query_id: "q1".into(),
                query: "{ users { name } }".into(),
            },
            TunnelMessage::QueryResponse {
                query_id: "q1".into(),
                data: serde_json::json!({"users": [{"name": "Alice"}]}),
            },
        ];

        for msg in &variants {
            let encoded = msg.encode().expect("encode should succeed");
            let decoded = TunnelMessage::decode(&encoded).expect("decode should succeed");
            assert_eq!(msg, &decoded, "roundtrip failed for {:?}", msg);
        }
    }

    #[test]
    fn sync_request_roundtrip() {
        let msg = TunnelMessage::SyncRequest {
            collection: "devices".into(),
            since_cid: None,
        };
        let encoded = msg.encode().unwrap();
        let decoded = TunnelMessage::decode(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn dag_block_roundtrip() {
        let msg = TunnelMessage::DagBlock {
            cid: "bafyblock".into(),
            data: (0..=255).collect(),
            links: vec!["link1".into(), "link2".into(), "link3".into()],
        };
        let encoded = msg.encode().unwrap();
        let decoded = TunnelMessage::decode(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn head_update_roundtrip() {
        let msg = TunnelMessage::HeadUpdate {
            collection: "signals".into(),
            doc_id: "doc-42".into(),
            head_cid: "bafyhead42".into(),
        };
        let encoded = msg.encode().unwrap();
        let decoded = TunnelMessage::decode(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn query_forward_roundtrip() {
        let msg = TunnelMessage::QueryForward {
            query_id: "qf-99".into(),
            query: "mutation { create_User(input: {name: \"Bob\"}) { _docID } }".into(),
        };
        let encoded = msg.encode().unwrap();
        let decoded = TunnelMessage::decode(&encoded).unwrap();
        assert_eq!(msg, decoded);
    }
}
