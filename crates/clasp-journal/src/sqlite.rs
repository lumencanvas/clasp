//! SQLite-backed journal implementation

use async_trait::async_trait;
use clasp_core::SignalType;
#[cfg(feature = "integrity")]
use hmac::{Hmac, Mac};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
#[cfg(feature = "integrity")]
use sha2::Sha256;
#[cfg(feature = "integrity")]
use hex;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

use crate::entry::{JournalEntry, ParamSnapshot};
use crate::error::{JournalError, Result};
use crate::journal::Journal;

#[cfg(feature = "integrity")]
type HmacSha256 = Hmac<Sha256>;

/// SQL filter strategy for address patterns.
enum PatternFilter {
    /// No wildcards — use `WHERE address = ?`
    Exact(String),
    /// Ends with `/**` — use `WHERE address LIKE '<prefix>%'`
    Prefix(String),
    /// Ends with `/*` — use `LIKE '<prefix>%' AND NOT LIKE '<prefix>%/%'`
    PrefixOneLevel(String),
    /// `/**` or `**` — no address filter needed
    MatchAll,
    /// Wildcards in middle segments — fall back to Rust glob_match
    Complex,
}

/// SQLite-backed journal for persistent storage.
///
/// Uses WAL mode for concurrent read/write access.
/// Optionally verifies entry integrity via HMAC-SHA256 (requires `integrity` feature).
pub struct SqliteJournal {
    pub(crate) conn: Mutex<Connection>,
    #[cfg(feature = "integrity")]
    hmac_key: Option<[u8; 32]>,
}

impl SqliteJournal {
    /// Create a new SQLite journal at the given path.
    pub fn new(path: &str) -> Result<Self> {
        let conn =
            Connection::open(path).map_err(|e| JournalError::StorageError(e.to_string()))?;
        let journal = Self {
            conn: Mutex::new(conn),
            #[cfg(feature = "integrity")]
            hmac_key: None,
        };
        journal.init_tables()?;
        Ok(journal)
    }

    /// Create a new SQLite journal with HMAC integrity verification.
    ///
    /// When an HMAC key is provided, all writes compute HMAC-SHA256 over
    /// `address || value_json || timestamp`, and all reads verify the HMAC.
    /// Entries written without HMAC (NULL hmac column) are accepted without verification.
    #[cfg(feature = "integrity")]
    pub fn with_hmac(path: &str, key: [u8; 32]) -> Result<Self> {
        let conn =
            Connection::open(path).map_err(|e| JournalError::StorageError(e.to_string()))?;
        let journal = Self {
            conn: Mutex::new(conn),
            hmac_key: Some(key),
        };
        journal.init_tables()?;
        Ok(journal)
    }

    /// Create an in-memory SQLite journal (for testing).
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| JournalError::StorageError(e.to_string()))?;
        let journal = Self {
            conn: Mutex::new(conn),
            #[cfg(feature = "integrity")]
            hmac_key: None,
        };
        journal.init_tables()?;
        Ok(journal)
    }

    /// Create an in-memory journal with HMAC integrity verification (for testing).
    #[cfg(feature = "integrity")]
    pub fn in_memory_with_hmac(key: [u8; 32]) -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| JournalError::StorageError(e.to_string()))?;
        let journal = Self {
            conn: Mutex::new(conn),
            hmac_key: Some(key),
        };
        journal.init_tables()?;
        Ok(journal)
    }

    fn init_tables(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;

            CREATE TABLE IF NOT EXISTS journal_entries (
                seq INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp INTEGER NOT NULL,
                author TEXT NOT NULL,
                address TEXT NOT NULL,
                signal_type TEXT NOT NULL,
                value_json TEXT NOT NULL,
                revision INTEGER,
                msg_type INTEGER NOT NULL,
                hmac TEXT
            );

            CREATE INDEX IF NOT EXISTS idx_entries_address ON journal_entries(address);
            CREATE INDEX IF NOT EXISTS idx_entries_timestamp ON journal_entries(timestamp);
            CREATE INDEX IF NOT EXISTS idx_entries_signal_type ON journal_entries(signal_type);

            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at INTEGER NOT NULL,
                seq_at INTEGER NOT NULL,
                data_json TEXT NOT NULL
            );
            ",
        )
        .map_err(|e| JournalError::StorageError(e.to_string()))?;

        // Migration: add hmac column to existing databases that lack it
        let has_hmac: bool = conn
            .prepare("SELECT hmac FROM journal_entries LIMIT 0")
            .is_ok();
        if !has_hmac {
            conn.execute_batch("ALTER TABLE journal_entries ADD COLUMN hmac TEXT")
                .map_err(|e| JournalError::StorageError(format!("migration failed: {}", e)))?;
        }

        Ok(())
    }

    /// Classify a CLASP address pattern for SQL-level filtering.
    ///
    /// Pushes simple patterns to SQL WHERE clauses instead of fetching all rows
    /// and filtering in Rust, which is O(n) on the full journal.
    fn pattern_to_sql_filter(pattern: &str) -> PatternFilter {
        // /** or ** matches everything
        if pattern == "/**" || pattern == "**" {
            return PatternFilter::MatchAll;
        }

        // No wildcards at all => exact match
        if !pattern.contains('*') {
            return PatternFilter::Exact(pattern.to_string());
        }

        let parts: Vec<&str> = pattern.split('/').collect();
        // Check if only the last segment contains a wildcard
        let wildcard_in_middle = parts[..parts.len().saturating_sub(1)]
            .iter()
            .any(|p| p.contains('*'));

        if wildcard_in_middle {
            return PatternFilter::Complex;
        }

        let last = *parts.last().unwrap_or(&"");
        let prefix = parts[..parts.len() - 1].join("/");
        let prefix_with_slash = format!("{}/", prefix);

        if last == "**" {
            // /foo/** -> LIKE '/foo/%'
            PatternFilter::Prefix(prefix_with_slash)
        } else if last == "*" {
            // /foo/* -> LIKE '/foo/%' AND NOT LIKE '/foo/%/%'
            PatternFilter::PrefixOneLevel(prefix_with_slash)
        } else {
            // Last segment has embedded wildcard like "temp*" — complex
            PatternFilter::Complex
        }
    }

    fn signal_type_to_str(st: &SignalType) -> &'static str {
        match st {
            SignalType::Param => "param",
            SignalType::Event => "event",
            SignalType::Stream => "stream",
            SignalType::Gesture => "gesture",
            SignalType::Timeline => "timeline",
        }
    }

    fn str_to_signal_type(s: &str) -> SignalType {
        match s {
            "param" => SignalType::Param,
            "event" => SignalType::Event,
            "stream" => SignalType::Stream,
            "gesture" => SignalType::Gesture,
            "timeline" => SignalType::Timeline,
            _ => SignalType::Event,
        }
    }

    /// Compute HMAC-SHA256 over address, value_json, and timestamp.
    #[cfg(feature = "integrity")]
    fn compute_hmac(key: &[u8; 32], address: &str, value_json: &str, timestamp: u64) -> String {
        let mut mac =
            HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
        mac.update(address.as_bytes());
        mac.update(value_json.as_bytes());
        mac.update(&timestamp.to_le_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Verify an HMAC against computed value.
    #[cfg(feature = "integrity")]
    fn verify_hmac(
        key: &[u8; 32],
        address: &str,
        value_json: &str,
        timestamp: u64,
        expected: &str,
    ) -> bool {
        let computed = Self::compute_hmac(key, address, value_json, timestamp);
        // Constant-time comparison via the hmac crate
        let mut mac =
            HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
        mac.update(address.as_bytes());
        mac.update(value_json.as_bytes());
        mac.update(&timestamp.to_le_bytes());
        // Decode expected hex and verify
        if let Ok(expected_bytes) = hex::decode(expected) {
            mac.verify_slice(&expected_bytes).is_ok()
        } else {
            // Invalid hex in stored HMAC
            computed == expected
        }
    }
}

#[async_trait]
impl Journal for SqliteJournal {
    async fn append(&self, entry: JournalEntry) -> Result<u64> {
        let conn = self.conn.lock();
        let value_json = serde_json::to_string(&entry.value)
            .map_err(|e| JournalError::SerializationError(e.to_string()))?;

        #[cfg(feature = "integrity")]
        let hmac_value: Option<String> = self
            .hmac_key
            .map(|key| Self::compute_hmac(&key, &entry.address, &value_json, entry.timestamp));
        #[cfg(not(feature = "integrity"))]
        let hmac_value: Option<String> = None;

        conn.execute(
            "INSERT INTO journal_entries (timestamp, author, address, signal_type, value_json, revision, msg_type, hmac)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.timestamp as i64,
                entry.author,
                entry.address,
                Self::signal_type_to_str(&entry.signal_type),
                value_json,
                entry.revision.map(|r| r as i64),
                entry.msg_type as i64,
                hmac_value,
            ],
        )
        .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let seq = conn.last_insert_rowid() as u64;
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
        let conn = self.conn.lock();

        let mut sql = String::from(
            "SELECT seq, timestamp, author, address, signal_type, value_json, revision, msg_type, hmac FROM journal_entries WHERE 1=1",
        );
        let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        // Push pattern filtering to SQL where possible
        let sql_filter = Self::pattern_to_sql_filter(pattern);
        let needs_rust_filter = match &sql_filter {
            PatternFilter::Exact(addr) => {
                sql.push_str(&format!(" AND address = ?{}", sql_params.len() + 1));
                sql_params.push(Box::new(addr.clone()));
                false
            }
            PatternFilter::Prefix(prefix) => {
                // Globstar: /foo/** -> LIKE '/foo/%'
                let like = format!("{}%", prefix);
                sql.push_str(&format!(" AND address LIKE ?{}", sql_params.len() + 1));
                sql_params.push(Box::new(like));
                false
            }
            PatternFilter::PrefixOneLevel(prefix) => {
                // Single wildcard at end: /foo/* -> LIKE '/foo/%' AND NOT LIKE '/foo/%/%'
                let like = format!("{}%", prefix);
                let not_like = format!("{}%/%", prefix);
                sql.push_str(&format!(" AND address LIKE ?{}", sql_params.len() + 1));
                sql_params.push(Box::new(like));
                sql.push_str(&format!(" AND address NOT LIKE ?{}", sql_params.len() + 1));
                sql_params.push(Box::new(not_like));
                false
            }
            PatternFilter::MatchAll => false,
            PatternFilter::Complex => true,
        };

        if let Some(from) = from {
            sql.push_str(&format!(
                " AND timestamp >= ?{}",
                sql_params.len() + 1
            ));
            sql_params.push(Box::new(from as i64));
        }
        if let Some(to) = to {
            sql.push_str(&format!(
                " AND timestamp <= ?{}",
                sql_params.len() + 1
            ));
            sql_params.push(Box::new(to as i64));
        }
        if !types.is_empty() {
            let type_strs: Vec<String> = types
                .iter()
                .map(|t| format!("'{}'", Self::signal_type_to_str(t)))
                .collect();
            sql.push_str(&format!(
                " AND signal_type IN ({})",
                type_strs.join(",")
            ));
        }

        sql.push_str(" ORDER BY seq ASC");

        if let Some(limit) = limit {
            sql.push_str(&format!(
                " LIMIT ?{}",
                sql_params.len() + 1
            ));
            sql_params.push(Box::new(limit as i64));
        }

        let param_refs: Vec<&dyn rusqlite::types::ToSql> = sql_params.iter().map(|p| p.as_ref()).collect();
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                let value_json: String = row.get(5)?;
                let revision: Option<i64> = row.get(6)?;
                let sig_str: String = row.get(4)?;
                let stored_hmac: Option<String> = row.get(8)?;

                Ok((
                    JournalEntry {
                        seq: row.get::<_, i64>(0)? as u64,
                        timestamp: row.get::<_, i64>(1)? as u64,
                        author: row.get(2)?,
                        address: row.get(3)?,
                        signal_type: Self::str_to_signal_type(&sig_str),
                        value: serde_json::from_str(&value_json).unwrap_or(clasp_core::Value::Null),
                        revision: revision.map(|r| r as u64),
                        msg_type: row.get::<_, i64>(7)? as u8,
                    },
                    value_json,
                    stored_hmac,
                ))
            })
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (entry, _value_json, stored_hmac) =
                row.map_err(|e| JournalError::StorageError(e.to_string()))?;

            // Verify HMAC integrity if key is configured and entry has an HMAC
            #[cfg(feature = "integrity")]
            if let Some(ref key) = self.hmac_key {
                if let Some(ref expected_hmac) = stored_hmac {
                    if !Self::verify_hmac(key, &entry.address, &_value_json, entry.timestamp, expected_hmac) {
                        return Err(JournalError::IntegrityViolation {
                            seq: entry.seq,
                            reason: "HMAC mismatch".to_string(),
                        });
                    }
                }
                // If stored_hmac is None, entry was written without integrity — allow it
            }
            let _ = stored_hmac; // Suppress unused warning when integrity feature is off

            // Only fall back to Rust-side glob filtering for complex patterns
            if needs_rust_filter {
                if clasp_core::address::glob_match(pattern, &entry.address) {
                    results.push(entry);
                }
            } else {
                results.push(entry);
            }
        }

        Ok(results)
    }

    async fn since(&self, seq: u64, limit: Option<u32>) -> Result<Vec<JournalEntry>> {
        let conn = self.conn.lock();

        let sql = if let Some(limit) = limit {
            format!(
                "SELECT seq, timestamp, author, address, signal_type, value_json, revision, msg_type, hmac \
                 FROM journal_entries WHERE seq > ?1 ORDER BY seq ASC LIMIT {}",
                limit
            )
        } else {
            "SELECT seq, timestamp, author, address, signal_type, value_json, revision, msg_type, hmac \
             FROM journal_entries WHERE seq > ?1 ORDER BY seq ASC"
                .to_string()
        };

        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let rows = stmt
            .query_map(params![seq as i64], |row| {
                let value_json: String = row.get(5)?;
                let revision: Option<i64> = row.get(6)?;
                let sig_str: String = row.get(4)?;
                let stored_hmac: Option<String> = row.get(8)?;

                Ok((
                    JournalEntry {
                        seq: row.get::<_, i64>(0)? as u64,
                        timestamp: row.get::<_, i64>(1)? as u64,
                        author: row.get(2)?,
                        address: row.get(3)?,
                        signal_type: Self::str_to_signal_type(&sig_str),
                        value: serde_json::from_str(&value_json).unwrap_or(clasp_core::Value::Null),
                        revision: revision.map(|r| r as u64),
                        msg_type: row.get::<_, i64>(7)? as u8,
                    },
                    value_json,
                    stored_hmac,
                ))
            })
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let (entry, _value_json, stored_hmac) =
                row.map_err(|e| JournalError::StorageError(e.to_string()))?;

            #[cfg(feature = "integrity")]
            if let Some(ref key) = self.hmac_key {
                if let Some(ref expected_hmac) = stored_hmac {
                    if !Self::verify_hmac(key, &entry.address, &_value_json, entry.timestamp, expected_hmac) {
                        return Err(JournalError::IntegrityViolation {
                            seq: entry.seq,
                            reason: "HMAC mismatch".to_string(),
                        });
                    }
                }
            }
            let _ = stored_hmac;

            results.push(entry);
        }

        Ok(results)
    }

    async fn latest_seq(&self) -> Result<u64> {
        let conn = self.conn.lock();
        let seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM journal_entries",
                [],
                |row| row.get(0),
            )
            .map_err(|e| JournalError::StorageError(e.to_string()))?;
        Ok(seq as u64)
    }

    async fn snapshot(&self, state: &[ParamSnapshot]) -> Result<u64> {
        let conn = self.conn.lock();
        let seq: i64 = conn
            .query_row(
                "SELECT COALESCE(MAX(seq), 0) FROM journal_entries",
                [],
                |row| row.get(0),
            )
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let data_json = serde_json::to_string(state)
            .map_err(|e| JournalError::SerializationError(e.to_string()))?;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_micros() as i64)
            .unwrap_or(0);

        conn.execute(
            "INSERT INTO snapshots (created_at, seq_at, data_json) VALUES (?1, ?2, ?3)",
            params![now, seq, data_json],
        )
        .map_err(|e| JournalError::StorageError(e.to_string()))?;

        Ok(seq as u64)
    }

    async fn load_snapshot(&self) -> Result<Option<Vec<ParamSnapshot>>> {
        let conn = self.conn.lock();

        let result: std::result::Result<String, rusqlite::Error> = conn.query_row(
            "SELECT data_json FROM snapshots ORDER BY id DESC LIMIT 1",
            [],
            |row| row.get(0),
        );

        match result {
            Ok(json) => {
                let snapshots: Vec<ParamSnapshot> = serde_json::from_str(&json)
                    .map_err(|e| JournalError::SerializationError(e.to_string()))?;
                Ok(Some(snapshots))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(JournalError::StorageError(e.to_string())),
        }
    }

    async fn compact(&self, before_seq: u64) -> Result<u64> {
        let conn = self.conn.lock();
        let removed = conn
            .execute(
                "DELETE FROM journal_entries WHERE seq < ?1",
                params![before_seq as i64],
            )
            .map_err(|e| JournalError::StorageError(e.to_string()))?;
        Ok(removed as u64)
    }

    async fn len(&self) -> Result<usize> {
        let conn = self.conn.lock();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM journal_entries", [], |row| {
                row.get(0)
            })
            .map_err(|e| JournalError::StorageError(e.to_string()))?;
        Ok(count as usize)
    }
}

/// Command sent to the batching writer thread.
enum BatchCommand {
    Write(JournalEntry, oneshot::Sender<Result<u64>>),
    Shutdown,
}

/// Batching wrapper around `SqliteJournal`.
///
/// Writes are sent to a dedicated writer task that batches multiple INSERTs
/// into a single SQLite transaction, significantly improving write throughput.
/// Reads are delegated directly to the underlying `SqliteJournal`.
pub struct BatchingSqliteJournal {
    writer_tx: mpsc::Sender<BatchCommand>,
    inner: Arc<SqliteJournal>,
}

impl BatchingSqliteJournal {
    /// Create a new batching journal wrapping a `SqliteJournal`.
    ///
    /// `batch_size` — max entries per batch (default 100).
    /// `batch_timeout_ms` — max wait time before flushing a partial batch (default 10ms).
    pub fn new(inner: SqliteJournal, batch_size: usize, batch_timeout_ms: u64) -> Self {
        let inner = Arc::new(inner);
        let (tx, rx) = mpsc::channel::<BatchCommand>(4096);

        let write_inner = Arc::clone(&inner);
        tokio::spawn(async move {
            Self::writer_loop(write_inner, rx, batch_size, batch_timeout_ms).await;
        });

        Self {
            writer_tx: tx,
            inner,
        }
    }

    /// Create with default batch parameters (100 entries, 10ms timeout).
    pub fn with_defaults(inner: SqliteJournal) -> Self {
        Self::new(inner, 100, 10)
    }

    async fn writer_loop(
        inner: Arc<SqliteJournal>,
        mut rx: mpsc::Receiver<BatchCommand>,
        batch_size: usize,
        batch_timeout_ms: u64,
    ) {
        let timeout = std::time::Duration::from_millis(batch_timeout_ms);
        let mut batch: Vec<(JournalEntry, oneshot::Sender<Result<u64>>)> = Vec::with_capacity(batch_size);

        loop {
            // Wait for the first message
            let cmd = rx.recv().await;
            match cmd {
                Some(BatchCommand::Write(entry, tx)) => {
                    batch.push((entry, tx));
                }
                Some(BatchCommand::Shutdown) | None => {
                    // Flush remaining and exit
                    if !batch.is_empty() {
                        Self::flush_batch(&inner, &mut batch);
                    }
                    return;
                }
            }

            // Collect more messages up to batch_size or timeout
            let deadline = tokio::time::Instant::now() + timeout;
            while batch.len() < batch_size {
                match tokio::time::timeout_at(deadline, rx.recv()).await {
                    Ok(Some(BatchCommand::Write(entry, tx))) => {
                        batch.push((entry, tx));
                    }
                    Ok(Some(BatchCommand::Shutdown)) | Ok(None) => {
                        Self::flush_batch(&inner, &mut batch);
                        return;
                    }
                    Err(_) => break, // Timeout — flush what we have
                }
            }

            Self::flush_batch(&inner, &mut batch);
        }
    }

    fn flush_batch(
        inner: &SqliteJournal,
        batch: &mut Vec<(JournalEntry, oneshot::Sender<Result<u64>>)>,
    ) {
        let conn = inner.conn.lock();

        // Execute all INSERTs in a single transaction
        let tx_result = conn.execute_batch("BEGIN");
        if let Err(e) = tx_result {
            let err_msg = format!("batch BEGIN failed: {}", e);
            for (_, sender) in batch.drain(..) {
                let _ = sender.send(Err(JournalError::StorageError(err_msg.clone())));
            }
            return;
        }

        let mut results: Vec<std::result::Result<u64, String>> = Vec::with_capacity(batch.len());

        for (entry, _) in batch.iter() {
            let value_json = match serde_json::to_string(&entry.value) {
                Ok(j) => j,
                Err(e) => {
                    results.push(Err(e.to_string()));
                    continue;
                }
            };

            #[cfg(feature = "integrity")]
            let hmac_value: Option<String> = inner
                .hmac_key
                .map(|key| SqliteJournal::compute_hmac(&key, &entry.address, &value_json, entry.timestamp));
            #[cfg(not(feature = "integrity"))]
            let hmac_value: Option<String> = None;

            match conn.execute(
                "INSERT INTO journal_entries (timestamp, author, address, signal_type, value_json, revision, msg_type, hmac)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    entry.timestamp as i64,
                    entry.author,
                    entry.address,
                    SqliteJournal::signal_type_to_str(&entry.signal_type),
                    value_json,
                    entry.revision.map(|r| r as i64),
                    entry.msg_type as i64,
                    hmac_value,
                ],
            ) {
                Ok(_) => results.push(Ok(conn.last_insert_rowid() as u64)),
                Err(e) => results.push(Err(e.to_string())),
            }
        }

        let commit_result = conn.execute_batch("COMMIT");
        drop(conn); // Release lock before sending results

        if let Err(e) = commit_result {
            let err_msg = format!("batch COMMIT failed: {}", e);
            for (_, sender) in batch.drain(..) {
                let _ = sender.send(Err(JournalError::StorageError(err_msg.clone())));
            }
            return;
        }

        for ((_, sender), result) in batch.drain(..).zip(results) {
            let _ = sender.send(match result {
                Ok(seq) => Ok(seq),
                Err(e) => Err(JournalError::StorageError(e)),
            });
        }
    }
}

#[async_trait]
impl Journal for BatchingSqliteJournal {
    async fn append(&self, entry: JournalEntry) -> Result<u64> {
        let (tx, rx) = oneshot::channel();
        self.writer_tx
            .send(BatchCommand::Write(entry, tx))
            .await
            .map_err(|_| JournalError::StorageError("writer channel closed".to_string()))?;
        rx.await
            .map_err(|_| JournalError::StorageError("writer dropped response".to_string()))?
    }

    async fn query(
        &self,
        pattern: &str,
        from: Option<u64>,
        to: Option<u64>,
        limit: Option<u32>,
        types: &[SignalType],
    ) -> Result<Vec<JournalEntry>> {
        self.inner.query(pattern, from, to, limit, types).await
    }

    async fn since(&self, seq: u64, limit: Option<u32>) -> Result<Vec<JournalEntry>> {
        self.inner.since(seq, limit).await
    }

    async fn latest_seq(&self) -> Result<u64> {
        self.inner.latest_seq().await
    }

    async fn snapshot(&self, state: &[ParamSnapshot]) -> Result<u64> {
        self.inner.snapshot(state).await
    }

    async fn load_snapshot(&self) -> Result<Option<Vec<ParamSnapshot>>> {
        self.inner.load_snapshot().await
    }

    async fn compact(&self, before_seq: u64) -> Result<u64> {
        self.inner.compact(before_seq).await
    }

    async fn len(&self) -> Result<usize> {
        self.inner.len().await
    }
}

impl Drop for BatchingSqliteJournal {
    fn drop(&mut self) {
        let _ = self.writer_tx.try_send(BatchCommand::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clasp_core::Value;

    #[tokio::test]
    async fn test_sqlite_append_and_query() {
        let journal = SqliteJournal::in_memory().unwrap();

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
        assert_eq!(results[0].seq, 1);
    }

    #[tokio::test]
    async fn test_sqlite_since() {
        let journal = SqliteJournal::in_memory().unwrap();

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
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].seq, 4);
        assert_eq!(results[1].seq, 5);
    }

    #[tokio::test]
    async fn test_sqlite_snapshot() {
        let journal = SqliteJournal::in_memory().unwrap();

        let snapshots = vec![ParamSnapshot {
            address: "/test/a".to_string(),
            value: Value::Float(1.0),
            revision: 5,
            writer: "s1".to_string(),
            timestamp: 1000,
        }];

        journal.snapshot(&snapshots).await.unwrap();

        let loaded = journal.load_snapshot().await.unwrap().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].address, "/test/a");
    }

    #[tokio::test]
    async fn test_sqlite_compact() {
        let journal = SqliteJournal::in_memory().unwrap();

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
        assert_eq!(removed, 5);

        let len = journal.len().await.unwrap();
        assert_eq!(len, 5);
    }

    #[tokio::test]
    async fn test_sqlite_empty_snapshot() {
        let journal = SqliteJournal::in_memory().unwrap();
        let loaded = journal.load_snapshot().await.unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_sql_pattern_filtering() {
        let journal = SqliteJournal::in_memory().unwrap();

        // Insert entries at various addresses
        for (addr, val) in [
            ("/sensors/temp", 1),
            ("/sensors/humidity", 2),
            ("/sensors/temp/room1", 3),
            ("/sensors/temp/room2", 4),
            ("/lights/room1", 5),
        ] {
            let entry = JournalEntry::from_set(
                addr.to_string(),
                Value::Int(val),
                val as u64,
                "s1".to_string(),
                1000 * val as u64,
            );
            journal.append(entry).await.unwrap();
        }

        // Exact match
        let results = journal
            .query("/sensors/temp", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/sensors/temp");

        // Globstar — /sensors/**
        let results = journal
            .query("/sensors/**", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 4);

        // Single wildcard — /sensors/*
        let results = journal
            .query("/sensors/*", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 2); // temp, humidity (not temp/room1, temp/room2)

        // Match all — /**
        let results = journal
            .query("/**", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 5);

        // Exact miss
        let results = journal
            .query("/sensors/pressure", None, None, None, &[])
            .await
            .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_pattern_to_sql_filter() {
        // Exact
        match SqliteJournal::pattern_to_sql_filter("/sensors/temp") {
            PatternFilter::Exact(s) => assert_eq!(s, "/sensors/temp"),
            _ => panic!("expected Exact"),
        }
        // Globstar
        match SqliteJournal::pattern_to_sql_filter("/sensors/**") {
            PatternFilter::Prefix(s) => assert_eq!(s, "/sensors/"),
            _ => panic!("expected Prefix"),
        }
        // Single wildcard
        match SqliteJournal::pattern_to_sql_filter("/sensors/*") {
            PatternFilter::PrefixOneLevel(s) => assert_eq!(s, "/sensors/"),
            _ => panic!("expected PrefixOneLevel"),
        }
        // Match all
        assert!(matches!(
            SqliteJournal::pattern_to_sql_filter("/**"),
            PatternFilter::MatchAll
        ));
        // Complex (wildcard in middle)
        assert!(matches!(
            SqliteJournal::pattern_to_sql_filter("/sensors/*/temp"),
            PatternFilter::Complex
        ));
    }

    #[tokio::test]
    async fn test_batching_journal_basic() {
        let inner = SqliteJournal::in_memory().unwrap();
        let journal = BatchingSqliteJournal::with_defaults(inner);

        let entry = JournalEntry::from_set(
            "/test/batch".to_string(),
            Value::Float(1.0),
            1,
            "s1".to_string(),
            1000,
        );
        let seq = journal.append(entry).await.unwrap();
        assert!(seq > 0);

        let results = journal.query("/**", None, None, None, &[]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/test/batch");
    }

    #[tokio::test]
    async fn test_batching_journal_concurrent_writes() {
        let inner = SqliteJournal::in_memory().unwrap();
        let journal = Arc::new(BatchingSqliteJournal::with_defaults(inner));

        // Spawn many concurrent writes to exercise batching
        let mut handles = Vec::new();
        for i in 0..50 {
            let j = Arc::clone(&journal);
            handles.push(tokio::spawn(async move {
                let entry = JournalEntry::from_set(
                    format!("/test/{}", i),
                    Value::Int(i),
                    (i + 1) as u64,
                    "s1".to_string(),
                    1000 * i as u64,
                );
                j.append(entry).await.unwrap()
            }));
        }

        for h in handles {
            let seq = h.await.unwrap();
            assert!(seq > 0);
        }

        let len = journal.len().await.unwrap();
        assert_eq!(len, 50);
    }

    #[cfg(feature = "integrity")]
    #[tokio::test]
    async fn test_hmac_integrity_roundtrip() {
        let key = [42u8; 32];
        let journal = SqliteJournal::in_memory_with_hmac(key).unwrap();

        let entry = JournalEntry::from_set(
            "/test/hmac".to_string(),
            Value::Float(3.14),
            1,
            "author1".to_string(),
            1000000,
        );
        journal.append(entry).await.unwrap();

        // Query should succeed — HMAC is valid
        let results = journal.query("/**", None, None, None, &[]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/test/hmac");
    }

    #[cfg(feature = "integrity")]
    #[tokio::test]
    async fn test_hmac_detects_tampering() {
        let key = [42u8; 32];
        let journal = SqliteJournal::in_memory_with_hmac(key).unwrap();

        let entry = JournalEntry::from_set(
            "/test/tamper".to_string(),
            Value::Int(100),
            1,
            "author1".to_string(),
            1000000,
        );
        journal.append(entry).await.unwrap();

        // Tamper with the stored value
        {
            let conn = journal.conn.lock();
            conn.execute(
                "UPDATE journal_entries SET value_json = '999' WHERE address = '/test/tamper'",
                [],
            )
            .unwrap();
        }

        // Query should fail with IntegrityViolation
        let result = journal.query("/**", None, None, None, &[]).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            JournalError::IntegrityViolation { seq, reason } => {
                assert_eq!(seq, 1);
                assert_eq!(reason, "HMAC mismatch");
            }
            other => panic!("Expected IntegrityViolation, got: {:?}", other),
        }
    }

    #[cfg(feature = "integrity")]
    #[tokio::test]
    async fn test_hmac_allows_legacy_entries() {
        let key = [42u8; 32];
        let journal = SqliteJournal::in_memory_with_hmac(key).unwrap();

        // Insert entry without HMAC (simulating legacy data)
        {
            let conn = journal.conn.lock();
            conn.execute(
                "INSERT INTO journal_entries (timestamp, author, address, signal_type, value_json, revision, msg_type, hmac)
                 VALUES (1000, 'legacy', '/test/old', 'param', '42', 1, 33, NULL)",
                [],
            )
            .unwrap();
        }

        // Query should succeed — NULL HMAC entries are accepted
        let results = journal.query("/**", None, None, None, &[]).await.unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].address, "/test/old");
    }
}
