//! Entity storage trait and in-memory implementation

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

use crate::entity::{Entity, EntityId, EntityStatus};
use crate::error::{RegistryError, Result};

/// Storage backend for entities
#[async_trait]
pub trait EntityStore: Send + Sync {
    /// Create a new entity
    async fn create(&self, entity: &Entity) -> Result<()>;

    /// Get an entity by ID
    async fn get(&self, id: &EntityId) -> Result<Option<Entity>>;

    /// Find an entity by its public key
    async fn find_by_public_key(&self, key: &[u8]) -> Result<Option<Entity>>;

    /// Find entities by tag
    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Entity>>;

    /// Find entities by namespace pattern
    async fn find_by_namespace(&self, namespace: &str) -> Result<Vec<Entity>>;

    /// List entities with pagination
    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<Entity>>;

    /// Update an entity
    async fn update(&self, entity: &Entity) -> Result<()>;

    /// Update entity status
    async fn update_status(&self, id: &EntityId, status: EntityStatus) -> Result<()>;

    /// Delete an entity
    async fn delete(&self, id: &EntityId) -> Result<bool>;

    /// Count total entities
    async fn count(&self) -> Result<usize>;
}

/// In-memory entity store for development and testing
pub struct MemoryEntityStore {
    entities: RwLock<HashMap<String, Entity>>,
}

impl MemoryEntityStore {
    pub fn new() -> Self {
        Self {
            entities: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for MemoryEntityStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EntityStore for MemoryEntityStore {
    async fn create(&self, entity: &Entity) -> Result<()> {
        let mut entities = self.entities.write().unwrap();
        let key = entity.id.as_str().to_string();
        if entities.contains_key(&key) {
            return Err(RegistryError::AlreadyExists(key));
        }
        entities.insert(key, entity.clone());
        Ok(())
    }

    async fn get(&self, id: &EntityId) -> Result<Option<Entity>> {
        let entities = self.entities.read().unwrap();
        Ok(entities.get(id.as_str()).cloned())
    }

    async fn find_by_public_key(&self, key: &[u8]) -> Result<Option<Entity>> {
        let entities = self.entities.read().unwrap();
        Ok(entities.values().find(|e| e.public_key == key).cloned())
    }

    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Entity>> {
        let entities = self.entities.read().unwrap();
        Ok(entities
            .values()
            .filter(|e| e.tags.iter().any(|t| t == tag))
            .cloned()
            .collect())
    }

    async fn find_by_namespace(&self, namespace: &str) -> Result<Vec<Entity>> {
        let entities = self.entities.read().unwrap();
        Ok(entities
            .values()
            .filter(|e| e.namespaces.iter().any(|ns| ns == namespace))
            .cloned()
            .collect())
    }

    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<Entity>> {
        let entities = self.entities.read().unwrap();
        Ok(entities
            .values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect())
    }

    async fn update(&self, entity: &Entity) -> Result<()> {
        let mut entities = self.entities.write().unwrap();
        let key = entity.id.as_str().to_string();
        if !entities.contains_key(&key) {
            return Err(RegistryError::NotFound(key));
        }
        entities.insert(key, entity.clone());
        Ok(())
    }

    async fn update_status(&self, id: &EntityId, status: EntityStatus) -> Result<()> {
        let mut entities = self.entities.write().unwrap();
        let key = id.as_str().to_string();
        match entities.get_mut(&key) {
            Some(entity) => {
                entity.status = status;
                Ok(())
            }
            None => Err(RegistryError::NotFound(key)),
        }
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        let mut entities = self.entities.write().unwrap();
        Ok(entities.remove(id.as_str()).is_some())
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.entities.read().unwrap().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityKeypair, EntityType};

    fn create_test_entity(name: &str) -> Entity {
        let keypair = EntityKeypair::generate().unwrap();
        keypair.to_entity(EntityType::Device, name.to_string())
    }

    #[tokio::test]
    async fn test_memory_store_create_get() {
        let store = MemoryEntityStore::new();
        let entity = create_test_entity("test-device");

        store.create(&entity).await.unwrap();

        let found = store.get(&entity.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test-device");
    }

    #[tokio::test]
    async fn test_memory_store_duplicate() {
        let store = MemoryEntityStore::new();
        let entity = create_test_entity("test-device");

        store.create(&entity).await.unwrap();
        assert!(store.create(&entity).await.is_err());
    }

    #[tokio::test]
    async fn test_memory_store_find_by_key() {
        let store = MemoryEntityStore::new();
        let entity = create_test_entity("test-device");
        let key = entity.public_key.clone();

        store.create(&entity).await.unwrap();

        let found = store.find_by_public_key(&key).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, entity.id);
    }

    #[tokio::test]
    async fn test_memory_store_find_by_tag() {
        let store = MemoryEntityStore::new();
        let mut entity = create_test_entity("test-device");
        entity.tags = vec!["lighting".to_string(), "dmx".to_string()];

        store.create(&entity).await.unwrap();

        let found = store.find_by_tag("lighting").await.unwrap();
        assert_eq!(found.len(), 1);

        let found = store.find_by_tag("audio").await.unwrap();
        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn test_memory_store_update_status() {
        let store = MemoryEntityStore::new();
        let entity = create_test_entity("test-device");
        let id = entity.id.clone();

        store.create(&entity).await.unwrap();
        store
            .update_status(&id, EntityStatus::Suspended)
            .await
            .unwrap();

        let found = store.get(&id).await.unwrap().unwrap();
        assert_eq!(found.status, EntityStatus::Suspended);
    }

    #[tokio::test]
    async fn test_memory_store_delete() {
        let store = MemoryEntityStore::new();
        let entity = create_test_entity("test-device");
        let id = entity.id.clone();

        store.create(&entity).await.unwrap();
        assert!(store.delete(&id).await.unwrap());
        assert!(store.get(&id).await.unwrap().is_none());
        assert!(!store.delete(&id).await.unwrap());
    }

    #[tokio::test]
    async fn test_memory_store_list() {
        let store = MemoryEntityStore::new();

        for i in 0..5 {
            let entity = create_test_entity(&format!("device-{}", i));
            store.create(&entity).await.unwrap();
        }

        assert_eq!(store.count().await.unwrap(), 5);

        let page1 = store.list(0, 3).await.unwrap();
        assert_eq!(page1.len(), 3);

        let page2 = store.list(3, 3).await.unwrap();
        assert_eq!(page2.len(), 2);
    }
}
