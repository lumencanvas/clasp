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
        Ok(self
            .group_keys
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
        Ok(self
            .tofu_records
            .lock()
            .map_err(|_| CryptoError::Storage("key store lock poisoned".into()))?
            .get(id)
            .cloned())
    }
}

/// Filesystem-backed key store for persistent storage.
/// Uses atomic writes (temp + rename) for crash safety.
///
/// Layout:
///   `<base_dir>/group-keys/<sha256(session_id)>.json`
///   `<base_dir>/tofu/<sha256(id)>.json`
#[cfg(feature = "fs-store")]
pub struct FileSystemKeyStore {
    base_dir: std::path::PathBuf,
}

#[cfg(feature = "fs-store")]
impl FileSystemKeyStore {
    pub fn new(base_dir: impl Into<std::path::PathBuf>) -> Self {
        Self {
            base_dir: base_dir.into(),
        }
    }

    fn hash_id(id: &str) -> String {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(id.as_bytes());
        hash.iter().map(|b| format!("{b:02x}")).collect()
    }

    fn group_key_path(&self, session_id: &str) -> std::path::PathBuf {
        self.base_dir
            .join("group-keys")
            .join(format!("{}.json", Self::hash_id(session_id)))
    }

    fn tofu_path(&self, id: &str) -> std::path::PathBuf {
        self.base_dir
            .join("tofu")
            .join(format!("{}.json", Self::hash_id(id)))
    }

    async fn atomic_write(
        path: &std::path::Path,
        data: &[u8],
    ) -> std::result::Result<(), CryptoError> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|e| CryptoError::Storage(format!("create dir: {e}")))?;
        }
        let tmp = path.with_extension("tmp");
        tokio::fs::write(&tmp, data)
            .await
            .map_err(|e| CryptoError::Storage(format!("write tmp: {e}")))?;
        tokio::fs::rename(&tmp, path)
            .await
            .map_err(|e| CryptoError::Storage(format!("rename: {e}")))?;
        Ok(())
    }
}

#[cfg(feature = "fs-store")]
#[async_trait]
impl KeyStore for FileSystemKeyStore {
    async fn save_group_key(&self, session_id: &str, data: KeyData) -> Result<()> {
        let path = self.group_key_path(session_id);
        let json = serde_json::to_vec_pretty(&data)
            .map_err(|e| CryptoError::Serialization(e.to_string()))?;
        Self::atomic_write(&path, &json).await
    }

    async fn load_group_key(&self, session_id: &str) -> Result<Option<KeyData>> {
        let path = self.group_key_path(session_id);
        match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let data: KeyData = serde_json::from_slice(&bytes)
                    .map_err(|e| CryptoError::Storage(format!("parse group key: {e}")))?;
                Ok(Some(data))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CryptoError::Storage(format!("read group key: {e}"))),
        }
    }

    async fn delete_group_key(&self, session_id: &str) -> Result<()> {
        let path = self.group_key_path(session_id);
        match tokio::fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(CryptoError::Storage(format!("delete group key: {e}"))),
        }
    }

    async fn save_tofu_record(&self, id: &str, record: TofuRecord) -> Result<()> {
        let path = self.tofu_path(id);
        let json = serde_json::to_vec_pretty(&record)
            .map_err(|e| CryptoError::Serialization(e.to_string()))?;
        Self::atomic_write(&path, &json).await
    }

    async fn load_tofu_record(&self, id: &str) -> Result<Option<TofuRecord>> {
        let path = self.tofu_path(id);
        match tokio::fs::read(&path).await {
            Ok(bytes) => {
                let record: TofuRecord = serde_json::from_slice(&bytes)
                    .map_err(|e| CryptoError::Storage(format!("parse tofu: {e}")))?;
                Ok(Some(record))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(CryptoError::Storage(format!("read tofu: {e}"))),
        }
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
        store
            .save_group_key("session-1", data.clone())
            .await
            .unwrap();
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

#[cfg(all(test, feature = "fs-store"))]
mod fs_tests {
    use super::*;

    #[tokio::test]
    async fn fs_store_group_key_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileSystemKeyStore::new(dir.path());
        let data = KeyData {
            key: serde_json::json!({"kty": "oct", "k": "dGVzdA=="}),
            stored_at: 1000,
        };
        store
            .save_group_key("session-1", data.clone())
            .await
            .unwrap();
        let loaded = store.load_group_key("session-1").await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.key["kty"], "oct");
        assert_eq!(loaded.stored_at, 1000);
    }

    #[tokio::test]
    async fn fs_store_group_key_missing() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileSystemKeyStore::new(dir.path());
        let loaded = store.load_group_key("nonexistent").await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn fs_store_delete_group_key() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileSystemKeyStore::new(dir.path());
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
    async fn fs_store_tofu_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileSystemKeyStore::new(dir.path());
        let record = TofuRecord {
            fingerprint: "abcd 1234".to_string(),
            first_seen: 5000,
        };
        store.save_tofu_record("peer-1", record).await.unwrap();
        let loaded = store.load_tofu_record("peer-1").await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().fingerprint, "abcd 1234");
    }

    #[tokio::test]
    async fn fs_store_persist_and_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().to_path_buf();

        // Save with one store instance
        {
            let store = FileSystemKeyStore::new(&path);
            let data = KeyData {
                key: serde_json::json!({"kty": "oct", "k": "dGVzdGtleQ=="}),
                stored_at: 42000,
            };
            store.save_group_key("persist-test", data).await.unwrap();
        }

        // Load with a new store instance (simulates restart)
        {
            let store = FileSystemKeyStore::new(&path);
            let loaded = store.load_group_key("persist-test").await.unwrap();
            assert!(loaded.is_some());
            let loaded = loaded.unwrap();
            assert_eq!(loaded.key["k"], "dGVzdGtleQ==");
            assert_eq!(loaded.stored_at, 42000);
        }
    }
}
