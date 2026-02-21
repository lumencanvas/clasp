//! Journal error types

use thiserror::Error;

pub type Result<T> = std::result::Result<T, JournalError>;

#[derive(Error, Debug)]
pub enum JournalError {
    #[error("journal full: capacity {0}")]
    Full(usize),

    #[error("invalid sequence: expected {expected}, got {got}")]
    InvalidSequence { expected: u64, got: u64 },

    #[error("entry not found: seq {0}")]
    NotFound(u64),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("snapshot error: {0}")]
    SnapshotError(String),
}
