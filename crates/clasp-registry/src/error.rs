//! Registry error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, RegistryError>;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("entity already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid entity ID format: {0}")]
    InvalidId(String),

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("signature verification failed: {0}")]
    SignatureError(String),

    #[error("token error: {0}")]
    TokenError(String),

    #[error("entity is not active: {0}")]
    NotActive(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("scope error: {0}")]
    ScopeError(String),
}
