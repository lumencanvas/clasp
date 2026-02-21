//! State management for Clasp params
//!
//! Provides conflict resolution and revision tracking for stateful parameters.

use crate::{ConflictStrategy, Value};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Eviction strategy when the state store reaches capacity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvictionStrategy {
    /// Evict least recently accessed entries (default)
    #[default]
    Lru,
    /// Evict oldest entries by creation time
    OldestFirst,
    /// Reject new entries when at capacity
    RejectNew,
}

/// Configuration for state store limits
#[derive(Debug, Clone)]
pub struct StateStoreConfig {
    /// Maximum number of parameters (None = unlimited)
    pub max_params: Option<usize>,
    /// Time-to-live for parameters without access (None = never expire)
    pub param_ttl: Option<Duration>,
    /// Strategy for eviction when at capacity
    pub eviction: EvictionStrategy,
}

impl Default for StateStoreConfig {
    fn default() -> Self {
        Self {
            max_params: Some(10_000),
            param_ttl: Some(Duration::from_secs(3600)), // 1 hour
            eviction: EvictionStrategy::Lru,
        }
    }
}

impl StateStoreConfig {
    /// Create config with no limits (for backwards compatibility)
    pub fn unlimited() -> Self {
        Self {
            max_params: None,
            param_ttl: None,
            eviction: EvictionStrategy::Lru,
        }
    }

    /// Create config with custom limits
    pub fn with_limits(max_params: usize, ttl_secs: u64) -> Self {
        Self {
            max_params: Some(max_params),
            param_ttl: Some(Duration::from_secs(ttl_secs)),
            eviction: EvictionStrategy::Lru,
        }
    }
}

/// State of a single parameter
#[derive(Debug, Clone)]
pub struct ParamState {
    /// Current value
    pub value: Value,
    /// Monotonic revision number
    pub revision: u64,
    /// Session ID of last writer
    pub writer: String,
    /// Timestamp of last write (microseconds)
    pub timestamp: u64,
    /// Timestamp of last access (microseconds) - for TTL eviction
    pub last_accessed: u64,
    /// Conflict resolution strategy
    pub strategy: ConflictStrategy,
    /// Lock holder (if locked)
    pub lock_holder: Option<String>,
    /// Metadata
    pub meta: Option<ParamMeta>,
    /// Origin router ID (for federation loop prevention)
    pub origin: Option<String>,
}

/// Parameter metadata
#[derive(Debug, Clone)]
pub struct ParamMeta {
    pub unit: Option<String>,
    pub range: Option<(f64, f64)>,
    pub default: Option<Value>,
}

impl ParamState {
    /// Create a new param state
    pub fn new(value: Value, writer: String) -> Self {
        let now = current_timestamp();
        Self {
            value,
            revision: 1,
            writer,
            timestamp: now,
            last_accessed: now,
            strategy: ConflictStrategy::Lww,
            lock_holder: None,
            meta: None,
            origin: None,
        }
    }

    /// Update the last_accessed timestamp
    pub fn touch(&mut self) {
        self.last_accessed = current_timestamp();
    }

    /// Create with specific strategy
    pub fn with_strategy(mut self, strategy: ConflictStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Create with metadata
    pub fn with_meta(mut self, meta: ParamMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    /// Attempt to update the value
    ///
    /// Returns Ok(new_revision) if update was accepted,
    /// Err with reason if rejected.
    pub fn try_update(
        &mut self,
        new_value: Value,
        writer: &str,
        expected_revision: Option<u64>,
        request_lock: bool,
        release_lock: bool,
    ) -> Result<u64, UpdateError> {
        let timestamp = current_timestamp();

        // Check optimistic lock (if revision specified)
        if let Some(expected) = expected_revision {
            if expected != self.revision {
                return Err(UpdateError::RevisionConflict {
                    expected,
                    actual: self.revision,
                });
            }
        }

        // Check lock
        if let Some(ref holder) = self.lock_holder {
            if holder != writer && !release_lock {
                return Err(UpdateError::LockHeld {
                    holder: holder.clone(),
                });
            }
        }

        // Handle lock release
        if release_lock {
            if self.lock_holder.as_deref() == Some(writer) {
                self.lock_holder = None;
            }
        }

        // Apply conflict resolution
        let should_update = match self.strategy {
            ConflictStrategy::Lww => timestamp >= self.timestamp,
            ConflictStrategy::Max => {
                match (&new_value, &self.value) {
                    (Value::Float(new), Value::Float(old)) => new > old,
                    (Value::Int(new), Value::Int(old)) => new > old,
                    _ => true, // Fall back to LWW for non-numeric
                }
            }
            ConflictStrategy::Min => match (&new_value, &self.value) {
                (Value::Float(new), Value::Float(old)) => new < old,
                (Value::Int(new), Value::Int(old)) => new < old,
                _ => true,
            },
            ConflictStrategy::Lock => {
                self.lock_holder.is_none() || self.lock_holder.as_deref() == Some(writer)
            }
            ConflictStrategy::Merge => true, // App handles merge
        };

        if !should_update {
            return Err(UpdateError::ConflictRejected);
        }

        // Handle lock request
        if request_lock {
            if self.lock_holder.is_some() && self.lock_holder.as_deref() != Some(writer) {
                return Err(UpdateError::LockHeld {
                    holder: self.lock_holder.clone().unwrap(),
                });
            }
            self.lock_holder = Some(writer.to_string());
        }

        // Apply update
        self.value = new_value;
        self.revision += 1;
        self.writer = writer.to_string();
        self.timestamp = timestamp;
        self.last_accessed = timestamp;

        Ok(self.revision)
    }

    /// Check if value is within range (if specified)
    pub fn validate_range(&self, value: &Value) -> bool {
        if let Some(meta) = &self.meta {
            if let Some((min, max)) = meta.range {
                if let Some(v) = value.as_f64() {
                    return v >= min && v <= max;
                }
            }
        }
        true
    }
}

/// Errors that can occur during state updates
#[derive(Debug, Clone)]
pub enum UpdateError {
    RevisionConflict { expected: u64, actual: u64 },
    LockHeld { holder: String },
    ConflictRejected,
    OutOfRange,
    AtCapacity,
}

impl std::fmt::Display for UpdateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RevisionConflict { expected, actual } => {
                write!(
                    f,
                    "Revision conflict: expected {}, got {}",
                    expected, actual
                )
            }
            Self::LockHeld { holder } => {
                write!(f, "Parameter locked by {}", holder)
            }
            Self::ConflictRejected => {
                write!(f, "Update rejected by conflict strategy")
            }
            Self::OutOfRange => {
                write!(f, "Value out of allowed range")
            }
            Self::AtCapacity => {
                write!(f, "State store at capacity")
            }
        }
    }
}

impl std::error::Error for UpdateError {}

/// Error returned when state store is at capacity
#[derive(Debug, Clone)]
pub struct CapacityError;

impl std::fmt::Display for CapacityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "State store at capacity")
    }
}

impl std::error::Error for CapacityError {}

/// State store for multiple params
#[derive(Debug)]
pub struct StateStore {
    params: HashMap<String, ParamState>,
    config: StateStoreConfig,
}

impl Default for StateStore {
    fn default() -> Self {
        Self {
            params: HashMap::new(),
            config: StateStoreConfig::unlimited(), // Backwards compatible default
        }
    }
}

impl StateStore {
    /// Create a new state store with default (unlimited) config
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new state store with the specified config
    pub fn with_config(config: StateStoreConfig) -> Self {
        Self {
            params: HashMap::new(),
            config,
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &StateStoreConfig {
        &self.config
    }

    /// Get a param's current state (does not update last_accessed)
    pub fn get(&self, address: &str) -> Option<&ParamState> {
        self.params.get(address)
    }

    /// Get a param's current state and update last_accessed
    pub fn get_mut(&mut self, address: &str) -> Option<&mut ParamState> {
        let param = self.params.get_mut(address)?;
        param.touch();
        Some(param)
    }

    /// Get a param's current value (does not update last_accessed)
    pub fn get_value(&self, address: &str) -> Option<&Value> {
        self.params.get(address).map(|p| &p.value)
    }

    /// Get a param's current value and update last_accessed
    pub fn get_value_mut(&mut self, address: &str) -> Option<&Value> {
        let param = self.params.get_mut(address)?;
        param.touch();
        Some(&param.value)
    }

    /// Set a param value, creating if necessary
    pub fn set(
        &mut self,
        address: &str,
        value: Value,
        writer: &str,
        revision: Option<u64>,
        lock: bool,
        unlock: bool,
    ) -> Result<u64, UpdateError> {
        if let Some(param) = self.params.get_mut(address) {
            param.try_update(value, writer, revision, lock, unlock)
        } else {
            // Check capacity before creating new param
            if let Some(max) = self.config.max_params {
                if self.params.len() >= max {
                    match self.config.eviction {
                        EvictionStrategy::RejectNew => {
                            return Err(UpdateError::AtCapacity);
                        }
                        EvictionStrategy::Lru => {
                            self.evict_lru();
                        }
                        EvictionStrategy::OldestFirst => {
                            self.evict_oldest();
                        }
                    }
                }
            }

            // Create new param
            let mut param = ParamState::new(value, writer.to_string());
            if lock {
                param.lock_holder = Some(writer.to_string());
            }
            let rev = param.revision;
            self.params.insert(address.to_string(), param);
            Ok(rev)
        }
    }

    /// Evict the least recently accessed param
    fn evict_lru(&mut self) {
        if let Some(oldest_key) = self
            .params
            .iter()
            .min_by_key(|(_, v)| v.last_accessed)
            .map(|(k, _)| k.clone())
        {
            self.params.remove(&oldest_key);
        }
    }

    /// Evict the oldest param by creation time (lowest revision is oldest)
    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .params
            .iter()
            .min_by_key(|(_, v)| v.timestamp)
            .map(|(k, _)| k.clone())
        {
            self.params.remove(&oldest_key);
        }
    }

    /// Remove params that haven't been accessed within the TTL
    /// Returns the number of params removed
    pub fn cleanup_stale(&mut self, ttl: Duration) -> usize {
        let now = current_timestamp();
        let ttl_micros = ttl.as_micros() as u64;
        let cutoff = now.saturating_sub(ttl_micros);

        let before = self.params.len();
        self.params.retain(|_, v| v.last_accessed >= cutoff);
        before - self.params.len()
    }

    /// Run cleanup using the configured TTL (if any)
    /// Returns the number of params removed
    pub fn cleanup_stale_with_config(&mut self) -> usize {
        if let Some(ttl) = self.config.param_ttl {
            self.cleanup_stale(ttl)
        } else {
            0
        }
    }

    /// Get all params matching a pattern
    pub fn get_matching(&self, pattern: &str) -> Vec<(&str, &ParamState)> {
        use crate::address::glob_match;

        self.params
            .iter()
            .filter(|(addr, _)| glob_match(pattern, addr))
            .map(|(addr, state)| (addr.as_str(), state))
            .collect()
    }

    /// Get all params as a snapshot
    pub fn snapshot(&self) -> Vec<(&str, &ParamState)> {
        self.params.iter().map(|(k, v)| (k.as_str(), v)).collect()
    }

    /// Number of params
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Remove a param
    pub fn remove(&mut self, address: &str) -> Option<ParamState> {
        self.params.remove(address)
    }

    /// Clear all params
    pub fn clear(&mut self) {
        self.params.clear();
    }
}

/// Get current timestamp in microseconds
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_micros() as u64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_update() {
        let mut state = ParamState::new(Value::Float(0.5), "session1".to_string());

        let result = state.try_update(Value::Float(0.75), "session2", None, false, false);

        assert!(result.is_ok());
        assert_eq!(state.revision, 2);
        assert_eq!(state.value, Value::Float(0.75));
        assert_eq!(state.writer, "session2");
    }

    #[test]
    fn test_revision_conflict() {
        let mut state = ParamState::new(Value::Float(0.5), "session1".to_string());

        let result = state.try_update(
            Value::Float(0.75),
            "session2",
            Some(999), // Wrong revision
            false,
            false,
        );

        assert!(matches!(result, Err(UpdateError::RevisionConflict { .. })));
    }

    #[test]
    fn test_locking() {
        let mut state = ParamState::new(Value::Float(0.5), "session1".to_string());

        // Session 1 takes lock
        let result = state.try_update(
            Value::Float(0.6),
            "session1",
            None,
            true, // Request lock
            false,
        );
        assert!(result.is_ok());
        assert_eq!(state.lock_holder, Some("session1".to_string()));

        // Session 2 tries to update - should fail
        let result = state.try_update(Value::Float(0.7), "session2", None, false, false);
        assert!(matches!(result, Err(UpdateError::LockHeld { .. })));

        // Session 1 can still update
        let result = state.try_update(Value::Float(0.8), "session1", None, false, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_max_strategy() {
        let mut state = ParamState::new(Value::Float(0.5), "session1".to_string())
            .with_strategy(ConflictStrategy::Max);

        // Higher value wins
        let result = state.try_update(Value::Float(0.8), "session2", None, false, false);
        assert!(result.is_ok());
        assert_eq!(state.value, Value::Float(0.8));

        // Lower value rejected
        let result = state.try_update(Value::Float(0.3), "session3", None, false, false);
        assert!(matches!(result, Err(UpdateError::ConflictRejected)));
        assert_eq!(state.value, Value::Float(0.8)); // Unchanged
    }

    #[test]
    fn test_state_store() {
        let mut store = StateStore::new();

        store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false)
            .unwrap();
        store
            .set("/test/b", Value::Float(2.0), "s1", None, false, false)
            .unwrap();
        store
            .set("/other/c", Value::Float(3.0), "s1", None, false, false)
            .unwrap();

        assert_eq!(store.len(), 3);

        let matching = store.get_matching("/test/*");
        assert_eq!(matching.len(), 2);
    }

    #[test]
    fn test_state_store_capacity_reject() {
        let config = StateStoreConfig {
            max_params: Some(2),
            param_ttl: None,
            eviction: EvictionStrategy::RejectNew,
        };
        let mut store = StateStore::with_config(config);

        store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false)
            .unwrap();
        store
            .set("/test/b", Value::Float(2.0), "s1", None, false, false)
            .unwrap();

        // Third should fail
        let result = store.set("/test/c", Value::Float(3.0), "s1", None, false, false);
        assert!(matches!(result, Err(UpdateError::AtCapacity)));
        assert_eq!(store.len(), 2);

        // Updating existing should still work
        store
            .set("/test/a", Value::Float(1.5), "s1", None, false, false)
            .unwrap();
        assert_eq!(store.get_value("/test/a"), Some(&Value::Float(1.5)));
    }

    #[test]
    fn test_state_store_capacity_lru_eviction() {
        let config = StateStoreConfig {
            max_params: Some(2),
            param_ttl: None,
            eviction: EvictionStrategy::Lru,
        };
        let mut store = StateStore::with_config(config);

        store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        store
            .set("/test/b", Value::Float(2.0), "s1", None, false, false)
            .unwrap();

        // Access /test/a to make it more recent
        std::thread::sleep(std::time::Duration::from_millis(1));
        store.get_mut("/test/a");

        // Third should evict /test/b (least recently accessed)
        store
            .set("/test/c", Value::Float(3.0), "s1", None, false, false)
            .unwrap();

        assert_eq!(store.len(), 2);
        assert!(store.get("/test/a").is_some());
        assert!(store.get("/test/b").is_none()); // Evicted
        assert!(store.get("/test/c").is_some());
    }

    #[test]
    fn test_state_store_capacity_oldest_eviction() {
        let config = StateStoreConfig {
            max_params: Some(2),
            param_ttl: None,
            eviction: EvictionStrategy::OldestFirst,
        };
        let mut store = StateStore::with_config(config);

        store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false)
            .unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1));
        store
            .set("/test/b", Value::Float(2.0), "s1", None, false, false)
            .unwrap();

        // Third should evict /test/a (oldest)
        store
            .set("/test/c", Value::Float(3.0), "s1", None, false, false)
            .unwrap();

        assert_eq!(store.len(), 2);
        assert!(store.get("/test/a").is_none()); // Evicted
        assert!(store.get("/test/b").is_some());
        assert!(store.get("/test/c").is_some());
    }

    #[test]
    fn test_state_store_cleanup_stale() {
        let mut store = StateStore::new();

        store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false)
            .unwrap();
        store
            .set("/test/b", Value::Float(2.0), "s1", None, false, false)
            .unwrap();

        // Sleep a bit, then access /test/a
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.get_mut("/test/a");

        // Cleanup with a very short TTL - should remove /test/b but not /test/a
        let removed = store.cleanup_stale(Duration::from_millis(5));
        assert_eq!(removed, 1);
        assert!(store.get("/test/a").is_some());
        assert!(store.get("/test/b").is_none());
    }

    #[test]
    fn test_state_store_cleanup_stale_with_config() {
        let config = StateStoreConfig {
            max_params: None,
            param_ttl: Some(Duration::from_millis(5)),
            eviction: EvictionStrategy::Lru,
        };
        let mut store = StateStore::with_config(config);

        store
            .set("/test/a", Value::Float(1.0), "s1", None, false, false)
            .unwrap();

        // Immediate cleanup should remove nothing
        let removed = store.cleanup_stale_with_config();
        assert_eq!(removed, 0);

        // Wait and cleanup
        std::thread::sleep(std::time::Duration::from_millis(10));
        let removed = store.cleanup_stale_with_config();
        assert_eq!(removed, 1);
        assert!(store.is_empty());
    }

    #[test]
    fn test_last_accessed_tracking() {
        let mut state = ParamState::new(Value::Float(0.5), "session1".to_string());
        let initial_accessed = state.last_accessed;

        std::thread::sleep(std::time::Duration::from_millis(1));
        state.touch();

        assert!(state.last_accessed > initial_accessed);
    }
}
