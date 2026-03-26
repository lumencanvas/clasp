//! Rust types that map to the DefraDB config schemas.

use serde::{Deserialize, Serialize};

/// Router configuration stored in DefraDB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RouterConfig {
    pub config_id: String,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub transports: Vec<String>,
    pub security_mode: String,
    pub max_sessions: u32,
    pub param_ttl_secs: u32,
    pub features: Vec<String>,
    pub owner: String,
    pub updated_at: u64,
    pub version: u32,
}

impl RouterConfig {
    /// Create a new router config with sensible defaults.
    pub fn new(config_id: impl Into<String>, name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            config_id: config_id.into(),
            name: name.into(),
            host: "0.0.0.0".into(),
            port: 9100,
            transports: vec!["websocket".into()],
            security_mode: "none".into(),
            max_sessions: 256,
            param_ttl_secs: 300,
            features: Vec::new(),
            owner: owner.into(),
            updated_at: 0,
            version: 1,
        }
    }
}

/// Connection configuration stored in DefraDB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConnectionConfig {
    pub config_id: String,
    pub name: String,
    pub router_url: String,
    pub transport: String,
    pub token: String,
    pub reconnect: bool,
    pub features: Vec<String>,
    pub owner: String,
    pub updated_at: u64,
    pub version: u32,
}

impl ConnectionConfig {
    /// Create a new connection config with sensible defaults.
    pub fn new(config_id: impl Into<String>, name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            config_id: config_id.into(),
            name: name.into(),
            router_url: "ws://localhost:9100".into(),
            transport: "websocket".into(),
            token: String::new(),
            reconnect: true,
            features: Vec::new(),
            owner: owner.into(),
            updated_at: 0,
            version: 1,
        }
    }
}

/// Bridge configuration stored in DefraDB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BridgeConfig {
    pub config_id: String,
    pub name: String,
    pub protocol: String,
    pub source_addr: String,
    pub target_addr: String,
    /// JSON-serialized mapping rules.
    pub mappings: String,
    pub active: bool,
    pub owner: String,
    pub updated_at: u64,
    pub version: u32,
}

impl BridgeConfig {
    /// Create a new bridge config with sensible defaults.
    pub fn new(config_id: impl Into<String>, name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            config_id: config_id.into(),
            name: name.into(),
            protocol: "osc".into(),
            source_addr: String::new(),
            target_addr: String::new(),
            mappings: "[]".into(),
            active: true,
            owner: owner.into(),
            updated_at: 0,
            version: 1,
        }
    }
}

/// Rule configuration stored in DefraDB.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuleConfig {
    pub config_id: String,
    pub name: String,
    /// JSON-serialized trigger definition.
    pub trigger: String,
    /// JSON-serialized conditions array.
    pub conditions: String,
    /// JSON-serialized actions array.
    pub actions: String,
    pub cooldown_secs: u32,
    pub enabled: bool,
    pub owner: String,
    pub updated_at: u64,
    pub version: u32,
}

impl RuleConfig {
    /// Create a new rule config with sensible defaults.
    pub fn new(config_id: impl Into<String>, name: impl Into<String>, owner: impl Into<String>) -> Self {
        Self {
            config_id: config_id.into(),
            name: name.into(),
            trigger: "{}".into(),
            conditions: "[]".into(),
            actions: "[]".into(),
            cooldown_secs: 0,
            enabled: true,
            owner: owner.into(),
            updated_at: 0,
            version: 1,
        }
    }
}

/// Full configuration snapshot for versioning.
///
/// Captures the complete state of all config entities at a point
/// in time. Nested configs are stored as JSON strings in DefraDB
/// but deserialized into typed vectors in Rust.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConfigSnapshot {
    pub snapshot_id: String,
    pub name: String,
    pub description: String,
    pub routers: Vec<RouterConfig>,
    pub connections: Vec<ConnectionConfig>,
    pub bridges: Vec<BridgeConfig>,
    pub rules: Vec<RuleConfig>,
    pub owner: String,
    pub created_at: u64,
}

impl ConfigSnapshot {
    /// Create a new empty snapshot.
    pub fn new(
        snapshot_id: impl Into<String>,
        name: impl Into<String>,
        owner: impl Into<String>,
    ) -> Self {
        Self {
            snapshot_id: snapshot_id.into(),
            name: name.into(),
            description: String::new(),
            routers: Vec::new(),
            connections: Vec::new(),
            bridges: Vec::new(),
            rules: Vec::new(),
            owner: owner.into(),
            created_at: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn router_config_roundtrip() {
        let config = RouterConfig {
            config_id: "r-001".into(),
            name: "Main Router".into(),
            host: "0.0.0.0".into(),
            port: 9100,
            transports: vec!["websocket".into(), "quic".into()],
            security_mode: "tls".into(),
            max_sessions: 512,
            param_ttl_secs: 600,
            features: vec!["federation".into()],
            owner: "did:key:z6Mk...".into(),
            updated_at: 1700000000,
            version: 3,
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: RouterConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, restored);
    }

    #[test]
    fn connection_config_roundtrip() {
        let config = ConnectionConfig {
            config_id: "c-001".into(),
            name: "Studio Link".into(),
            router_url: "ws://192.168.1.10:9100".into(),
            transport: "websocket".into(),
            token: "secret-token".into(),
            reconnect: true,
            features: vec!["compression".into()],
            owner: "did:key:z6Mk...".into(),
            updated_at: 1700000000,
            version: 1,
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, restored);
    }

    #[test]
    fn bridge_config_roundtrip() {
        let config = BridgeConfig {
            config_id: "b-001".into(),
            name: "OSC Bridge".into(),
            protocol: "osc".into(),
            source_addr: "/synth/**".into(),
            target_addr: "/daw/track1/**".into(),
            mappings: r#"[{"from":"/freq","to":"/pitch"}]"#.into(),
            active: true,
            owner: "did:key:z6Mk...".into(),
            updated_at: 1700000000,
            version: 2,
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: BridgeConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, restored);
    }

    #[test]
    fn rule_config_roundtrip() {
        let config = RuleConfig {
            config_id: "rule-001".into(),
            name: "Auto-mute".into(),
            trigger: r#"{"type":"signal","address":"/mixer/master/clip"}"#.into(),
            conditions: r#"[{"op":"gt","value":0.95}]"#.into(),
            actions: r#"[{"type":"set","address":"/mixer/master/mute","value":true}]"#.into(),
            cooldown_secs: 5,
            enabled: true,
            owner: "did:key:z6Mk...".into(),
            updated_at: 1700000000,
            version: 1,
        };

        let json = serde_json::to_string(&config).unwrap();
        let restored: RuleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, restored);
    }

    #[test]
    fn snapshot_roundtrip() {
        let snapshot = ConfigSnapshot {
            snapshot_id: "snap-001".into(),
            name: "Production Setup".into(),
            description: "Full studio config for live show".into(),
            routers: vec![RouterConfig::new("r-001", "Main", "owner-a")],
            connections: vec![ConnectionConfig::new("c-001", "Link", "owner-a")],
            bridges: vec![BridgeConfig::new("b-001", "OSC", "owner-a")],
            rules: vec![RuleConfig::new("rule-001", "Auto-mute", "owner-a")],
            owner: "owner-a".into(),
            created_at: 1700000000,
        };

        let json = serde_json::to_string(&snapshot).unwrap();
        let restored: ConfigSnapshot = serde_json::from_str(&json).unwrap();
        assert_eq!(snapshot, restored);
    }

    #[test]
    fn import_export_json() {
        let snapshot = ConfigSnapshot {
            snapshot_id: "snap-export".into(),
            name: "Export Test".into(),
            description: "Testing JSON round-trip".into(),
            routers: vec![RouterConfig {
                config_id: "r-100".into(),
                name: "Router A".into(),
                host: "10.0.0.1".into(),
                port: 9200,
                transports: vec!["quic".into()],
                security_mode: "mtls".into(),
                max_sessions: 128,
                param_ttl_secs: 60,
                features: vec!["ttl".into(), "persist".into()],
                owner: "did:key:test".into(),
                updated_at: 1700000000,
                version: 5,
            }],
            connections: vec![ConnectionConfig {
                config_id: "c-100".into(),
                name: "Conn A".into(),
                router_url: "wss://example.com:9200".into(),
                transport: "quic".into(),
                token: "tok-abc".into(),
                reconnect: false,
                features: vec![],
                owner: "did:key:test".into(),
                updated_at: 1700000000,
                version: 2,
            }],
            bridges: vec![],
            rules: vec![],
            owner: "did:key:test".into(),
            created_at: 1700000000,
        };

        let exported = serde_json::to_string_pretty(&snapshot).unwrap();
        let imported: ConfigSnapshot = serde_json::from_str(&exported).unwrap();
        assert_eq!(snapshot, imported);
        assert_eq!(imported.routers.len(), 1);
        assert_eq!(imported.connections.len(), 1);
        assert_eq!(imported.routers[0].port, 9200);
        assert_eq!(imported.connections[0].transport, "quic");
    }
}
