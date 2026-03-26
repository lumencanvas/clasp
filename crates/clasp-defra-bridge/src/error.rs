//! Error types for the DefraDB bridge.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum BridgeError {
    #[error("DefraDB query failed: {0}")]
    Defra(String),

    #[error("address parse error: {0}")]
    Address(String),

    #[error("signal send failed: {0}")]
    Signal(String),

    #[error("subscription failed: {0}")]
    Subscription(String),

    #[error("value conversion error: {0}")]
    Conversion(String),
}

pub type Result<T> = std::result::Result<T, BridgeError>;
