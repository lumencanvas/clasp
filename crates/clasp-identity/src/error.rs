use thiserror::Error;

/// Errors that can occur in identity operations.
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("invalid DID: {0}")]
    InvalidDid(String),

    #[error("invalid PeerID: {0}")]
    InvalidPeerId(String),

    #[error("invalid signature: {0}")]
    InvalidSignature(String),

    #[error("decoding error: {0}")]
    DecodingError(String),
}

pub type Result<T> = std::result::Result<T, IdentityError>;
