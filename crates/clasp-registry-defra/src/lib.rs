//! DefraDB backend for the CLASP entity registry.
//!
//! Stores entity identity records in [DefraDB](https://docs.source.network/defradb),
//! a peer-to-peer document database built on Merkle CRDTs. This enables
//! automatic multi-node replication of registry state without explicit
//! federation logic -- DefraDB handles conflict resolution at the
//! document level.
//!
//! # Usage
//!
//! ```no_run
//! use clasp_registry_defra::DefraEntityStore;
//!
//! # async fn example() -> clasp_registry::Result<()> {
//! let store = DefraEntityStore::connect("http://localhost:9181").await?;
//! # Ok(())
//! # }
//! ```

mod convert;
mod error;
mod schema;
mod store;

pub use store::DefraEntityStore;
