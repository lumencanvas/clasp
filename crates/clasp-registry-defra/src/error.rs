//! Error types for the DefraDB registry backend.

use clasp_registry::RegistryError;
use thiserror::Error;

/// Errors specific to the DefraDB registry backend.
///
/// These are converted to [`RegistryError`] at the trait boundary so
/// callers never need to depend on this crate's error type directly.
#[derive(Error, Debug)]
pub enum DefraRegistryError {
    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("schema provisioning failed: {0}")]
    Schema(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("DefraDB unavailable at {0}")]
    Unavailable(String),
}

impl From<DefraRegistryError> for RegistryError {
    fn from(err: DefraRegistryError) -> Self {
        match err {
            DefraRegistryError::Http(msg) => RegistryError::StorageError(msg),
            DefraRegistryError::GraphQL(msg) => RegistryError::StorageError(msg),
            DefraRegistryError::Schema(msg) => RegistryError::StorageError(msg),
            DefraRegistryError::Deserialization(msg) => RegistryError::StorageError(msg),
            DefraRegistryError::Unavailable(url) => {
                RegistryError::StorageError(format!("DefraDB unavailable at {url}"))
            }
        }
    }
}

