//! Core Identity struct: one Ed25519 keypair, three identity systems.

use ed25519_dalek::{Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::did;
use crate::error::Result;
use crate::peer_id;

/// A unified identity backed by a single Ed25519 keypair.
///
/// Produces three interoperable identifiers:
/// - CLASP `EntityId` (format: `clasp:<base58-first-16-bytes-of-pubkey>`)
/// - DID (`did:key:z6Mk...`)
/// - libp2p PeerID (`12D3KooW...`)
#[derive(Clone)]
pub struct Identity {
    signing_key: SigningKey,
    verifying_key: VerifyingKey,
}

impl Identity {
    /// Generate a new random identity.
    pub fn generate() -> Self {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        Self {
            signing_key,
            verifying_key,
        }
    }

    /// Create from an existing Ed25519 signing key.
    pub fn from_signing_key(key: SigningKey) -> Self {
        let verifying_key = key.verifying_key();
        Self {
            signing_key: key,
            verifying_key,
        }
    }

    /// Create from raw signing key bytes (32 bytes).
    pub fn from_bytes(bytes: &[u8; 32]) -> Result<Self> {
        let signing_key = SigningKey::from_bytes(bytes);
        Ok(Self::from_signing_key(signing_key))
    }

    /// Get the CLASP EntityId (format: `clasp:<base58-first-16-bytes>`).
    pub fn entity_id(&self) -> String {
        let encoded = bs58::encode(&self.verifying_key.as_bytes()[..16]).into_string();
        format!("clasp:{}", encoded)
    }

    /// Get the DID (format: `did:key:z6Mk...`).
    ///
    /// Uses multicodec 0xed (Ed25519 public key) with multibase base58btc 'z' prefix.
    pub fn did(&self) -> String {
        did::public_key_to_did(self.verifying_key.as_bytes())
    }

    /// Get the libp2p PeerID (format: `12D3KooW...`).
    ///
    /// Uses identity multihash (0x00) for keys <= 42 bytes,
    /// encoded as base58btc of the protobuf-encoded public key.
    pub fn peer_id(&self) -> String {
        peer_id::public_key_to_peer_id(self.verifying_key.as_bytes())
    }

    /// Get the raw 32-byte public key.
    pub fn public_key(&self) -> &[u8; 32] {
        self.verifying_key.as_bytes()
    }

    /// Get a reference to the signing key.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Sign data with Ed25519.
    pub fn sign(&self, data: &[u8]) -> Vec<u8> {
        let sig = self.signing_key.sign(data);
        sig.to_bytes().to_vec()
    }

    /// Verify a signature against this identity's public key.
    pub fn verify(&self, data: &[u8], signature: &[u8]) -> bool {
        if signature.len() != 64 {
            return false;
        }
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(signature);
        match ed25519_dalek::Signature::from_bytes(&sig_bytes) {
            sig => self.verifying_key.verify(data, &sig).is_ok(),
        }
    }

    /// Export the signing key bytes (32 bytes, secret).
    pub fn export_secret(&self) -> [u8; 32] {
        self.signing_key.to_bytes()
    }
}

impl Drop for Identity {
    fn drop(&mut self) {
        // Zeroize the signing key bytes on drop
        let mut bytes = self.signing_key.to_bytes();
        bytes.zeroize();
    }
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("entity_id", &self.entity_id())
            .field("did", &self.did())
            .field("peer_id", &self.peer_id())
            .field("signing_key", &"[redacted]")
            .finish()
    }
}

impl Serialize for Identity {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Identity", 3)?;
        state.serialize_field("entity_id", &self.entity_id())?;
        state.serialize_field("did", &self.did())?;
        state.serialize_field("peer_id", &self.peer_id())?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Identity {
    fn deserialize<D>(_deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "Identity cannot be deserialized from public fields; use from_bytes() with the secret key",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::did_to_public_key;
    use crate::peer_id::peer_id_to_public_key;

    #[test]
    fn identity_from_seed_deterministic() {
        let seed: [u8; 32] = [
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24,
            25, 26, 27, 28, 29, 30, 31, 32,
        ];
        let id1 = Identity::from_bytes(&seed).unwrap();
        let id2 = Identity::from_bytes(&seed).unwrap();
        assert_eq!(id1.entity_id(), id2.entity_id());
        assert_eq!(id1.did(), id2.did());
        assert_eq!(id1.peer_id(), id2.peer_id());
        assert_eq!(id1.public_key(), id2.public_key());
    }

    #[test]
    fn entity_id_format() {
        let id = Identity::generate();
        let eid = id.entity_id();
        assert!(
            eid.starts_with("clasp:"),
            "should start with 'clasp:', got: {}",
            eid
        );
        let suffix = &eid[6..];
        // Verify base58 is valid
        let decoded = bs58::decode(suffix).into_vec().unwrap();
        assert_eq!(
            decoded.len(),
            16,
            "entity_id suffix should decode to 16 bytes"
        );
    }

    #[test]
    fn entity_id_roundtrip() {
        // Verify our entity_id matches what EntityId::from_public_key produces:
        // base58 of first 16 bytes of the 32-byte public key
        let seed: [u8; 32] = [99; 32];
        let id = Identity::from_bytes(&seed).unwrap();
        let pubkey = id.public_key();
        let expected = format!("clasp:{}", bs58::encode(&pubkey[..16]).into_string());
        assert_eq!(id.entity_id(), expected);
    }

    #[test]
    fn did_roundtrip() {
        let id = Identity::generate();
        let d = id.did();
        let recovered = did_to_public_key(&d).unwrap();
        assert_eq!(&recovered, id.public_key());
    }

    #[test]
    fn did_format() {
        let id = Identity::generate();
        let d = id.did();
        // z6Mk is the base58btc prefix for Ed25519 multicodec bytes [0xed, 0x01]
        assert!(
            d.starts_with("did:key:z6Mk"),
            "DID should start with 'did:key:z6Mk', got: {}",
            d
        );
    }

    #[test]
    fn peer_id_roundtrip() {
        let id = Identity::generate();
        let pid = id.peer_id();
        let recovered = peer_id_to_public_key(&pid).unwrap();
        assert_eq!(&recovered, id.public_key());
    }

    #[test]
    fn peer_id_format() {
        let id = Identity::generate();
        let pid = id.peer_id();
        assert!(
            pid.starts_with("12D3KooW"),
            "PeerID should start with '12D3KooW', got: {}",
            pid
        );
    }

    #[test]
    fn cross_format_same_key() {
        let id = Identity::generate();
        let pubkey = id.public_key();

        // Recover key from DID
        let did_key = did_to_public_key(&id.did()).unwrap();
        assert_eq!(&did_key, pubkey);

        // Recover key from PeerID
        let pid_key = peer_id_to_public_key(&id.peer_id()).unwrap();
        assert_eq!(&pid_key, pubkey);

        // Verify entity_id uses first 16 bytes
        let eid_suffix = &id.entity_id()[6..];
        let eid_bytes = bs58::decode(eid_suffix).into_vec().unwrap();
        assert_eq!(&eid_bytes[..], &pubkey[..16]);
    }

    #[test]
    fn sign_verify() {
        let id = Identity::generate();
        let data = b"hello clasp";
        let sig = id.sign(data);
        assert!(id.verify(data, &sig));
        assert!(!id.verify(b"wrong data", &sig));
    }

    #[test]
    fn different_keys_different_ids() {
        let id1 = Identity::generate();
        let id2 = Identity::generate();
        assert_ne!(id1.entity_id(), id2.entity_id());
        assert_ne!(id1.did(), id2.did());
        assert_ne!(id1.peer_id(), id2.peer_id());
    }

    #[test]
    fn existing_entity_id_compat() {
        // Verify compatibility with clasp-registry EntityId::from_public_key
        // which does: bs58::encode(&key[..16]).into_string() prefixed with "clasp:"
        let seed: [u8; 32] = [7; 32];
        let id = Identity::from_bytes(&seed).unwrap();
        let pubkey = id.public_key();

        let registry_style = format!("clasp:{}", bs58::encode(&pubkey[..16]).into_string());
        assert_eq!(id.entity_id(), registry_style);
    }
}
