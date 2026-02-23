use thiserror::Error;

#[derive(Debug, Error)]
pub enum CryptoError {
    #[error("encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("key derivation failed: {0}")]
    KeyDerivationFailed(String),

    #[error("invalid key: {0}")]
    InvalidKey(String),

    #[error("signing failed: {0}")]
    SigningFailed(String),

    #[error("verification failed: {0}")]
    VerificationFailed(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("session destroyed")]
    SessionDestroyed,

    #[error("no group key available")]
    NoGroupKey,

    #[error("TOFU key change rejected for peer: {0}")]
    TofuViolation(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, CryptoError>;
