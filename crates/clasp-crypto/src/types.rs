use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

/// E2E encrypted envelope that flows through CLASP as a normal map value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct E2EEnvelope {
    /// Marker field, always 1.
    pub _e2e: u8,
    /// Base64-encoded ciphertext.
    pub ct: String,
    /// Base64-encoded IV (12 bytes for AES-GCM).
    pub iv: String,
    /// Envelope version.
    pub v: u8,
}

/// Stored key material with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyData {
    /// The group key in JWK JSON format (interop with JS).
    pub key: serde_json::Value,
    /// When this key was stored (Unix ms).
    pub stored_at: u64,
}

/// Stored TOFU fingerprint record.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TofuRecord {
    /// Hex fingerprint of the peer's ECDH public key.
    pub fingerprint: String,
    /// When this key was first seen (Unix ms).
    pub first_seen: u64,
}

/// ECDH key pair: public key (SEC1 encoded) + private key (scalar bytes).
/// Private key material is zeroed on drop. Not `Clone` to prevent
/// uncontrolled duplication of private key material.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct ECDHKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

/// ECDSA signing key pair.
/// Private key material is zeroed on drop. Not `Clone` to prevent
/// uncontrolled duplication of private key material.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SigningKeyPair {
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
}

/// Key exchange message sent between peers (camelCase for JS interop).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct KeyExchangeMessage {
    pub from_id: String,
    pub encrypted_key: String,
    pub iv: String,
    /// ECDH public key in JWK JSON format (interop with JS).
    pub sender_public_key: serde_json::Value,
}

/// Public key announcement (camelCase for JS interop).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicKeyAnnouncement {
    /// ECDH public key in JWK JSON format.
    pub public_key: serde_json::Value,
    pub timestamp: u64,
}
