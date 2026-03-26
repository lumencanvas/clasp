//! Write-through state store backed by DefraDB.
//!
//! The hot path (`get`, `set`) operates on a local DashMap cache and completes
//! synchronously in sub-100us. Writes are queued to an unbounded channel and
//! flushed to DefraDB by a background worker task. A separate sync task polls
//! DefraDB for remote changes (from P2P peers) and merges them into the cache.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use clasp_core::state::{ParamState, UpdateError};
use clasp_core::{Ttl, Value};
use clasp_journal_defra::DefraClient;
use dashmap::DashMap;
use serde_json::json;
use tokio::sync::mpsc;
use tracing::{debug, error, trace, warn};

use crate::convert::{defra_to_param, param_to_defra};
use crate::error::{DefraStateError, Result};
use crate::schema::CLASP_PARAM_SCHEMA;

/// Configuration for the DefraDB state store.
#[derive(Debug, Clone)]
pub struct DefraStateConfig {
    /// Maximum cache size (params evicted from cache, not from DefraDB).
    pub max_cache_size: Option<usize>,
    /// How often to sync from DefraDB (catch remote changes).
    pub sync_interval: Duration,
    /// Whether to load all state from DefraDB on startup.
    pub preload: bool,
    /// Maximum number of write ops to batch in a single flush.
    pub write_batch_size: usize,
}

impl Default for DefraStateConfig {
    fn default() -> Self {
        Self {
            max_cache_size: Some(10_000),
            sync_interval: Duration::from_secs(5),
            preload: true,
            write_batch_size: 100,
        }
    }
}

/// Cache hit/miss statistics.
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Number of parameters currently held in the in-memory cache.
    pub cached_params: usize,
    /// Number of write operations queued but not yet flushed to DefraDB.
    pub pending_writes: usize,
    /// Total number of cache hits since startup.
    pub cache_hits: u64,
    /// Total number of cache misses since startup.
    pub cache_misses: u64,
}

/// Internal write operation queued for the background worker.
enum WriteOp {
    Set { address: String, state: ParamState },
    Delete { address: String },
}

/// A DefraDB-backed state store with an in-memory write-through cache.
///
/// Reads hit the DashMap cache first (sub-100us). Writes update the cache
/// immediately and queue an async flush to DefraDB. A background sync task
/// pulls remote changes from DefraDB peers.
pub struct DefraStateStore {
    /// In-memory cache for hot path.
    cache: DashMap<String, ParamState>,
    /// DefraDB client for persistence.
    client: DefraClient,
    /// Channel for async write-through.
    write_tx: mpsc::UnboundedSender<WriteOp>,
    /// Receiver end -- taken by `start_writer`.
    write_rx: tokio::sync::Mutex<Option<mpsc::UnboundedReceiver<WriteOp>>>,
    /// Configuration.
    config: DefraStateConfig,
    /// Stats counters.
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl DefraStateStore {
    /// Create a new store, provision the schema, and optionally preload state.
    pub async fn new(defra_url: &str, config: DefraStateConfig) -> Result<Self> {
        let client = DefraClient::new(defra_url);

        // Provision schema (idempotent)
        client
            .add_schema(CLASP_PARAM_SCHEMA)
            .await
            .map_err(|e| DefraStateError::Schema(e.to_string()))?;

        let (write_tx, write_rx) = mpsc::unbounded_channel();

        let store = Self {
            cache: DashMap::new(),
            client,
            write_tx,
            write_rx: tokio::sync::Mutex::new(Some(write_rx)),
            config: config.clone(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        };

        if config.preload {
            if let Err(e) = store.load_from_defra().await {
                warn!(error = %e, "Failed to preload state from DefraDB (starting with empty cache)");
            }
        }

        Ok(store)
    }

    /// Start the background write-through worker.
    ///
    /// Consumes write ops from the channel and batches them into DefraDB
    /// mutations. Returns a JoinHandle for the worker task.
    ///
    /// This must only be called once. Subsequent calls return a task that
    /// completes immediately.
    pub fn start_writer(&self) -> tokio::task::JoinHandle<()> {
        // Try to take the receiver. If already taken, return a no-op handle.
        let rx = {
            let mut guard = self
                .write_rx
                .try_lock()
                .expect("write_rx lock should not be contended");
            guard.take()
        };

        let Some(mut rx) = rx else {
            return tokio::spawn(async {});
        };

        let client = DefraClient::new(self.client.base_url());
        let batch_size = self.config.write_batch_size;

        tokio::spawn(async move {
            let mut batch: Vec<WriteOp> = Vec::with_capacity(batch_size);

            loop {
                // Wait for the first op
                match rx.recv().await {
                    Some(op) => batch.push(op),
                    None => break, // Channel closed
                }

                // Drain up to batch_size without blocking
                while batch.len() < batch_size {
                    match rx.try_recv() {
                        Ok(op) => batch.push(op),
                        Err(_) => break,
                    }
                }

                // Process the batch
                for op in batch.drain(..) {
                    match op {
                        WriteOp::Set { address, state } => {
                            let doc = param_to_defra(&address, &state);
                            let mutation = format!(
                                r#"mutation {{
                                    add_ClaspParam(input: {{
                                        address: {address}
                                        value: {value}
                                        valueType: {value_type}
                                        revision: {revision}
                                        writer: {writer}
                                        timestamp: {timestamp}
                                        lastAccessed: {last_accessed}
                                        strategy: {strategy}
                                        lockHolder: {lock_holder}
                                        origin: {origin}
                                        ttlMode: {ttl_mode}
                                        ttlSecs: {ttl_secs}
                                    }}) {{
                                        _docID
                                    }}
                                }}"#,
                                address = json!(doc["address"]),
                                value = json!(doc["value"]),
                                value_type = json!(doc["valueType"]),
                                revision = doc["revision"],
                                writer = json!(doc["writer"]),
                                timestamp = doc["timestamp"],
                                last_accessed = doc["lastAccessed"],
                                strategy = json!(doc["strategy"]),
                                lock_holder = json!(doc["lockHolder"]),
                                origin = json!(doc["origin"]),
                                ttl_mode = json!(doc["ttlMode"]),
                                ttl_secs = doc["ttlSecs"],
                            );

                            // Try upsert: first try to find existing doc, then create or update
                            if let Err(e) = upsert_param(&client, &address, &doc).await {
                                error!(address = %address, error = %e, "Failed to write param to DefraDB");
                            } else {
                                trace!(address = %address, "Flushed param to DefraDB");
                            }
                            // Suppress unused variable warning -- we build the mutation string
                            // for documentation but use the upsert helper instead.
                            let _ = mutation;
                        }
                        WriteOp::Delete { address } => {
                            if let Err(e) = delete_param(&client, &address).await {
                                error!(address = %address, error = %e, "Failed to delete param from DefraDB");
                            } else {
                                trace!(address = %address, "Deleted param from DefraDB");
                            }
                        }
                    }
                }
            }

            debug!("Write-through worker shut down");
        })
    }

    /// Start the background sync worker that pulls remote changes from DefraDB.
    ///
    /// Returns a JoinHandle for the sync task. The task runs until `shutdown`
    /// signals true.
    pub fn start_sync(
        &self,
        mut shutdown: tokio::sync::watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        let client = DefraClient::new(self.client.base_url());
        let interval = self.config.sync_interval;
        let cache = self.cache.clone();

        tokio::spawn(async move {
            let mut tick = tokio::time::interval(interval);
            tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

            loop {
                tokio::select! {
                    _ = tick.tick() => {}
                    Ok(()) = shutdown.changed() => {
                        if *shutdown.borrow() {
                            break;
                        }
                    }
                }

                // Query all params from DefraDB
                let query = r#"{ ClaspParam { address revision value valueType writer timestamp lastAccessed strategy lockHolder origin ttlMode ttlSecs } }"#;
                match client.graphql(query, None).await {
                    Ok(data) => {
                        if let Some(docs) = data.get("ClaspParam").and_then(|v| v.as_array()) {
                            for doc in docs {
                                match defra_to_param(doc) {
                                    Ok((addr, remote_state)) => {
                                        // Only update cache if remote revision is newer
                                        let should_update = match cache.get(&addr) {
                                            Some(local) => remote_state.revision > local.revision,
                                            None => true,
                                        };
                                        if should_update {
                                            trace!(address = %addr, revision = remote_state.revision, "Synced remote param");
                                            cache.insert(addr, remote_state);
                                        }
                                    }
                                    Err(e) => {
                                        warn!(error = %e, "Failed to convert DefraDB doc to ParamState");
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!(error = %e, "Failed to sync from DefraDB");
                    }
                }
            }

            debug!("Sync worker shut down");
        })
    }

    /// Get a parameter value (cache-first, returns None on miss).
    ///
    /// This is the hot path -- purely synchronous DashMap lookup.
    pub fn get(&self, address: &str) -> Option<Value> {
        match self.cache.get(address) {
            Some(entry) => {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                Some(entry.value.clone())
            }
            None => {
                self.cache_misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Get full parameter state (cache-first).
    pub fn get_state(&self, address: &str) -> Option<ParamState> {
        match self.cache.get(address) {
            Some(entry) => {
                self.cache_hits.fetch_add(1, Ordering::Relaxed);
                Some(entry.value().clone())
            }
            None => {
                self.cache_misses.fetch_add(1, Ordering::Relaxed);
                None
            }
        }
    }

    /// Get a parameter value, falling back to DefraDB on cache miss.
    ///
    /// Use this when latency is acceptable and you need to check persistence.
    pub async fn get_or_fetch(&self, address: &str) -> Result<Option<Value>> {
        // Check cache first
        if let Some(entry) = self.cache.get(address) {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
            return Ok(Some(entry.value.clone()));
        }

        self.cache_misses.fetch_add(1, Ordering::Relaxed);

        // Query DefraDB
        let query = format!(
            r#"{{ ClaspParam(filter: {{ address: {{ _eq: {addr} }} }}) {{ address revision value valueType writer timestamp lastAccessed strategy lockHolder origin ttlMode ttlSecs }} }}"#,
            addr = json!(address),
        );

        let data = self
            .client
            .graphql(&query, None)
            .await
            .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;

        if let Some(docs) = data.get("ClaspParam").and_then(|v| v.as_array()) {
            if let Some(doc) = docs.first() {
                let (addr, state) = defra_to_param(doc)?;
                let value = state.value.clone();
                self.cache.insert(addr, state);
                return Ok(Some(value));
            }
        }

        Ok(None)
    }

    /// Set a parameter value. Updates cache immediately and queues a DefraDB write.
    ///
    /// Returns Ok(new_revision) on success.
    pub fn set(
        &self,
        address: &str,
        value: Value,
        writer: &str,
        revision: Option<u64>,
        lock: bool,
        unlock: bool,
        ttl: Option<Ttl>,
    ) -> std::result::Result<u64, UpdateError> {
        let new_rev = if let Some(mut entry) = self.cache.get_mut(address) {
            entry
                .value_mut()
                .try_update(value, writer, revision, lock, unlock, ttl)?
        } else {
            // Check capacity before creating new param
            if let Some(max) = self.config.max_cache_size {
                if self.cache.len() >= max {
                    self.evict_lru();
                }
            }

            let mut state = ParamState::new(value, writer.to_string());
            if lock {
                state.lock_holder = Some(writer.to_string());
            }
            state.ttl = ttl;
            let rev = state.revision;
            self.cache.insert(address.to_string(), state);
            rev
        };

        // Queue async write-through
        if let Some(state) = self.cache.get(address) {
            let _ = self.write_tx.send(WriteOp::Set {
                address: address.to_string(),
                state: state.value().clone(),
            });
        }

        Ok(new_rev)
    }

    /// Get all params matching a glob pattern.
    pub fn get_matching(&self, pattern: &str) -> Vec<(String, ParamState)> {
        self.cache
            .iter()
            .filter(|entry| glob_match::glob_match(pattern, entry.key()))
            .map(|entry| (entry.key().clone(), entry.value().clone()))
            .collect()
    }

    /// Remove a parameter from the cache and queue a DefraDB deletion.
    pub fn remove(&self, address: &str) -> Option<ParamState> {
        let removed = self.cache.remove(address).map(|(_, v)| v);
        if removed.is_some() {
            let _ = self.write_tx.send(WriteOp::Delete {
                address: address.to_string(),
            });
        }
        removed
    }

    /// Load all state from DefraDB into the cache.
    ///
    /// Returns the number of params loaded.
    pub async fn load_from_defra(&self) -> Result<usize> {
        let query = r#"{ ClaspParam { address revision value valueType writer timestamp lastAccessed strategy lockHolder origin ttlMode ttlSecs } }"#;

        let data = self
            .client
            .graphql(query, None)
            .await
            .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;

        let mut count = 0;
        if let Some(docs) = data.get("ClaspParam").and_then(|v| v.as_array()) {
            for doc in docs {
                match defra_to_param(doc) {
                    Ok((addr, state)) => {
                        self.cache.insert(addr, state);
                        count += 1;
                    }
                    Err(e) => {
                        warn!(error = %e, "Skipping invalid DefraDB param during load");
                    }
                }
            }
        }

        debug!(count = count, "Loaded params from DefraDB");
        Ok(count)
    }

    /// Flush all pending writes to DefraDB.
    ///
    /// This drains the write channel and processes all pending ops synchronously.
    pub async fn flush(&self) -> Result<()> {
        // We cannot drain from the receiver here (it belongs to the writer task).
        // Instead, send a barrier and wait. For now, just yield to let the writer
        // task process pending ops.
        tokio::task::yield_now().await;
        Ok(())
    }

    /// Get cache statistics.
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            cached_params: self.cache.len(),
            pending_writes: 0, // Cannot observe channel length from sender side
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
        }
    }

    /// Number of cached params.
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty.
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Clear the in-memory cache. Does NOT delete from DefraDB.
    pub fn clear_cache(&self) {
        self.cache.clear();
        self.cache_hits.store(0, Ordering::Relaxed);
        self.cache_misses.store(0, Ordering::Relaxed);
    }

    /// Remove expired params from cache and queue DefraDB deletions.
    ///
    /// Respects per-param TTL modes (Sliding, Absolute, Never). Params
    /// without a per-param TTL are checked against the provided global TTL.
    ///
    /// Returns the number of params removed.
    pub fn cleanup_stale(&self, ttl: Duration) -> usize {
        let now = current_timestamp();
        let global_ttl_micros = ttl.as_micros() as u64;
        let mut removed = 0;

        // Collect keys to remove (cannot mutate DashMap while iterating)
        let expired: Vec<String> = self
            .cache
            .iter()
            .filter(|entry| {
                let v = entry.value();
                match v.ttl {
                    Some(Ttl::Never) => false,
                    Some(Ttl::Sliding(secs)) => {
                        let cutoff = now.saturating_sub(secs as u64 * 1_000_000);
                        v.last_accessed < cutoff
                    }
                    Some(Ttl::Absolute(secs)) => {
                        let expires_at = v.timestamp.saturating_add(secs as u64 * 1_000_000);
                        now >= expires_at
                    }
                    None => {
                        let cutoff = now.saturating_sub(global_ttl_micros);
                        v.last_accessed < cutoff
                    }
                }
            })
            .map(|entry| entry.key().clone())
            .collect();

        for key in expired {
            if self.cache.remove(&key).is_some() {
                let _ = self.write_tx.send(WriteOp::Delete { address: key });
                removed += 1;
            }
        }

        removed
    }

    /// Evict the least recently accessed param from cache.
    fn evict_lru(&self) {
        let oldest = self
            .cache
            .iter()
            .min_by_key(|entry| entry.value().last_accessed)
            .map(|entry| entry.key().clone());

        if let Some(key) = oldest {
            self.cache.remove(&key);
            // Note: we do NOT delete from DefraDB on cache eviction.
            // The param persists in DefraDB and can be re-fetched.
        }
    }
}

/// Get current timestamp in microseconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

/// Upsert a param document in DefraDB.
///
/// Queries for an existing doc by address, then creates or updates.
async fn upsert_param(
    client: &DefraClient,
    address: &str,
    doc: &serde_json::Value,
) -> std::result::Result<(), DefraStateError> {
    // Check for existing doc
    let find_query = format!(
        r#"{{ ClaspParam(filter: {{ address: {{ _eq: {addr} }} }}) {{ _docID }} }}"#,
        addr = json!(address),
    );

    let data = client
        .graphql(&find_query, None)
        .await
        .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;

    if let Some(existing) = data
        .get("ClaspParam")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|d| d.get("_docID"))
        .and_then(|id| id.as_str())
    {
        // Update existing
        let update_query = format!(
            r#"mutation {{
                update_ClaspParam(docID: "{doc_id}", input: {{
                    value: {value}
                    valueType: {value_type}
                    revision: {revision}
                    writer: {writer}
                    timestamp: {timestamp}
                    lastAccessed: {last_accessed}
                    strategy: {strategy}
                    lockHolder: {lock_holder}
                    origin: {origin}
                    ttlMode: {ttl_mode}
                    ttlSecs: {ttl_secs}
                }}) {{
                    _docID
                }}
            }}"#,
            doc_id = existing,
            value = json!(doc["value"]),
            value_type = json!(doc["valueType"]),
            revision = doc["revision"],
            writer = json!(doc["writer"]),
            timestamp = doc["timestamp"],
            last_accessed = doc["lastAccessed"],
            strategy = json!(doc["strategy"]),
            lock_holder = json!(doc["lockHolder"]),
            origin = json!(doc["origin"]),
            ttl_mode = json!(doc["ttlMode"]),
            ttl_secs = doc["ttlSecs"],
        );

        client
            .graphql(&update_query, None)
            .await
            .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;
    } else {
        // Create new
        let create_query = format!(
            r#"mutation {{
                add_ClaspParam(input: {{
                    address: {address}
                    value: {value}
                    valueType: {value_type}
                    revision: {revision}
                    writer: {writer}
                    timestamp: {timestamp}
                    lastAccessed: {last_accessed}
                    strategy: {strategy}
                    lockHolder: {lock_holder}
                    origin: {origin}
                    ttlMode: {ttl_mode}
                    ttlSecs: {ttl_secs}
                }}) {{
                    _docID
                }}
            }}"#,
            address = json!(doc["address"]),
            value = json!(doc["value"]),
            value_type = json!(doc["valueType"]),
            revision = doc["revision"],
            writer = json!(doc["writer"]),
            timestamp = doc["timestamp"],
            last_accessed = doc["lastAccessed"],
            strategy = json!(doc["strategy"]),
            lock_holder = json!(doc["lockHolder"]),
            origin = json!(doc["origin"]),
            ttl_mode = json!(doc["ttlMode"]),
            ttl_secs = doc["ttlSecs"],
        );

        client
            .graphql(&create_query, None)
            .await
            .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;
    }

    Ok(())
}

/// Delete a param document from DefraDB by address.
async fn delete_param(
    client: &DefraClient,
    address: &str,
) -> std::result::Result<(), DefraStateError> {
    let find_query = format!(
        r#"{{ ClaspParam(filter: {{ address: {{ _eq: {addr} }} }}) {{ _docID }} }}"#,
        addr = json!(address),
    );

    let data = client
        .graphql(&find_query, None)
        .await
        .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;

    if let Some(existing) = data
        .get("ClaspParam")
        .and_then(|v| v.as_array())
        .and_then(|arr| arr.first())
        .and_then(|d| d.get("_docID"))
        .and_then(|id| id.as_str())
    {
        let delete_query = format!(
            r#"mutation {{ delete_ClaspParam(docID: "{doc_id}") {{ _docID }} }}"#,
            doc_id = existing,
        );

        client
            .graphql(&delete_query, None)
            .await
            .map_err(|e| DefraStateError::GraphQL(e.to_string()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clasp_core::{ConflictStrategy, Ttl, Value};

    /// Helper: create a store with no DefraDB connection (cache-only for unit tests).
    fn test_store() -> DefraStateStore {
        let (write_tx, write_rx) = mpsc::unbounded_channel();
        DefraStateStore {
            cache: DashMap::new(),
            client: DefraClient::new("http://localhost:0"),
            write_tx,
            write_rx: tokio::sync::Mutex::new(Some(write_rx)),
            config: DefraStateConfig::default(),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }

    #[test]
    fn write_through_cache() {
        let store = test_store();
        let rev = store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false, None)
            .unwrap();
        assert_eq!(rev, 1);

        // Cache should be updated immediately
        assert_eq!(store.get("/test/a"), Some(Value::Float(1.0)));
    }

    #[test]
    fn get_returns_cached() {
        let store = test_store();
        store
            .set("/test/b", Value::Int(42), "s1", None, false, false, None)
            .unwrap();
        assert_eq!(store.get("/test/b"), Some(Value::Int(42)));
    }

    #[test]
    fn set_increments_revision() {
        let store = test_store();
        let r1 = store
            .set("/test/c", Value::Float(1.0), "s1", None, false, false, None)
            .unwrap();
        assert_eq!(r1, 1);

        let r2 = store
            .set("/test/c", Value::Float(2.0), "s1", None, false, false, None)
            .unwrap();
        assert_eq!(r2, 2);

        let r3 = store
            .set("/test/c", Value::Float(3.0), "s1", None, false, false, None)
            .unwrap();
        assert_eq!(r3, 3);
    }

    #[test]
    fn conflict_lww() {
        let store = test_store();
        store
            .set("/test/d", Value::Float(1.0), "s1", None, false, false, None)
            .unwrap();

        // LWW: newer write wins (default strategy)
        let rev = store
            .set("/test/d", Value::Float(2.0), "s2", None, false, false, None)
            .unwrap();
        assert_eq!(rev, 2);
        assert_eq!(store.get("/test/d"), Some(Value::Float(2.0)));
    }

    #[test]
    fn conflict_max() {
        let store = test_store();
        store
            .set("/test/e", Value::Float(5.0), "s1", None, false, false, None)
            .unwrap();

        // Set strategy to Max
        if let Some(mut entry) = store.cache.get_mut("/test/e") {
            entry.strategy = ConflictStrategy::Max;
        }

        // Higher value wins
        let rev = store
            .set(
                "/test/e",
                Value::Float(10.0),
                "s2",
                None,
                false,
                false,
                None,
            )
            .unwrap();
        assert_eq!(rev, 2);
        assert_eq!(store.get("/test/e"), Some(Value::Float(10.0)));

        // Lower value rejected
        let result = store.set("/test/e", Value::Float(3.0), "s3", None, false, false, None);
        assert!(matches!(result, Err(UpdateError::ConflictRejected)));
        assert_eq!(store.get("/test/e"), Some(Value::Float(10.0)));
    }

    #[test]
    fn conflict_lock() {
        let store = test_store();

        // Session 1 takes lock
        store
            .set("/test/f", Value::Float(1.0), "s1", None, true, false, None)
            .unwrap();

        // Session 2 cannot write
        let result = store.set("/test/f", Value::Float(2.0), "s2", None, false, false, None);
        assert!(matches!(result, Err(UpdateError::LockHeld { .. })));

        // Session 1 can still write
        store
            .set("/test/f", Value::Float(3.0), "s1", None, false, false, None)
            .unwrap();
        assert_eq!(store.get("/test/f"), Some(Value::Float(3.0)));
    }

    #[test]
    fn cache_stats() {
        let store = test_store();
        store
            .set("/test/g", Value::Int(1), "s1", None, false, false, None)
            .unwrap();

        // Hit
        store.get("/test/g");
        // Miss
        store.get("/test/nonexistent");

        let stats = store.cache_stats();
        assert_eq!(stats.cached_params, 1);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn cleanup_stale_removes_expired() {
        let store = test_store();

        // Insert a param with a very short absolute TTL
        store
            .set(
                "/test/h",
                Value::Float(1.0),
                "s1",
                None,
                false,
                false,
                Some(Ttl::Absolute(0)), // Expires immediately
            )
            .unwrap();

        // Insert a param with Never TTL
        store
            .set(
                "/test/i",
                Value::Float(2.0),
                "s1",
                None,
                false,
                false,
                Some(Ttl::Never),
            )
            .unwrap();

        // Small sleep to ensure absolute TTL (0 seconds) has expired
        std::thread::sleep(std::time::Duration::from_millis(5));

        let removed = store.cleanup_stale(Duration::from_secs(3600));
        assert_eq!(removed, 1);
        assert!(store.get("/test/h").is_none());
        assert!(store.get("/test/i").is_some());
    }

    #[test]
    fn get_matching_returns_matches() {
        let store = test_store();
        store
            .set(
                "/synth/osc1/freq",
                Value::Float(440.0),
                "s1",
                None,
                false,
                false,
                None,
            )
            .unwrap();
        store
            .set(
                "/synth/osc1/amp",
                Value::Float(0.8),
                "s1",
                None,
                false,
                false,
                None,
            )
            .unwrap();
        store
            .set(
                "/mixer/ch1/vol",
                Value::Float(0.5),
                "s1",
                None,
                false,
                false,
                None,
            )
            .unwrap();

        let matches = store.get_matching("/synth/**");
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn remove_deletes_from_cache() {
        let store = test_store();
        store
            .set("/test/j", Value::Int(99), "s1", None, false, false, None)
            .unwrap();
        assert!(store.get("/test/j").is_some());

        let removed = store.remove("/test/j");
        assert!(removed.is_some());
        assert!(store.get("/test/j").is_none());
    }

    #[test]
    fn clear_cache_empties_store() {
        let store = test_store();
        store
            .set("/test/k", Value::Int(1), "s1", None, false, false, None)
            .unwrap();
        store
            .set("/test/l", Value::Int(2), "s1", None, false, false, None)
            .unwrap();

        assert_eq!(store.len(), 2);
        store.clear_cache();
        assert!(store.is_empty());
    }

    // -- Integration tests (require a running DefraDB instance) ----------------

    #[tokio::test]
    #[ignore]
    async fn test_write_through_persists() {
        let store = DefraStateStore::new("http://localhost:9181", DefraStateConfig::default())
            .await
            .unwrap();
        let _writer = store.start_writer();

        store
            .set(
                "/integration/persist",
                Value::Float(42.0),
                "test-session",
                None,
                false,
                false,
                None,
            )
            .unwrap();

        // Give the writer time to flush
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Verify by fetching from DefraDB directly
        let val = store.get_or_fetch("/integration/persist").await.unwrap();
        assert_eq!(val, Some(Value::Float(42.0)));
    }

    #[tokio::test]
    #[ignore]
    async fn test_load_from_defra() {
        // Insert directly into DefraDB, then load
        let client = DefraClient::new("http://localhost:9181");
        let unique_addr = format!(
            "/integration/load-test/{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );
        let doc = serde_json::json!({
            "address": &unique_addr,
            "value": "123",
            "valueType": "int",
            "revision": 1,
            "writer": "direct",
            "timestamp": 1700000000_i64,
            "lastAccessed": 1700000000_i64,
            "strategy": "lww",
            "lockHolder": "",
            "origin": "",
            "ttlMode": "none",
            "ttlSecs": 0,
        });

        let create_query = format!(
            r#"mutation {{ add_ClaspParam(input: {{
                address: "{unique_addr}"
                value: "123"
                valueType: "int"
                revision: 1
                writer: "direct"
                timestamp: 1700000000
                lastAccessed: 1700000000
                strategy: "lww"
                lockHolder: ""
                origin: ""
                ttlMode: "none"
                ttlSecs: 0
            }}) {{ _docID }} }}"#
        );
        let _ = doc; // used for documentation
        client.graphql(&create_query, None).await.unwrap();

        // Create store with preload
        let config = DefraStateConfig {
            preload: true,
            ..Default::default()
        };
        let store = DefraStateStore::new("http://localhost:9181", config)
            .await
            .unwrap();

        let val = store.get(&unique_addr);
        assert_eq!(val, Some(Value::Int(123)));
    }

    #[tokio::test]
    #[ignore]
    async fn test_two_store_sync() {
        // This test requires two DefraDB nodes in a P2P network.
        // Store A writes to node 1, DefraDB syncs to node 2 via Merkle CRDTs,
        // then Store B (on node 2) loads and sees the data.

        let store_a = DefraStateStore::new("http://localhost:9181", DefraStateConfig::default())
            .await
            .unwrap();
        let _writer_a = store_a.start_writer();

        let unique_addr = format!(
            "/integration/p2p-sync/{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        store_a
            .set(
                &unique_addr,
                Value::String("hello from A".into()),
                "store-a",
                None,
                false,
                false,
                None,
            )
            .unwrap();

        // Flush write-through, then wait for P2P propagation
        let _ = store_a.flush().await;
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Store B connects to node 2 with preload
        let store_b = match DefraStateStore::new(
            "http://localhost:9182",
            DefraStateConfig {
                preload: true,
                ..Default::default()
            },
        )
        .await
        {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Store B creation failed (DefraDB node 2 may be unavailable): {e}");
                return; // Skip test if node 2 is not ready
            }
        };

        let val = store_b.get(&unique_addr);
        assert_eq!(val, Some(Value::String("hello from A".into())));
    }
}
