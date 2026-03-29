//! DefraDB-backed CLASP configuration storage.
//!
//! Stores router, connection, bridge, and rule configurations in
//! [DefraDB](https://docs.source.network/defradb), a peer-to-peer document
//! database built on Merkle CRDTs. This enables:
//!
//! - **P2P config sync** between team members via DefraDB's Merkle CRDTs
//! - **Version history** via DefraDB's Merkle DAG (time-travel queries)
//! - **Access control** via DID-based owner fields
//!
//! # Usage
//!
//! ```no_run
//! use clasp_config_defra::DefraConfigStore;
//!
//! # async fn example() -> clasp_config_defra::Result<()> {
//! let store = DefraConfigStore::new("http://localhost:9181").await?;
//! let routers = store.list_routers().await?;
//! # Ok(())
//! # }
//! ```

mod convert;
mod error;
pub mod policy;
mod schema;
mod store;
mod types;

pub use error::{ConfigDefraError, Result};
pub use store::DefraConfigStore;
pub use types::{BridgeConfig, ConfigSnapshot, ConnectionConfig, RouterConfig, RuleConfig};
