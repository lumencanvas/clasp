//! Entity token validator implementing the clasp-core TokenValidator trait

use std::sync::Arc;

use clasp_core::security::{Scope, TokenInfo, TokenValidator, ValidationResult};

use crate::entity::Entity;
use crate::store::EntityStore;
use crate::token::{parse_token, verify_token_signature, ENTITY_TOKEN_PREFIX};

/// Token validator for entity-signed tokens.
///
/// Plugs into the existing `ValidatorChain` alongside `CpskValidator`.
/// Token format: "ent_<base64url(msgpack(entity_id + timestamp + signature))>"
pub struct EntityValidator {
    store: Arc<dyn EntityStore>,
    /// Maximum token age in seconds (0 = no limit)
    max_token_age: u64,
}

impl EntityValidator {
    /// Create a new entity validator
    pub fn new(store: Arc<dyn EntityStore>) -> Self {
        Self {
            store,
            max_token_age: 0,
        }
    }

    /// Set maximum token age (tokens older than this are rejected)
    pub fn with_max_token_age(mut self, seconds: u64) -> Self {
        self.max_token_age = seconds;
        self
    }
}

impl TokenValidator for EntityValidator {
    fn validate(&self, token: &str) -> ValidationResult {
        // Check prefix -- if not ours, pass to next validator
        if !token.starts_with(ENTITY_TOKEN_PREFIX) {
            return ValidationResult::NotMyToken;
        }

        // Parse the token payload
        let payload = match parse_token(token) {
            Ok(p) => p,
            Err(e) => return ValidationResult::Invalid(format!("malformed entity token: {}", e)),
        };

        // Check token age if configured
        if self.max_token_age > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0);

            if now.saturating_sub(payload.timestamp) > self.max_token_age {
                return ValidationResult::Expired;
            }
        }

        // Look up entity -- we need to block on the async store
        // Since TokenValidator::validate is sync, we use a thread-local runtime
        // or assume we're called from within a tokio runtime
        let store = self.store.clone();
        let entity_id_str = payload.entity_id.clone();

        let entity_id = match crate::entity::EntityId::parse(&entity_id_str) {
            Ok(id) => id,
            Err(e) => return ValidationResult::Invalid(format!("invalid entity ID: {}", e)),
        };

        // Use tokio::task::block_in_place to call async from sync context
        let entity: Entity = match tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(store.get(&entity_id))
        }) {
            Ok(Some(e)) => e,
            Ok(None) => {
                return ValidationResult::Invalid(format!(
                    "entity not found: {}",
                    entity_id_str
                ))
            }
            Err(e) => return ValidationResult::Invalid(format!("store error: {}", e)),
        };

        // Check entity status
        if !entity.is_active() {
            return ValidationResult::Invalid(format!(
                "entity {} is {}",
                entity_id_str, entity.status
            ));
        }

        // Verify signature
        if let Err(e) = verify_token_signature(&payload, &entity.public_key) {
            return ValidationResult::Invalid(format!("signature error: {}", e));
        }

        // Build scopes from entity
        let scopes: Vec<Scope> = entity
            .scopes
            .iter()
            .filter_map(|s| Scope::parse(s).ok())
            .collect();

        // If no scopes defined, grant based on namespaces
        let scopes = if scopes.is_empty() && !entity.namespaces.is_empty() {
            entity
                .namespaces
                .iter()
                .filter_map(|ns: &String| {
                    let pattern = if ns.ends_with("/**") {
                        ns.clone()
                    } else if ns.ends_with('/') {
                        format!("{}**", ns)
                    } else {
                        format!("{}/**", ns)
                    };
                    Scope::parse(&format!("admin:{}", pattern)).ok()
                })
                .collect()
        } else {
            scopes
        };

        let info = TokenInfo::new(token.to_string(), scopes)
            .with_subject(entity_id_str)
            .with_metadata("entity_type", entity.entity_type.to_string())
            .with_metadata("entity_name", entity.name);

        ValidationResult::Valid(info)
    }

    fn name(&self) -> &str {
        "Entity"
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityKeypair, EntityType};
    use crate::store::MemoryEntityStore;
    use crate::token::generate_token;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_validator_valid() {
        let store = Arc::new(MemoryEntityStore::new());
        let keypair = EntityKeypair::generate().unwrap();
        let mut entity = keypair.to_entity(EntityType::Device, "test-device".to_string());
        entity.scopes = vec!["admin:/**".to_string()];
        store.create(&entity).await.unwrap();

        let validator = EntityValidator::new(store);
        let token = generate_token(&keypair).unwrap();

        match validator.validate(&token) {
            ValidationResult::Valid(info) => {
                assert!(info.has_scope(clasp_core::Action::Read, "/any/path"));
                assert_eq!(info.subject.as_deref(), Some(keypair.entity_id.as_str()));
            }
            other => panic!("expected Valid, got {:?}", other),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_validator_not_my_token() {
        let store = Arc::new(MemoryEntityStore::new());
        let validator = EntityValidator::new(store);

        match validator.validate("cpsk_some_token") {
            ValidationResult::NotMyToken => {}
            other => panic!("expected NotMyToken, got {:?}", other),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_validator_unknown_entity() {
        let store = Arc::new(MemoryEntityStore::new());
        let validator = EntityValidator::new(store);
        let keypair = EntityKeypair::generate().unwrap();
        let token = generate_token(&keypair).unwrap();

        match validator.validate(&token) {
            ValidationResult::Invalid(msg) => {
                assert!(msg.contains("not found"));
            }
            other => panic!("expected Invalid, got {:?}", other),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_validator_wrong_signature() {
        let store = Arc::new(MemoryEntityStore::new());
        let keypair = EntityKeypair::generate().unwrap();
        let entity = keypair.to_entity(EntityType::Device, "test-device".to_string());
        store.create(&entity).await.unwrap();

        // Generate token with a different keypair
        let other_keypair = EntityKeypair::generate().unwrap();
        // Manually create a token that claims to be the first entity but signed by the second
        let token = {
            use base64::engine::general_purpose::URL_SAFE_NO_PAD;
            use base64::Engine;
            use ed25519_dalek::Signer;

            let timestamp = 0u64;
            let entity_id = keypair.entity_id.as_str().to_string();
            let mut message = entity_id.as_bytes().to_vec();
            message.extend_from_slice(&timestamp.to_be_bytes());

            let signature = other_keypair.signing_key.sign(&message);

            let payload = crate::token::EntityTokenPayload {
                entity_id,
                timestamp,
                signature: signature.to_bytes().to_vec(),
            };

            let encoded = rmp_serde::to_vec(&payload).unwrap();
            format!("ent_{}", URL_SAFE_NO_PAD.encode(&encoded))
        };

        let validator = EntityValidator::new(store);
        match validator.validate(&token) {
            ValidationResult::Invalid(msg) => {
                assert!(msg.contains("signature"));
            }
            other => panic!("expected Invalid, got {:?}", other),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_validator_suspended() {
        let store = Arc::new(MemoryEntityStore::new());
        let keypair = EntityKeypair::generate().unwrap();
        let mut entity = keypair.to_entity(EntityType::Device, "test-device".to_string());
        entity.status = crate::entity::EntityStatus::Suspended;
        store.create(&entity).await.unwrap();

        let validator = EntityValidator::new(store);
        let token = generate_token(&keypair).unwrap();

        match validator.validate(&token) {
            ValidationResult::Invalid(msg) => {
                assert!(msg.contains("suspended"));
            }
            other => panic!("expected Invalid, got {:?}", other),
        }
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_entity_validator_namespace_scopes() {
        let store = Arc::new(MemoryEntityStore::new());
        let keypair = EntityKeypair::generate().unwrap();
        let mut entity = keypair.to_entity(EntityType::Device, "light-controller".to_string());
        entity.namespaces = vec!["/lights".to_string()];
        store.create(&entity).await.unwrap();

        let validator = EntityValidator::new(store);
        let token = generate_token(&keypair).unwrap();

        match validator.validate(&token) {
            ValidationResult::Valid(info) => {
                assert!(info.has_scope(clasp_core::Action::Admin, "/lights/room1"));
                assert!(!info.has_scope(clasp_core::Action::Read, "/audio/mixer"));
            }
            other => panic!("expected Valid, got {:?}", other),
        }
    }
}
