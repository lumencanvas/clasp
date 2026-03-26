//! DID (did:key) encoding and decoding for Ed25519 public keys.
//!
//! Follows the did:key method specification:
//! 1. Prepend multicodec varint for Ed25519 (0xed -> varint bytes 0xed, 0x01)
//! 2. Base58btc encode
//! 3. Prepend "did:key:z" (z = base58btc multibase prefix)

use crate::error::{IdentityError, Result};

/// Ed25519 multicodec code: 0xed
/// Encoded as unsigned varint: [0xed, 0x01]
const ED25519_MULTICODEC: [u8; 2] = [0xed, 0x01];

/// Create a did:key string from a 32-byte Ed25519 public key.
pub fn public_key_to_did(key: &[u8; 32]) -> String {
    let mut buf = Vec::with_capacity(2 + 32);
    buf.extend_from_slice(&ED25519_MULTICODEC);
    buf.extend_from_slice(key);
    let encoded = bs58::encode(&buf).into_string();
    format!("did:key:z{}", encoded)
}

/// Parse a did:key string back to a 32-byte Ed25519 public key.
pub fn did_to_public_key(did: &str) -> Result<[u8; 32]> {
    let rest = did
        .strip_prefix("did:key:z")
        .ok_or_else(|| IdentityError::InvalidDid("must start with \"did:key:z\"".into()))?;

    let decoded = bs58::decode(rest)
        .into_vec()
        .map_err(|e| IdentityError::InvalidDid(format!("invalid base58: {}", e)))?;

    if decoded.len() != 34 {
        return Err(IdentityError::InvalidDid(format!(
            "expected 34 bytes (2 multicodec + 32 key), got {}",
            decoded.len()
        )));
    }

    if decoded[0] != ED25519_MULTICODEC[0] || decoded[1] != ED25519_MULTICODEC[1] {
        return Err(IdentityError::InvalidDid(format!(
            "unsupported multicodec: [{:#x}, {:#x}], expected Ed25519 [{:#x}, {:#x}]",
            decoded[0], decoded[1], ED25519_MULTICODEC[0], ED25519_MULTICODEC[1]
        )));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded[2..34]);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn did_roundtrip() {
        let key = [42u8; 32];
        let did = public_key_to_did(&key);
        let recovered = did_to_public_key(&did).unwrap();
        assert_eq!(recovered, key);
    }

    #[test]
    fn did_format_prefix() {
        let key = [0u8; 32];
        let did = public_key_to_did(&key);
        assert!(did.starts_with("did:key:z"));
    }

    #[test]
    fn did_parse_invalid() {
        assert!(did_to_public_key("not-a-did").is_err());
        assert!(did_to_public_key("did:key:zBadData").is_err());
    }
}
