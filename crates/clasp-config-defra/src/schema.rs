//! DefraDB schema definitions for CLASP configuration storage.
//!
//! These GraphQL SDL schemas are provisioned on startup via the
//! DefraDB schema API. Each config entity becomes a conflict-free
//! replicated document, synced automatically across peers.

/// Schema for router configurations.
pub const ROUTER_CONFIG_SCHEMA: &str = r#"
type ClaspRouterConfig {
    configId: String @index
    name: String
    host: String
    port: Int
    transports: [String]
    securityMode: String
    maxSessions: Int
    paramTtlSecs: Int
    features: [String]
    owner: String @index
    updatedAt: Int
    version: Int
}
"#;

/// Schema for connection configurations.
pub const CONNECTION_CONFIG_SCHEMA: &str = r#"
type ClaspConnectionConfig {
    configId: String @index
    name: String
    routerUrl: String
    transport: String
    token: String
    reconnect: Boolean
    features: [String]
    owner: String @index
    updatedAt: Int
    version: Int
}
"#;

/// Schema for bridge configurations.
pub const BRIDGE_CONFIG_SCHEMA: &str = r#"
type ClaspBridgeConfig {
    configId: String @index
    name: String
    protocol: String
    sourceAddr: String
    targetAddr: String
    mappings: String
    active: Boolean
    owner: String @index
    updatedAt: Int
    version: Int
}
"#;

/// Schema for rule configurations.
pub const RULE_CONFIG_SCHEMA: &str = r#"
type ClaspRuleConfig {
    configId: String @index
    name: String
    trigger: String
    conditions: String
    actions: String
    cooldownSecs: Int
    enabled: Boolean
    owner: String @index
    updatedAt: Int
    version: Int
}
"#;

/// Schema for full configuration snapshots.
///
/// Nested config arrays are stored as JSON strings since DefraDB
/// uses a flat document model without nested collections.
pub const CONFIG_SNAPSHOT_SCHEMA: &str = r#"
type ClaspConfigSnapshot {
    snapshotId: String @index
    name: String
    description: String
    routers: String
    connections: String
    bridges: String
    rules: String
    owner: String @index
    createdAt: Int
}
"#;

/// All schemas that must be provisioned for config storage.
pub const ALL_SCHEMAS: &[&str] = &[
    ROUTER_CONFIG_SCHEMA,
    CONNECTION_CONFIG_SCHEMA,
    BRIDGE_CONFIG_SCHEMA,
    RULE_CONFIG_SCHEMA,
    CONFIG_SNAPSHOT_SCHEMA,
];
