//! CryptoClient wrapper for transparent E2E encryption over a CLASP client.
//! Behind the `client` feature flag since it depends on clasp-client.

#[cfg(feature = "client")]
mod inner {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use crate::error::{CryptoError, Result};
    use crate::protocol::{E2ESession, E2ESessionConfig, OnKeyChange};
    use crate::storage::KeyStore;
    use crate::types::E2EEnvelope;

    /// Configuration for CryptoClient.
    pub struct CryptoClientConfig {
        pub identity_id: String,
        pub store: Arc<dyn KeyStore>,
        pub on_key_change: Option<OnKeyChange>,
        /// Default rotation interval applied to all new sessions.
        pub rotation_interval: Option<Duration>,
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
                    rotation_interval: self.config.rotation_interval,
                    on_rotation: None,
                    max_announcement_age: None,
                });
                self.sessions.insert(base_path.to_string(), session);
            }
            self.sessions.get_mut(base_path).unwrap()
        }

        /// Encrypt a string value and set it via the inner client.
        /// Drives automatic key rotation before encrypting if due.
        pub async fn set_encrypted(
            &mut self,
            address: &str,
            session_path: &str,
            value: &str,
        ) -> Result<()> {
            self.drive_rotation(session_path).await?;
            let session = self
                .sessions
                .get(session_path)
                .ok_or(CryptoError::NoGroupKey)?;
            let envelope = session.encrypt(value)?;
            let json = serde_json::to_value(&envelope)
                .map_err(|e| CryptoError::Serialization(e.to_string()))?;
            self.inner
                .set(address, clasp_core::Value::from(json.to_string()))
                .await
                .map_err(|e| CryptoError::Other(e.to_string()))
        }

        /// Emit an encrypted event via the inner client.
        /// Drives automatic key rotation before encrypting if due.
        pub async fn emit_encrypted(
            &mut self,
            address: &str,
            session_path: &str,
            value: &str,
        ) -> Result<()> {
            self.drive_rotation(session_path).await?;
            let session = self
                .sessions
                .get(session_path)
                .ok_or(CryptoError::NoGroupKey)?;
            let envelope = session.encrypt(value)?;
            let json = serde_json::to_value(&envelope)
                .map_err(|e| CryptoError::Serialization(e.to_string()))?;
            self.inner
                .emit(address, clasp_core::Value::from(json.to_string()))
                .await
                .map_err(|e| CryptoError::Other(e.to_string()))
        }

        /// Tick all sessions for automatic rotation. Call this from a
        /// `tokio::select!` loop or periodic timer to drive rotation
        /// when not actively sending messages.
        pub async fn tick_rotations(&mut self) -> Result<()> {
            let paths: Vec<String> = self.sessions.keys().cloned().collect();
            for path in paths {
                self.drive_rotation(&path).await?;
            }
            Ok(())
        }

        /// Drive rotation for a single session, distributing key exchange
        /// messages via the inner client if rotation occurred.
        async fn drive_rotation(&mut self, session_path: &str) -> Result<()> {
            let session = match self.sessions.get_mut(session_path) {
                Some(s) => s,
                None => return Ok(()),
            };
            if let Some((messages, announcement)) = session.maybe_rotate().await? {
                // Publish the new announcement
                let ann_json = serde_json::to_value(&announcement)
                    .map_err(|e| CryptoError::Serialization(e.to_string()))?;
                let pubkey_path =
                    format!("{}/_e2e/pubkey/{}", session_path, self.config.identity_id);
                self.inner
                    .set(&pubkey_path, clasp_core::Value::from(ann_json.to_string()))
                    .await
                    .map_err(|e| CryptoError::Other(e.to_string()))?;

                // Distribute key exchange messages to peers
                for (peer_id, keyex) in messages {
                    let keyex_json = serde_json::to_value(&keyex)
                        .map_err(|e| CryptoError::Serialization(e.to_string()))?;
                    let keyex_path = format!("{}/_e2e/keyex/{}", session_path, peer_id);
                    self.inner
                        .emit(&keyex_path, clasp_core::Value::from(keyex_json.to_string()))
                        .await
                        .map_err(|e| CryptoError::Other(e.to_string()))?;
                }
            }
            Ok(())
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
