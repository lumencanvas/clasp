//! Bidirectional bridge between DefraDB documents and CLASP real-time signals.
//!
//! This crate watches DefraDB for document changes and emits corresponding
//! CLASP signals, and conversely listens for CLASP SET signals on `/defra/**`
//! addresses to update DefraDB documents. This enables CLASP clients to
//! subscribe to DefraDB mutations using standard wildcard patterns.
//!
//! # Address Convention
//!
//! ```text
//! /defra/{collection}/{docID}          -- whole document
//! /defra/{collection}/{docID}/{field}  -- specific field
//! ```
//!
//! # Architecture
//!
//! The CLASP side is abstracted behind [`SignalSender`] and [`SignalReceiver`]
//! traits so the crate can be tested without a running router.

mod address;
mod bridge;
mod convert;
mod error;
mod traits;
mod watcher;
mod writer;

pub use address::{defra_to_clasp_address, parse_defra_address, DefraAddress};
pub use bridge::{DefraBridge, OriginTracker};
pub use convert::{clasp_value_to_json, json_doc_to_clasp_map, json_to_clasp_value};
pub use error::{BridgeError, Result};
pub use traits::{SignalReceiver, SignalSender};
pub use watcher::DefraWatcher;
pub use writer::DefraWriter;
