//! DefraDB schema definitions for entity registry storage.
//!
//! This GraphQL SDL schema is provisioned on startup via the
//! DefraDB schema API. DefraDB stores documents as Merkle CRDTs,
//! so each field becomes a conflict-free replicated register.

/// Schema for registry entities.
///
/// `publicKey` is hex-encoded (64 chars for 32-byte Ed25519 keys).
/// `metadata` is stored as a JSON-serialized string because DefraDB
/// does not have a native map type.
/// `createdAt` is a unix timestamp in seconds.
pub const ENTITY_SCHEMA: &str = r#"
type ClaspEntity {
    entityId: String @index
    entityType: String
    name: String
    publicKey: String @index
    createdAt: Int
    metadata: String
    tags: [String]
    namespaces: [String]
    scopes: [String]
    status: String
}
"#;
