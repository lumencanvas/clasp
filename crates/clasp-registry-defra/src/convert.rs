//! Conversion helpers between CLASP entity types and DefraDB representations.

use std::collections::HashMap;
use std::time::{Duration, UNIX_EPOCH};

use clasp_registry::{Entity, EntityId, EntityStatus, EntityType};

use crate::error::DefraRegistryError;

/// Convert an [`Entity`] to a DefraDB-compatible JSON value for GraphQL input.
pub fn entity_to_defra(entity: &Entity) -> serde_json::Value {
    let created_at = entity
        .created_at
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_secs() as i64;

    let metadata = serde_json::to_string(&entity.metadata).unwrap_or_else(|_| "{}".to_string());

    serde_json::json!({
        "entityId": entity.id.as_str(),
        "entityType": entity.entity_type.to_string(),
        "name": entity.name,
        "publicKey": hex_encode(&entity.public_key),
        "createdAt": created_at,
        "metadata": metadata,
        "tags": entity.tags,
        "namespaces": entity.namespaces,
        "scopes": entity.scopes,
        "status": entity.status.to_string(),
    })
}

/// Convert a DefraDB document back into an [`Entity`].
pub fn defra_to_entity(doc: &serde_json::Value) -> std::result::Result<Entity, DefraRegistryError> {
    let entity_id = doc
        .get("entityId")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DefraRegistryError::Deserialization("missing entityId".into()))?;

    let entity_type_str = doc
        .get("entityType")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DefraRegistryError::Deserialization("missing entityType".into()))?;

    let name = doc
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DefraRegistryError::Deserialization("missing name".into()))?;

    let public_key_hex = doc
        .get("publicKey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| DefraRegistryError::Deserialization("missing publicKey".into()))?;

    let created_at_secs = doc
        .get("createdAt")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    let metadata_str = doc
        .get("metadata")
        .and_then(|v| v.as_str())
        .unwrap_or("{}");

    let tags = json_str_array(doc.get("tags"));
    let namespaces = json_str_array(doc.get("namespaces"));
    let scopes = json_str_array(doc.get("scopes"));

    let status_str = doc
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("active");

    let id = EntityId::parse(entity_id).map_err(|e| {
        DefraRegistryError::Deserialization(format!("invalid entityId: {e}"))
    })?;

    let entity_type = parse_entity_type(entity_type_str).ok_or_else(|| {
        DefraRegistryError::Deserialization(format!("unknown entityType: {entity_type_str}"))
    })?;

    let public_key = hex_decode(public_key_hex).ok_or_else(|| {
        DefraRegistryError::Deserialization(format!("invalid hex publicKey: {public_key_hex}"))
    })?;

    let created_at = UNIX_EPOCH + Duration::from_secs(created_at_secs as u64);

    let metadata: HashMap<String, String> =
        serde_json::from_str(metadata_str).unwrap_or_default();

    let status = parse_entity_status(status_str);

    Ok(Entity {
        id,
        entity_type,
        name: name.to_string(),
        public_key,
        created_at,
        metadata,
        tags,
        namespaces,
        scopes,
        status,
    })
}

/// Hex-encode a byte slice.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Hex-decode a string into bytes. Returns `None` on invalid hex.
pub fn hex_decode(hex: &str) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).ok())
        .collect()
}

fn parse_entity_type(s: &str) -> Option<EntityType> {
    match s {
        "device" => Some(EntityType::Device),
        "user" => Some(EntityType::User),
        "service" => Some(EntityType::Service),
        "router" => Some(EntityType::Router),
        _ => None,
    }
}

fn parse_entity_status(s: &str) -> EntityStatus {
    match s {
        "suspended" => EntityStatus::Suspended,
        "revoked" => EntityStatus::Revoked,
        _ => EntityStatus::Active,
    }
}

fn json_str_array(val: Option<&serde_json::Value>) -> Vec<String> {
    val.and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_test_entity() -> Entity {
        Entity {
            id: EntityId::parse("clasp:3vQB7B6mGGskg").unwrap(),
            entity_type: EntityType::Device,
            name: "test-device".to_string(),
            public_key: vec![
                0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
                0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a,
                0x1b, 0x1c, 0x1d, 0x1e, 0x1f, 0x20,
            ],
            created_at: UNIX_EPOCH + Duration::from_secs(1700000000),
            metadata: {
                let mut m = HashMap::new();
                m.insert("location".to_string(), "stage-left".to_string());
                m.insert("firmware".to_string(), "2.1.0".to_string());
                m
            },
            tags: vec!["lighting".to_string(), "dmx".to_string()],
            namespaces: vec!["/venue/main".to_string()],
            scopes: vec!["read".to_string(), "write".to_string()],
            status: EntityStatus::Active,
        }
    }

    #[test]
    fn convert_entity_roundtrip() {
        let entity = make_test_entity();
        let defra_doc = entity_to_defra(&entity);
        let restored = defra_to_entity(&defra_doc).unwrap();

        assert_eq!(restored.id, entity.id);
        assert_eq!(restored.entity_type, entity.entity_type);
        assert_eq!(restored.name, entity.name);
        assert_eq!(restored.public_key, entity.public_key);
        assert_eq!(restored.created_at, entity.created_at);
        assert_eq!(restored.metadata, entity.metadata);
        assert_eq!(restored.tags, entity.tags);
        assert_eq!(restored.namespaces, entity.namespaces);
        assert_eq!(restored.scopes, entity.scopes);
        assert_eq!(restored.status, entity.status);
    }

    #[test]
    fn convert_entity_types() {
        let types = [
            (EntityType::Device, "device"),
            (EntityType::User, "user"),
            (EntityType::Service, "service"),
            (EntityType::Router, "router"),
        ];

        for (et, expected_str) in types {
            assert_eq!(et.to_string(), expected_str);
            assert_eq!(parse_entity_type(expected_str), Some(et));
        }

        assert_eq!(parse_entity_type("unknown"), None);
    }

    #[test]
    fn convert_entity_status() {
        let statuses = [
            (EntityStatus::Active, "active"),
            (EntityStatus::Suspended, "suspended"),
            (EntityStatus::Revoked, "revoked"),
        ];

        for (status, expected_str) in statuses {
            assert_eq!(status.to_string(), expected_str);
            assert_eq!(parse_entity_status(expected_str), status);
        }

        // Unknown falls back to Active
        assert_eq!(parse_entity_status("bogus"), EntityStatus::Active);
    }

    #[test]
    fn convert_public_key_hex() {
        let key = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF];
        let hex = hex_encode(&key);
        assert_eq!(hex, "deadbeef00ff");
        let decoded = hex_decode(&hex).unwrap();
        assert_eq!(decoded, key);
    }

    #[test]
    fn convert_public_key_hex_invalid() {
        assert!(hex_decode("zzzz").is_none());
        assert!(hex_decode("abc").is_none()); // odd length
    }

    #[test]
    fn convert_metadata() {
        let mut map = HashMap::new();
        map.insert("key1".to_string(), "value1".to_string());
        map.insert("key2".to_string(), "value2".to_string());

        let json_str = serde_json::to_string(&map).unwrap();
        let restored: HashMap<String, String> = serde_json::from_str(&json_str).unwrap();
        assert_eq!(restored, map);
    }
}
