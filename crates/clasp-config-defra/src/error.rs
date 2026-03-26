//! Error types for the DefraDB config backend.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigDefraError {
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

    #[error("invalid config: {0}")]
    InvalidConfig(String),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ConfigDefraError>;
