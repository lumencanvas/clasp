//! Conversion between Rust config types and DefraDB GraphQL representations.

use serde_json::Value;

use crate::error::{ConfigDefraError, Result};
use crate::types::*;

// -- Escape helpers -----------------------------------------------------------

/// Escape a string for inline use in a GraphQL query.
fn gql_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Serialize a `Vec<String>` to the DefraDB JSON array string format.
fn vec_to_gql_array(items: &[String]) -> String {
    let escaped: Vec<String> = items.iter().map(|s| format!(r#""{}""#, gql_escape(s))).collect();
    format!("[{}]", escaped.join(", "))
}

// -- RouterConfig -------------------------------------------------------------

/// Build a GraphQL mutation input for creating a router config.
pub fn router_to_create_mutation(config: &RouterConfig) -> String {
    format!(
        r#"mutation {{
            add_ClaspRouterConfig(input: {{
                configId: "{config_id}",
                name: "{name}",
                host: "{host}",
                port: {port},
                transports: {transports},
                securityMode: "{security_mode}",
                maxSessions: {max_sessions},
                paramTtlSecs: {param_ttl_secs},
                features: {features},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        config_id = gql_escape(&config.config_id),
        name = gql_escape(&config.name),
        host = gql_escape(&config.host),
        port = config.port,
        transports = vec_to_gql_array(&config.transports),
        security_mode = gql_escape(&config.security_mode),
        max_sessions = config.max_sessions,
        param_ttl_secs = config.param_ttl_secs,
        features = vec_to_gql_array(&config.features),
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Build a GraphQL mutation to update an existing router config by docID.
pub fn router_to_update_mutation(doc_id: &str, config: &RouterConfig) -> String {
    format!(
        r#"mutation {{
            update_ClaspRouterConfig(docID: "{doc_id}", input: {{
                name: "{name}",
                host: "{host}",
                port: {port},
                transports: {transports},
                securityMode: "{security_mode}",
                maxSessions: {max_sessions},
                paramTtlSecs: {param_ttl_secs},
                features: {features},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        doc_id = gql_escape(doc_id),
        name = gql_escape(&config.name),
        host = gql_escape(&config.host),
        port = config.port,
        transports = vec_to_gql_array(&config.transports),
        security_mode = gql_escape(&config.security_mode),
        max_sessions = config.max_sessions,
        param_ttl_secs = config.param_ttl_secs,
        features = vec_to_gql_array(&config.features),
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Parse a router config from a DefraDB document.
pub fn router_from_doc(doc: &Value) -> Result<RouterConfig> {
    Ok(RouterConfig {
        config_id: str_field(doc, "configId")?,
        name: str_field(doc, "name")?,
        host: str_field(doc, "host")?,
        port: int_field(doc, "port")? as u16,
        transports: str_array_field(doc, "transports"),
        security_mode: str_field(doc, "securityMode").unwrap_or_default(),
        max_sessions: int_field(doc, "maxSessions").unwrap_or(256) as u32,
        param_ttl_secs: int_field(doc, "paramTtlSecs").unwrap_or(300) as u32,
        features: str_array_field(doc, "features"),
        owner: str_field(doc, "owner").unwrap_or_default(),
        updated_at: int_field(doc, "updatedAt").unwrap_or(0) as u64,
        version: int_field(doc, "version").unwrap_or(1) as u32,
    })
}

// -- ConnectionConfig ---------------------------------------------------------

/// Build a GraphQL mutation input for creating a connection config.
pub fn connection_to_create_mutation(config: &ConnectionConfig) -> String {
    format!(
        r#"mutation {{
            add_ClaspConnectionConfig(input: {{
                configId: "{config_id}",
                name: "{name}",
                routerUrl: "{router_url}",
                transport: "{transport}",
                token: "{token}",
                reconnect: {reconnect},
                features: {features},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        config_id = gql_escape(&config.config_id),
        name = gql_escape(&config.name),
        router_url = gql_escape(&config.router_url),
        transport = gql_escape(&config.transport),
        token = gql_escape(&config.token),
        reconnect = config.reconnect,
        features = vec_to_gql_array(&config.features),
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Build a GraphQL mutation to update an existing connection config.
pub fn connection_to_update_mutation(doc_id: &str, config: &ConnectionConfig) -> String {
    format!(
        r#"mutation {{
            update_ClaspConnectionConfig(docID: "{doc_id}", input: {{
                name: "{name}",
                routerUrl: "{router_url}",
                transport: "{transport}",
                token: "{token}",
                reconnect: {reconnect},
                features: {features},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        doc_id = gql_escape(doc_id),
        name = gql_escape(&config.name),
        router_url = gql_escape(&config.router_url),
        transport = gql_escape(&config.transport),
        token = gql_escape(&config.token),
        reconnect = config.reconnect,
        features = vec_to_gql_array(&config.features),
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Parse a connection config from a DefraDB document.
pub fn connection_from_doc(doc: &Value) -> Result<ConnectionConfig> {
    Ok(ConnectionConfig {
        config_id: str_field(doc, "configId")?,
        name: str_field(doc, "name")?,
        router_url: str_field(doc, "routerUrl").unwrap_or_default(),
        transport: str_field(doc, "transport").unwrap_or_else(|_| "websocket".into()),
        token: str_field(doc, "token").unwrap_or_default(),
        reconnect: bool_field(doc, "reconnect").unwrap_or(true),
        features: str_array_field(doc, "features"),
        owner: str_field(doc, "owner").unwrap_or_default(),
        updated_at: int_field(doc, "updatedAt").unwrap_or(0) as u64,
        version: int_field(doc, "version").unwrap_or(1) as u32,
    })
}

// -- BridgeConfig -------------------------------------------------------------

/// Build a GraphQL mutation input for creating a bridge config.
pub fn bridge_to_create_mutation(config: &BridgeConfig) -> String {
    format!(
        r#"mutation {{
            add_ClaspBridgeConfig(input: {{
                configId: "{config_id}",
                name: "{name}",
                protocol: "{protocol}",
                sourceAddr: "{source_addr}",
                targetAddr: "{target_addr}",
                mappings: "{mappings}",
                active: {active},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        config_id = gql_escape(&config.config_id),
        name = gql_escape(&config.name),
        protocol = gql_escape(&config.protocol),
        source_addr = gql_escape(&config.source_addr),
        target_addr = gql_escape(&config.target_addr),
        mappings = gql_escape(&config.mappings),
        active = config.active,
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Build a GraphQL mutation to update an existing bridge config.
pub fn bridge_to_update_mutation(doc_id: &str, config: &BridgeConfig) -> String {
    format!(
        r#"mutation {{
            update_ClaspBridgeConfig(docID: "{doc_id}", input: {{
                name: "{name}",
                protocol: "{protocol}",
                sourceAddr: "{source_addr}",
                targetAddr: "{target_addr}",
                mappings: "{mappings}",
                active: {active},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        doc_id = gql_escape(doc_id),
        name = gql_escape(&config.name),
        protocol = gql_escape(&config.protocol),
        source_addr = gql_escape(&config.source_addr),
        target_addr = gql_escape(&config.target_addr),
        mappings = gql_escape(&config.mappings),
        active = config.active,
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Parse a bridge config from a DefraDB document.
pub fn bridge_from_doc(doc: &Value) -> Result<BridgeConfig> {
    Ok(BridgeConfig {
        config_id: str_field(doc, "configId")?,
        name: str_field(doc, "name")?,
        protocol: str_field(doc, "protocol").unwrap_or_else(|_| "osc".into()),
        source_addr: str_field(doc, "sourceAddr").unwrap_or_default(),
        target_addr: str_field(doc, "targetAddr").unwrap_or_default(),
        mappings: str_field(doc, "mappings").unwrap_or_else(|_| "[]".into()),
        active: bool_field(doc, "active").unwrap_or(true),
        owner: str_field(doc, "owner").unwrap_or_default(),
        updated_at: int_field(doc, "updatedAt").unwrap_or(0) as u64,
        version: int_field(doc, "version").unwrap_or(1) as u32,
    })
}

// -- RuleConfig ---------------------------------------------------------------

/// Build a GraphQL mutation input for creating a rule config.
pub fn rule_to_create_mutation(config: &RuleConfig) -> String {
    format!(
        r#"mutation {{
            add_ClaspRuleConfig(input: {{
                configId: "{config_id}",
                name: "{name}",
                trigger: "{trigger}",
                conditions: "{conditions}",
                actions: "{actions}",
                cooldownSecs: {cooldown_secs},
                enabled: {enabled},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        config_id = gql_escape(&config.config_id),
        name = gql_escape(&config.name),
        trigger = gql_escape(&config.trigger),
        conditions = gql_escape(&config.conditions),
        actions = gql_escape(&config.actions),
        cooldown_secs = config.cooldown_secs,
        enabled = config.enabled,
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Build a GraphQL mutation to update an existing rule config.
pub fn rule_to_update_mutation(doc_id: &str, config: &RuleConfig) -> String {
    format!(
        r#"mutation {{
            update_ClaspRuleConfig(docID: "{doc_id}", input: {{
                name: "{name}",
                trigger: "{trigger}",
                conditions: "{conditions}",
                actions: "{actions}",
                cooldownSecs: {cooldown_secs},
                enabled: {enabled},
                owner: "{owner}",
                updatedAt: {updated_at},
                version: {version}
            }}) {{
                _docID
            }}
        }}"#,
        doc_id = gql_escape(doc_id),
        name = gql_escape(&config.name),
        trigger = gql_escape(&config.trigger),
        conditions = gql_escape(&config.conditions),
        actions = gql_escape(&config.actions),
        cooldown_secs = config.cooldown_secs,
        enabled = config.enabled,
        owner = gql_escape(&config.owner),
        updated_at = config.updated_at,
        version = config.version,
    )
}

/// Parse a rule config from a DefraDB document.
pub fn rule_from_doc(doc: &Value) -> Result<RuleConfig> {
    Ok(RuleConfig {
        config_id: str_field(doc, "configId")?,
        name: str_field(doc, "name")?,
        trigger: str_field(doc, "trigger").unwrap_or_else(|_| "{}".into()),
        conditions: str_field(doc, "conditions").unwrap_or_else(|_| "[]".into()),
        actions: str_field(doc, "actions").unwrap_or_else(|_| "[]".into()),
        cooldown_secs: int_field(doc, "cooldownSecs").unwrap_or(0) as u32,
        enabled: bool_field(doc, "enabled").unwrap_or(true),
        owner: str_field(doc, "owner").unwrap_or_default(),
        updated_at: int_field(doc, "updatedAt").unwrap_or(0) as u64,
        version: int_field(doc, "version").unwrap_or(1) as u32,
    })
}

// -- ConfigSnapshot -----------------------------------------------------------

/// Build a GraphQL mutation input for creating a snapshot.
pub fn snapshot_to_create_mutation(snapshot: &ConfigSnapshot) -> Result<String> {
    let routers_json = serde_json::to_string(&snapshot.routers)?;
    let connections_json = serde_json::to_string(&snapshot.connections)?;
    let bridges_json = serde_json::to_string(&snapshot.bridges)?;
    let rules_json = serde_json::to_string(&snapshot.rules)?;

    Ok(format!(
        r#"mutation {{
            add_ClaspConfigSnapshot(input: {{
                snapshotId: "{snapshot_id}",
                name: "{name}",
                description: "{description}",
                routers: "{routers}",
                connections: "{connections}",
                bridges: "{bridges}",
                rules: "{rules}",
                owner: "{owner}",
                createdAt: {created_at}
            }}) {{
                _docID
            }}
        }}"#,
        snapshot_id = gql_escape(&snapshot.snapshot_id),
        name = gql_escape(&snapshot.name),
        description = gql_escape(&snapshot.description),
        routers = gql_escape(&routers_json),
        connections = gql_escape(&connections_json),
        bridges = gql_escape(&bridges_json),
        rules = gql_escape(&rules_json),
        owner = gql_escape(&snapshot.owner),
        created_at = snapshot.created_at,
    ))
}

/// Parse a config snapshot from a DefraDB document.
pub fn snapshot_from_doc(doc: &Value) -> Result<ConfigSnapshot> {
    let routers_str = str_field(doc, "routers").unwrap_or_else(|_| "[]".into());
    let connections_str = str_field(doc, "connections").unwrap_or_else(|_| "[]".into());
    let bridges_str = str_field(doc, "bridges").unwrap_or_else(|_| "[]".into());
    let rules_str = str_field(doc, "rules").unwrap_or_else(|_| "[]".into());

    let routers: Vec<RouterConfig> = serde_json::from_str(&routers_str)
        .map_err(|e| ConfigDefraError::Deserialization(format!("routers: {e}")))?;
    let connections: Vec<ConnectionConfig> = serde_json::from_str(&connections_str)
        .map_err(|e| ConfigDefraError::Deserialization(format!("connections: {e}")))?;
    let bridges: Vec<BridgeConfig> = serde_json::from_str(&bridges_str)
        .map_err(|e| ConfigDefraError::Deserialization(format!("bridges: {e}")))?;
    let rules: Vec<RuleConfig> = serde_json::from_str(&rules_str)
        .map_err(|e| ConfigDefraError::Deserialization(format!("rules: {e}")))?;

    Ok(ConfigSnapshot {
        snapshot_id: str_field(doc, "snapshotId")?,
        name: str_field(doc, "name")?,
        description: str_field(doc, "description").unwrap_or_default(),
        routers,
        connections,
        bridges,
        rules,
        owner: str_field(doc, "owner").unwrap_or_default(),
        created_at: int_field(doc, "createdAt").unwrap_or(0) as u64,
    })
}

// -- Field extraction helpers -------------------------------------------------

fn str_field(doc: &Value, field: &str) -> Result<String> {
    doc.get(field)
        .and_then(|v| v.as_str())
        .map(String::from)
        .ok_or_else(|| {
            ConfigDefraError::Deserialization(format!("missing or invalid field: {field}"))
        })
}

fn int_field(doc: &Value, field: &str) -> Result<i64> {
    doc.get(field)
        .and_then(|v| v.as_i64())
        .ok_or_else(|| {
            ConfigDefraError::Deserialization(format!("missing or invalid field: {field}"))
        })
}

fn bool_field(doc: &Value, field: &str) -> Result<bool> {
    doc.get(field)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| {
            ConfigDefraError::Deserialization(format!("missing or invalid field: {field}"))
        })
}

fn str_array_field(doc: &Value, field: &str) -> Vec<String> {
    doc.get(field)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn router_mutation_format() {
        let config = RouterConfig::new("r-001", "Test Router", "owner-a");
        let mutation = router_to_create_mutation(&config);
        assert!(mutation.contains("add_ClaspRouterConfig"));
        assert!(mutation.contains(r#"configId: "r-001""#));
        assert!(mutation.contains(r#"name: "Test Router""#));
    }

    #[test]
    fn router_from_doc_parses() {
        let doc = json!({
            "configId": "r-001",
            "name": "Main Router",
            "host": "0.0.0.0",
            "port": 9100,
            "transports": ["websocket"],
            "securityMode": "none",
            "maxSessions": 256,
            "paramTtlSecs": 300,
            "features": [],
            "owner": "test",
            "updatedAt": 1700000000,
            "version": 1
        });

        let config = router_from_doc(&doc).unwrap();
        assert_eq!(config.config_id, "r-001");
        assert_eq!(config.port, 9100);
        assert_eq!(config.transports, vec!["websocket"]);
    }

    #[test]
    fn connection_from_doc_parses() {
        let doc = json!({
            "configId": "c-001",
            "name": "Link",
            "routerUrl": "ws://localhost:9100",
            "transport": "websocket",
            "token": "",
            "reconnect": true,
            "features": [],
            "owner": "test",
            "updatedAt": 0,
            "version": 1
        });

        let config = connection_from_doc(&doc).unwrap();
        assert_eq!(config.config_id, "c-001");
        assert!(config.reconnect);
    }

    #[test]
    fn bridge_from_doc_parses() {
        let doc = json!({
            "configId": "b-001",
            "name": "OSC",
            "protocol": "osc",
            "sourceAddr": "/in/**",
            "targetAddr": "/out/**",
            "mappings": "[]",
            "active": true,
            "owner": "test",
            "updatedAt": 0,
            "version": 1
        });

        let config = bridge_from_doc(&doc).unwrap();
        assert_eq!(config.config_id, "b-001");
        assert_eq!(config.protocol, "osc");
    }

    #[test]
    fn rule_from_doc_parses() {
        let doc = json!({
            "configId": "rule-001",
            "name": "Auto-mute",
            "trigger": "{}",
            "conditions": "[]",
            "actions": "[]",
            "cooldownSecs": 5,
            "enabled": true,
            "owner": "test",
            "updatedAt": 0,
            "version": 1
        });

        let config = rule_from_doc(&doc).unwrap();
        assert_eq!(config.config_id, "rule-001");
        assert_eq!(config.cooldown_secs, 5);
    }

    #[test]
    fn snapshot_from_doc_parses() {
        let routers = serde_json::to_string(&vec![RouterConfig::new("r-1", "R", "o")]).unwrap();
        let doc = json!({
            "snapshotId": "snap-001",
            "name": "Test",
            "description": "desc",
            "routers": routers,
            "connections": "[]",
            "bridges": "[]",
            "rules": "[]",
            "owner": "test",
            "createdAt": 1700000000
        });

        let snap = snapshot_from_doc(&doc).unwrap();
        assert_eq!(snap.snapshot_id, "snap-001");
        assert_eq!(snap.routers.len(), 1);
    }

    #[test]
    fn gql_escape_special_chars() {
        assert_eq!(gql_escape(r#"say "hello""#), r#"say \"hello\""#);
        assert_eq!(gql_escape(r"back\slash"), r"back\\slash");
    }
}
