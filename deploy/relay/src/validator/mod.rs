//! Chat-specific write validation and snapshot filtering.
//!
//! Enforces server-side authorization rules for room admin/ban/meta paths
//! and namespace metadata paths. Also filters sensitive fields from snapshots.

mod filter;
pub(crate) mod helpers;
pub(crate) mod paths;
mod write;

#[cfg(test)]
mod tests;

pub use filter::ChatSnapshotFilter;
pub use write::ChatWriteValidator;
