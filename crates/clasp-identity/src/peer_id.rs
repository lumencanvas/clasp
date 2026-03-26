//! libp2p PeerID encoding and decoding for Ed25519 public keys.
//!
//! Encoding steps:
//! 1. Create protobuf-encoded public key:
//!    - Field 1 (KeyType) varint = 1 (Ed25519)  -> 0x08 0x01
//!    - Field 2 (Data) length-delimited = 32 bytes -> 0x12 0x20 + key bytes
//!    - Total: 36 bytes
//! 2. Since 36 <= 42, use identity multihash:
//!    - 0x00 (identity hash function code)
//!    - 0x24 (varint length = 36)
//!    - + 36 bytes of protobuf
//!    - Total: 38 bytes
//! 3. Base58btc encode

use crate::error::{IdentityError, Result};

/// Protobuf prefix for an Ed25519 public key:
/// field 1 (KeyType), varint, value 1 = 0x08 0x01
/// field 2 (Data), length-delimited, length 32 = 0x12 0x20
const PROTO_PREFIX: [u8; 4] = [0x08, 0x01, 0x12, 0x20];

/// Identity multihash prefix: code 0x00, length 36 (0x24)
const IDENTITY_MULTIHASH_PREFIX: [u8; 2] = [0x00, 0x24];

/// Encode a 32-byte Ed25519 public key as a libp2p PeerID string.
pub fn public_key_to_peer_id(key: &[u8; 32]) -> String {
    let mut buf = Vec::with_capacity(38);
    // Identity multihash wrapping the protobuf-encoded key
    buf.extend_from_slice(&IDENTITY_MULTIHASH_PREFIX);
    buf.extend_from_slice(&PROTO_PREFIX);
    buf.extend_from_slice(key);
    bs58::encode(&buf).into_string()
}

/// Decode a libp2p PeerID string back to a 32-byte Ed25519 public key.
pub fn peer_id_to_public_key(peer_id: &str) -> Result<[u8; 32]> {
    let decoded = bs58::decode(peer_id)
        .into_vec()
        .map_err(|e| IdentityError::InvalidPeerId(format!("invalid base58: {}", e)))?;

    if decoded.len() != 38 {
        return Err(IdentityError::InvalidPeerId(format!(
            "expected 38 bytes, got {}",
            decoded.len()
        )));
    }

    // Check identity multihash prefix
    if decoded[0] != IDENTITY_MULTIHASH_PREFIX[0] || decoded[1] != IDENTITY_MULTIHASH_PREFIX[1] {
        return Err(IdentityError::InvalidPeerId(
            "not an identity multihash".into(),
        ));
    }

    // Check protobuf prefix (Ed25519 key type)
    if decoded[2..6] != PROTO_PREFIX {
        return Err(IdentityError::InvalidPeerId(
            "not an Ed25519 protobuf-encoded key".into(),
        ));
    }

    let mut key = [0u8; 32];
    key.copy_from_slice(&decoded[6..38]);
    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peer_id_roundtrip() {
        let key = [42u8; 32];
        let pid = public_key_to_peer_id(&key);
        let recovered = peer_id_to_public_key(&pid).unwrap();
        assert_eq!(recovered, key);
    }

    #[test]
    fn peer_id_format_prefix() {
        let key = [0u8; 32];
        let pid = public_key_to_peer_id(&key);
        assert!(pid.starts_with("12D3KooW"), "PeerID should start with 12D3KooW, got: {}", pid);
    }

    #[test]
    fn peer_id_parse_invalid() {
        assert!(peer_id_to_public_key("not-valid").is_err());
    }
}
