//! CLASP Entity Registry
//!
//! Persistent identity for devices, users, services, and routers in the CLASP network.
//! Each entity has an Ed25519 keypair and a stable ID derived from its public key.
//!
//! # Entity ID Format
//! ```text
//! clasp:<base58-ed25519-pubkey-prefix>
//! ```
//!
//! # Token Format
//! ```text
//! ent_<base64url(msgpack(entity_id + timestamp + ed25519_signature))>
//! ```
//!
//! # Storage Backends
//! - `MemoryEntityStore` -- default, no deps, for dev/testing
//! - `SqliteEntityStore` -- feature-gated behind `sqlite`, single file, WAL mode
//!
//! # Integration
//!
//! `EntityValidator` implements `clasp_core::TokenValidator` and plugs into the existing
//! `ValidatorChain`. Existing `cpsk_` tokens continue working unchanged.
//!
//! ```rust,no_run
//! use std::sync::Arc;
//! use clasp_core::ValidatorChain;
//! use clasp_registry::{EntityValidator, MemoryEntityStore};
//!
//! let store = Arc::new(MemoryEntityStore::new());
//! let chain = ValidatorChain::new()
//!     .with(clasp_core::CpskValidator::new())
//!     .with(EntityValidator::new(store));
//! ```

pub mod entity;
pub mod error;
#[cfg(feature = "sqlite")]
pub mod sqlite;
pub mod store;
pub mod token;
pub mod validator;

pub use entity::{Entity, EntityId, EntityKeypair, EntityStatus, EntityType};
pub use error::{RegistryError, Result};
#[cfg(feature = "sqlite")]
pub use sqlite::SqliteEntityStore;
pub use store::{EntityStore, MemoryEntityStore};
pub use token::{generate_token, parse_token, ENTITY_TOKEN_PREFIX};
pub use validator::EntityValidator;
