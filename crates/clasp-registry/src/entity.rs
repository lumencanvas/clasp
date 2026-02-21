//! Entity types for the CLASP registry

use ed25519_dalek::{SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::SystemTime;

use crate::error::{RegistryError, Result};

/// Entity ID format: "clasp:<base58-ed25519-pubkey-prefix>"
/// Uses first 16 bytes of the 32-byte public key for a shorter but still unique ID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(String);

impl EntityId {
    /// Create an EntityId from a public key
    pub fn from_public_key(key: &[u8]) -> Result<Self> {
        if key.len() != 32 {
            return Err(RegistryError::InvalidKey(format!(
                "expected 32-byte Ed25519 public key, got {} bytes",
                key.len()
            )));
        }
        let encoded = bs58::encode(&key[..16]).into_string();
        Ok(Self(format!("clasp:{}", encoded)))
    }

    /// Parse an EntityId from string
    pub fn parse(s: &str) -> Result<Self> {
        if !s.starts_with("clasp:") {
            return Err(RegistryError::InvalidId(format!(
                "entity ID must start with 'clasp:', got: {}",
                s
            )));
        }
        let suffix = &s[6..];
        // Validate base58
        bs58::decode(suffix)
            .into_vec()
            .map_err(|e| RegistryError::InvalidId(format!("invalid base58 in entity ID: {}", e)))?;
        Ok(Self(s.to_string()))
    }

    /// Get the raw string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for EntityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<EntityId> for String {
    fn from(id: EntityId) -> Self {
        id.0
    }
}

/// Type of entity in the registry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EntityType {
    Device,
    User,
    Service,
    Router,
}

impl fmt::Display for EntityType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityType::Device => write!(f, "device"),
            EntityType::User => write!(f, "user"),
            EntityType::Service => write!(f, "service"),
            EntityType::Router => write!(f, "router"),
        }
    }
}

/// Status of an entity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum EntityStatus {
    #[default]
    Active,
    Suspended,
    Revoked,
}

impl fmt::Display for EntityStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EntityStatus::Active => write!(f, "active"),
            EntityStatus::Suspended => write!(f, "suspended"),
            EntityStatus::Revoked => write!(f, "revoked"),
        }
    }
}

/// A registered entity in the CLASP network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub entity_type: EntityType,
    pub name: String,
    #[serde(with = "hex_bytes")]
    pub public_key: Vec<u8>,
    pub created_at: SystemTime,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub namespaces: Vec<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    #[serde(default)]
    pub status: EntityStatus,
}

impl Entity {
    /// Check if this entity is currently active
    pub fn is_active(&self) -> bool {
        self.status == EntityStatus::Active
    }
}

/// An entity keypair (private + public key)
pub struct EntityKeypair {
    pub entity_id: EntityId,
    pub signing_key: SigningKey,
    pub verifying_key: VerifyingKey,
}

impl EntityKeypair {
    /// Generate a new random keypair
    pub fn generate() -> Result<Self> {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let entity_id = EntityId::from_public_key(verifying_key.as_bytes())?;

        Ok(Self {
            entity_id,
            signing_key,
            verifying_key,
        })
    }

    /// Create from an existing signing key
    pub fn from_signing_key(signing_key: SigningKey) -> Result<Self> {
        let verifying_key = signing_key.verifying_key();
        let entity_id = EntityId::from_public_key(verifying_key.as_bytes())?;

        Ok(Self {
            entity_id,
            signing_key,
            verifying_key,
        })
    }

    /// Get the public key bytes
    pub fn public_key_bytes(&self) -> &[u8] {
        self.verifying_key.as_bytes()
    }

    /// Create an Entity from this keypair
    pub fn to_entity(&self, entity_type: EntityType, name: String) -> Entity {
        Entity {
            id: self.entity_id.clone(),
            entity_type,
            name,
            public_key: self.verifying_key.as_bytes().to_vec(),
            created_at: SystemTime::now(),
            metadata: HashMap::new(),
            tags: Vec::new(),
            namespaces: Vec::new(),
            scopes: Vec::new(),
            status: EntityStatus::Active,
        }
    }
}

impl fmt::Debug for EntityKeypair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EntityKeypair")
            .field("entity_id", &self.entity_id)
            .field("verifying_key", &"[redacted]")
            .finish()
    }
}

/// Serde helper for hex-encoded byte arrays
mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &Vec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string: String = bytes.iter().map(|b| format!("{:02x}", b)).collect();
        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(serde::de::Error::custom))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_from_public_key() {
        let keypair = EntityKeypair::generate().unwrap();
        let id = &keypair.entity_id;
        assert!(id.as_str().starts_with("clasp:"));
        assert!(id.as_str().len() > 10);
    }

    #[test]
    fn test_entity_id_parse() {
        let keypair = EntityKeypair::generate().unwrap();
        let id_str = keypair.entity_id.as_str();
        let parsed = EntityId::parse(id_str).unwrap();
        assert_eq!(parsed, keypair.entity_id);
    }

    #[test]
    fn test_entity_id_parse_invalid() {
        assert!(EntityId::parse("invalid").is_err());
        assert!(EntityId::parse("clasp:!!!").is_err());
    }

    #[test]
    fn test_keypair_generate() {
        let kp1 = EntityKeypair::generate().unwrap();
        let kp2 = EntityKeypair::generate().unwrap();
        assert_ne!(kp1.entity_id, kp2.entity_id);
    }

    #[test]
    fn test_to_entity() {
        let keypair = EntityKeypair::generate().unwrap();
        let entity = keypair.to_entity(EntityType::Device, "test-device".to_string());
        assert_eq!(entity.id, keypair.entity_id);
        assert_eq!(entity.entity_type, EntityType::Device);
        assert_eq!(entity.name, "test-device");
        assert!(entity.is_active());
    }

    #[test]
    fn test_entity_status() {
        let keypair = EntityKeypair::generate().unwrap();
        let mut entity = keypair.to_entity(EntityType::User, "test-user".to_string());
        assert!(entity.is_active());

        entity.status = EntityStatus::Suspended;
        assert!(!entity.is_active());

        entity.status = EntityStatus::Revoked;
        assert!(!entity.is_active());
    }
}
