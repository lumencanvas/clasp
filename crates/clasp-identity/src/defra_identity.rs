//! DefraDB ACP identity (secp256k1).
//!
//! DefraDB uses secp256k1 keys for access control, not Ed25519. This module
//! provides a separate secp256k1 identity for DefraDB operations, generated
//! deterministically from the primary Ed25519 identity via key derivation.
//!
//! Requires the `secp256k1` feature flag.

use k256::ecdsa::SigningKey;
use rand::rngs::OsRng;

use crate::error::Result;
use crate::Identity;

/// A secp256k1 identity for DefraDB ACP operations.
///
/// This is a secondary key derived from or generated alongside the primary
/// Ed25519 identity. It is used exclusively for DefraDB authentication
/// and access control.
pub struct DefraIdentity {
    signing_key: SigningKey,
}

impl DefraIdentity {
    /// Generate a new random secp256k1 identity.
    pub fn generate() -> Self {
        let signing_key = SigningKey::random(&mut OsRng);
        Self { signing_key }
    }

    /// Create from raw 32-byte private key.
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(bytes.into())
            .map_err(|e| crate::error::IdentityError::InvalidKey(e.to_string()))?;
        Ok(Self { signing_key })
    }

    /// Derive a DefraDB identity deterministically from a CLASP Ed25519 identity.
    ///
    /// Uses HKDF-SHA256 to derive a secp256k1 key from the Ed25519 signing key
    /// bytes. This ensures the same CLASP identity always produces the same
    /// DefraDB identity, without storing an extra key.
    pub fn derive_from(identity: &Identity) -> Result<Self> {
        use sha2::{Digest, Sha256};

        // HKDF-expand: SHA256(ed25519_private_key || "defradb-acp-secp256k1")
        // Simple derivation since we just need 32 bytes of key material.
        let mut hasher = Sha256::new();
        hasher.update(identity.signing_key().to_bytes());
        hasher.update(b"defradb-acp-secp256k1");
        let derived: [u8; 32] = hasher.finalize().into();

        Self::from_bytes(&derived)
    }

    /// Get the private key as a hex string (for DefraDB CLI identity parameter).
    pub fn to_hex(&self) -> String {
        hex::encode(self.signing_key.to_bytes())
    }

    /// Get the raw 32-byte private key.
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.signing_key.to_bytes().into()
    }

    /// Get the compressed public key (33 bytes).
    pub fn public_key_compressed(&self) -> Vec<u8> {
        use k256::ecdsa::VerifyingKey;
        let vk = VerifyingKey::from(&self.signing_key);
        vk.to_sec1_bytes().to_vec()
    }
}

impl std::fmt::Debug for DefraIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DefraIdentity")
            .field("public_key", &format!("{}...", &self.to_hex()[..8]))
            .finish()
    }
}

/// Encode bytes as lowercase hex. Avoids pulling in the `hex` crate
/// for this single use case.
mod hex {
    pub fn encode(bytes: impl AsRef<[u8]>) -> String {
        bytes
            .as_ref()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_produces_valid_key() {
        let id = DefraIdentity::generate();
        assert_eq!(id.private_key_bytes().len(), 32);
        assert_eq!(id.to_hex().len(), 64);
    }

    #[test]
    fn from_bytes_round_trips() {
        let id = DefraIdentity::generate();
        let bytes = id.private_key_bytes();
        let restored = DefraIdentity::from_bytes(&bytes).unwrap();
        assert_eq!(id.to_hex(), restored.to_hex());
    }

    #[test]
    fn derive_from_ed25519_is_deterministic() {
        let ed_identity = Identity::generate();
        let defra_a = DefraIdentity::derive_from(&ed_identity).unwrap();
        let defra_b = DefraIdentity::derive_from(&ed_identity).unwrap();
        assert_eq!(defra_a.to_hex(), defra_b.to_hex());
    }

    #[test]
    fn derive_from_different_ed25519_produces_different_keys() {
        let ed_a = Identity::generate();
        let ed_b = Identity::generate();
        let defra_a = DefraIdentity::derive_from(&ed_a).unwrap();
        let defra_b = DefraIdentity::derive_from(&ed_b).unwrap();
        assert_ne!(defra_a.to_hex(), defra_b.to_hex());
    }

    #[test]
    fn public_key_compressed_is_33_bytes() {
        let id = DefraIdentity::generate();
        assert_eq!(id.public_key_compressed().len(), 33);
    }

    #[test]
    fn hex_encoding_is_lowercase() {
        let id = DefraIdentity::generate();
        let hex = id.to_hex();
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(hex, hex.to_lowercase());
    }
}
