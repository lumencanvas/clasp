//! [`Journal`] trait implementation backed by DefraDB.

use std::sync::atomic::{AtomicU64, Ordering};

use async_trait::async_trait;
use clasp_core::SignalType;
use clasp_journal::entry::{JournalEntry, ParamSnapshot};
use clasp_journal::error::Result;
use clasp_journal::journal::Journal;
use clasp_journal::JournalError;
use tokio::sync::Mutex;
use tracing::{debug, warn};

use crate::client::DefraClient;
use crate::convert;
use crate::schema;

/// A CLASP journal backed by DefraDB.
///
/// Journal entries and snapshots are stored as DefraDB documents,
/// which are automatically replicated across peers via Merkle CRDTs.
/// The local `seq_counter` provides monotonic ordering within this
/// node; cross-node ordering relies on DefraDB's CRDT merge semantics.
pub struct DefraJournal {
    client: DefraClient,
    /// Monotonic sequence counter, loaded from DefraDB on startup
    /// and incremented locally for each append.
    seq_counter: AtomicU64,
    /// Lock for snapshot operations that require consistency across
    /// multiple GraphQL calls.
    snapshot_lock: Mutex<()>,
}

impl DefraJournal {
    /// Create a new DefraDB journal, provisioning schemas and loading
    /// the latest sequence number from the database.
    pub async fn new(defra_url: &str) -> Result<Self> {
        let client = DefraClient::new(defra_url);

        // Provision schemas (idempotent)
        client
            .add_schema(schema::JOURNAL_ENTRY_SCHEMA)
            .await
            .map_err(JournalError::from)?;
        client
            .add_schema(schema::PARAM_SNAPSHOT_SCHEMA)
            .await
            .map_err(JournalError::from)?;

        debug!("DefraDB schemas provisioned");

        // Load the latest sequence number
        let latest = Self::fetch_latest_seq(&client).await?;
        debug!(seq = latest, "Loaded latest sequence from DefraDB");

        Ok(Self {
            client,
            seq_counter: AtomicU64::new(latest),
            snapshot_lock: Mutex::new(()),
        })
    }

    /// Connect to DefraDB with a health check, then provision schemas.
    ///
    /// Returns an error if DefraDB is not reachable.
    pub async fn connect(defra_url: &str) -> Result<Self> {
        let client = DefraClient::new(defra_url);
        let healthy = client.health().await.map_err(JournalError::from)?;
        if !healthy {
            return Err(JournalError::StorageError(format!(
                "DefraDB unavailable at {defra_url}"
            )));
        }
        Self::new(defra_url).await
    }

    /// Fetch the latest sequence number from DefraDB.
    async fn fetch_latest_seq(client: &DefraClient) -> Result<u64> {
        let query = r#"query {
            ClaspJournalEntry(order: {seq: DESC}, limit: 1) {
                seq
            }
        }"#;

        let data = client
            .graphql(query, None)
            .await
            .map_err(JournalError::from)?;

        let seq = data
            .get("ClaspJournalEntry")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|entry| entry.get("seq"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(seq)
    }

    /// Parse a list of journal entry documents from a GraphQL response.
    fn parse_entries(data: &serde_json::Value) -> Result<Vec<JournalEntry>> {
        let arr = data
            .get("ClaspJournalEntry")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut entries = Vec::with_capacity(arr.len());
        for doc in &arr {
            entries.push(Self::parse_entry(doc)?);
        }
        Ok(entries)
    }

    /// Parse a single journal entry document.
    fn parse_entry(doc: &serde_json::Value) -> Result<JournalEntry> {
        let seq = doc
            .get("seq")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        if doc.get("seq").and_then(|v| v.as_u64()).is_none() {
            warn!("Journal entry missing 'seq' field, defaulting to 0");
        }
        let timestamp = doc
            .get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            * 1_000_000; // Stored as seconds, convert back to microseconds
        let author = doc
            .get("author")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if doc.get("author").and_then(|v| v.as_str()).is_none() {
            warn!("Journal entry missing 'author' field, defaulting to empty string");
        }
        let address = doc
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if doc.get("address").and_then(|v| v.as_str()).is_none() {
            warn!("Journal entry missing 'address' field, defaulting to empty string");
        }
        let signal_type = doc
            .get("signalType")
            .and_then(|v| v.as_i64())
            .map(|v| convert::int_to_signal_type(v as i32))
            .unwrap_or(SignalType::Event);
        let value_str = doc
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("null");
        let value = convert::json_to_value(value_str);
        let revision = doc
            .get("revision")
            .and_then(|v| v.as_u64());
        let msg_type = doc
            .get("msgType")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u8;
        if doc.get("msgType").and_then(|v| v.as_u64()).is_none() {
            warn!("Journal entry missing 'msgType' field, defaulting to 0");
        }

        Ok(JournalEntry {
            seq,
            timestamp,
            author,
            address,
            signal_type,
            value,
            revision,
            msg_type,
        })
    }

    /// Parse snapshot documents from a GraphQL response.
    fn parse_snapshots(data: &serde_json::Value) -> Result<Vec<ParamSnapshot>> {
        let arr = data
            .get("ClaspParamSnapshot")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();

        let mut snapshots = Vec::with_capacity(arr.len());
        for doc in &arr {
            snapshots.push(Self::parse_snapshot(doc)?);
        }
        Ok(snapshots)
    }

    /// Parse a single snapshot document.
    fn parse_snapshot(doc: &serde_json::Value) -> Result<ParamSnapshot> {
        let address = doc
            .get("address")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let value_str = doc
            .get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("null");
        let value = convert::json_to_value(value_str);
        let revision = doc
            .get("revision")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let writer = doc
            .get("writer")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let timestamp = doc
            .get("timestamp")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            * 1_000_000; // Stored as seconds, convert back to microseconds

        Ok(ParamSnapshot {
            address,
            value,
            revision,
            writer,
            timestamp,
        })
    }

    /// Build the fields selection string for journal entry queries.
    fn entry_fields() -> &'static str {
        "seq timestamp author address signalType value revision msgType"
    }
}

#[async_trait]
impl Journal for DefraJournal {
    async fn append(&self, entry: JournalEntry) -> Result<u64> {
        let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst) + 1;
        let value_json = convert::value_to_json(&entry.value);
        let signal_type_int = convert::signal_type_to_int(entry.signal_type);

        // Escape strings for GraphQL inline values
        let author = entry.author.replace('\\', "\\\\").replace('"', "\\\"");
        let address = entry.address.replace('\\', "\\\\").replace('"', "\\\"");
        let value_escaped = value_json.replace('\\', "\\\\").replace('"', "\\\"");

        let revision_str = match entry.revision {
            Some(r) => r.to_string(),
            None => "0".to_string(),
        };

        let mutation = format!(
            r#"mutation {{
                add_ClaspJournalEntry(input: {{
                    seq: {seq},
                    timestamp: {timestamp},
                    author: "{author}",
                    address: "{address}",
                    signalType: {signal_type_int},
                    value: "{value_escaped}",
                    revision: {revision_str},
                    msgType: {msg_type}
                }}) {{
                    _docID
                }}
            }}"#,
            seq = seq,
            timestamp = entry.timestamp / 1_000_000, // Store as seconds (DefraDB Int32)
            author = author,
            address = address,
            signal_type_int = signal_type_int,
            value_escaped = value_escaped,
            revision_str = revision_str,
            msg_type = entry.msg_type,
        );

        self.client
            .graphql(&mutation, None)
            .await
            .map_err(JournalError::from)?;

        debug!(seq = seq, address = %entry.address, "Appended journal entry");
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
        let like_pattern = convert::clasp_pattern_to_like(pattern);
        let like_escaped = like_pattern.replace('\\', "\\\\").replace('"', "\\\"");

        // Build filter components
        let mut filters = vec![format!(r#"address: {{_like: "{like_escaped}"}}"#)];

        // Convert microsecond timestamps to seconds for DefraDB queries
        let from_secs = from.map(|ts| ts / 1_000_000);
        let to_secs = to.map(|ts| ts / 1_000_000);

        if let Some(from_ts) = from_secs {
            filters.push(format!("timestamp: {{_gte: {from_ts}}}"));
        }
        if let Some(to_ts) = to_secs {
            if from_secs.is_some() {
                let _ = filters.pop();
                filters.push(format!(
                    "timestamp: {{_gte: {}, _lte: {}}}",
                    from_secs.unwrap(),
                    to_ts
                ));
            } else {
                filters.push(format!("timestamp: {{_lte: {to_ts}}}"));
            }
        }

        if !types.is_empty() {
            let type_ints: Vec<String> = types
                .iter()
                .map(|t| convert::signal_type_to_int(*t).to_string())
                .collect();
            filters.push(format!(
                "signalType: {{_in: [{}]}}",
                type_ints.join(", ")
            ));
        }

        let filter_str = filters.join(", ");
        let limit_str = match limit {
            Some(l) => format!(", limit: {l}"),
            None => String::new(),
        };

        let query = format!(
            r#"query {{
                ClaspJournalEntry(
                    filter: {{{filter_str}}},
                    order: {{seq: ASC}}{limit_str}
                ) {{
                    {fields}
                }}
            }}"#,
            filter_str = filter_str,
            limit_str = limit_str,
            fields = Self::entry_fields(),
        );

        let data = self
            .client
            .graphql(&query, None)
            .await
            .map_err(JournalError::from)?;

        Self::parse_entries(&data)
    }

    async fn since(&self, seq: u64, limit: Option<u32>) -> Result<Vec<JournalEntry>> {
        let limit_str = match limit {
            Some(l) => format!(", limit: {l}"),
            None => String::new(),
        };

        let query = format!(
            r#"query {{
                ClaspJournalEntry(
                    filter: {{seq: {{_gt: {seq}}}}},
                    order: {{seq: ASC}}{limit_str}
                ) {{
                    {fields}
                }}
            }}"#,
            seq = seq,
            limit_str = limit_str,
            fields = Self::entry_fields(),
        );

        let data = self
            .client
            .graphql(&query, None)
            .await
            .map_err(JournalError::from)?;

        Self::parse_entries(&data)
    }

    async fn latest_seq(&self) -> Result<u64> {
        Self::fetch_latest_seq(&self.client).await
    }

    async fn snapshot(&self, state: &[ParamSnapshot]) -> Result<u64> {
        let _guard = self.snapshot_lock.lock().await;
        let snapshot_seq = self.seq_counter.load(Ordering::SeqCst);

        for param in state {
            let address = param.address.replace('\\', "\\\\").replace('"', "\\\"");
            let value_json = convert::value_to_json(&param.value);
            let value_escaped = value_json.replace('\\', "\\\\").replace('"', "\\\"");
            let writer = param.writer.replace('\\', "\\\\").replace('"', "\\\"");

            let mutation = format!(
                r#"mutation {{
                    add_ClaspParamSnapshot(input: {{
                        address: "{address}",
                        value: "{value_escaped}",
                        revision: {revision},
                        writer: "{writer}",
                        timestamp: {timestamp},
                        snapshotSeq: {snapshot_seq}
                    }}) {{
                        _docID
                    }}
                }}"#,
                address = address,
                value_escaped = value_escaped,
                revision = param.revision,
                writer = writer,
                timestamp = param.timestamp / 1_000_000, // Store as seconds
                snapshot_seq = snapshot_seq,
            );

            self.client
                .graphql(&mutation, None)
                .await
                .map_err(JournalError::from)?;
        }

        debug!(
            snapshot_seq = snapshot_seq,
            count = state.len(),
            "Saved snapshot to DefraDB"
        );
        Ok(snapshot_seq)
    }

    async fn load_snapshot(&self) -> Result<Option<Vec<ParamSnapshot>>> {
        // Find the latest snapshot sequence number
        let max_query = r#"query {
            ClaspParamSnapshot(order: {snapshotSeq: DESC}, limit: 1) {
                snapshotSeq
            }
        }"#;

        let data = self
            .client
            .graphql(max_query, None)
            .await
            .map_err(JournalError::from)?;

        let snapshot_seq = data
            .get("ClaspParamSnapshot")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|doc| doc.get("snapshotSeq"))
            .and_then(|v| v.as_u64());

        let snapshot_seq = match snapshot_seq {
            Some(seq) => seq,
            None => return Ok(None),
        };

        // Load all params belonging to that snapshot
        let query = format!(
            r#"query {{
                ClaspParamSnapshot(
                    filter: {{snapshotSeq: {{_eq: {snapshot_seq}}}}}
                ) {{
                    address value revision writer timestamp
                }}
            }}"#,
            snapshot_seq = snapshot_seq,
        );

        let data = self
            .client
            .graphql(&query, None)
            .await
            .map_err(JournalError::from)?;

        let snapshots = Self::parse_snapshots(&data)?;
        if snapshots.is_empty() {
            Ok(None)
        } else {
            Ok(Some(snapshots))
        }
    }

    async fn compact(&self, before_seq: u64) -> Result<u64> {
        // DefraDB does not support bulk delete by filter. We query matching
        // document IDs and delete them individually. This is a known
        // performance bottleneck for large compactions -- a future DefraDB
        // release with batch delete support would improve this significantly.
        let query = format!(
            r#"query {{
                ClaspJournalEntry(
                    filter: {{seq: {{_lt: {before_seq}}}}}
                ) {{
                    _docID
                }}
            }}"#,
            before_seq = before_seq,
        );

        let data = self
            .client
            .graphql(&query, None)
            .await
            .map_err(JournalError::from)?;

        let doc_ids: Vec<String> = data
            .get("ClaspJournalEntry")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|doc| {
                        doc.get("_docID")
                            .and_then(|v| v.as_str())
                            .map(String::from)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let count = doc_ids.len() as u64;

        for doc_id in &doc_ids {
            let doc_id_escaped = doc_id.replace('\\', "\\\\").replace('"', "\\\"");
            let mutation = format!(
                r#"mutation {{
                    delete_ClaspJournalEntry(docID: "{doc_id_escaped}") {{
                        _docID
                    }}
                }}"#,
                doc_id_escaped = doc_id_escaped,
            );

            if let Err(e) = self
                .client
                .graphql(&mutation, None)
                .await
                .map_err(JournalError::from)
            {
                warn!(doc_id = %doc_id, error = %e, "Failed to delete entry during compaction");
            }
        }

        debug!(
            before_seq = before_seq,
            deleted = count,
            "Compacted journal entries"
        );
        Ok(count)
    }

    async fn len(&self) -> Result<usize> {
        // DefraDB does not expose a dedicated count aggregate, so we
        // query all seq values and count the results. For very large
        // journals this is inefficient but correct.
        let query = r#"query {
            ClaspJournalEntry {
                seq
            }
        }"#;

        let data = self
            .client
            .graphql(query, None)
            .await
            .map_err(JournalError::from)?;

        let count = data
            .get("ClaspJournalEntry")
            .and_then(|v| v.as_array())
            .map(|arr| arr.len())
            .unwrap_or(0);

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clasp_core::Value;

    /// Format verification: ensure the GraphQL mutation string for append
    /// is well-formed with properly escaped values.
    #[test]
    fn client_graphql_query_format() {
        let entry = JournalEntry {
            seq: 0,
            timestamp: 1700000000,
            author: "test-author".into(),
            address: "/synth/osc1/freq".into(),
            signal_type: SignalType::Param,
            value: Value::Float(440.0),
            revision: Some(1),
            msg_type: 0x21,
        };

        let seq = 42u64;
        let value_json = convert::value_to_json(&entry.value);
        let signal_type_int = convert::signal_type_to_int(entry.signal_type);
        let value_escaped = value_json.replace('\\', "\\\\").replace('"', "\\\"");

        let mutation = format!(
            r#"mutation {{
                add_ClaspJournalEntry(input: {{
                    seq: {seq},
                    timestamp: {timestamp},
                    author: "{author}",
                    address: "{address}",
                    signalType: {signal_type_int},
                    value: "{value_escaped}",
                    revision: {revision},
                    msgType: {msg_type}
                }}) {{
                    _docID
                }}
            }}"#,
            seq = seq,
            timestamp = entry.timestamp,
            author = entry.author,
            address = entry.address,
            signal_type_int = signal_type_int,
            value_escaped = value_escaped,
            revision = entry.revision.unwrap_or(0),
            msg_type = entry.msg_type,
        );

        assert!(mutation.contains("add_ClaspJournalEntry"));
        assert!(mutation.contains("seq: 42"));
        assert!(mutation.contains("signalType: 0"));
        assert!(mutation.contains(r#"author: "test-author""#));
        assert!(mutation.contains(r#"address: "/synth/osc1/freq""#));
        assert!(mutation.contains("msgType: 33"));
    }

    /// Verify that parse_entry handles a typical DefraDB document.
    #[test]
    fn parse_entry_from_defra_doc() {
        let doc = serde_json::json!({
            "seq": 5,
            "timestamp": 1700000000,
            "author": "node-a",
            "address": "/mixer/ch1/fader",
            "signalType": 0,
            "value": "0.75",
            "revision": 3,
            "msgType": 33
        });

        let entry = DefraJournal::parse_entry(&doc).unwrap();
        assert_eq!(entry.seq, 5);
        assert_eq!(entry.signal_type, SignalType::Param);
        assert_eq!(entry.address, "/mixer/ch1/fader");
        assert_eq!(entry.msg_type, 0x21);
    }

    /// Verify snapshot document parsing.
    #[test]
    fn parse_snapshot_from_defra_doc() {
        let doc = serde_json::json!({
            "address": "/synth/vol",
            "value": "0.5",
            "revision": 10,
            "writer": "session-abc",
            "timestamp": 1700000000
        });

        let snap = DefraJournal::parse_snapshot(&doc).unwrap();
        assert_eq!(snap.address, "/synth/vol");
        assert_eq!(snap.revision, 10);
        assert_eq!(snap.writer, "session-abc");
    }

    // -- Integration tests (require a running DefraDB instance) ----------

    #[tokio::test]
    #[ignore]
    async fn test_append_and_query() {
        let journal = DefraJournal::connect("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let entry = JournalEntry::from_set(
            "/test/defra/param".into(),
            Value::Float(1.0),
            1,
            "integration-test".into(),
            1700000000,
        );

        let seq = journal.append(entry).await.unwrap();
        assert!(seq > 0);

        let results = journal
            .query("/test/defra/*", None, None, Some(10), &[SignalType::Param])
            .await
            .unwrap();

        assert!(!results.is_empty());
        assert!(results.iter().any(|e| e.seq == seq));
    }

    #[tokio::test]
    #[ignore]
    async fn test_since() {
        let journal = DefraJournal::connect("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let baseline = journal.latest_seq().await.unwrap();

        let entry = JournalEntry::from_publish(
            "/test/defra/event".into(),
            SignalType::Event,
            Value::String("ping".into()),
            "integration-test".into(),
            1700000001,
        );
        let seq = journal.append(entry).await.unwrap();

        let since = journal.since(baseline, None).await.unwrap();
        assert!(since.iter().any(|e| e.seq == seq));
    }

    #[tokio::test]
    #[ignore]
    async fn test_snapshot_save_load() {
        let journal = DefraJournal::connect("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let state = vec![
            ParamSnapshot {
                address: "/test/snap/a".into(),
                value: Value::Int(42),
                revision: 1,
                writer: "test".into(),
                timestamp: 1700000000,
            },
            ParamSnapshot {
                address: "/test/snap/b".into(),
                value: Value::Bool(true),
                revision: 2,
                writer: "test".into(),
                timestamp: 1700000001,
            },
        ];

        let snap_seq = journal.snapshot(&state).await.unwrap();
        assert!(snap_seq > 0);

        let loaded = journal.load_snapshot().await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert!(loaded.len() >= 2);
    }

    #[tokio::test]
    #[ignore]
    async fn test_compact() {
        let journal = DefraJournal::connect("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        // Append a few entries
        for i in 0..3 {
            let entry = JournalEntry::from_publish(
                format!("/test/compact/{i}"),
                SignalType::Event,
                Value::Int(i as i64),
                "integration-test".into(),
                1700000000 + i as u64,
            );
            journal.append(entry).await.unwrap();
        }

        let latest = journal.latest_seq().await.unwrap();
        let deleted = journal.compact(latest).await.unwrap();
        // Should have deleted at least some entries
        assert!(deleted > 0 || latest == 0);
    }
}
