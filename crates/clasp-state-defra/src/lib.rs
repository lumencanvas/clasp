//! DefraDB-backed state store for CLASP with write-through cache and P2P sync.
//!
//! Provides a [`DefraStateStore`] that keeps an in-memory cache (DashMap) for
//! hot-path reads/writes at sub-100us, while asynchronously flushing writes to
//! DefraDB for persistence. On startup, state is loaded from DefraDB -- no
//! journal replay needed.
//!
//! P2P state sync comes for free via DefraDB's Merkle CRDTs: a background sync
//! task polls DefraDB for remote changes and merges them into the local cache.
//!
//! # Usage
//!
//! ```no_run
//! use clasp_state_defra::{DefraStateStore, DefraStateConfig};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let store = DefraStateStore::new("http://localhost:9181", DefraStateConfig::default()).await?;
//! let writer_handle = store.start_writer();
//! # Ok(())
//! # }
//! ```

mod convert;
mod error;
mod schema;
mod store;

pub use error::{DefraStateError, Result};
pub use store::{CacheStats, DefraStateConfig, DefraStateStore};
