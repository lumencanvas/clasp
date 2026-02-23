//! # clasp-crypto
//!
//! E2E encryption add-on for the CLASP protocol.
//!
//! ## Layers
//!
//! - **Primitives** (`primitives`): Pure crypto operations — AES-256-GCM,
//!   ECDH P-256, HKDF-SHA256, ECDSA P-256. No CLASP dependency.
//! - **Protocol** (`protocol`): E2ESession state machine for key exchange
//!   over CLASP paths.
//! - **Storage** (`storage`): KeyStore trait with MemoryKeyStore.
//! - **Client** (`client`, behind `client` feature): CryptoClient wrapper
//!   for transparent encrypt/decrypt over a `clasp_client::Clasp` instance.

pub mod error;
pub mod primitives;
pub mod protocol;
pub mod storage;
pub mod types;

#[cfg(feature = "client")]
pub mod client;

pub use error::{CryptoError, Result};
pub use primitives::{
    constant_time_eq, decrypt, derive_shared_key, encrypt, export_group_key, export_public_key,
    fingerprint, fingerprint_jwk, generate_ecdh_key_pair, generate_group_key,
    generate_signing_key_pair, group_key_to_jwk, import_group_key, import_public_key,
    jwk_to_group_key, jwk_to_public_key, public_key_to_jwk, sign, verify,
};
pub use protocol::E2ESession;
pub use storage::{KeyStore, MemoryKeyStore};
pub use types::{E2EEnvelope, ECDHKeyPair, KeyData, KeyExchangeMessage, SigningKeyPair, TofuRecord};
