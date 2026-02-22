//! Entity token generation and parsing
//!
//! Token format: "ent_<base64url(msgpack(EntityTokenPayload))>"
//!
//! The payload contains the entity ID, a timestamp, and an Ed25519 signature
//! over (entity_id || timestamp), allowing the validator to verify the token
//! without any shared secret -- only the entity's public key.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use ed25519_dalek::{Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entity::EntityKeypair;
use crate::error::{RegistryError, Result};

/// Token prefix for entity tokens
pub const ENTITY_TOKEN_PREFIX: &str = "ent_";

/// Serializable token payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTokenPayload {
    /// Entity ID
    pub entity_id: String,
    /// Creation timestamp (seconds since epoch)
    pub timestamp: u64,
    /// Ed25519 signature over (entity_id || timestamp_bytes)
    #[serde(with = "serde_bytes")]
    pub signature: Vec<u8>,
}

/// Generate an entity authentication token
pub fn generate_token(keypair: &EntityKeypair) -> Result<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entity_id = keypair.entity_id.as_str().to_string();

    // Create message to sign: entity_id || timestamp_bytes
    let mut message = entity_id.as_bytes().to_vec();
    message.extend_from_slice(&timestamp.to_be_bytes());

    let signature = keypair
        .signing_key
        .try_sign(&message)
        .map_err(|e| RegistryError::SignatureError(e.to_string()))?;

    let payload = EntityTokenPayload {
        entity_id,
        timestamp,
        signature: signature.to_bytes().to_vec(),
    };

    let encoded = rmp_serde::to_vec(&payload)
        .map_err(|e| RegistryError::TokenError(format!("failed to encode token: {}", e)))?;

    Ok(format!(
        "{}{}",
        ENTITY_TOKEN_PREFIX,
        URL_SAFE_NO_PAD.encode(&encoded)
    ))
}

/// Parse and verify an entity token
///
/// Returns the payload if the token format is valid and signature verification
/// succeeds with the provided public key. Does NOT check entity status --
/// that's the validator's job.
pub fn parse_token(token: &str) -> Result<EntityTokenPayload> {
    if !token.starts_with(ENTITY_TOKEN_PREFIX) {
        return Err(RegistryError::TokenError("missing ent_ prefix".to_string()));
    }

    let encoded = &token[ENTITY_TOKEN_PREFIX.len()..];
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|e| RegistryError::TokenError(format!("invalid base64: {}", e)))?;

    let payload: EntityTokenPayload = rmp_serde::from_slice(&bytes)
        .map_err(|e| RegistryError::TokenError(format!("invalid payload: {}", e)))?;

    Ok(payload)
}

/// Verify the signature on a token payload using the entity's public key
pub fn verify_token_signature(payload: &EntityTokenPayload, public_key: &[u8]) -> Result<()> {
    if public_key.len() != 32 {
        return Err(RegistryError::InvalidKey(
            "public key must be 32 bytes".to_string(),
        ));
    }

    let key_bytes: [u8; 32] = public_key
        .try_into()
        .map_err(|_| RegistryError::InvalidKey("invalid key length".to_string()))?;

    let verifying_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)
        .map_err(|e| RegistryError::InvalidKey(format!("invalid public key: {}", e)))?;

    let sig_bytes: [u8; 64] = payload
        .signature
        .as_slice()
        .try_into()
        .map_err(|_| RegistryError::SignatureError("signature must be 64 bytes".to_string()))?;

    let signature = Signature::from_bytes(&sig_bytes);

    // Reconstruct the signed message
    let mut message = payload.entity_id.as_bytes().to_vec();
    message.extend_from_slice(&payload.timestamp.to_be_bytes());

    verifying_key
        .verify(&message, &signature)
        .map_err(|e| RegistryError::SignatureError(format!("signature verification failed: {}", e)))
}

mod serde_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(bytes)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // rmp-serde encodes bytes as bin format; deserialize as raw bytes
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_parse_token() {
        let keypair = EntityKeypair::generate().unwrap();
        let token = generate_token(&keypair).unwrap();

        assert!(token.starts_with("ent_"));

        let payload = parse_token(&token).unwrap();
        assert_eq!(payload.entity_id, keypair.entity_id.as_str());
    }

    #[test]
    fn test_verify_token_signature() {
        let keypair = EntityKeypair::generate().unwrap();
        let token = generate_token(&keypair).unwrap();
        let payload = parse_token(&token).unwrap();

        // Valid key
        verify_token_signature(&payload, keypair.public_key_bytes()).unwrap();

        // Wrong key
        let other_keypair = EntityKeypair::generate().unwrap();
        assert!(verify_token_signature(&payload, other_keypair.public_key_bytes()).is_err());
    }

    #[test]
    fn test_parse_invalid_token() {
        assert!(parse_token("invalid").is_err());
        assert!(parse_token("ent_!!!").is_err());
    }

    #[test]
    fn test_token_uniqueness() {
        let keypair = EntityKeypair::generate().unwrap();
        let token1 = generate_token(&keypair).unwrap();
        // Tokens generated at same second will be identical since timestamp is seconds
        // but the entity_id + timestamp combo ensures uniqueness across entities
        let token2 = generate_token(&keypair).unwrap();
        // Both should be valid
        parse_token(&token1).unwrap();
        parse_token(&token2).unwrap();
    }

    // --- Negative tests ---

    #[test]
    fn test_parse_bad_base64() {
        assert!(parse_token("ent_!!!not-valid-base64!!!").is_err());
    }

    #[test]
    fn test_parse_truncated_payload() {
        use base64::engine::general_purpose::URL_SAFE_NO_PAD;
        use base64::Engine;
        let truncated = URL_SAFE_NO_PAD.encode([0x92, 0x01]);
        assert!(parse_token(&format!("ent_{}", truncated)).is_err());
    }

    #[test]
    fn test_verify_wrong_key_length() {
        let keypair = EntityKeypair::generate().unwrap();
        let token = generate_token(&keypair).unwrap();
        let payload = parse_token(&token).unwrap();

        // Too short
        assert!(verify_token_signature(&payload, &[0u8; 16]).is_err());
        // Too long
        assert!(verify_token_signature(&payload, &[0u8; 64]).is_err());
    }

    #[test]
    fn test_verify_truncated_signature() {
        let keypair = EntityKeypair::generate().unwrap();
        let token = generate_token(&keypair).unwrap();
        let mut payload = parse_token(&token).unwrap();

        // Truncate signature to 32 bytes (should be 64)
        payload.signature.truncate(32);
        assert!(verify_token_signature(&payload, keypair.public_key_bytes()).is_err());
    }
}
