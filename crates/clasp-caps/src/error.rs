//! Capability token error types

use thiserror::Error;

/// Capability token errors
#[derive(Debug, Error)]
pub enum CapError {
    /// Token encoding/decoding failed
    #[error("encoding error: {0}")]
    Encoding(String),

    /// Signature verification failed
    #[error("invalid signature")]
    InvalidSignature,

    /// Token has expired
    #[error("token expired")]
    Expired,

    /// Scope attenuation violation (child tried to widen parent's scope)
    #[error("scope attenuation violation: {0}")]
    AttenuationViolation(String),

    /// Delegation chain too deep
    #[error("delegation chain too deep: {depth} exceeds max {max}")]
    ChainTooDeep { depth: usize, max: usize },

    /// Issuer not trusted (not in trust anchors)
    #[error("untrusted issuer: {0}")]
    UntrustedIssuer(String),

    /// Invalid proof chain
    #[error("invalid proof chain: {0}")]
    InvalidProof(String),

    /// Key error
    #[error("key error: {0}")]
    KeyError(String),
}

/// Result type for capability operations
pub type Result<T> = std::result::Result<T, CapError>;
