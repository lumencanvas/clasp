//! DefraDB schema definition for parameter state.
//!
//! This GraphQL SDL schema is provisioned on startup via the DefraDB schema
//! API. Each field becomes a conflict-free replicated register backed by a
//! Merkle CRDT, enabling automatic P2P sync of parameter state.

/// Schema for CLASP parameter state documents.
///
/// `value` is stored as a JSON-serialized string because DefraDB does not
/// have a native CLASP Value type. Enums (strategy, ttlMode) are stored as
/// lowercase strings for readability in the DefraDB explorer.
pub const CLASP_PARAM_SCHEMA: &str = r#"
type ClaspParam {
    address: String @index
    value: String
    valueType: String
    revision: Int
    writer: String
    timestamp: Int
    lastAccessed: Int
    strategy: String
    lockHolder: String
    origin: String
    ttlMode: String
    ttlSecs: Int
}
"#;
