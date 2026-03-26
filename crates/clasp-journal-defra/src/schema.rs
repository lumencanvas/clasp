//! DefraDB schema definitions for journal storage.
//!
//! These GraphQL SDL schemas are provisioned on startup via the
//! DefraDB schema API. DefraDB stores documents as Merkle CRDTs,
//! so each field becomes a conflict-free replicated register.

/// Schema for journal entries.
///
/// `value` is stored as a JSON-serialized string because DefraDB
/// does not have a native CLASP Value type. Signal types are stored
/// as integers for compact indexing.
///
/// DefraDB `Int` is 32-bit signed (max 2,147,483,647).
/// Timestamps are stored as seconds since epoch.
/// This format is valid until January 19, 2038 (Y2038 problem).
/// If DefraDB adds Int64 support, migrate timestamps to microseconds.
pub const JOURNAL_ENTRY_SCHEMA: &str = r#"
type ClaspJournalEntry {
    seq: Int @index
    timestamp: Int
    author: String
    address: String @index
    signalType: Int
    value: String
    revision: Int
    msgType: Int
}
"#;

/// Schema for param snapshots.
///
/// Each snapshot is tagged with a `snapshotSeq` so that all params
/// belonging to the same snapshot can be loaded atomically.
pub const PARAM_SNAPSHOT_SCHEMA: &str = r#"
type ClaspParamSnapshot {
    address: String @index
    value: String
    revision: Int
    writer: String
    timestamp: Int
    snapshotSeq: Int @index
}
"#;
