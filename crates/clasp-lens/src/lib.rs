//! LensVM WASM transform host for CLASP
//!
//! Loads and executes LensVM WASM modules (compiled via `lens_sdk`) as signal
//! transforms. Each lens is a WASM module exporting `alloc`, `transform`, and
//! optionally `inverse` and `set_param`.
//!
//! The host implements the LensVM protocol: it provides the `next()` import
//! that feeds input values to the lens, and reads output via the transport
//! buffer format.
//!
//! # Transport Buffer Format
//!
//! Data crosses the WASM boundary as:
//! ```text
//! [TypeId: i8] [Length: u32 LE] [Payload: bytes]
//!
//! TypeId values:
//!   -1 = error (payload is UTF-8 error string)
//!    0 = nil (no length/payload)
//!    1 = JSON item (payload is JSON bytes)
//!  127 = end of stream
//! ```

pub mod error;
pub mod host;
pub mod transport;

pub use error::{LensError, Result};
pub use host::LensHost;
