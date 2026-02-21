//! SQLite-backed journal implementation

use async_trait::async_trait;
use clasp_core::SignalType;
use parking_lot::Mutex;
use rusqlite::{params, Connection};

use crate::entry::{JournalEntry, ParamSnapshot};
use crate::error::{JournalError, Result};
use crate::journal::Journal;

/// SQLite-backed journal for persistent storage.
///
/// Uses WAL mode for concurrent read/write access.
pub struct SqliteJournal {
    conn: Mutex<Connection>,
}

impl SqliteJournal {
    /// Create a new SQLite journal at the given path.
    pub fn new(path: &str) -> Result<Self> {
        let conn =
            Connection::open(path).map_err(|e| JournalError::StorageError(e.to_string()))?;
        let journal = Self {
            conn: Mutex::new(conn),
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
                msg_type INTEGER NOT NULL
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
        Ok(())
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
}

#[async_trait]
impl Journal for SqliteJournal {
    async fn append(&self, entry: JournalEntry) -> Result<u64> {
        let conn = self.conn.lock();
        let value_json = serde_json::to_string(&entry.value)
            .map_err(|e| JournalError::SerializationError(e.to_string()))?;

        conn.execute(
            "INSERT INTO journal_entries (timestamp, author, address, signal_type, value_json, revision, msg_type)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry.timestamp as i64,
                entry.author,
                entry.address,
                Self::signal_type_to_str(&entry.signal_type),
                value_json,
                entry.revision.map(|r| r as i64),
                entry.msg_type as i64,
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

        // Build query dynamically -- we use a broad SELECT and filter by pattern in Rust
        // since glob_match is Rust-native, not SQL
        let mut sql = String::from(
            "SELECT seq, timestamp, author, address, signal_type, value_json, revision, msg_type FROM journal_entries WHERE 1=1",
        );
        let mut sql_params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

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

                Ok(JournalEntry {
                    seq: row.get::<_, i64>(0)? as u64,
                    timestamp: row.get::<_, i64>(1)? as u64,
                    author: row.get(2)?,
                    address: row.get(3)?,
                    signal_type: Self::str_to_signal_type(&sig_str),
                    value: serde_json::from_str(&value_json).unwrap_or(clasp_core::Value::Null),
                    revision: revision.map(|r| r as u64),
                    msg_type: row.get::<_, i64>(7)? as u8,
                })
            })
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            let entry = row.map_err(|e| JournalError::StorageError(e.to_string()))?;
            // Apply pattern filter in Rust
            if clasp_core::address::glob_match(pattern, &entry.address) {
                results.push(entry);
            }
        }

        Ok(results)
    }

    async fn since(&self, seq: u64, limit: Option<u32>) -> Result<Vec<JournalEntry>> {
        let conn = self.conn.lock();

        let sql = if let Some(limit) = limit {
            format!(
                "SELECT seq, timestamp, author, address, signal_type, value_json, revision, msg_type \
                 FROM journal_entries WHERE seq > ?1 ORDER BY seq ASC LIMIT {}",
                limit
            )
        } else {
            "SELECT seq, timestamp, author, address, signal_type, value_json, revision, msg_type \
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

                Ok(JournalEntry {
                    seq: row.get::<_, i64>(0)? as u64,
                    timestamp: row.get::<_, i64>(1)? as u64,
                    author: row.get(2)?,
                    address: row.get(3)?,
                    signal_type: Self::str_to_signal_type(&sig_str),
                    value: serde_json::from_str(&value_json).unwrap_or(clasp_core::Value::Null),
                    revision: revision.map(|r| r as u64),
                    msg_type: row.get::<_, i64>(7)? as u8,
                })
            })
            .map_err(|e| JournalError::StorageError(e.to_string()))?;

        let mut results = Vec::new();
        for row in rows {
            results.push(row.map_err(|e| JournalError::StorageError(e.to_string()))?);
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
}
