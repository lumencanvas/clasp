//! Error types for the DefraDB transport tunnel.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum TunnelError {
    #[error("encoding error: {0}")]
    Encode(String),

    #[error("decoding error: {0}")]
    Decode(String),

    #[error("DefraDB client error: {0}")]
    Defra(String),

    #[error("peer not found: {0}")]
    PeerNotFound(String),

    #[error("invalid address: {0}")]
    InvalidAddress(String),

    #[error("GraphQL error: {0}")]
    GraphQL(String),
}

impl From<serde_json::Error> for TunnelError {
    fn from(err: serde_json::Error) -> Self {
        TunnelError::Decode(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, TunnelError>;
