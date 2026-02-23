//! Key storage traits and implementations.

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::error::{CryptoError, Result};
use crate::types::{KeyData, TofuRecord};

/// Pluggable persistence interface for crypto keys and TOFU records.
#[async_trait]
pub trait KeyStore: Send + Sync {
    /// Save a group key for a session/group.
    async fn save_group_key(&self, session_id: &str, data: KeyData) -> Result<()>;
    /// Load a group key for a session/group.
    async fn load_group_key(&self, session_id: &str) -> Result<Option<KeyData>>;
    /// Delete a group key for a session/group.
    async fn delete_group_key(&self, session_id: &str) -> Result<()>;
    /// Save a TOFU fingerprint record.
    async fn save_tofu_record(&self, id: &str, record: TofuRecord) -> Result<()>;
    /// Load a TOFU fingerprint record.
    async fn load_tofu_record(&self, id: &str) -> Result<Option<TofuRecord>>;
}

/// In-memory key store for testing or ephemeral use.
pub struct MemoryKeyStore {
    group_keys: Mutex<HashMap<String, KeyData>>,
    tofu_records: Mutex<HashMap<String, TofuRecord>>,
}

impl MemoryKeyStore {
    pub fn new() -> Self {
        Self {
            group_keys: Mutex::new(HashMap::new()),
            tofu_records: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for MemoryKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KeyStore for MemoryKeyStore {
    async fn save_group_key(&self, session_id: &str, data: KeyData) -> Result<()> {
        self.group_keys
            .lock()
            .map_err(|_| CryptoError::Storage("key store lock poisoned".into()))?
            .insert(session_id.to_string(), data);
        Ok(())
    }

    async fn load_group_key(&self, session_id: &str) -> Result<Option<KeyData>> {
        Ok(self.group_keys
            .lock()
            .map_err(|_| CryptoError::Storage("key store lock poisoned".into()))?
            .get(session_id)
            .cloned())
    }

    async fn delete_group_key(&self, session_id: &str) -> Result<()> {
        self.group_keys
            .lock()
            .map_err(|_| CryptoError::Storage("key store lock poisoned".into()))?
            .remove(session_id);
        Ok(())
    }

    async fn save_tofu_record(&self, id: &str, record: TofuRecord) -> Result<()> {
        self.tofu_records
            .lock()
            .map_err(|_| CryptoError::Storage("key store lock poisoned".into()))?
            .insert(id.to_string(), record);
        Ok(())
    }

    async fn load_tofu_record(&self, id: &str) -> Result<Option<TofuRecord>> {
        Ok(self.tofu_records
            .lock()
            .map_err(|_| CryptoError::Storage("key store lock poisoned".into()))?
            .get(id)
            .cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn memory_store_group_key_round_trip() {
        let store = MemoryKeyStore::new();
        let data = KeyData {
            key: serde_json::json!({"kty": "oct", "k": "dGVzdA=="}),
            stored_at: 1000,
        };
        store.save_group_key("session-1", data.clone()).await.unwrap();
        let loaded = store.load_group_key("session-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().key["kty"], "oct");
    }

    #[tokio::test]
    async fn memory_store_group_key_missing() {
        let store = MemoryKeyStore::new();
        let loaded = store.load_group_key("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn memory_store_delete_group_key() {
        let store = MemoryKeyStore::new();
        let data = KeyData {
            key: serde_json::json!({"kty": "oct", "k": "dGVzdA=="}),
            stored_at: 1000,
        };
        store.save_group_key("s1", data).await.unwrap();
        store.delete_group_key("s1").await.unwrap();
        let loaded = store.load_group_key("s1").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn memory_store_tofu_round_trip() {
        let store = MemoryKeyStore::new();
        let record = TofuRecord {
            fingerprint: "abcd 1234".to_string(),
            first_seen: 5000,
        };
        store.save_tofu_record("peer-1", record).await.unwrap();
        let loaded = store.load_tofu_record("peer-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().fingerprint, "abcd 1234");
    }
}
