//! DefraDB backend for the CLASP journal.
//!
//! Stores journal entries and snapshots in [DefraDB](https://docs.source.network/defradb),
//! a peer-to-peer document database built on Merkle CRDTs. This enables
//! automatic multi-node replication of journal state without explicit
//! federation logic -- DefraDB handles conflict resolution at the
//! document level.
//!
//! # Usage
//!
//! ```no_run
//! use clasp_journal_defra::DefraJournal;
//!
//! # async fn example() -> clasp_journal::Result<()> {
//! let journal = DefraJournal::connect("http://localhost:9181").await?;
//! # Ok(())
//! # }
//! ```

mod client;
mod convert;
mod error;
mod journal;
mod schema;

pub use client::DefraClient;
pub use journal::DefraJournal;
