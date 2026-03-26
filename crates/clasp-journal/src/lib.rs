//! Event log and state persistence for CLASP routers.
//!
//! Provides append-only journal storage for all state mutations,
//! enabling crash recovery, state replay, and federation sync.
//!
//! # Backends
//!
//! - [`MemoryJournal`] -- in-memory ring buffer for dev/testing
//! - [`SqliteJournal`] -- persistent SQLite storage (requires `sqlite` feature)
//! - `DefraJournal` -- DefraDB P2P backend via Merkle CRDTs (see `clasp-journal-defra` crate)

pub mod entry;
pub mod error;
pub mod journal;
pub mod memory;

#[cfg(feature = "sqlite")]
pub mod sqlite;

// Re-exports
pub use entry::{JournalEntry, ParamSnapshot};
pub use error::{JournalError, Result};
pub use journal::Journal;
pub use memory::MemoryJournal;

#[cfg(feature = "sqlite")]
pub use sqlite::{BatchingSqliteJournal, SqliteJournal};
