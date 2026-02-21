//! Router state management

use clasp_core::state::{ParamState, StateStore, StateStoreConfig, UpdateError};
use clasp_core::{ParamValue, SetMessage, SignalDefinition, SnapshotMessage, Value};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::time::{Duration, Instant};

#[cfg(feature = "journal")]
use std::sync::Arc;
#[cfg(feature = "journal")]
use clasp_core::SignalType;
#[cfg(feature = "journal")]
use clasp_journal::{Journal, JournalEntry};

use crate::SessionId;

/// Signal entry with registration time for cleanup
#[derive(Debug, Clone)]
pub struct SignalEntry {
    /// The signal definition
    pub definition: SignalDefinition,
    /// When this signal was registered
    pub registered_at: Instant,
    /// Last time this signal was accessed or updated
    pub last_accessed: Instant,
}

/// Configuration for router state management
#[derive(Debug, Clone)]
pub struct RouterStateConfig {
    /// Parameter store configuration
    pub param_config: StateStoreConfig,
    /// TTL for signal definitions (None = never expire)
    pub signal_ttl: Option<Duration>,
    /// Maximum number of signals (None = unlimited)
    pub max_signals: Option<usize>,
}

impl Default for RouterStateConfig {
    fn default() -> Self {
        Self {
            param_config: StateStoreConfig::default(),
            signal_ttl: Some(Duration::from_secs(3600)), // 1 hour
            max_signals: Some(10_000),
        }
    }
}

impl RouterStateConfig {
    /// Create config with no limits (for backwards compatibility)
    pub fn unlimited() -> Self {
        Self {
            param_config: StateStoreConfig::unlimited(),
            signal_ttl: None,
            max_signals: None,
        }
    }
}

/// Global router state
pub struct RouterState {
    /// Parameter state store
    params: RwLock<StateStore>,
    /// Change listeners (for reactive updates)
    listeners: DashMap<String, Vec<Box<dyn Fn(&str, &Value) + Send + Sync>>>,
    /// Signal registry (announced signals from clients) with timestamps
    signals: DashMap<String, SignalEntry>,
    /// Configuration
    config: RouterStateConfig,
    /// Optional journal for state persistence and replay
    #[cfg(feature = "journal")]
    journal: Option<Arc<dyn Journal>>,
}

impl RouterState {
    pub fn new() -> Self {
        Self::with_config(RouterStateConfig::unlimited())
    }

    /// Create with specific configuration
    pub fn with_config(config: RouterStateConfig) -> Self {
        Self {
            params: RwLock::new(StateStore::with_config(config.param_config.clone())),
            listeners: DashMap::new(),
            signals: DashMap::new(),
            config,
            #[cfg(feature = "journal")]
            journal: None,
        }
    }

    /// Set the journal for state persistence and replay
    #[cfg(feature = "journal")]
    pub fn set_journal(&mut self, journal: Arc<dyn Journal>) {
        self.journal = Some(journal);
    }

    /// Get a reference to the journal (if configured)
    #[cfg(feature = "journal")]
    pub fn journal(&self) -> Option<&Arc<dyn Journal>> {
        self.journal.as_ref()
    }

    /// Register signals from an ANNOUNCE message
    pub fn register_signals(&self, signals: Vec<SignalDefinition>) {
        let now = Instant::now();
        for signal in signals {
            let address = signal.address.clone();
            self.signals.insert(
                address,
                SignalEntry {
                    definition: signal,
                    registered_at: now,
                    last_accessed: now,
                },
            );
        }
    }

    /// Query signals matching a pattern
    pub fn query_signals(&self, pattern: &str) -> Vec<SignalDefinition> {
        self.signals
            .iter()
            .filter(|entry| clasp_core::address::glob_match(pattern, entry.key()))
            .map(|entry| entry.value().definition.clone())
            .collect()
    }

    /// Get all registered signals
    pub fn all_signals(&self) -> Vec<SignalDefinition> {
        self.signals
            .iter()
            .map(|entry| entry.value().definition.clone())
            .collect()
    }

    /// Get signal count
    pub fn signal_count(&self) -> usize {
        self.signals.len()
    }

    /// Remove stale signals that haven't been accessed within the TTL
    /// Returns the number of signals removed
    pub fn cleanup_stale_signals(&self, ttl: Duration) -> usize {
        let now = Instant::now();
        let before = self.signals.len();
        self.signals
            .retain(|_, entry| now.duration_since(entry.last_accessed) < ttl);
        before - self.signals.len()
    }

    /// Remove stale params using the configured TTL
    /// Returns the number of params removed
    pub fn cleanup_stale_params(&self, ttl: Duration) -> usize {
        self.params.write().cleanup_stale(ttl)
    }

    /// Run all cleanup operations using configured TTLs
    /// Returns (params_removed, signals_removed)
    pub fn cleanup_stale(&self) -> (usize, usize) {
        let params_removed = if let Some(ttl) = self.config.param_config.param_ttl {
            self.params.write().cleanup_stale(ttl)
        } else {
            0
        };

        let signals_removed = if let Some(ttl) = self.config.signal_ttl {
            self.cleanup_stale_signals(ttl)
        } else {
            0
        };

        (params_removed, signals_removed)
    }

    /// Get a parameter value
    pub fn get(&self, address: &str) -> Option<Value> {
        self.params.read().get_value(address).cloned()
    }

    /// Get full parameter state
    pub fn get_state(&self, address: &str) -> Option<ParamState> {
        self.params.read().get(address).cloned()
    }

    /// Set a parameter value
    pub fn set(
        &self,
        address: &str,
        value: Value,
        writer: &SessionId,
        revision: Option<u64>,
        lock: bool,
        unlock: bool,
    ) -> Result<u64, UpdateError> {
        let result =
            self.params
                .write()
                .set(address, value.clone(), writer, revision, lock, unlock)?;

        // Notify listeners
        if let Some(listeners) = self.listeners.get(address) {
            for listener in listeners.iter() {
                listener(address, &value);
            }
        }

        Ok(result)
    }

    /// Apply a SET message
    pub fn apply_set(&self, msg: &SetMessage, writer: &SessionId) -> Result<u64, UpdateError> {
        let result = self.set(
            &msg.address,
            msg.value.clone(),
            writer,
            msg.revision,
            msg.lock,
            msg.unlock,
        )?;

        // Fire-and-forget journal append
        #[cfg(feature = "journal")]
        if let Some(ref journal) = self.journal {
            let entry = JournalEntry::from_set(
                msg.address.clone(),
                msg.value.clone(),
                result,
                writer.clone(),
                clasp_core::time::now(),
            );
            let journal = Arc::clone(journal);
            tokio::spawn(async move {
                let _ = journal.append(entry).await;
            });
        }

        Ok(result)
    }

    /// Record a PUBLISH event in the journal (fire-and-forget)
    #[cfg(feature = "journal")]
    pub fn journal_publish(
        &self,
        address: &str,
        signal_type: SignalType,
        value: Option<&Value>,
        author: &str,
    ) {
        if let Some(ref journal) = self.journal {
            let entry = JournalEntry::from_publish(
                address.to_string(),
                signal_type,
                value.cloned().unwrap_or(Value::Null),
                author.to_string(),
                clasp_core::time::now(),
            );
            let journal = Arc::clone(journal);
            tokio::spawn(async move {
                let _ = journal.append(entry).await;
            });
        }
    }

    /// Get all parameters matching a pattern
    pub fn get_matching(&self, pattern: &str) -> Vec<(String, ParamState)> {
        self.params
            .read()
            .get_matching(pattern)
            .into_iter()
            .map(|(k, v)| (k.to_string(), v.clone()))
            .collect()
    }

    /// Create a snapshot of all params matching a pattern
    pub fn snapshot(&self, pattern: &str) -> SnapshotMessage {
        let params: Vec<ParamValue> = self
            .get_matching(pattern)
            .into_iter()
            .map(|(address, state)| ParamValue {
                address,
                value: state.value,
                revision: state.revision,
                writer: Some(state.writer),
                timestamp: Some(state.timestamp),
            })
            .collect();

        SnapshotMessage { params }
    }

    /// Create a full snapshot
    pub fn full_snapshot(&self) -> SnapshotMessage {
        self.snapshot("**")
    }

    /// Recover state from journal snapshot and replay entries.
    ///
    /// Loads the most recent snapshot, then replays any entries appended after
    /// the snapshot was taken. This enables crash recovery.
    #[cfg(feature = "journal")]
    pub async fn recover_from_journal(&self) -> std::result::Result<usize, String> {
        let journal = self
            .journal
            .as_ref()
            .ok_or_else(|| "No journal configured".to_string())?;

        let mut recovered = 0;

        // Load snapshot if available
        if let Ok(Some(snapshots)) = journal.load_snapshot().await {
            for snap in &snapshots {
                let _ = self.set(
                    &snap.address,
                    snap.value.clone(),
                    &snap.writer,
                    Some(snap.revision),
                    false,
                    false,
                );
                recovered += 1;
            }
            tracing::info!("Recovered {} params from journal snapshot", recovered);
        }

        // Replay entries since the snapshot
        // For simplicity, replay all SET entries (they're idempotent with LWW)
        if let Ok(entries) = journal.since(0, None).await {
            for entry in &entries {
                if entry.msg_type == 0x21 {
                    // SET
                    if let Some(revision) = entry.revision {
                        let _ = self.set(
                            &entry.address,
                            entry.value.clone(),
                            &entry.author,
                            Some(revision),
                            false,
                            false,
                        );
                        recovered += 1;
                    }
                }
            }
            tracing::info!(
                "Replayed {} journal entries ({} were SET operations)",
                entries.len(),
                entries.iter().filter(|e| e.msg_type == 0x21).count()
            );
        }

        Ok(recovered)
    }

    /// Save current state as a journal snapshot.
    #[cfg(feature = "journal")]
    pub async fn save_snapshot(&self) -> std::result::Result<u64, String> {
        let journal = self
            .journal
            .as_ref()
            .ok_or_else(|| "No journal configured".to_string())?;

        let all_params = self.get_matching("**");
        let snapshots: Vec<clasp_journal::ParamSnapshot> = all_params
            .into_iter()
            .map(|(address, state)| clasp_journal::ParamSnapshot {
                address,
                value: state.value,
                revision: state.revision,
                writer: state.writer,
                timestamp: state.timestamp,
            })
            .collect();

        journal
            .snapshot(&snapshots)
            .await
            .map_err(|e| e.to_string())
    }

    /// Number of parameters
    pub fn len(&self) -> usize {
        self.params.read().len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.params.read().is_empty()
    }

    /// Clear all state
    pub fn clear(&self) {
        self.params.write().clear();
    }
}

impl Default for RouterState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_state() {
        let state = RouterState::new();

        state
            .set(
                "/test/value",
                Value::Float(0.5),
                &"session1".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        let value = state.get("/test/value").unwrap();
        assert_eq!(value, Value::Float(0.5));
    }

    #[test]
    fn test_snapshot() {
        let state = RouterState::new();

        state
            .set(
                "/test/a",
                Value::Float(1.0),
                &"s1".to_string(),
                None,
                false,
                false,
            )
            .unwrap();
        state
            .set(
                "/test/b",
                Value::Float(2.0),
                &"s1".to_string(),
                None,
                false,
                false,
            )
            .unwrap();
        state
            .set(
                "/other/c",
                Value::Float(3.0),
                &"s1".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        let snapshot = state.snapshot("/test/**");
        assert_eq!(snapshot.params.len(), 2);
    }

    #[test]
    fn test_register_signals() {
        use clasp_core::SignalType;

        let state = RouterState::new();

        let signals = vec![
            SignalDefinition {
                address: "/test/signal1".to_string(),
                signal_type: SignalType::Param,
                datatype: Some("float".to_string()),
                access: None,
                meta: None,
            },
            SignalDefinition {
                address: "/test/signal2".to_string(),
                signal_type: SignalType::Event,
                datatype: Some("bool".to_string()),
                access: None,
                meta: None,
            },
        ];

        state.register_signals(signals);
        assert_eq!(state.signal_count(), 2);

        let queried = state.query_signals("/test/**");
        assert_eq!(queried.len(), 2);
    }

    #[test]
    fn test_cleanup_stale_signals() {
        use clasp_core::SignalType;

        let config = RouterStateConfig {
            param_config: StateStoreConfig::unlimited(),
            signal_ttl: Some(Duration::from_millis(10)),
            max_signals: None,
        };
        let state = RouterState::with_config(config);

        let signals = vec![SignalDefinition {
            address: "/test/signal".to_string(),
            signal_type: SignalType::Param,
            datatype: Some("float".to_string()),
            access: None,
            meta: None,
        }];

        state.register_signals(signals);
        assert_eq!(state.signal_count(), 1);

        // Immediate cleanup should remove nothing
        let removed = state.cleanup_stale_signals(Duration::from_millis(10));
        assert_eq!(removed, 0);

        // Wait and cleanup
        std::thread::sleep(Duration::from_millis(15));
        let removed = state.cleanup_stale_signals(Duration::from_millis(10));
        assert_eq!(removed, 1);
        assert_eq!(state.signal_count(), 0);
    }

    #[test]
    fn test_cleanup_stale_all() {
        use clasp_core::SignalType;

        let config = RouterStateConfig {
            param_config: StateStoreConfig::with_limits(1000, 1), // 1 second TTL
            signal_ttl: Some(Duration::from_millis(10)),
            max_signals: None,
        };
        let state = RouterState::with_config(config);

        // Add a param and signal
        state
            .set(
                "/test/param",
                Value::Float(1.0),
                &"s1".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        let signals = vec![SignalDefinition {
            address: "/test/signal".to_string(),
            signal_type: SignalType::Param,
            datatype: Some("float".to_string()),
            access: None,
            meta: None,
        }];
        state.register_signals(signals);

        assert_eq!(state.len(), 1);
        assert_eq!(state.signal_count(), 1);

        // Wait for signal TTL to expire
        std::thread::sleep(Duration::from_millis(15));
        let (params_removed, signals_removed) = state.cleanup_stale();

        // Signal should be removed, param should still be there (1 second TTL)
        assert_eq!(signals_removed, 1);
        assert_eq!(params_removed, 0);
        assert_eq!(state.signal_count(), 0);
        assert_eq!(state.len(), 1);
    }
}
