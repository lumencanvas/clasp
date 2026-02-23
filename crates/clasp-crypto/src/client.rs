//! CryptoClient wrapper for transparent E2E encryption over a CLASP client.
//! Behind the `client` feature flag since it depends on clasp-client.

#[cfg(feature = "client")]
mod inner {
    use std::collections::HashMap;
    use std::sync::Arc;

    use crate::error::{CryptoError, Result};
    use crate::protocol::{E2ESession, E2ESessionConfig, OnKeyChange};
    use crate::storage::KeyStore;
    use crate::types::E2EEnvelope;

    /// Configuration for CryptoClient.
    pub struct CryptoClientConfig {
        pub identity_id: String,
        pub store: Arc<dyn KeyStore>,
        pub on_key_change: Option<OnKeyChange>,
    }

    /// Wraps a `clasp_client::Clasp` instance to provide transparent E2E encryption.
    ///
    /// The `inner` field provides direct access to the underlying Clasp client
    /// for operations that should not be encrypted.
    pub struct CryptoClient {
        /// Direct access to the underlying CLASP client for unencrypted operations.
        pub inner: clasp_client::Clasp,
        config: CryptoClientConfig,
        sessions: HashMap<String, E2ESession>,
    }

    impl CryptoClient {
        pub fn new(client: clasp_client::Clasp, config: CryptoClientConfig) -> Self {
            Self {
                inner: client,
                config,
                sessions: HashMap::new(),
            }
        }

        /// Get or create an E2ESession for a base path.
        pub fn session(&mut self, base_path: &str) -> &mut E2ESession {
            if !self.sessions.contains_key(base_path) {
                let session = E2ESession::new(E2ESessionConfig {
                    identity_id: self.config.identity_id.clone(),
                    base_path: base_path.to_string(),
                    store: self.config.store.clone(),
                    on_key_change: self.config.on_key_change.clone(),
                    password_hash: None,
                });
                self.sessions.insert(base_path.to_string(), session);
            }
            self.sessions.get_mut(base_path).unwrap()
        }

        /// Encrypt a string value and set it via the inner client.
        pub async fn set_encrypted(
            &self,
            address: &str,
            session: &E2ESession,
            value: &str,
        ) -> Result<()> {
            let envelope = session.encrypt(value)?;
            let json = serde_json::to_value(&envelope)
                .map_err(|e| CryptoError::Serialization(e.to_string()))?;
            self.inner
                .set(address, clasp_core::Value::from(json.to_string()))
                .await
                .map_err(|e| CryptoError::Other(e.to_string()))
        }

        /// Emit an encrypted event via the inner client.
        pub async fn emit_encrypted(
            &self,
            address: &str,
            session: &E2ESession,
            value: &str,
        ) -> Result<()> {
            let envelope = session.encrypt(value)?;
            let json = serde_json::to_value(&envelope)
                .map_err(|e| CryptoError::Serialization(e.to_string()))?;
            self.inner
                .emit(address, clasp_core::Value::from(json.to_string()))
                .await
                .map_err(|e| CryptoError::Other(e.to_string()))
        }

        /// Check if a value is an E2E envelope.
        pub fn is_envelope(value: &serde_json::Value) -> bool {
            value.get("_e2e").and_then(|v| v.as_u64()) == Some(1)
                && value.get("ct").and_then(|v| v.as_str()).is_some()
                && value.get("iv").and_then(|v| v.as_str()).is_some()
        }

        /// Try to parse and decrypt an envelope.
        pub async fn try_decrypt(
            session: &mut E2ESession,
            value: &serde_json::Value,
        ) -> Option<String> {
            let envelope: E2EEnvelope = serde_json::from_value(value.clone()).ok()?;
            session.decrypt(&envelope).await.ok()
        }

        /// Find the session whose base_path is a prefix of the given address.
        /// Uses longest-prefix match with path boundary check.
        pub fn find_session(&self, address: &str) -> Option<&E2ESession> {
            let mut best: Option<&E2ESession> = None;
            let mut best_len = 0;
            for (base_path, session) in &self.sessions {
                if (address == base_path.as_str()
                    || address.starts_with(&format!("{}/", base_path)))
                    && base_path.len() > best_len
                {
                    best = Some(session);
                    best_len = base_path.len();
                }
            }
            best
        }

        /// Find the session (mutable) whose base_path is a prefix of the given address.
        pub fn find_session_mut(&mut self, address: &str) -> Option<&mut E2ESession> {
            let mut best_path: Option<String> = None;
            let mut best_len = 0;
            for base_path in self.sessions.keys() {
                if (address == base_path.as_str()
                    || address.starts_with(&format!("{}/", base_path)))
                    && base_path.len() > best_len
                {
                    best_path = Some(base_path.clone());
                    best_len = base_path.len();
                }
            }
            best_path.and_then(move |p| self.sessions.get_mut(&p))
        }

        /// Close all sessions.
        pub fn close(&mut self) {
            for session in self.sessions.values_mut() {
                session.destroy();
            }
            self.sessions.clear();
        }
    }
}

#[cfg(feature = "client")]
pub use inner::*;
