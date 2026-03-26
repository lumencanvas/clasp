//! Error types for the DefraDB journal backend.

use clasp_journal::JournalError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DefraError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("GraphQL error: {0}")]
    GraphQL(String),

    #[error("schema provisioning failed: {0}")]
    Schema(String),

    #[error("deserialization error: {0}")]
    Deserialization(String),

    #[error("DefraDB unavailable at {0}")]
    Unavailable(String),
}

impl From<DefraError> for JournalError {
    fn from(err: DefraError) -> Self {
        match err {
            DefraError::Http(e) => JournalError::StorageError(e.to_string()),
            DefraError::GraphQL(msg) => JournalError::StorageError(msg),
            DefraError::Schema(msg) => JournalError::StorageError(msg),
            DefraError::Deserialization(msg) => JournalError::SerializationError(msg),
            DefraError::Unavailable(url) => {
                JournalError::StorageError(format!("DefraDB unavailable at {url}"))
            }
        }
    }
}

pub type Result<T> = std::result::Result<T, DefraError>;
