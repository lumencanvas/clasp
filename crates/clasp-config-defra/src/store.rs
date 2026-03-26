//! Main DefraDB configuration store.

use clasp_journal_defra::DefraClient;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::convert;
use crate::error::{ConfigDefraError, Result};
use crate::schema;
use crate::types::*;

/// DefraDB-backed configuration store for CLASP.
///
/// Provides CRUD operations for router, connection, bridge, and rule
/// configs, plus full-config snapshots for versioning. All data is
/// stored in DefraDB and automatically synced across peers via
/// Merkle CRDTs.
pub struct DefraConfigStore {
    client: DefraClient,
}

impl DefraConfigStore {
    /// Create a new config store, provisioning all schemas.
    ///
    /// Schemas are provisioned idempotently -- calling this multiple
    /// times against the same DefraDB instance is safe.
    pub async fn new(defra_url: &str) -> Result<Self> {
        let client = DefraClient::new(defra_url);

        // Provision all config schemas
        for sdl in schema::ALL_SCHEMAS {
            client.add_schema(sdl).await.map_err(|e| {
                ConfigDefraError::Schema(e.to_string())
            })?;
        }

        debug!("Config schemas provisioned");
        Ok(Self { client })
    }

    // -- Router configs -------------------------------------------------------

    /// Save a router config (upsert: creates or updates by configId).
    pub async fn save_router(&self, config: &RouterConfig) -> Result<()> {
        if let Some(doc_id) = self.find_doc_id("ClaspRouterConfig", &config.config_id).await? {
            let mutation = convert::router_to_update_mutation(&doc_id, config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Updated router config");
        } else {
            let mutation = convert::router_to_create_mutation(config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Created router config");
        }
        Ok(())
    }

    /// Get a router config by configId.
    pub async fn get_router(&self, config_id: &str) -> Result<Option<RouterConfig>> {
        let query = format!(
            r#"query {{
                ClaspRouterConfig(filter: {{configId: {{_eq: "{id}"}}}}) {{
                    {fields}
                }}
            }}"#,
            id = gql_escape(config_id),
            fields = ROUTER_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspRouterConfig");
        match docs.first() {
            Some(doc) => Ok(Some(convert::router_from_doc(doc)?)),
            None => Ok(None),
        }
    }

    /// List all router configs.
    pub async fn list_routers(&self) -> Result<Vec<RouterConfig>> {
        let query = format!(
            r#"query {{
                ClaspRouterConfig {{
                    {fields}
                }}
            }}"#,
            fields = ROUTER_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspRouterConfig");
        docs.iter().map(|d| convert::router_from_doc(d)).collect()
    }

    /// List router configs owned by the given owner.
    pub async fn list_routers_by_owner(&self, owner: &str) -> Result<Vec<RouterConfig>> {
        let query = format!(
            r#"query {{
                ClaspRouterConfig(filter: {{owner: {{_eq: "{owner}"}}}}) {{
                    {fields}
                }}
            }}"#,
            owner = gql_escape(owner),
            fields = ROUTER_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspRouterConfig");
        docs.iter().map(|d| convert::router_from_doc(d)).collect()
    }

    /// Delete a router config by configId. Returns true if found and deleted.
    pub async fn delete_router(&self, config_id: &str) -> Result<bool> {
        self.delete_by_config_id("ClaspRouterConfig", config_id).await
    }

    // -- Connection configs ---------------------------------------------------

    /// Save a connection config (upsert).
    pub async fn save_connection(&self, config: &ConnectionConfig) -> Result<()> {
        if let Some(doc_id) = self.find_doc_id("ClaspConnectionConfig", &config.config_id).await? {
            let mutation = convert::connection_to_update_mutation(&doc_id, config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Updated connection config");
        } else {
            let mutation = convert::connection_to_create_mutation(config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Created connection config");
        }
        Ok(())
    }

    /// Get a connection config by configId.
    pub async fn get_connection(&self, config_id: &str) -> Result<Option<ConnectionConfig>> {
        let query = format!(
            r#"query {{
                ClaspConnectionConfig(filter: {{configId: {{_eq: "{id}"}}}}) {{
                    {fields}
                }}
            }}"#,
            id = gql_escape(config_id),
            fields = CONNECTION_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspConnectionConfig");
        match docs.first() {
            Some(doc) => Ok(Some(convert::connection_from_doc(doc)?)),
            None => Ok(None),
        }
    }

    /// List all connection configs.
    pub async fn list_connections(&self) -> Result<Vec<ConnectionConfig>> {
        let query = format!(
            r#"query {{
                ClaspConnectionConfig {{
                    {fields}
                }}
            }}"#,
            fields = CONNECTION_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspConnectionConfig");
        docs.iter().map(|d| convert::connection_from_doc(d)).collect()
    }

    /// Delete a connection config by configId.
    pub async fn delete_connection(&self, config_id: &str) -> Result<bool> {
        self.delete_by_config_id("ClaspConnectionConfig", config_id).await
    }

    // -- Bridge configs -------------------------------------------------------

    /// Save a bridge config (upsert).
    pub async fn save_bridge(&self, config: &BridgeConfig) -> Result<()> {
        if let Some(doc_id) = self.find_doc_id("ClaspBridgeConfig", &config.config_id).await? {
            let mutation = convert::bridge_to_update_mutation(&doc_id, config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Updated bridge config");
        } else {
            let mutation = convert::bridge_to_create_mutation(config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Created bridge config");
        }
        Ok(())
    }

    /// Get a bridge config by configId.
    pub async fn get_bridge(&self, config_id: &str) -> Result<Option<BridgeConfig>> {
        let query = format!(
            r#"query {{
                ClaspBridgeConfig(filter: {{configId: {{_eq: "{id}"}}}}) {{
                    {fields}
                }}
            }}"#,
            id = gql_escape(config_id),
            fields = BRIDGE_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspBridgeConfig");
        match docs.first() {
            Some(doc) => Ok(Some(convert::bridge_from_doc(doc)?)),
            None => Ok(None),
        }
    }

    /// List all bridge configs.
    pub async fn list_bridges(&self) -> Result<Vec<BridgeConfig>> {
        let query = format!(
            r#"query {{
                ClaspBridgeConfig {{
                    {fields}
                }}
            }}"#,
            fields = BRIDGE_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspBridgeConfig");
        docs.iter().map(|d| convert::bridge_from_doc(d)).collect()
    }

    /// Delete a bridge config by configId.
    pub async fn delete_bridge(&self, config_id: &str) -> Result<bool> {
        self.delete_by_config_id("ClaspBridgeConfig", config_id).await
    }

    // -- Rule configs ---------------------------------------------------------

    /// Save a rule config (upsert).
    pub async fn save_rule(&self, config: &RuleConfig) -> Result<()> {
        if let Some(doc_id) = self.find_doc_id("ClaspRuleConfig", &config.config_id).await? {
            let mutation = convert::rule_to_update_mutation(&doc_id, config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Updated rule config");
        } else {
            let mutation = convert::rule_to_create_mutation(config);
            self.execute_mutation(&mutation).await?;
            debug!(config_id = %config.config_id, "Created rule config");
        }
        Ok(())
    }

    /// Get a rule config by configId.
    pub async fn get_rule(&self, config_id: &str) -> Result<Option<RuleConfig>> {
        let query = format!(
            r#"query {{
                ClaspRuleConfig(filter: {{configId: {{_eq: "{id}"}}}}) {{
                    {fields}
                }}
            }}"#,
            id = gql_escape(config_id),
            fields = RULE_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspRuleConfig");
        match docs.first() {
            Some(doc) => Ok(Some(convert::rule_from_doc(doc)?)),
            None => Ok(None),
        }
    }

    /// List all rule configs.
    pub async fn list_rules(&self) -> Result<Vec<RuleConfig>> {
        let query = format!(
            r#"query {{
                ClaspRuleConfig {{
                    {fields}
                }}
            }}"#,
            fields = RULE_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspRuleConfig");
        docs.iter().map(|d| convert::rule_from_doc(d)).collect()
    }

    /// Delete a rule config by configId.
    pub async fn delete_rule(&self, config_id: &str) -> Result<bool> {
        self.delete_by_config_id("ClaspRuleConfig", config_id).await
    }

    // -- Snapshots ------------------------------------------------------------

    /// Save a full configuration snapshot.
    pub async fn save_snapshot(&self, snapshot: &ConfigSnapshot) -> Result<()> {
        let mutation = convert::snapshot_to_create_mutation(snapshot)?;
        self.execute_mutation(&mutation).await?;
        debug!(snapshot_id = %snapshot.snapshot_id, "Saved config snapshot");
        Ok(())
    }

    /// Get a snapshot by snapshotId.
    pub async fn get_snapshot(&self, snapshot_id: &str) -> Result<Option<ConfigSnapshot>> {
        let query = format!(
            r#"query {{
                ClaspConfigSnapshot(filter: {{snapshotId: {{_eq: "{id}"}}}}) {{
                    {fields}
                }}
            }}"#,
            id = gql_escape(snapshot_id),
            fields = SNAPSHOT_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspConfigSnapshot");
        match docs.first() {
            Some(doc) => Ok(Some(convert::snapshot_from_doc(doc)?)),
            None => Ok(None),
        }
    }

    /// List all snapshots, ordered by createdAt descending.
    pub async fn list_snapshots(&self) -> Result<Vec<ConfigSnapshot>> {
        let query = format!(
            r#"query {{
                ClaspConfigSnapshot(order: {{createdAt: DESC}}) {{
                    {fields}
                }}
            }}"#,
            fields = SNAPSHOT_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspConfigSnapshot");
        docs.iter().map(|d| convert::snapshot_from_doc(d)).collect()
    }

    /// Get the most recent snapshot.
    pub async fn latest_snapshot(&self) -> Result<Option<ConfigSnapshot>> {
        let query = format!(
            r#"query {{
                ClaspConfigSnapshot(order: {{createdAt: DESC}}, limit: 1) {{
                    {fields}
                }}
            }}"#,
            fields = SNAPSHOT_FIELDS,
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, "ClaspConfigSnapshot");
        match docs.first() {
            Some(doc) => Ok(Some(convert::snapshot_from_doc(doc)?)),
            None => Ok(None),
        }
    }

    // -- Import/Export --------------------------------------------------------

    /// Import a JSON config string (compatible with the bridge app's file
    /// format) and store it as individual configs plus a snapshot.
    ///
    /// Returns the snapshot that was created from the import.
    pub async fn import_json(&self, json: &str, owner: &str) -> Result<ConfigSnapshot> {
        let snapshot: ConfigSnapshot = serde_json::from_str(json).map_err(|e| {
            ConfigDefraError::Deserialization(format!("invalid config JSON: {e}"))
        })?;

        // Save individual configs
        for router in &snapshot.routers {
            self.save_router(router).await?;
        }
        for conn in &snapshot.connections {
            self.save_connection(conn).await?;
        }
        for bridge in &snapshot.bridges {
            self.save_bridge(bridge).await?;
        }
        for rule in &snapshot.rules {
            self.save_rule(rule).await?;
        }

        // Create a snapshot record for the import
        let import_snapshot = ConfigSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            name: format!("Import: {}", snapshot.name),
            description: format!("Imported from JSON by {owner}"),
            routers: snapshot.routers,
            connections: snapshot.connections,
            bridges: snapshot.bridges,
            rules: snapshot.rules,
            owner: owner.into(),
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        self.save_snapshot(&import_snapshot).await?;
        debug!(owner = %owner, "Imported config from JSON");
        Ok(import_snapshot)
    }

    /// Export the current config state as a JSON string compatible with
    /// the bridge app's file format.
    pub async fn export_json(&self) -> Result<String> {
        let routers = self.list_routers().await?;
        let connections = self.list_connections().await?;
        let bridges = self.list_bridges().await?;
        let rules = self.list_rules().await?;

        let snapshot = ConfigSnapshot {
            snapshot_id: Uuid::new_v4().to_string(),
            name: "Export".into(),
            description: "Exported from DefraDB config store".into(),
            routers,
            connections,
            bridges,
            rules,
            owner: String::new(),
            created_at: chrono::Utc::now().timestamp() as u64,
        };

        let json = serde_json::to_string_pretty(&snapshot)?;
        Ok(json)
    }

    // -- History --------------------------------------------------------------

    /// Query DefraDB's commit history for a specific config document.
    ///
    /// Uses DefraDB's `_commits` introspection to retrieve the Merkle
    /// DAG history for the document identified by `config_id` in the
    /// given `collection`.
    pub async fn config_history(
        &self,
        config_id: &str,
        collection: &str,
    ) -> Result<Vec<serde_json::Value>> {
        Self::validate_collection(collection)?;

        // First find the docID for this configId
        let doc_id = self.find_doc_id(collection, config_id).await?;
        let doc_id = match doc_id {
            Some(id) => id,
            None => return Ok(Vec::new()),
        };

        let query = format!(
            r#"query {{
                commits(docID: "{doc_id}", order: {{height: DESC}}) {{
                    cid
                    height
                    delta
                    links {{
                        cid
                        name
                    }}
                }}
            }}"#,
            doc_id = gql_escape(&doc_id),
        );

        let data = self.execute_query(&query).await?;
        let commits = extract_array(&data, "commits");
        Ok(commits)
    }

    // -- Internal helpers -----------------------------------------------------

    /// Allowed collection names for GraphQL interpolation.
    const VALID_COLLECTIONS: &'static [&'static str] = &[
        "ClaspRouterConfig",
        "ClaspConnectionConfig",
        "ClaspBridgeConfig",
        "ClaspRuleConfig",
        "ClaspConfigSnapshot",
    ];

    /// Validate that `collection` is in the whitelist before interpolating
    /// into a GraphQL query.
    fn validate_collection(collection: &str) -> Result<()> {
        if !Self::VALID_COLLECTIONS.contains(&collection) {
            return Err(ConfigDefraError::InvalidConfig(
                format!("unknown collection: {}", collection),
            ));
        }
        Ok(())
    }

    /// Find the DefraDB _docID for a document with the given configId.
    async fn find_doc_id(&self, collection: &str, config_id: &str) -> Result<Option<String>> {
        Self::validate_collection(collection)?;
        let query = format!(
            r#"query {{
                {collection}(filter: {{configId: {{_eq: "{id}"}}}}) {{
                    _docID
                }}
            }}"#,
            collection = collection,
            id = gql_escape(config_id),
        );

        let data = self.execute_query(&query).await?;
        let docs = extract_array(&data, collection);
        Ok(docs
            .first()
            .and_then(|d| d.get("_docID"))
            .and_then(|v| v.as_str())
            .map(String::from))
    }

    /// Delete a document by configId from a collection.
    async fn delete_by_config_id(&self, collection: &str, config_id: &str) -> Result<bool> {
        let doc_id = match self.find_doc_id(collection, config_id).await? {
            Some(id) => id,
            None => return Ok(false),
        };

        let mutation = format!(
            r#"mutation {{
                delete_{collection}(docID: "{doc_id}") {{
                    _docID
                }}
            }}"#,
            collection = collection,
            doc_id = gql_escape(&doc_id),
        );

        match self.execute_mutation(&mutation).await {
            Ok(_) => {
                debug!(config_id = %config_id, collection = %collection, "Deleted config");
                Ok(true)
            }
            Err(e) => {
                warn!(config_id = %config_id, error = %e, "Failed to delete config");
                Err(e)
            }
        }
    }

    /// Execute a GraphQL query against DefraDB.
    async fn execute_query(&self, query: &str) -> Result<serde_json::Value> {
        self.client
            .graphql(query, None)
            .await
            .map_err(|e| ConfigDefraError::GraphQL(e.to_string()))
    }

    /// Execute a GraphQL mutation against DefraDB.
    async fn execute_mutation(&self, mutation: &str) -> Result<serde_json::Value> {
        self.client
            .graphql(mutation, None)
            .await
            .map_err(|e| ConfigDefraError::GraphQL(e.to_string()))
    }
}

// -- Field selection constants ------------------------------------------------

const ROUTER_FIELDS: &str =
    "configId name host port transports securityMode maxSessions paramTtlSecs features owner updatedAt version";

const CONNECTION_FIELDS: &str =
    "configId name routerUrl transport token reconnect features owner updatedAt version";

const BRIDGE_FIELDS: &str =
    "configId name protocol sourceAddr targetAddr mappings active owner updatedAt version";

const RULE_FIELDS: &str =
    "configId name trigger conditions actions cooldownSecs enabled owner updatedAt version";

const SNAPSHOT_FIELDS: &str =
    "snapshotId name description routers connections bridges rules owner createdAt";

/// Escape a string for inline use in a GraphQL query.
fn gql_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Extract an array from a GraphQL data response.
fn extract_array(data: &serde_json::Value, key: &str) -> Vec<serde_json::Value> {
    data.get(key)
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn uid() -> String {
        uuid::Uuid::new_v4().to_string()[..8].to_string()
    }

    // -- Integration tests (require a running DefraDB instance) ---------------

    #[tokio::test]
    #[ignore]
    async fn test_save_and_get_router() {
        let store = DefraConfigStore::new("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let id = format!("test-r-{}", uid());
        let config = RouterConfig {
            config_id: id.clone(),
            name: "Integration Test Router".into(),
            host: "127.0.0.1".into(),
            port: 9200,
            transports: vec!["websocket".into(), "quic".into()],
            security_mode: "tls".into(),
            max_sessions: 64,
            param_ttl_secs: 120,
            features: vec!["federation".into()],
            owner: "test-owner".into(),
            updated_at: 1700000000,
            version: 1,
        };

        store.save_router(&config).await.unwrap();
        let loaded = store.get_router(&id).await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "Integration Test Router");
        assert_eq!(loaded.port, 9200);

        // Update
        let mut updated = config.clone();
        updated.port = 9300;
        updated.version = 2;
        store.save_router(&updated).await.unwrap();
        let loaded = store.get_router(&id).await.unwrap().unwrap();
        assert_eq!(loaded.port, 9300);
        assert_eq!(loaded.version, 2);

        // Cleanup
        store.delete_router(&id).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_by_owner() {
        let store = DefraConfigStore::new("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let id_a = format!("owner-a-{}", uid());
        let id_b = format!("owner-b-{}", uid());
        let owner_a = format!("alpha-{}", uid());
        let owner_b = format!("beta-{}", uid());
        let config_a = RouterConfig::new(&id_a, "Router A1", &owner_a);
        let config_b = RouterConfig::new(&id_b, "Router B1", &owner_b);

        store.save_router(&config_a).await.unwrap();
        store.save_router(&config_b).await.unwrap();

        let alpha_routers = store.list_routers_by_owner(&owner_a).await.unwrap();
        assert!(alpha_routers.iter().any(|r| r.config_id == id_a));
        assert!(!alpha_routers.iter().any(|r| r.config_id == id_b));

        // Cleanup
        store.delete_router(&id_a).await.unwrap();
        store.delete_router(&id_b).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_snapshot_save_and_load() {
        let store = DefraConfigStore::new("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let snap_id = format!("snap-{}", uid());
        let snapshot = ConfigSnapshot {
            snapshot_id: snap_id.clone(),
            name: "Test Snapshot".into(),
            description: "Integration test".into(),
            routers: vec![RouterConfig::new(&format!("sr-{}", uid()), "R1", "test")],
            connections: vec![ConnectionConfig::new(&format!("sc-{}", uid()), "C1", "test")],
            bridges: vec![],
            rules: vec![],
            owner: "test".into(),
            created_at: 1700000000,
        };

        store.save_snapshot(&snapshot).await.unwrap();
        let loaded = store.get_snapshot(&snap_id).await.unwrap();
        assert!(loaded.is_some());
        let loaded = loaded.unwrap();
        assert_eq!(loaded.name, "Test Snapshot");
        assert_eq!(loaded.routers.len(), 1);
        assert_eq!(loaded.connections.len(), 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_import_export() {
        let store = DefraConfigStore::new("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let rid = format!("imp-r-{}", uid());
        let original = ConfigSnapshot {
            snapshot_id: format!("imp-{}", uid()),
            name: "Import Test".into(),
            description: "Testing import/export".into(),
            routers: vec![RouterConfig::new(&rid, "Imported Router", "importer")],
            connections: vec![],
            bridges: vec![],
            rules: vec![],
            owner: "importer".into(),
            created_at: 1700000000,
        };

        let json = serde_json::to_string_pretty(&original).unwrap();
        let imported = store.import_json(&json, "importer").await.unwrap();
        assert_eq!(imported.routers.len(), 1);

        // Verify the router was saved individually
        let router = store.get_router(&rid).await.unwrap();
        assert!(router.is_some());
        assert_eq!(router.unwrap().name, "Imported Router");

        // Cleanup
        store.delete_router(&rid).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_delete() {
        let store = DefraConfigStore::new("http://localhost:9181")
            .await
            .expect("DefraDB must be running");

        let cid = format!("del-c-{}", uid());
        let config = ConnectionConfig::new(&cid, "Delete Me", "test");
        store.save_connection(&config).await.unwrap();

        let deleted = store.delete_connection(&cid).await.unwrap();
        assert!(deleted);

        let loaded = store.get_connection(&cid).await.unwrap();
        assert!(loaded.is_none());

        // Deleting again should return false
        let deleted_again = store.delete_connection(&cid).await.unwrap();
        assert!(!deleted_again);
    }
}
