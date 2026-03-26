//! Error types for the DefraDB state store.

use thiserror::Error;

/// Errors from the DefraDB state store.
#[derive(Error, Debug)]
pub enum DefraStateError {
    #[error("DefraDB HTTP error: {0}")]
    Http(String),

    #[error("DefraDB GraphQL error: {0}")]
    GraphQL(String),

    #[error("schema provisioning failed: {0}")]
    Schema(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("DefraDB unavailable at {0}")]
    Unavailable(String),

    #[error("write channel closed")]
    ChannelClosed,
}

pub type Result<T> = std::result::Result<T, DefraStateError>;
