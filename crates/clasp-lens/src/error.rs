//! Error types for the lens host.

use thiserror::Error;

pub type Result<T> = std::result::Result<T, LensError>;

#[derive(Debug, Error)]
pub enum LensError {
    #[error("failed to compile WASM module: {0}")]
    Compile(#[from] wasmtime::Error),

    #[error("WASM module missing required export: {name}")]
    MissingExport { name: String },

    #[error("lens returned error: {0}")]
    LensError(String),

    #[error("transport buffer decode error: {0}")]
    Decode(String),

    #[error("WASM memory error: {0}")]
    Memory(String),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("WASM memory access fault: {0}")]
    MemoryAccess(#[from] wasmtime::MemoryAccessError),

    #[error("lens returned end of stream (no output)")]
    EndOfStream,
}
