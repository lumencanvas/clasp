//! CLASP ACP policy definition for DefraDB.
//!
//! Defines the Zanzibar-style relationship-based access control policy
//! that governs who can read, write, and delete CLASP configuration
//! documents stored in DefraDB.
//!
//! The policy is registered with DefraDB on startup (if ACP is enabled),
//! and the returned policy ID is referenced by `@policy` directives in
//! the schema definitions.
//!
//! # Identity
//!
//! DefraDB ACP uses secp256k1 keys (not Ed25519). The identity is passed
//! as either a raw hex private key or a JWT on the Authorization header.
//! CLASP's native identity system uses Ed25519, so ACP-enabled deployments
//! need a separate secp256k1 key for DefraDB operations.

/// The CLASP ACP policy in DefraDB Policy Interface (DPI) YAML format.
///
/// This policy defines four resource types matching the config schemas:
/// router_config, connection_config, bridge_config, rule_config.
///
/// Each resource has an owner relation (the document creator) and an
/// operator relation (users granted access by the owner). Permissions
/// follow the Zanzibar model: expressions combine relations with union (+).
pub const CLASP_ACP_POLICY: &str = r#"
description: CLASP Bridge configuration access control policy

actor:
  name: actor

resources:
  router_config:
    permissions:
      read:
        expr: owner + operator
      write:
        expr: owner + operator
      delete:
        expr: owner
    relations:
      owner:
        types:
          - actor
      operator:
        types:
          - actor

  connection_config:
    permissions:
      read:
        expr: owner + operator
      write:
        expr: owner + operator
      delete:
        expr: owner
    relations:
      owner:
        types:
          - actor
      operator:
        types:
          - actor

  bridge_config:
    permissions:
      read:
        expr: owner + operator
      write:
        expr: owner + operator
      delete:
        expr: owner
    relations:
      owner:
        types:
          - actor
      operator:
        types:
          - actor

  rule_config:
    permissions:
      read:
        expr: owner + operator
      write:
        expr: owner + operator
      delete:
        expr: owner
    relations:
      owner:
        types:
          - actor
      operator:
        types:
          - actor

  config_snapshot:
    permissions:
      read:
        expr: owner
      write:
        expr: owner
      delete:
        expr: owner
    relations:
      owner:
        types:
          - actor
"#;

/// Schema variant with @policy directives for ACP-enabled DefraDB.
///
/// These schemas reference a policy ID that must be registered first
/// via `DefraClient::add_policy()`. The `%POLICY_ID%` placeholder
/// is replaced at runtime with the actual policy hash.
pub const ROUTER_CONFIG_SCHEMA_ACP: &str = r#"
type ClaspRouterConfig @policy(
    id: "%POLICY_ID%",
    resource: "router_config"
) {
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

pub const CONNECTION_CONFIG_SCHEMA_ACP: &str = r#"
type ClaspConnectionConfig @policy(
    id: "%POLICY_ID%",
    resource: "connection_config"
) {
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

pub const BRIDGE_CONFIG_SCHEMA_ACP: &str = r#"
type ClaspBridgeConfig @policy(
    id: "%POLICY_ID%",
    resource: "bridge_config"
) {
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

pub const RULE_CONFIG_SCHEMA_ACP: &str = r#"
type ClaspRuleConfig @policy(
    id: "%POLICY_ID%",
    resource: "rule_config"
) {
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

pub const CONFIG_SNAPSHOT_SCHEMA_ACP: &str = r#"
type ClaspConfigSnapshot @policy(
    id: "%POLICY_ID%",
    resource: "config_snapshot"
) {
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

/// All ACP-enabled schemas (with `%POLICY_ID%` placeholder).
pub const ALL_SCHEMAS_ACP: &[&str] = &[
    ROUTER_CONFIG_SCHEMA_ACP,
    CONNECTION_CONFIG_SCHEMA_ACP,
    BRIDGE_CONFIG_SCHEMA_ACP,
    RULE_CONFIG_SCHEMA_ACP,
    CONFIG_SNAPSHOT_SCHEMA_ACP,
];

/// Replace the `%POLICY_ID%` placeholder in a schema with the actual ID.
pub fn resolve_policy_id(schema: &str, policy_id: &str) -> String {
    schema.replace("%POLICY_ID%", policy_id)
}
