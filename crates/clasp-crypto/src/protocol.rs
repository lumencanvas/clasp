//! E2E encryption session — manages key exchange for one group/room/channel.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::STANDARD as B64, Engine};
use zeroize::{Zeroize, Zeroizing};

use crate::error::{CryptoError, Result};
use crate::primitives;
use crate::storage::KeyStore;
use crate::types::{
    E2EEnvelope, ECDHKeyPair, KeyData, KeyExchangeMessage, PublicKeyAnnouncement, TofuRecord,
};

/// Callback for TOFU key change events.
/// Must return `true` to accept the new key, or `false` to reject.
/// If absent, key changes are rejected by default.
pub type OnKeyChange = Arc<dyn Fn(&str, &str, &str) -> bool + Send + Sync>;

/// Configuration for an E2E session.
pub struct E2ESessionConfig {
    pub identity_id: String,
    pub base_path: String,
    pub store: Arc<dyn KeyStore>,
    pub on_key_change: Option<OnKeyChange>,
    pub password_hash: Option<String>,
}

/// E2E encryption session state machine.
///
/// Manages ECDH key exchange, group key distribution, TOFU verification,
/// and encrypt/decrypt operations. This is the protocol layer that works
/// with raw bytes and messages — the CryptoClient (behind `client` feature)
/// wires it to a CLASP client.
pub struct E2ESession {
    config: E2ESessionConfig,
    group_key: Option<Vec<u8>>,
    ecdh_key_pair: Option<ECDHKeyPair>,
    peer_public_keys: HashMap<String, Vec<u8>>,
    started: bool,
    destroyed: bool,
}

impl E2ESession {
    pub fn new(config: E2ESessionConfig) -> Self {
        Self {
            config,
            group_key: None,
            ecdh_key_pair: None,
            peer_public_keys: HashMap::new(),
            started: false,
            destroyed: false,
        }
    }

    /// Whether this session has an active group key.
    pub fn encrypted(&self) -> bool {
        self.group_key.is_some()
    }

    /// The base path for this session's E2E subpaths.
    pub fn base_path(&self) -> &str {
        &self.config.base_path
    }

    /// Start the session: attempt to load a persisted group key.
    pub async fn start(&mut self) -> Result<()> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        if self.started {
            return Ok(());
        }
        self.started = true;

        // Try loading persisted key (stored as JWK)
        let session_id = self.session_id();
        if let Some(data) = self.config.store.load_group_key(&session_id).await? {
            match primitives::jwk_to_group_key(&data.key) {
                Ok(key) => self.group_key = Some(key),
                Err(_) => {
                    self.config.store.delete_group_key(&session_id).await?;
                }
            }
        }

        Ok(())
    }

    /// Enable encryption: generate a new group key.
    /// Returns a PublicKeyAnnouncement to be published via CLASP.
    pub async fn enable_encryption(&mut self) -> Result<PublicKeyAnnouncement> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }

        let mut key = Zeroizing::new(primitives::generate_group_key());
        self.group_key = Some(key.to_vec());

        // Persist as JWK for JS interop
        let jwk = primitives::group_key_to_jwk(&key)?;
        key.zeroize();
        self.config
            .store
            .save_group_key(
                &self.session_id(),
                KeyData {
                    key: jwk,
                    stored_at: now_ms(),
                },
            )
            .await?;

        self.make_public_key_announcement()
    }

    /// Create a public key announcement (for requestGroupKey).
    pub fn request_group_key(&mut self) -> Result<Option<PublicKeyAnnouncement>> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        if self.group_key.is_some() {
            return Ok(None);
        }
        self.make_public_key_announcement().map(Some)
    }

    /// Encrypt a string value into an E2EEnvelope.
    pub fn encrypt(&self, value: &str) -> Result<E2EEnvelope> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        let key = self.group_key.as_ref().ok_or(CryptoError::NoGroupKey)?;
        let plaintext = value.as_bytes();
        let (ciphertext, iv) = primitives::encrypt(key, plaintext)?;
        Ok(E2EEnvelope {
            _e2e: 1,
            ct: B64.encode(&ciphertext),
            iv: B64.encode(&iv),
            v: 1,
        })
    }

    /// Decrypt an E2EEnvelope back to a string.
    pub async fn decrypt(&mut self, envelope: &E2EEnvelope) -> Result<String> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        if envelope._e2e != 1 {
            return Err(CryptoError::DecryptionFailed("invalid E2E marker".into()));
        }
        if envelope.v != 1 {
            return Err(CryptoError::DecryptionFailed("unsupported envelope version".into()));
        }
        let key = Zeroizing::new(match &self.group_key {
            Some(k) => k.clone(),
            None => {
                let session_id = self.session_id();
                match self.config.store.load_group_key(&session_id).await? {
                    Some(data) => {
                        let k = primitives::jwk_to_group_key(&data.key)?;
                        self.group_key = Some(k.clone());
                        k
                    }
                    None => return Err(CryptoError::NoGroupKey),
                }
            }
        });

        let ciphertext = B64.decode(&envelope.ct)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid base64 ct: {e}")))?;
        let iv = B64.decode(&envelope.iv)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid base64 iv: {e}")))?;
        let plaintext = primitives::decrypt(&key, &ciphertext, &iv)?;
        String::from_utf8(plaintext)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid UTF-8: {e}")))
    }

    /// Handle a peer's public key announcement.
    /// Returns a KeyExchangeMessage if we have the group key and should distribute it.
    ///
    /// **Password-gated sessions**: If `password_hash` is set, the caller must
    /// verify the peer's password proof *before* calling this method. This method
    /// does not enforce password gating — it is the caller's responsibility.
    pub async fn handle_peer_pubkey(
        &mut self,
        peer_id: &str,
        announcement: &PublicKeyAnnouncement,
    ) -> Result<Option<KeyExchangeMessage>> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        if peer_id == self.config.identity_id {
            return Ok(None);
        }

        // Convert JWK to SEC1 for internal crypto operations
        let peer_pub_bytes = primitives::jwk_to_public_key(&announcement.public_key)?;

        // TOFU verification (uses JWK fingerprint for JS interop)
        self.verify_peer_key(peer_id, &announcement.public_key).await?;

        // Cache SEC1 bytes for crypto
        self.peer_public_keys.insert(peer_id.to_string(), peer_pub_bytes.clone());

        // Only distribute if we have the group key
        let group_key = Zeroizing::new(match &self.group_key {
            Some(k) => k.clone(),
            None => return Ok(None),
        });

        // Derive shared key and encrypt the group key as JWK JSON (JS interop)
        self.ensure_ecdh_key_pair();
        let kp = self.ecdh_key_pair();
        let shared = Zeroizing::new(primitives::derive_shared_key(&kp.private_key, &peer_pub_bytes, None)?);
        let group_key_jwk = primitives::group_key_to_jwk(&group_key)?;
        let mut group_key_json = Zeroizing::new(serde_json::to_string(&group_key_jwk)
            .map_err(|e| CryptoError::Serialization(e.to_string()))?);
        let (ct, iv) = primitives::encrypt(&shared, group_key_json.as_bytes())?;
        group_key_json.zeroize();

        let sender_pub_jwk = primitives::public_key_to_jwk(&kp.public_key)?;

        Ok(Some(KeyExchangeMessage {
            from_id: self.config.identity_id.clone(),
            encrypted_key: B64.encode(&ct),
            iv: B64.encode(&iv),
            sender_public_key: sender_pub_jwk,
        }))
    }

    /// Handle a key exchange message sent to us.
    /// Decrypts and stores the group key.
    pub async fn handle_key_exchange(&mut self, msg: &KeyExchangeMessage) -> Result<()> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        // Reject empty sender ID — prevents TOFU bypass
        if msg.from_id.is_empty() {
            return Err(CryptoError::InvalidKey("key exchange message missing sender ID".into()));
        }

        let sender_pub = primitives::jwk_to_public_key(&msg.sender_public_key)?;

        // TOFU verify sender (uses JWK fingerprint)
        self.verify_peer_key(&msg.from_id, &msg.sender_public_key).await?;

        // Cache sender's public key for future key rotations
        self.peer_public_keys.insert(msg.from_id.clone(), sender_pub.clone());

        self.ensure_ecdh_key_pair();
        let kp = self.ecdh_key_pair();
        let shared = Zeroizing::new(primitives::derive_shared_key(&kp.private_key, &sender_pub, None)?);

        let ct = B64.decode(&msg.encrypted_key)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid base64: {e}")))?;
        let iv = B64.decode(&msg.iv)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid base64: {e}")))?;

        let decrypted = primitives::decrypt(&shared, &ct, &iv)?;
        let mut key_json = Zeroizing::new(String::from_utf8(decrypted)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid UTF-8: {e}")))?);
        let key_jwk: serde_json::Value = serde_json::from_str(&key_json)
            .map_err(|e| CryptoError::DecryptionFailed(format!("invalid JWK JSON: {e}")))?;
        key_json.zeroize();
        let mut group_key = Zeroizing::new(primitives::jwk_to_group_key(&key_jwk)?);

        self.group_key = Some(group_key.to_vec());
        group_key.zeroize();

        // Persist as JWK
        self.config
            .store
            .save_group_key(
                &self.session_id(),
                KeyData {
                    key: key_jwk,
                    stored_at: now_ms(),
                },
            )
            .await?;

        Ok(())
    }

    /// Rotate the group key. Returns KeyExchangeMessages for all cached peers.
    pub async fn rotate_key(&mut self) -> Result<Vec<(String, KeyExchangeMessage)>> {
        if self.destroyed {
            return Err(CryptoError::SessionDestroyed);
        }
        if self.group_key.is_none() {
            return Ok(vec![]);
        }

        // Zeroize old key before replacing
        if let Some(ref mut old_key) = self.group_key {
            old_key.zeroize();
        }
        let mut new_key = Zeroizing::new(primitives::generate_group_key());
        self.group_key = Some(new_key.to_vec());

        let jwk = primitives::group_key_to_jwk(&new_key)?;
        let mut group_key_json = Zeroizing::new(serde_json::to_string(&jwk)
            .map_err(|e| CryptoError::Serialization(e.to_string()))?);
        new_key.zeroize();
        self.config
            .store
            .save_group_key(
                &self.session_id(),
                KeyData {
                    key: jwk,
                    stored_at: now_ms(),
                },
            )
            .await?;

        // Distribute to all cached peers
        self.ensure_ecdh_key_pair();
        let kp = self.ecdh_key_pair();
        let sender_pub_jwk = primitives::public_key_to_jwk(&kp.public_key)?;
        let mut messages = Vec::new();

        for (peer_id, peer_pub) in &self.peer_public_keys {
            if *peer_id == self.config.identity_id {
                continue;
            }
            if let Ok(shared) = primitives::derive_shared_key(&kp.private_key, peer_pub, None) {
                let mut shared = Zeroizing::new(shared);
                if let Ok((ct, iv)) = primitives::encrypt(&shared, group_key_json.as_bytes()) {
                    messages.push((
                        peer_id.clone(),
                        KeyExchangeMessage {
                            from_id: self.config.identity_id.clone(),
                            encrypted_key: B64.encode(&ct),
                            iv: B64.encode(&iv),
                            sender_public_key: sender_pub_jwk.clone(),
                        },
                    ));
                }
                shared.zeroize();
            }
        }
        group_key_json.zeroize();

        Ok(messages)
    }

    /// Remove a peer's cached public key.
    pub fn remove_peer(&mut self, peer_id: &str) {
        self.peer_public_keys.remove(peer_id);
    }

    /// Destroy the session, zeroing all key material.
    pub fn destroy(&mut self) {
        self.destroyed = true;
        if let Some(ref mut key) = self.group_key {
            key.zeroize();
        }
        self.group_key = None;
        // ECDHKeyPair implements ZeroizeOnDrop, so dropping clears it
        self.ecdh_key_pair = None;
        self.peer_public_keys.clear();
    }

    // --- Private ---

    fn session_id(&self) -> String {
        self.config.base_path.clone()
    }

    /// Ensure the ECDH key pair is initialized. Call this before accessing
    /// `self.ecdh_key_pair` to avoid unnecessary cloning of private key material.
    fn ensure_ecdh_key_pair(&mut self) {
        if self.ecdh_key_pair.is_none() {
            self.ecdh_key_pair = Some(primitives::generate_ecdh_key_pair());
        }
    }

    /// Access the ECDH key pair (must call ensure_ecdh_key_pair first).
    fn ecdh_key_pair(&self) -> &ECDHKeyPair {
        self.ecdh_key_pair.as_ref().unwrap()
    }

    fn make_public_key_announcement(&mut self) -> Result<PublicKeyAnnouncement> {
        self.ensure_ecdh_key_pair();
        let kp = self.ecdh_key_pair();
        let jwk = primitives::public_key_to_jwk(&kp.public_key)?;
        Ok(PublicKeyAnnouncement {
            public_key: jwk,
            timestamp: now_ms(),
        })
    }

    /// TOFU verification using JWK fingerprint (matches JS implementation).
    /// Always stores the record on first use. On key change, calls the
    /// `on_key_change` callback which must return `true` to accept.
    /// If no callback is set, key changes are rejected.
    async fn verify_peer_key(&self, peer_id: &str, public_key_jwk: &serde_json::Value) -> Result<()> {
        let fp = primitives::fingerprint_jwk(public_key_jwk);
        let record_id = format!("{}:{}", self.config.base_path, peer_id);

        let stored = self.config.store.load_tofu_record(&record_id).await?;

        match stored {
            None => {
                // First time — trust on first use
                self.config
                    .store
                    .save_tofu_record(
                        &record_id,
                        TofuRecord {
                            fingerprint: fp,
                            first_seen: now_ms(),
                        },
                    )
                    .await?;
            }
            Some(record) => {
                if !primitives::constant_time_eq(record.fingerprint.as_bytes(), fp.as_bytes()) {
                    // Key changed — check if caller accepts
                    let accepted = self.config.on_key_change
                        .as_ref()
                        .map(|cb| cb(peer_id, &record.fingerprint, &fp))
                        .unwrap_or(false);
                    if !accepted {
                        return Err(CryptoError::TofuViolation(peer_id.to_string()));
                    }
                    // Update the stored record to the new fingerprint,
                    // preserving original first_seen
                    self.config
                        .store
                        .save_tofu_record(
                            &record_id,
                            TofuRecord {
                                fingerprint: fp,
                                first_seen: record.first_seen,
                            },
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }
}

impl Drop for E2ESession {
    fn drop(&mut self) {
        if !self.destroyed {
            self.destroy();
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::MemoryKeyStore;

    fn test_config(store: Arc<dyn KeyStore>) -> E2ESessionConfig {
        E2ESessionConfig {
            identity_id: "alice".to_string(),
            base_path: "/test/room/1".to_string(),
            store,
            on_key_change: None,
            password_hash: None,
        }
    }

    #[tokio::test]
    async fn session_starts_without_key() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        assert!(!session.encrypted());
    }

    #[tokio::test]
    async fn enable_encryption_creates_key() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();
        assert!(session.encrypted());
    }

    #[tokio::test]
    async fn encrypt_decrypt_round_trip() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        let envelope = session.encrypt("Hello, world!").unwrap();
        assert_eq!(envelope._e2e, 1);
        assert_eq!(envelope.v, 1);

        let decrypted = session.decrypt(&envelope).await.unwrap();
        assert_eq!(decrypted, "Hello, world!");
    }

    #[tokio::test]
    async fn persists_and_loads_key() {
        let store = Arc::new(MemoryKeyStore::new());

        let mut session1 = E2ESession::new(test_config(store.clone()));
        session1.start().await.unwrap();
        session1.enable_encryption().await.unwrap();
        let envelope = session1.encrypt("hello").unwrap();

        let mut session2 = E2ESession::new(test_config(store));
        session2.start().await.unwrap();
        assert!(session2.encrypted());

        let decrypted = session2.decrypt(&envelope).await.unwrap();
        assert_eq!(decrypted, "hello");
    }

    #[tokio::test]
    async fn key_exchange_between_peers() {
        let store_a = Arc::new(MemoryKeyStore::new());
        let store_b = Arc::new(MemoryKeyStore::new());

        let mut alice = E2ESession::new(E2ESessionConfig {
            identity_id: "alice".to_string(),
            base_path: "/test/room/1".to_string(),
            store: store_a,
            on_key_change: None,
            password_hash: None,
        });
        alice.start().await.unwrap();
        let _alice_announcement = alice.enable_encryption().await.unwrap();

        let mut bob = E2ESession::new(E2ESessionConfig {
            identity_id: "bob".to_string(),
            base_path: "/test/room/1".to_string(),
            store: store_b,
            on_key_change: None,
            password_hash: None,
        });
        bob.start().await.unwrap();
        let bob_announcement = bob.request_group_key().unwrap().unwrap();

        let keyex = alice
            .handle_peer_pubkey("bob", &bob_announcement)
            .await
            .unwrap();
        assert!(keyex.is_some());

        bob.handle_key_exchange(&keyex.unwrap()).await.unwrap();
        assert!(bob.encrypted());

        let envelope = alice.encrypt("secret message").unwrap();
        let decrypted = bob.decrypt(&envelope).await.unwrap();
        assert_eq!(decrypted, "secret message");
    }

    #[tokio::test]
    async fn rotate_key_invalidates_old_messages() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        let old_envelope = session.encrypt("before rotation").unwrap();
        session.rotate_key().await.unwrap();

        let result = session.decrypt(&old_envelope).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn tofu_detects_key_change_and_accepts() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let store = Arc::new(MemoryKeyStore::new());
        let changed = Arc::new(AtomicBool::new(false));
        let changed_clone = changed.clone();

        let mut session = E2ESession::new(E2ESessionConfig {
            identity_id: "alice".to_string(),
            base_path: "/test/room/1".to_string(),
            store: store.clone(),
            on_key_change: Some(Arc::new(move |_peer, _old, _new| {
                changed_clone.store(true, Ordering::SeqCst);
                true // accept the key change
            })),
            password_hash: None,
        });
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        // First key from Bob — TOFU, trusted
        let bob_kp1 = primitives::generate_ecdh_key_pair();
        let ann1 = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp1.public_key).unwrap(),
            timestamp: now_ms(),
        };
        session.handle_peer_pubkey("bob", &ann1).await.unwrap();
        assert!(!changed.load(Ordering::SeqCst));

        // Different key from Bob — should trigger change, accepted by callback
        let bob_kp2 = primitives::generate_ecdh_key_pair();
        let ann2 = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp2.public_key).unwrap(),
            timestamp: now_ms(),
        };
        session.handle_peer_pubkey("bob", &ann2).await.unwrap();
        assert!(changed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn tofu_rejects_key_change_without_callback() {
        let store = Arc::new(MemoryKeyStore::new());

        // No onKeyChange callback — key changes should be rejected
        let mut session = E2ESession::new(test_config(store.clone()));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        // First key from Bob — TOFU, trusted
        let bob_kp1 = primitives::generate_ecdh_key_pair();
        let ann1 = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp1.public_key).unwrap(),
            timestamp: now_ms(),
        };
        session.handle_peer_pubkey("bob", &ann1).await.unwrap();

        // Different key from Bob — should be rejected (no callback)
        let bob_kp2 = primitives::generate_ecdh_key_pair();
        let ann2 = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp2.public_key).unwrap(),
            timestamp: now_ms(),
        };
        let result = session.handle_peer_pubkey("bob", &ann2).await;
        assert!(matches!(result, Err(CryptoError::TofuViolation(_))));
    }

    #[tokio::test]
    async fn tofu_rejects_key_change_when_callback_returns_false() {
        let store = Arc::new(MemoryKeyStore::new());

        let mut session = E2ESession::new(E2ESessionConfig {
            identity_id: "alice".to_string(),
            base_path: "/test/room/1".to_string(),
            store: store.clone(),
            on_key_change: Some(Arc::new(|_peer, _old, _new| false)),
            password_hash: None,
        });
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        let bob_kp1 = primitives::generate_ecdh_key_pair();
        let ann1 = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp1.public_key).unwrap(),
            timestamp: now_ms(),
        };
        session.handle_peer_pubkey("bob", &ann1).await.unwrap();

        let bob_kp2 = primitives::generate_ecdh_key_pair();
        let ann2 = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp2.public_key).unwrap(),
            timestamp: now_ms(),
        };
        let result = session.handle_peer_pubkey("bob", &ann2).await;
        assert!(matches!(result, Err(CryptoError::TofuViolation(_))));
    }

    #[tokio::test]
    async fn tofu_stores_records_without_callback() {
        let store = Arc::new(MemoryKeyStore::new());

        // No onKeyChange callback
        let mut session = E2ESession::new(test_config(store.clone()));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        let bob_kp = primitives::generate_ecdh_key_pair();
        let ann = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp.public_key).unwrap(),
            timestamp: now_ms(),
        };
        session.handle_peer_pubkey("bob", &ann).await.unwrap();

        // TOFU record should be stored even without callback
        let record = store.load_tofu_record("/test/room/1:bob").await.unwrap();
        assert!(record.is_some());
    }

    #[tokio::test]
    async fn empty_from_id_rejected() {
        let store_a = Arc::new(MemoryKeyStore::new());
        let store_b = Arc::new(MemoryKeyStore::new());

        let mut alice = E2ESession::new(E2ESessionConfig {
            identity_id: "alice".to_string(),
            base_path: "/test/room/1".to_string(),
            store: store_a,
            on_key_change: None,
            password_hash: None,
        });
        alice.start().await.unwrap();

        let mut bob = E2ESession::new(E2ESessionConfig {
            identity_id: "bob".to_string(),
            base_path: "/test/room/1".to_string(),
            store: store_b,
            on_key_change: None,
            password_hash: None,
        });
        bob.start().await.unwrap();
        bob.enable_encryption().await.unwrap();
        let bob_announcement = bob.request_group_key().unwrap();

        // Craft a message with empty from_id
        let msg = KeyExchangeMessage {
            from_id: String::new(),
            encrypted_key: "AAAA".to_string(),
            iv: "BBBB".to_string(),
            sender_public_key: serde_json::json!({}),
        };
        let result = alice.handle_key_exchange(&msg).await;
        assert!(matches!(result, Err(CryptoError::InvalidKey(_))));

        // Suppress unused variable warning
        drop(bob_announcement);
    }

    #[tokio::test]
    async fn encrypt_after_destroy_fails() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();
        session.destroy();

        let result = session.encrypt("test");
        assert!(matches!(result, Err(CryptoError::SessionDestroyed)));
    }

    #[tokio::test]
    async fn decrypt_after_destroy_fails() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();
        let envelope = session.encrypt("test").unwrap();
        session.destroy();

        let result = session.decrypt(&envelope).await;
        assert!(matches!(result, Err(CryptoError::SessionDestroyed)));
    }

    #[tokio::test]
    async fn handle_peer_pubkey_after_destroy_fails() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();
        session.destroy();

        let bob_kp = primitives::generate_ecdh_key_pair();
        let ann = PublicKeyAnnouncement {
            public_key: primitives::public_key_to_jwk(&bob_kp.public_key).unwrap(),
            timestamp: now_ms(),
        };
        let result = session.handle_peer_pubkey("bob", &ann).await;
        assert!(matches!(result, Err(CryptoError::SessionDestroyed)));
    }

    #[tokio::test]
    async fn handle_key_exchange_after_destroy_fails() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.destroy();

        let msg = KeyExchangeMessage {
            from_id: "bob".to_string(),
            encrypted_key: "AAAA".to_string(),
            iv: "BBBB".to_string(),
            sender_public_key: serde_json::json!({}),
        };
        let result = session.handle_key_exchange(&msg).await;
        assert!(matches!(result, Err(CryptoError::SessionDestroyed)));
    }

    #[tokio::test]
    async fn decrypt_rejects_unknown_envelope_version() {
        let store = Arc::new(MemoryKeyStore::new());
        let mut session = E2ESession::new(test_config(store));
        session.start().await.unwrap();
        session.enable_encryption().await.unwrap();

        let envelope = E2EEnvelope {
            _e2e: 1,
            ct: "AAAA".to_string(),
            iv: "BBBB".to_string(),
            v: 2,
        };
        let result = session.decrypt(&envelope).await;
        assert!(matches!(result, Err(CryptoError::DecryptionFailed(_))));
    }
}
