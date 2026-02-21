//! In-memory journal implementation (ring buffer)

use std::collections::VecDeque;

use async_trait::async_trait;
use clasp_core::SignalType;
use parking_lot::RwLock;

use crate::entry::{JournalEntry, ParamSnapshot};
use crate::error::Result;
use crate::journal::Journal;

/// In-memory journal backed by a ring buffer.
///
/// Useful for development, testing, and short-lived routers that don't
/// need persistence across restarts.
pub struct MemoryJournal {
    entries: RwLock<VecDeque<JournalEntry>>,
    snapshot: RwLock<Option<Vec<ParamSnapshot>>>,
    next_seq: RwLock<u64>,
    capacity: usize,
}

impl MemoryJournal {
    /// Create a new memory journal with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: RwLock::new(VecDeque::with_capacity(capacity)),
            snapshot: RwLock::new(None),
            next_seq: RwLock::new(1),
            capacity,
        }
    }

    /// Create with default capacity (10,000 entries)
    pub fn default_capacity() -> Self {
        Self::new(10_000)
    }
}

#[async_trait]
impl Journal for MemoryJournal {
    async fn append(&self, mut entry: JournalEntry) -> Result<u64> {
        let mut entries = self.entries.write();
        let mut next_seq = self.next_seq.write();

        // Assign sequence number
        entry.seq = *next_seq;
        *next_seq += 1;

        // Evict oldest if at capacity
        if entries.len() >= self.capacity {
            entries.pop_front();
        }

        let seq = entry.seq;
        entries.push_back(entry);
        Ok(seq)
    }

    async fn query(
        &self,
        pattern: &str,
        from: Option<u64>,
        to: Option<u64>,
        limit: Option<u32>,
        types: &[SignalType],
    ) -> Result<Vec<JournalEntry>> {
        let entries = self.entries.read();
        let limit = limit.unwrap_or(u32::MAX) as usize;

        let results: Vec<JournalEntry> = entries
            .iter()
            .filter(|e| {
                // Time range filter
                if let Some(from) = from {
                    if e.timestamp < from {
                        return false;
                    }
                }
                if let Some(to) = to {
                    if e.timestamp > to {
                        return false;
                    }
                }
                // Signal type filter
                if !types.is_empty() && !types.contains(&e.signal_type) {
                    return false;
                }
                // Pattern filter
                clasp_core::address::glob_match(pattern, &e.address)
            })
            .take(limit)
            .cloned()
            .collect();

        Ok(results)
    }

    async fn since(&self, seq: u64, limit: Option<u32>) -> Result<Vec<JournalEntry>> {
        let entries = self.entries.read();
        let limit = limit.unwrap_or(u32::MAX) as usize;

        let results: Vec<JournalEntry> = entries
            .iter()
            .filter(|e| e.seq > seq)
            .take(limit)
            .cloned()
            .collect();

        Ok(results)
    }

    async fn latest_seq(&self) -> Result<u64> {
        let next_seq = self.next_seq.read();
        Ok(next_seq.saturating_sub(1))
    }

    async fn snapshot(&self, state: &[ParamSnapshot]) -> Result<u64> {
        let seq = self.latest_seq().await?;
        *self.snapshot.write() = Some(state.to_vec());
        Ok(seq)
    }

    async fn load_snapshot(&self) -> Result<Option<Vec<ParamSnapshot>>> {
        Ok(self.snapshot.read().clone())
    }

    async fn compact(&self, before_seq: u64) -> Result<u64> {
        let mut entries = self.entries.write();
        let before = entries.len();
        entries.retain(|e| e.seq >= before_seq);
        Ok((before - entries.len()) as u64)
    }

    async fn len(&self) -> Result<usize> {
        Ok(self.entries.read().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clasp_core::Value;

    #[tokio::test]
    async fn test_append_and_query() {
        let journal = MemoryJournal::new(100);

        let entry = JournalEntry::from_set(
            "/test/value".to_string(),
            Value::Float(0.5),
            1,
            "session1".to_string(),
            1000000,
        );

        let seq = journal.append(entry).await.unwrap();
        assert_eq!(seq, 1);

        let results = journal.query("/**", None, None, None, &[]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/test/value");
    }

    #[tokio::test]
    async fn test_query_with_pattern() {
        let journal = MemoryJournal::new(100);

        for i in 0..5 {
            let addr = format!("/lights/room{}", i);
            let entry = JournalEntry::from_set(
                addr,
                Value::Float(i as f64 * 0.2),
                i + 1,
                "s1".to_string(),
                1000000 + i,
            );
            journal.append(entry).await.unwrap();
        }

        // Also add non-matching entries
        let entry = JournalEntry::from_set(
            "/audio/mixer".to_string(),
            Value::Float(0.8),
            1,
            "s1".to_string(),
            1000010,
        );
        journal.append(entry).await.unwrap();

        let results = journal
            .query("/lights/**", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 5);

        let results = journal
            .query("/audio/**", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_query_with_time_range() {
        let journal = MemoryJournal::new(100);

        for i in 0..10u64 {
            let entry = JournalEntry::from_set(
                "/test/value".to_string(),
                Value::Float(i as f64),
                i + 1,
                "s1".to_string(),
                i * 1000, // 0, 1000, 2000, ...
            );
            journal.append(entry).await.unwrap();
        }

        let results = journal
            .query("/**", Some(3000), Some(7000), None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 5); // timestamps 3000, 4000, 5000, 6000, 7000
    }

    #[tokio::test]
    async fn test_query_with_type_filter() {
        let journal = MemoryJournal::new(100);

        let set_entry = JournalEntry::from_set(
            "/test/param".to_string(),
            Value::Float(1.0),
            1,
            "s1".to_string(),
            1000,
        );
        journal.append(set_entry).await.unwrap();

        let pub_entry = JournalEntry::from_publish(
            "/test/event".to_string(),
            SignalType::Event,
            Value::Bool(true),
            "s1".to_string(),
            2000,
        );
        journal.append(pub_entry).await.unwrap();

        let results = journal
            .query("/**", None, None, None, &[SignalType::Param])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/test/param");

        let results = journal
            .query("/**", None, None, None, &[SignalType::Event])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/test/event");
    }

    #[tokio::test]
    async fn test_since() {
        let journal = MemoryJournal::new(100);

        for i in 0..5 {
            let entry = JournalEntry::from_set(
                format!("/test/{}", i),
                Value::Int(i),
                (i + 1) as u64,
                "s1".to_string(),
                1000 * i as u64,
            );
            journal.append(entry).await.unwrap();
        }

        let results = journal.since(3, None).await.unwrap();
        assert_eq!(results.len(), 2); // seq 4 and 5
        assert_eq!(results[0].seq, 4);
        assert_eq!(results[1].seq, 5);
    }

    #[tokio::test]
    async fn test_ring_buffer_eviction() {
        let journal = MemoryJournal::new(3);

        for i in 0..5 {
            let entry = JournalEntry::from_set(
                format!("/test/{}", i),
                Value::Int(i),
                (i + 1) as u64,
                "s1".to_string(),
                1000 * i as u64,
            );
            journal.append(entry).await.unwrap();
        }

        let len = journal.len().await.unwrap();
        assert_eq!(len, 3);

        // Should have entries 3, 4, 5 (oldest evicted)
        let results = journal.query("/**", None, None, None, &[]).await.unwrap();
        assert_eq!(results[0].seq, 3);
        assert_eq!(results[2].seq, 5);
    }

    #[tokio::test]
    async fn test_snapshot() {
        let journal = MemoryJournal::new(100);

        let snapshots = vec![
            ParamSnapshot {
                address: "/test/a".to_string(),
                value: Value::Float(1.0),
                revision: 5,
                writer: "s1".to_string(),
                timestamp: 1000,
            },
            ParamSnapshot {
                address: "/test/b".to_string(),
                value: Value::Float(2.0),
                revision: 3,
                writer: "s2".to_string(),
                timestamp: 2000,
            },
        ];

        journal.snapshot(&snapshots).await.unwrap();

        let loaded = journal.load_snapshot().await.unwrap().unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].address, "/test/a");
        assert_eq!(loaded[1].address, "/test/b");
    }

    #[tokio::test]
    async fn test_compact() {
        let journal = MemoryJournal::new(100);

        for i in 0..10 {
            let entry = JournalEntry::from_set(
                format!("/test/{}", i),
                Value::Int(i),
                (i + 1) as u64,
                "s1".to_string(),
                1000 * i as u64,
            );
            journal.append(entry).await.unwrap();
        }

        let removed = journal.compact(6).await.unwrap();
        assert_eq!(removed, 5); // Remove seq 1-5

        let len = journal.len().await.unwrap();
        assert_eq!(len, 5);
    }

    #[tokio::test]
    async fn test_latest_seq() {
        let journal = MemoryJournal::new(100);

        assert_eq!(journal.latest_seq().await.unwrap(), 0);

        let entry = JournalEntry::from_set(
            "/test".to_string(),
            Value::Null,
            1,
            "s1".to_string(),
            0,
        );
        journal.append(entry).await.unwrap();

        assert_eq!(journal.latest_seq().await.unwrap(), 1);
    }

    #[tokio::test]
    async fn test_query_with_limit() {
        let journal = MemoryJournal::new(100);

        for i in 0..10 {
            let entry = JournalEntry::from_set(
                "/test/value".to_string(),
                Value::Int(i),
                (i + 1) as u64,
                "s1".to_string(),
                1000 * i as u64,
            );
            journal.append(entry).await.unwrap();
        }

        let results = journal
            .query("/**", None, None, Some(3), &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 3);
    }
}
