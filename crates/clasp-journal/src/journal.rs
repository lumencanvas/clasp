//! Journal trait definition

use async_trait::async_trait;
use clasp_core::SignalType;

use crate::entry::{JournalEntry, ParamSnapshot};
use crate::error::Result;

/// Event journal for recording state changes and events.
///
/// The journal provides append-only storage for all state mutations,
/// enabling crash recovery, state replay, and federation sync.
#[async_trait]
pub trait Journal: Send + Sync {
    /// Append an entry to the journal. Returns the assigned sequence number.
    async fn append(&self, entry: JournalEntry) -> Result<u64>;

    /// Query entries matching a pattern within a time range.
    async fn query(
        &self,
        pattern: &str,
        from: Option<u64>,
        to: Option<u64>,
        limit: Option<u32>,
        types: &[SignalType],
    ) -> Result<Vec<JournalEntry>>;

    /// Get entries since a given sequence number.
    async fn since(&self, seq: u64, limit: Option<u32>) -> Result<Vec<JournalEntry>>;

    /// Get the latest sequence number.
    async fn latest_seq(&self) -> Result<u64>;

    /// Save a state snapshot for faster recovery.
    async fn snapshot(&self, state: &[ParamSnapshot]) -> Result<u64>;

    /// Load the most recent snapshot.
    async fn load_snapshot(&self) -> Result<Option<Vec<ParamSnapshot>>>;

    /// Remove entries older than the given sequence number.
    /// Returns the number of entries removed.
    async fn compact(&self, before_seq: u64) -> Result<u64>;

    /// Get the total number of entries in the journal.
    async fn len(&self) -> Result<usize>;
}
