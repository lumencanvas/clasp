//! SQLite-backed entity store
//!
//! Feature-gated behind `sqlite`. Uses WAL mode for concurrent read access.

use async_trait::async_trait;
use rusqlite::{params, Connection};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::entity::{Entity, EntityId, EntityStatus, EntityType};
use crate::error::{RegistryError, Result};
use crate::store::EntityStore;

/// SQLite-backed entity store
///
/// Uses a single SQLite file with WAL mode for good read concurrency.
pub struct SqliteEntityStore {
    conn: Mutex<Connection>,
}

impl SqliteEntityStore {
    /// Open or create a SQLite entity store at the given path
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .map_err(|e| RegistryError::StorageError(format!("failed to open database: {}", e)))?;

        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA synchronous=NORMAL;
             PRAGMA foreign_keys=ON;",
        )
        .map_err(|e| RegistryError::StorageError(format!("failed to set pragmas: {}", e)))?;

        let store = Self {
            conn: Mutex::new(conn),
        };
        store.create_tables()?;
        Ok(store)
    }

    /// Create an in-memory SQLite store (for testing)
    pub fn in_memory() -> Result<Self> {
        Self::open(":memory:")
    }

    fn create_tables(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                entity_type TEXT NOT NULL,
                name TEXT NOT NULL,
                public_key BLOB NOT NULL,
                created_at INTEGER NOT NULL,
                metadata TEXT NOT NULL DEFAULT '{}',
                tags TEXT NOT NULL DEFAULT '[]',
                namespaces TEXT NOT NULL DEFAULT '[]',
                scopes TEXT NOT NULL DEFAULT '[]',
                status TEXT NOT NULL DEFAULT 'active'
            );
            CREATE INDEX IF NOT EXISTS idx_entities_public_key ON entities(public_key);
            CREATE INDEX IF NOT EXISTS idx_entities_status ON entities(status);",
        )
        .map_err(|e| RegistryError::StorageError(format!("failed to create tables: {}", e)))?;
        Ok(())
    }

    fn row_to_entity(row: &rusqlite::Row) -> rusqlite::Result<Entity> {
        let id_str: String = row.get(0)?;
        let entity_type_str: String = row.get(1)?;
        let name: String = row.get(2)?;
        let public_key: Vec<u8> = row.get(3)?;
        let created_at_secs: u64 = row.get(4)?;
        let metadata_json: String = row.get(5)?;
        let tags_json: String = row.get(6)?;
        let namespaces_json: String = row.get(7)?;
        let scopes_json: String = row.get(8)?;
        let status_str: String = row.get(9)?;

        let entity_type = match entity_type_str.as_str() {
            "device" => EntityType::Device,
            "user" => EntityType::User,
            "service" => EntityType::Service,
            "router" => EntityType::Router,
            _ => EntityType::Device,
        };

        let status = match status_str.as_str() {
            "active" => EntityStatus::Active,
            "suspended" => EntityStatus::Suspended,
            "revoked" => EntityStatus::Revoked,
            _ => EntityStatus::Active,
        };

        let created_at = UNIX_EPOCH + std::time::Duration::from_secs(created_at_secs);
        let metadata = serde_json::from_str(&metadata_json).unwrap_or_default();
        let tags = serde_json::from_str(&tags_json).unwrap_or_default();
        let namespaces = serde_json::from_str(&namespaces_json).unwrap_or_default();
        let scopes = serde_json::from_str(&scopes_json).unwrap_or_default();

        Ok(Entity {
            id: EntityId::parse(&id_str).unwrap_or_else(|_| EntityId::parse("clasp:invalid").unwrap()),
            entity_type,
            name,
            public_key,
            created_at,
            metadata,
            tags,
            namespaces,
            scopes,
            status,
        })
    }
}

fn system_time_to_secs(t: SystemTime) -> u64 {
    t.duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

#[async_trait]
impl EntityStore for SqliteEntityStore {
    async fn create(&self, entity: &Entity) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let metadata = serde_json::to_string(&entity.metadata)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        let tags = serde_json::to_string(&entity.tags)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        let namespaces = serde_json::to_string(&entity.namespaces)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        let scopes = serde_json::to_string(&entity.scopes)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        conn.execute(
            "INSERT INTO entities (id, entity_type, name, public_key, created_at, metadata, tags, namespaces, scopes, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                entity.id.as_str(),
                entity.entity_type.to_string(),
                entity.name,
                entity.public_key,
                system_time_to_secs(entity.created_at),
                metadata,
                tags,
                namespaces,
                scopes,
                entity.status.to_string(),
            ],
        )
        .map_err(|e| match e {
            rusqlite::Error::SqliteFailure(err, _)
                if err.code == rusqlite::ErrorCode::ConstraintViolation =>
            {
                RegistryError::AlreadyExists(entity.id.as_str().to_string())
            }
            _ => RegistryError::StorageError(e.to_string()),
        })?;

        Ok(())
    }

    async fn get(&self, id: &EntityId) -> Result<Option<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, entity_type, name, public_key, created_at, metadata, tags, namespaces, scopes, status FROM entities WHERE id = ?1")
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let result = stmt
            .query_row(params![id.as_str()], Self::row_to_entity)
            .optional()
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_public_key(&self, key: &[u8]) -> Result<Option<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, entity_type, name, public_key, created_at, metadata, tags, namespaces, scopes, status FROM entities WHERE public_key = ?1")
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let result = stmt
            .query_row(params![key], Self::row_to_entity)
            .optional()
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        Ok(result)
    }

    async fn find_by_tag(&self, tag: &str) -> Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%\"{}\"%" , tag);
        let mut stmt = conn
            .prepare("SELECT id, entity_type, name, public_key, created_at, metadata, tags, namespaces, scopes, status FROM entities WHERE tags LIKE ?1")
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![pattern], Self::row_to_entity)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| RegistryError::StorageError(e.to_string()))?);
        }
        Ok(result)
    }

    async fn find_by_namespace(&self, namespace: &str) -> Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let pattern = format!("%\"{}\"%" , namespace);
        let mut stmt = conn
            .prepare("SELECT id, entity_type, name, public_key, created_at, metadata, tags, namespaces, scopes, status FROM entities WHERE namespaces LIKE ?1")
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![pattern], Self::row_to_entity)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| RegistryError::StorageError(e.to_string()))?);
        }
        Ok(result)
    }

    async fn list(&self, offset: usize, limit: usize) -> Result<Vec<Entity>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, entity_type, name, public_key, created_at, metadata, tags, namespaces, scopes, status FROM entities ORDER BY created_at DESC LIMIT ?1 OFFSET ?2")
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![limit as i64, offset as i64], Self::row_to_entity)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| RegistryError::StorageError(e.to_string()))?);
        }
        Ok(result)
    }

    async fn update(&self, entity: &Entity) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let metadata = serde_json::to_string(&entity.metadata)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        let tags = serde_json::to_string(&entity.tags)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        let namespaces = serde_json::to_string(&entity.namespaces)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        let scopes = serde_json::to_string(&entity.scopes)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        let rows = conn
            .execute(
                "UPDATE entities SET name = ?1, metadata = ?2, tags = ?3, namespaces = ?4, scopes = ?5, status = ?6 WHERE id = ?7",
                params![
                    entity.name,
                    metadata,
                    tags,
                    namespaces,
                    scopes,
                    entity.status.to_string(),
                    entity.id.as_str(),
                ],
            )
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        if rows == 0 {
            return Err(RegistryError::NotFound(entity.id.as_str().to_string()));
        }
        Ok(())
    }

    async fn update_status(&self, id: &EntityId, status: EntityStatus) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let rows = conn
            .execute(
                "UPDATE entities SET status = ?1 WHERE id = ?2",
                params![status.to_string(), id.as_str()],
            )
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        if rows == 0 {
            return Err(RegistryError::NotFound(id.as_str().to_string()));
        }
        Ok(())
    }

    async fn delete(&self, id: &EntityId) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn
            .execute("DELETE FROM entities WHERE id = ?1", params![id.as_str()])
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        Ok(rows > 0)
    }

    async fn count(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;
        Ok(count as usize)
    }
}

// Need this trait for optional() method
trait OptionalExt<T> {
    fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error>;
}

impl<T> OptionalExt<T> for std::result::Result<T, rusqlite::Error> {
    fn optional(self) -> std::result::Result<Option<T>, rusqlite::Error> {
        match self {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{EntityKeypair, EntityType};

    fn create_test_entity(name: &str) -> Entity {
        let keypair = EntityKeypair::generate().unwrap();
        let mut entity = keypair.to_entity(EntityType::Device, name.to_string());
        entity.tags = vec!["test".to_string()];
        entity.namespaces = vec!["/test".to_string()];
        entity.scopes = vec!["admin:/**".to_string()];
        entity
    }

    #[tokio::test]
    async fn test_sqlite_create_get() {
        let store = SqliteEntityStore::in_memory().unwrap();
        let entity = create_test_entity("test-device");

        store.create(&entity).await.unwrap();

        let found = store.get(&entity.id).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.name, "test-device");
        assert_eq!(found.entity_type, EntityType::Device);
        assert!(found.is_active());
    }

    #[tokio::test]
    async fn test_sqlite_duplicate() {
        let store = SqliteEntityStore::in_memory().unwrap();
        let entity = create_test_entity("test-device");

        store.create(&entity).await.unwrap();
        assert!(store.create(&entity).await.is_err());
    }

    #[tokio::test]
    async fn test_sqlite_find_by_key() {
        let store = SqliteEntityStore::in_memory().unwrap();
        let entity = create_test_entity("test-device");
        let key = entity.public_key.clone();

        store.create(&entity).await.unwrap();

        let found = store.find_by_public_key(&key).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_sqlite_find_by_tag() {
        let store = SqliteEntityStore::in_memory().unwrap();
        let entity = create_test_entity("test-device");
        store.create(&entity).await.unwrap();

        let found = store.find_by_tag("test").await.unwrap();
        assert_eq!(found.len(), 1);

        let found = store.find_by_tag("other").await.unwrap();
        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn test_sqlite_update_status() {
        let store = SqliteEntityStore::in_memory().unwrap();
        let entity = create_test_entity("test-device");
        let id = entity.id.clone();

        store.create(&entity).await.unwrap();
        store.update_status(&id, EntityStatus::Revoked).await.unwrap();

        let found = store.get(&id).await.unwrap().unwrap();
        assert_eq!(found.status, EntityStatus::Revoked);
    }

    #[tokio::test]
    async fn test_sqlite_delete() {
        let store = SqliteEntityStore::in_memory().unwrap();
        let entity = create_test_entity("test-device");
        let id = entity.id.clone();

        store.create(&entity).await.unwrap();
        assert!(store.delete(&id).await.unwrap());
        assert!(store.get(&id).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_sqlite_list_count() {
        let store = SqliteEntityStore::in_memory().unwrap();

        for i in 0..5 {
            let entity = create_test_entity(&format!("device-{}", i));
            store.create(&entity).await.unwrap();
        }

        assert_eq!(store.count().await.unwrap(), 5);

        let page = store.list(0, 3).await.unwrap();
        assert_eq!(page.len(), 3);
    }
}
