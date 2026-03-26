//! Top-level bridge that ties the watcher and writer together.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use clasp_journal_defra::DefraClient;

use crate::error::{BridgeError, Result};
use crate::traits::{SignalReceiver, SignalSender};
use crate::watcher::DefraWatcher;
use crate::writer::DefraWriter;

/// Tracks recently-written addresses to prevent echo loops.
///
/// When the bridge writes a value to DefraDB, it records the address. On
/// the next watcher poll, if the same address shows up as "changed", the
/// watcher knows it was the bridge's own write and suppresses the signal.
pub struct OriginTracker {
    recent_writes: RwLock<HashMap<String, Instant>>,
    ttl: Duration,
}

impl OriginTracker {
    /// Create a tracker with the given TTL for suppressed addresses.
    pub fn new(ttl: Duration) -> Self {
        Self {
            recent_writes: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    /// Record that the bridge just wrote to `address`.
    pub async fn mark(&self, address: &str) {
        let mut writes = self.recent_writes.write().await;
        writes.insert(address.to_string(), Instant::now());
    }

    /// Check if `address` was recently written by the bridge and should
    /// be suppressed. Also prunes expired entries.
    pub async fn should_suppress(&self, address: &str) -> bool {
        let mut writes = self.recent_writes.write().await;

        // Prune expired entries
        let now = Instant::now();
        writes.retain(|_, ts| now.duration_since(*ts) < self.ttl);

        writes.contains_key(address)
    }
}

/// Bidirectional bridge between DefraDB and CLASP.
///
/// Runs a [`DefraWatcher`] (DefraDB -> CLASP) and a [`DefraWriter`]
/// (CLASP -> DefraDB) concurrently, using an [`OriginTracker`] to
/// prevent echo loops.
pub struct DefraBridge {
    watcher: DefraWatcher,
    writer: DefraWriter,
    origin_tracker: Arc<OriginTracker>,
}

impl DefraBridge {
    /// Create a new bridge for the given DefraDB URL and collections.
    pub fn new(defra_url: &str, collections: Vec<String>) -> Self {
        let watch_client = DefraClient::new(defra_url);
        let write_client = DefraClient::new(defra_url);

        Self {
            watcher: DefraWatcher::new(watch_client, collections),
            writer: DefraWriter::new(write_client),
            origin_tracker: Arc::new(OriginTracker::new(Duration::from_secs(5))),
        }
    }

    /// Set the echo suppression TTL (default: 5 seconds).
    /// Changes within this window from the bridge's own writes are suppressed
    /// to prevent feedback loops.
    pub fn with_echo_ttl(mut self, ttl: Duration) -> Self {
        self.origin_tracker = Arc::new(OriginTracker::new(ttl));
        self
    }

    /// Override the watcher poll interval.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.watcher = self.watcher.with_poll_interval(interval);
        self
    }

    /// Start the bridge.
    ///
    /// Runs the watcher and writer concurrently until `shutdown` is set.
    pub async fn run(
        self,
        sender: Arc<dyn SignalSender>,
        receiver: Arc<dyn SignalReceiver>,
        shutdown: Arc<AtomicBool>,
    ) -> Result<()> {
        info!("DefraBridge starting");

        let tracker = self.origin_tracker.clone();
        let shutdown_watcher = shutdown.clone();
        let shutdown_writer = shutdown.clone();

        // Spawn watcher task (DefraDB -> CLASP)
        let watcher_handle = tokio::spawn(async move {
            self.watcher.run(sender, shutdown_watcher).await
        });

        // Spawn writer task (CLASP -> DefraDB)
        let writer = self.writer;
        let writer_handle = tokio::spawn(async move {
            let mut rx = receiver
                .subscribe("/defra/**")
                .await
                .map_err(|e| BridgeError::Subscription(e.to_string()))?;

            while !shutdown_writer.load(Ordering::Relaxed) {
                match rx.recv().await {
                    Some((address, value)) => {
                        // Mark this write so the watcher suppresses the echo
                        tracker.mark(&address).await;

                        if let Err(e) = writer.handle_signal(&address, value).await {
                            warn!(error = %e, address = %address, "writer failed");
                        }
                    }
                    None => {
                        debug!("signal receiver channel closed");
                        break;
                    }
                }
            }

            Ok::<(), BridgeError>(())
        });

        // Wait for both tasks
        let (watcher_result, writer_result) = tokio::join!(watcher_handle, writer_handle);

        // Propagate errors
        match watcher_result {
            Ok(Err(e)) => warn!(error = %e, "watcher error"),
            Err(e) => warn!(error = %e, "watcher task panicked"),
            _ => {}
        }
        match writer_result {
            Ok(Err(e)) => warn!(error = %e, "writer error"),
            Err(e) => warn!(error = %e, "writer task panicked"),
            _ => {}
        }

        info!("DefraBridge stopped");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn origin_tracker_prevents_echo() {
        let tracker = OriginTracker::new(Duration::from_millis(100));

        tracker.mark("/defra/User/bae-abc/name").await;
        assert!(tracker.should_suppress("/defra/User/bae-abc/name").await);
        assert!(!tracker.should_suppress("/defra/User/bae-abc/age").await);

        // Wait for TTL to expire
        tokio::time::sleep(Duration::from_millis(150)).await;
        assert!(!tracker.should_suppress("/defra/User/bae-abc/name").await);
    }
}
