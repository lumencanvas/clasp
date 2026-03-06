//! E2E encryption interop tests: Rust-to-Rust scenarios.
//!
//! Tests key exchange, rotation, multi-peer, and TOFU verification
//! using E2ESession directly (no router needed).

use std::sync::Arc;

use anyhow::Result;
use clasp_crypto::types::PublicKeyAnnouncement;
use clasp_crypto::{E2ESession, E2ESessionConfig, MemoryKeyStore};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    println!("=== CLASP Crypto Interop Tests (Rust-to-Rust) ===\n");

    test_key_exchange().await?;
    test_key_rotation().await?;
    test_multi_peer().await?;
    test_tofu_violation().await?;

    println!("\nAll crypto interop tests passed.");
    Ok(())
}

async fn test_key_exchange() -> Result<()> {
    print!("  Key exchange: Alice enables, Bob requests, Bob decrypts ... ");

    let store_a = Arc::new(MemoryKeyStore::new());
    let store_b = Arc::new(MemoryKeyStore::new());

    let mut alice = E2ESession::new(E2ESessionConfig {
        identity_id: "alice".into(),
        base_path: "/test/e2e".into(),
        store: store_a,
        on_key_change: None,
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    });
    alice.start().await?;
    let _ann = alice.enable_encryption().await?;

    let mut bob = E2ESession::new(E2ESessionConfig {
        identity_id: "bob".into(),
        base_path: "/test/e2e".into(),
        store: store_b,
        on_key_change: None,
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    });
    bob.start().await?;
    let bob_ann = bob.request_group_key()?.expect("Bob should announce");

    let keyex = alice
        .handle_peer_pubkey("bob", &bob_ann)
        .await?
        .expect("Alice should distribute key");
    bob.handle_key_exchange(&keyex).await?;

    assert!(bob.encrypted(), "Bob should have the group key");

    let envelope = alice.encrypt("Hello from Alice")?;
    let decrypted = bob.decrypt(&envelope).await?;
    assert_eq!(decrypted, "Hello from Alice");

    println!("OK");
    Ok(())
}

async fn test_key_rotation() -> Result<()> {
    print!("  Key rotation: Alice rotates, Bob gets new key ... ");

    let store_a = Arc::new(MemoryKeyStore::new());
    let store_b = Arc::new(MemoryKeyStore::new());

    let mut alice = E2ESession::new(E2ESessionConfig {
        identity_id: "alice".into(),
        base_path: "/test/rotation".into(),
        store: store_a,
        on_key_change: None,
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    });
    alice.start().await?;
    alice.enable_encryption().await?;

    let mut bob = E2ESession::new(E2ESessionConfig {
        identity_id: "bob".into(),
        base_path: "/test/rotation".into(),
        store: store_b,
        on_key_change: None,
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    });
    bob.start().await?;
    let bob_ann = bob.request_group_key()?.unwrap();
    let keyex = alice.handle_peer_pubkey("bob", &bob_ann).await?.unwrap();
    bob.handle_key_exchange(&keyex).await?;

    // Encrypt before rotation
    let old_envelope = alice.encrypt("before")?;

    // Rotate
    let rotation_msgs = alice.rotate_key().await?;
    assert!(!rotation_msgs.is_empty(), "Should distribute to Bob");

    // Bob receives the new key
    for (_, msg) in &rotation_msgs {
        bob.handle_key_exchange(msg).await?;
    }

    // Old message should fail to decrypt
    let old_result = bob.decrypt(&old_envelope).await;
    assert!(old_result.is_err(), "Old message should fail");

    // New message should work
    let new_envelope = alice.encrypt("after")?;
    let decrypted = bob.decrypt(&new_envelope).await?;
    assert_eq!(decrypted, "after");

    println!("OK");
    Ok(())
}

async fn test_multi_peer() -> Result<()> {
    print!("  Multi-peer: 3 clients, all decrypt after joining ... ");

    let store_a = Arc::new(MemoryKeyStore::new());
    let store_b = Arc::new(MemoryKeyStore::new());
    let store_c = Arc::new(MemoryKeyStore::new());

    let mk_config = |id: &str, store: Arc<MemoryKeyStore>| E2ESessionConfig {
        identity_id: id.into(),
        base_path: "/test/multi".into(),
        store,
        on_key_change: None,
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    };

    let mut alice = E2ESession::new(mk_config("alice", store_a));
    alice.start().await?;
    alice.enable_encryption().await?;

    let mut bob = E2ESession::new(mk_config("bob", store_b));
    bob.start().await?;
    let bob_ann = bob.request_group_key()?.unwrap();
    let keyex_b = alice.handle_peer_pubkey("bob", &bob_ann).await?.unwrap();
    bob.handle_key_exchange(&keyex_b).await?;

    let mut carol = E2ESession::new(mk_config("carol", store_c));
    carol.start().await?;
    let carol_ann = carol.request_group_key()?.unwrap();
    let keyex_c = alice
        .handle_peer_pubkey("carol", &carol_ann)
        .await?
        .unwrap();
    carol.handle_key_exchange(&keyex_c).await?;

    // All three should be able to encrypt/decrypt
    let envelope = alice.encrypt("group message")?;
    assert_eq!(bob.decrypt(&envelope).await?, "group message");
    assert_eq!(carol.decrypt(&envelope).await?, "group message");

    println!("OK");
    Ok(())
}

async fn test_tofu_violation() -> Result<()> {
    print!("  TOFU violation: key change rejected without callback ... ");

    let store = Arc::new(MemoryKeyStore::new());

    let mut session = E2ESession::new(E2ESessionConfig {
        identity_id: "alice".into(),
        base_path: "/test/tofu".into(),
        store,
        on_key_change: None, // no callback = reject key changes
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    });
    session.start().await?;
    session.enable_encryption().await?;

    // First key from Bob — trusted
    let bob_kp1 = clasp_crypto::generate_ecdh_key_pair();
    let ann1 = PublicKeyAnnouncement {
        public_key: clasp_crypto::public_key_to_jwk(&bob_kp1.public_key)?,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    session.handle_peer_pubkey("bob", &ann1).await?;

    // Different key from Bob — should be rejected
    let bob_kp2 = clasp_crypto::generate_ecdh_key_pair();
    let ann2 = PublicKeyAnnouncement {
        public_key: clasp_crypto::public_key_to_jwk(&bob_kp2.public_key)?,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64,
    };
    let result = session.handle_peer_pubkey("bob", &ann2).await;
    assert!(result.is_err(), "Should reject TOFU violation");

    println!("OK");
    Ok(())
}
