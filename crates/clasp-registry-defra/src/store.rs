//! DefraDB-backed entity store implementing the `EntityStore` trait.

use async_trait::async_trait;
use tracing::debug;

use clasp_journal_defra::{json_to_graphql_input, DefraClient};
use clasp_registry::error::{RegistryError, Result};
use clasp_registry::store::EntityStore;
use clasp_registry::{Entity, EntityId, EntityStatus};

use crate::convert::{defra_to_entity, entity_to_defra, hex_encode};
use crate::schema::ENTITY_SCHEMA;

/// DefraDB-backed entity store.
///
/// Uses the shared [`DefraClient`] from `clasp-journal-defra` for all
/// HTTP/GraphQL communication. The schema is provisioned automatically
/// on construction via [`DefraEntityStore::connect`].
pub struct DefraEntityStore {
    client: DefraClient,
}

impl DefraEntityStore {
    /// Connect to a DefraDB instance and provision the entity schema.
    ///
    /// The schema provision is idempotent -- calling this multiple times
    /// against the same DefraDB instance is safe.
    pub async fn connect(defra_url: &str) -> Result<Self> {
        let client = DefraClient::new(defra_url);

        client.add_schema(ENTITY_SCHEMA).await.map_err(|e| {
            RegistryError::StorageError(format!("schema provisioning failed: {e}"))
        })?;

        debug!(url = %defra_url, "DefraDB entity store connected");

        Ok(Self { client })
    }

    /// Look up a single entity document by its entityId field.
    ///
    /// Returns the raw DefraDB document (including `_docID`) or `None`.
    async fn find_doc_by_entity_id(
        &self,
        entity_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let query = r#"
            query {
                ClaspEntity(filter: { entityId: { _eq: $id } }) {
                    _docID
                    entityId
                    entityType
                    name
                    publicKey
                    createdAt
                    metadata
                    tags
                    namespaces
                    scopes
                    status
                }
            }
        "#
        .replace("$id", &format!("\"{}\"", entity_id));

        let data = self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("query failed: {e}"))
        })?;

        let docs = data
            .get("ClaspEntity")
            .and_then(|v| v.as_array());

        match docs {
            Some(arr) if !arr.is_empty() => Ok(Some(arr[0].clone())),
            _ => Ok(None),
        }
    }

    /// Extract the DefraDB `_docID` from a document value.
    fn doc_id(doc: &serde_json::Value) -> Result<String> {
        doc.get("_docID")
            .and_then(|v| v.as_str())
            .map(String::from)
            .ok_or_else(|| RegistryError::StorageError("missing _docID in document".into()))
    }
}

#[async_trait]
impl EntityStore for DefraEntityStore {
    async fn create(&self, entity: &Entity) -> Result<()> {
        // Check for existing entity with same ID
        if self.find_doc_by_entity_id(entity.id.as_str()).await?.is_some() {
            return Err(RegistryError::AlreadyExists(entity.id.as_str().to_string()));
        }

        let input = entity_to_defra(entity);
        let input_gql = json_to_graphql_input(&input);

        let query = format!(
            r#"mutation {{
                add_ClaspEntity(input: {}) {{
                    _docID
                }}
            }}"#,
            input_gql
        );

        self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("create failed: {e}"))
        })?;

        debug!(entity_id = %entity.id, "entity created in DefraDB");
        Ok(())
    }

    async fn get(&self, id: &EntityId) -> Result<Option<Entity>> {
        match self.find_doc_by_entity_id(id.as_str()).await? {
            Some(doc) => {
                let entity = defra_to_entity(&doc).map_err(RegistryError::from)?;
                Ok(Some(entity))
            }
            None => Ok(None),
        }
    }

    async fn find_by_public_key(&self, key: &[u8]) -> Result<Option<Entity>> {
        let hex_key = hex_encode(key);

        let query = format!(
            r#"query {{
                ClaspEntity(filter: {{ publicKey: {{ _eq: "{hex_key}" }} }}) {{
                    entityId
                    entityType
                    name
                    publicKey
                    createdAt
                    metadata
                    tags
                    namespaces
                    scopes
                    status
                }}
            }}"#
        );

        let data = self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("query failed: {e}"))
        })?;

        let docs = data
            .get("ClaspEntity")
            .and_then(|v| v.as_array());

        match docs {
            Some(arr) if !arr.is_empty() => {
                let entity = defra_to_entity(&arr[0]).map_err(RegistryError::from)?;
                Ok(Some(entity))
            }
            _ => Ok(None),
        }
    }

    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Entity>> {
        // DefraDB array filtering: query all and filter client-side
        // as array contains semantics vary across DefraDB versions
        let query = r#"query {
            ClaspEntity {
                entityId
                entityType
                name
                publicKey
                createdAt
                metadata
                tags
                namespaces
                scopes
                status
            }
        }"#;

        let data = self.client.graphql(query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("query failed: {e}"))
        })?;

        let docs = data
            .get("ClaspEntity")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for doc in &docs {
            let tags = doc
                .get("tags")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if tags.iter().any(|t| *t == tag) {
                let entity = defra_to_entity(doc).map_err(RegistryError::from)?;
                results.push(entity);
            }
        }

        Ok(results)
    }

    async fn find_by_namespace(&self, namespace: &str) -> Result<Vec<Entity>> {
        // Same approach as find_by_tag: query all, filter client-side
        let query = r#"query {
            ClaspEntity {
                entityId
                entityType
                name
                publicKey
                createdAt
                metadata
                tags
                namespaces
                scopes
                status
            }
        }"#;

        let data = self.client.graphql(query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("query failed: {e}"))
        })?;

        let docs = data
            .get("ClaspEntity")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut results = Vec::new();
        for doc in &docs {
            let namespaces = doc
                .get("namespaces")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if namespaces.iter().any(|ns| *ns == namespace) {
                let entity = defra_to_entity(doc).map_err(RegistryError::from)?;
                results.push(entity);
            }
        }

        Ok(results)
    }

    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<Entity>> {
        let query = format!(
            r#"query {{
                ClaspEntity(offset: {offset}, limit: {limit}) {{
                    entityId
                    entityType
                    name
                    publicKey
                    createdAt
                    metadata
                    tags
                    namespaces
                    scopes
                    status
                }}
            }}"#
        );

        let data = self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("query failed: {e}"))
        })?;

        let docs = data
            .get("ClaspEntity")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        docs.iter()
            .map(|doc| defra_to_entity(doc).map_err(RegistryError::from))
            .collect()
    }

    async fn update(&self, entity: &Entity) -> Result<()> {
        let doc = self
            .find_doc_by_entity_id(entity.id.as_str())
            .await?
            .ok_or_else(|| RegistryError::NotFound(entity.id.as_str().to_string()))?;

        let doc_id = Self::doc_id(&doc)?;
        let input = entity_to_defra(entity);
        let input_gql = json_to_graphql_input(&input);

        let query = format!(
            r#"mutation {{
                update_ClaspEntity(docID: "{doc_id}", input: {}) {{
                    _docID
                }}
            }}"#,
            input_gql
        );

        self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("update failed: {e}"))
        })?;

        debug!(entity_id = %entity.id, "entity updated in DefraDB");
        Ok(())
    }

    async fn update_status(&self, id: &EntityId, status: EntityStatus) -> Result<()> {
        let doc = self
            .find_doc_by_entity_id(id.as_str())
            .await?
            .ok_or_else(|| RegistryError::NotFound(id.as_str().to_string()))?;

        let doc_id = Self::doc_id(&doc)?;

        let query = format!(
            r#"mutation {{
                update_ClaspEntity(docID: "{doc_id}", input: {{ status: "{status}" }}) {{
                    _docID
                }}
            }}"#
        );

        self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("update_status failed: {e}"))
        })?;

        debug!(entity_id = %id, %status, "entity status updated in DefraDB");
        Ok(())
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        let doc = match self.find_doc_by_entity_id(id.as_str()).await? {
            Some(doc) => doc,
            None => return Ok(false),
        };

        let doc_id = Self::doc_id(&doc)?;

        let query = format!(
            r#"mutation {{
                delete_ClaspEntity(docID: "{doc_id}") {{
                    _docID
                }}
            }}"#
        );

        self.client.graphql(&query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("delete failed: {e}"))
        })?;

        debug!(entity_id = %id, "entity deleted from DefraDB");
        Ok(true)
    }

    async fn count(&self) -> Result<usize> {
        let query = r#"query {
            ClaspEntity {
                entityId
            }
        }"#;

        let data = self.client.graphql(query, None).await.map_err(|e| {
            RegistryError::StorageError(format!("count query failed: {e}"))
        })?;

        let count = data
            .get("ClaspEntity")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clasp_registry::{EntityId, EntityStatus, EntityType};
    use std::collections::HashMap;
    use std::time::{Duration, UNIX_EPOCH};

    fn make_test_entity(name: &str) -> Entity {
        // Generate unique key per call so tests don't collide
        let mut key = [0u8; 32];
        let nanos = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH).unwrap().as_nanos();
        key[..16].copy_from_slice(&nanos.to_le_bytes());
        let id = EntityId::from_public_key(&key).unwrap();
        Entity {
            id,
            entity_type: EntityType::Device,
            name: name.to_string(),
            public_key: key.to_vec(),
            created_at: UNIX_EPOCH + Duration::from_secs(1700000000),
            metadata: HashMap::new(),
            tags: vec!["lighting".to_string()],
            namespaces: vec!["/venue/main".to_string()],
            scopes: vec!["read".to_string()],
            status: EntityStatus::Active,
        }
    }

    // -- Integration tests (require a running DefraDB instance) ----------------

    #[tokio::test]
    #[ignore]
    async fn test_create_and_get() {
        let store = DefraEntityStore::connect("http://localhost:9181")
            .await
            .unwrap();
        let entity = make_test_entity("integ-device");

        // Clean up from previous runs
        let _ = store.delete(&entity.id).await;

        store.create(&entity).await.unwrap();

        let found = store.get(&entity.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "integ-device");
        assert_eq!(found.entity_type, EntityType::Device);
        assert_eq!(found.public_key, entity.public_key);

        // Cleanup
        store.delete(&entity.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_public_key() {
        let store = DefraEntityStore::connect("http://localhost:9181")
            .await
            .unwrap();
        let entity = make_test_entity("pk-device");
        let _ = store.delete(&entity.id).await;

        store.create(&entity).await.unwrap();

        let found = store.find_by_public_key(&entity.public_key).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, entity.id);

        store.delete(&entity.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_find_by_tag() {
        let store = DefraEntityStore::connect("http://localhost:9181")
            .await
            .unwrap();
        let entity = make_test_entity("tag-device");
        let _ = store.delete(&entity.id).await;

        store.create(&entity).await.unwrap();

        let found = store.find_by_tag("lighting").await.unwrap();
        assert!(!found.is_empty());
        assert!(found.iter().any(|e| e.id == entity.id));

        let not_found = store.find_by_tag("nonexistent").await.unwrap();
        assert!(!not_found.iter().any(|e| e.id == entity.id));

        store.delete(&entity.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_update_status() {
        let store = DefraEntityStore::connect("http://localhost:9181")
            .await
            .unwrap();
        let entity = make_test_entity("status-device");
        let _ = store.delete(&entity.id).await;

        store.create(&entity).await.unwrap();
        store
            .update_status(&entity.id, EntityStatus::Suspended)
            .await
            .unwrap();

        let found = store.get(&entity.id).await.unwrap().unwrap();
        assert_eq!(found.status, EntityStatus::Suspended);

        store.delete(&entity.id).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete() {
        let store = DefraEntityStore::connect("http://localhost:9181")
            .await
            .unwrap();
        let entity = make_test_entity("delete-device");
        let _ = store.delete(&entity.id).await;

        store.create(&entity).await.unwrap();

        assert!(store.delete(&entity.id).await.unwrap());
        assert!(store.get(&entity.id).await.unwrap().is_none());
        assert!(!store.delete(&entity.id).await.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_and_count() {
        let store = DefraEntityStore::connect("http://localhost:9181")
            .await
            .unwrap();
        let entity = make_test_entity("list-device");
        let _ = store.delete(&entity.id).await;

        store.create(&entity).await.unwrap();

        let count = store.count().await.unwrap();
        assert!(count >= 1);

        let page = store.list(0, 100).await.unwrap();
        assert!(!page.is_empty());
        assert!(page.iter().any(|e| e.id == entity.id));

        store.delete(&entity.id).await.unwrap();
    }
}
