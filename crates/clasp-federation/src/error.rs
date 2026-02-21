//! Federation error types

use thiserror::Error;

/// Federation-specific errors
#[derive(Debug, Error)]
pub enum FederationError {
    /// Peer connection failed
    #[error("connection to peer failed: {0}")]
    ConnectionFailed(String),

    /// Handshake failed (peer did not announce federation feature)
    #[error("federation handshake failed: {0}")]
    HandshakeFailed(String),

    /// Namespace conflict (two peers claim the same namespace)
    #[error("namespace conflict: {pattern} claimed by both {a} and {b}")]
    NamespaceConflict {
        pattern: String,
        a: String,
        b: String,
    },

    /// Peer not found
    #[error("peer not found: {0}")]
    PeerNotFound(String),

    /// Transport error
    #[error("transport error: {0}")]
    Transport(String),

    /// Codec error
    #[error("codec error: {0}")]
    Codec(String),

    /// Sync error
    #[error("sync error: {0}")]
    Sync(String),

    /// Already connected to this peer
    #[error("already connected to peer: {0}")]
    AlreadyConnected(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    Config(String),
}

/// Result type for federation operations
pub type Result<T> = std::result::Result<T, FederationError>;

impl From<clasp_core::Error> for FederationError {
    fn from(e: clasp_core::Error) -> Self {
        FederationError::Codec(e.to_string())
    }
}

impl From<clasp_transport::TransportError> for FederationError {
    fn from(e: clasp_transport::TransportError) -> Self {
        FederationError::Transport(e.to_string())
    }
}
