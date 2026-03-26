//! Thin HTTP client for the DefraDB GraphQL API.

use std::time::Duration;

use serde_json::json;
use tracing::{debug, warn};

use crate::error::{DefraError, Result};

/// HTTP client for DefraDB's GraphQL and management APIs.
///
/// Includes configurable retry with exponential backoff for transient
/// network failures. Retries are applied to `graphql()` calls only --
/// schema provisioning and health checks are not retried.
pub struct DefraClient {
    base_url: String,
    http: reqwest::Client,
    /// Maximum number of retry attempts for transient failures (default: 3).
    max_retries: u32,
    /// Base delay between retries, doubled on each attempt (default: 100ms).
    retry_base_delay: Duration,
}

impl DefraClient {
    /// Create a new client pointing at the given DefraDB instance.
    ///
    /// Does not perform a health check -- use [`DefraClient::health`] to
    /// verify connectivity before issuing queries.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .connect_timeout(Duration::from_secs(5))
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            max_retries: 3,
            retry_base_delay: Duration::from_millis(100),
        }
    }

    /// Set the maximum number of retry attempts for transient failures.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Set the base delay between retries (doubled on each attempt).
    pub fn with_retry_delay(mut self, delay: Duration) -> Self {
        self.retry_base_delay = delay;
        self
    }

    /// Execute a GraphQL query or mutation against DefraDB.
    ///
    /// Returns the top-level `"data"` value on success, or a
    /// [`DefraError::GraphQL`] if the response contains errors.
    pub async fn graphql(
        &self,
        query: &str,
        variables: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v0/graphql", self.base_url);

        let mut body = json!({ "query": query });
        if let Some(ref vars) = variables {
            body["variables"] = vars.clone();
        }

        debug!(url = %url, "DefraDB GraphQL request");

        let mut last_err = None;
        for attempt in 0..=self.max_retries {
            if attempt > 0 {
                let delay = self.retry_base_delay * 2u32.saturating_pow(attempt - 1);
                warn!(attempt, delay_ms = delay.as_millis() as u64, "Retrying DefraDB request");
                tokio::time::sleep(delay).await;
            }

            let resp = match self.http.post(&url).json(&body).send().await {
                Ok(r) => r,
                Err(e) => {
                    // Network error -- retryable
                    last_err = Some(DefraError::Http(e));
                    continue;
                }
            };

            // 5xx = server error, retryable; 4xx = client error, not retryable
            if resp.status().is_server_error() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                last_err = Some(DefraError::GraphQL(format!("server error {status}: {text}")));
                continue;
            }

            let resp = resp.error_for_status().map_err(DefraError::Http)?;
            let payload: serde_json::Value = resp.json().await?;

            // Check for GraphQL-level errors (not retryable)
            if let Some(errors) = payload.get("errors") {
                if let Some(arr) = errors.as_array() {
                    if !arr.is_empty() {
                        let messages: Vec<String> = arr
                            .iter()
                            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                            .map(String::from)
                            .collect();
                        return Err(DefraError::GraphQL(messages.join("; ")));
                    }
                }
            }

            return Ok(payload
                .get("data")
                .cloned()
                .unwrap_or(serde_json::Value::Null));
        }

        Err(last_err.unwrap_or_else(|| DefraError::GraphQL("max retries exceeded".into())))
    }

    /// Add a GraphQL SDL schema to DefraDB.
    ///
    /// This is idempotent -- if the schema already exists, the
    /// "already exists" error is silently ignored.
    pub async fn add_schema(&self, sdl: &str) -> Result<()> {
        let url = format!("{}/api/v0/collections", self.base_url);

        debug!(url = %url, "DefraDB schema provision");

        let resp = self
            .http
            .post(&url)
            .header("Content-Type", "text/plain")
            .body(sdl.to_string())
            .send()
            .await?;

        // Accept 200 and 400 (schema already exists)
        if resp.status().is_success() {
            return Ok(());
        }

        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();

        // DefraDB returns an error when the schema already exists.
        // Treat this as success since we want idempotent provisioning.
        if text.contains("already exists") {
            debug!("Schema already provisioned, skipping");
            return Ok(());
        }

        warn!(status = %status, body = %text, "Schema provisioning failed");
        Err(DefraError::Schema(format!(
            "status {status}: {text}"
        )))
    }

    /// Check whether the DefraDB instance is reachable.
    pub async fn health(&self) -> Result<bool> {
        let url = format!("{}/api/v0/graphql", self.base_url);
        let body = json!({ "query": "{ __typename }" });

        match self.http.post(&url).json(&body).send().await {
            Ok(resp) => Ok(resp.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    /// Return the base URL this client is configured with.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

/// Convert a `serde_json::Value` to GraphQL input literal syntax.
///
/// JSON uses quoted keys (`{"name": "alice"}`), GraphQL uses unquoted
/// (`{name: "alice"}`). This function performs the conversion recursively.
pub fn json_to_graphql_input(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) => {
            let entries: Vec<String> = map
                .iter()
                .map(|(k, v)| format!("{}: {}", k, json_to_graphql_input(v)))
                .collect();
            format!("{{{}}}", entries.join(", "))
        }
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_to_graphql_input).collect();
            format!("[{}]", items.join(", "))
        }
        serde_json::Value::String(s) => {
            let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
            format!("\"{}\"", escaped)
        }
        other => other.to_string(),
    }
}
