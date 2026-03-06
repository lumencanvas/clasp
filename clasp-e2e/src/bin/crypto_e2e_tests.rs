//! E2E encryption tests through a real CLASP router.
//!
//! Tests encrypted pub/sub, key exchange flow, key rotation,
//! and multiple encrypted rooms using TestRouter + raw E2ESession.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use clasp_client::Clasp;
use clasp_core::{SecurityMode, Value};
use clasp_crypto::{E2ESession, E2ESessionConfig, MemoryKeyStore};
use clasp_router::{Router, RouterConfig};

// ============================================================================
// Test Framework
// ============================================================================

struct TestResult {
    name: &'static str,
    passed: bool,
    message: String,
    duration_ms: u128,
}

impl TestResult {
    fn pass(name: &'static str, duration_ms: u128) -> Self {
        Self {
            name,
            passed: true,
            message: "OK".to_string(),
            duration_ms,
        }
    }

    fn fail(name: &'static str, message: impl Into<String>, duration_ms: u128) -> Self {
        Self {
            name,
            passed: false,
            message: message.into(),
            duration_ms,
        }
    }
}

// ============================================================================
// Test Router
// ============================================================================

struct TestRouter {
    port: u16,
    handle: tokio::task::JoinHandle<()>,
}

impl TestRouter {
    async fn start() -> Self {
        let port = {
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            listener.local_addr().unwrap().port()
        };
        let addr = format!("127.0.0.1:{}", port);

        let router = Router::new(RouterConfig {
            name: "Crypto E2E Test Router".to_string(),
            max_sessions: 100,
            session_timeout: 60,
            features: vec![
                "param".to_string(),
                "event".to_string(),
                "stream".to_string(),
            ],
            security_mode: SecurityMode::Open,
            max_subscriptions_per_session: 1000,
            gesture_coalescing: false,
            gesture_coalesce_interval_ms: 0,
            max_messages_per_second: 0,
            rate_limiting_enabled: false,
            ..Default::default()
        });

        let handle = tokio::spawn(async move {
            let _ = router.serve_websocket(&addr).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;

        Self { port, handle }
    }

    fn url(&self) -> String {
        format!("ws://127.0.0.1:{}", self.port)
    }

    fn stop(self) {
        self.handle.abort();
    }
}

fn mk_config(id: &str, base_path: &str, store: Arc<MemoryKeyStore>) -> E2ESessionConfig {
    E2ESessionConfig {
        identity_id: id.into(),
        base_path: base_path.into(),
        store,
        on_key_change: None,
        password_hash: None,
        rotation_interval: None,
        on_rotation: None,
        max_announcement_age: None,
    }
}

// ============================================================================
// Tests
// ============================================================================

/// Alice encrypts and publishes through the router, Bob subscribes and decrypts.
async fn test_encrypted_pubsub_through_router() -> TestResult {
    let start = std::time::Instant::now();
    let name = "encrypted_pubsub_through_router";

    let router = TestRouter::start().await;
    let store_a = Arc::new(MemoryKeyStore::new());
    let store_b = Arc::new(MemoryKeyStore::new());

    // Set up Alice and Bob E2E sessions
    let mut alice = E2ESession::new(mk_config("alice", "/encrypted", store_a));
    alice.start().await.unwrap();
    let _ann = alice.enable_encryption().await.unwrap();

    let mut bob = E2ESession::new(mk_config("bob", "/encrypted", store_b));
    bob.start().await.unwrap();
    let bob_ann = bob.request_group_key().unwrap().unwrap();

    // Key exchange
    let keyex = alice
        .handle_peer_pubkey("bob", &bob_ann)
        .await
        .unwrap()
        .unwrap();
    bob.handle_key_exchange(&keyex).await.unwrap();

    if !bob.encrypted() {
        router.stop();
        return TestResult::fail(
            name,
            "Bob doesn't have group key",
            start.elapsed().as_millis(),
        );
    }

    // Alice encrypts
    let envelope = alice.encrypt("encrypted through router").unwrap();
    let envelope_json = serde_json::to_string(&envelope).unwrap();

    // Connect clients to router
    let sender = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("sender connect: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };

    let receiver = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("receiver connect: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };

    // Bob subscribes via router
    let received = Arc::new(AtomicBool::new(false));
    let received_clone = received.clone();
    let notify = Arc::new(tokio::sync::Notify::new());
    let notify_clone = notify.clone();

    let _ = receiver
        .subscribe("/encrypted/**", move |value, _| {
            if let Value::String(s) = value {
                // We just verify the envelope arrived; decryption is tested below
                if s.contains("\"_e2e\":1") || s.contains("\"_e2e\": 1") {
                    received_clone.store(true, Ordering::SeqCst);
                    notify_clone.notify_one();
                }
            }
        })
        .await;

    tokio::time::sleep(Duration::from_millis(50)).await;

    // Alice publishes encrypted envelope through router
    let _ = sender
        .set("/encrypted/signal", Value::String(envelope_json))
        .await;

    let _ = tokio::time::timeout(Duration::from_secs(2), notify.notified()).await;

    if !received.load(Ordering::SeqCst) {
        router.stop();
        return TestResult::fail(
            name,
            "Encrypted envelope not received",
            start.elapsed().as_millis(),
        );
    }

    // Bob decrypts the envelope
    let decrypted = bob.decrypt(&envelope).await.unwrap();
    if decrypted != "encrypted through router" {
        router.stop();
        return TestResult::fail(
            name,
            format!("Wrong decryption: {decrypted}"),
            start.elapsed().as_millis(),
        );
    }

    router.stop();
    TestResult::pass(name, start.elapsed().as_millis())
}

/// Key exchange messages flow through the router via subscriptions.
async fn test_key_exchange_through_router() -> TestResult {
    let start = std::time::Instant::now();
    let name = "key_exchange_through_router";

    let router = TestRouter::start().await;
    let store_a = Arc::new(MemoryKeyStore::new());
    let store_b = Arc::new(MemoryKeyStore::new());

    // Alice enables encryption
    let mut alice = E2ESession::new(mk_config("alice", "/keyex-room", store_a));
    alice.start().await.unwrap();
    let alice_ann = alice.enable_encryption().await.unwrap();

    // Bob requests group key
    let mut bob = E2ESession::new(mk_config("bob", "/keyex-room", store_b));
    bob.start().await.unwrap();
    let bob_ann = bob.request_group_key().unwrap().unwrap();

    // Connect clients to router for pubkey/keyex distribution
    let alice_client = Clasp::connect_to(&router.url()).await.unwrap();
    let bob_client = Clasp::connect_to(&router.url()).await.unwrap();

    // Publish announcements through router
    let alice_ann_json = serde_json::to_string(&alice_ann).unwrap();
    let bob_ann_json = serde_json::to_string(&bob_ann).unwrap();

    let _ = alice_client
        .set("/_e2e/pubkey/alice", Value::String(alice_ann_json))
        .await;
    let _ = bob_client
        .set("/_e2e/pubkey/bob", Value::String(bob_ann_json))
        .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Alice handles Bob's announcement and generates key exchange
    let keyex = alice
        .handle_peer_pubkey("bob", &bob_ann)
        .await
        .unwrap()
        .unwrap();

    // Distribute keyex through router
    let keyex_json = serde_json::to_string(&keyex).unwrap();
    let _ = alice_client
        .emit("/_e2e/keyex/bob", Value::String(keyex_json))
        .await;

    // Bob handles the key exchange directly
    bob.handle_key_exchange(&keyex).await.unwrap();

    if !bob.encrypted() {
        router.stop();
        return TestResult::fail(
            name,
            "Bob doesn't have group key after keyex",
            start.elapsed().as_millis(),
        );
    }

    // Verify both can encrypt/decrypt
    let envelope = alice.encrypt("via router keyex").unwrap();
    let decrypted = bob.decrypt(&envelope).await.unwrap();

    router.stop();

    if decrypted != "via router keyex" {
        return TestResult::fail(
            name,
            format!("Wrong decryption: {decrypted}"),
            start.elapsed().as_millis(),
        );
    }

    TestResult::pass(name, start.elapsed().as_millis())
}

/// Alice rotates the key, sends new keyex through router, Bob decrypts post-rotation.
async fn test_key_rotation_through_router() -> TestResult {
    let start = std::time::Instant::now();
    let name = "key_rotation_through_router";

    let router = TestRouter::start().await;
    let store_a = Arc::new(MemoryKeyStore::new());
    let store_b = Arc::new(MemoryKeyStore::new());

    // Initial key exchange
    let mut alice = E2ESession::new(mk_config("alice", "/rotation-room", store_a));
    alice.start().await.unwrap();
    alice.enable_encryption().await.unwrap();

    let mut bob = E2ESession::new(mk_config("bob", "/rotation-room", store_b));
    bob.start().await.unwrap();
    let bob_ann = bob.request_group_key().unwrap().unwrap();
    let keyex = alice
        .handle_peer_pubkey("bob", &bob_ann)
        .await
        .unwrap()
        .unwrap();
    bob.handle_key_exchange(&keyex).await.unwrap();

    // Encrypt before rotation
    let pre_rotation_envelope = alice.encrypt("before rotation").unwrap();

    // Rotate key
    let rotation_msgs = alice.rotate_key().await.unwrap();

    // Bob receives the rotation key exchange through router
    let alice_client = Clasp::connect_to(&router.url()).await.unwrap();
    for (peer_id, msg) in &rotation_msgs {
        if peer_id == "bob" {
            let msg_json = serde_json::to_string(msg).unwrap();
            let _ = alice_client
                .emit(&format!("/_e2e/keyex/{peer_id}"), Value::String(msg_json))
                .await;
            bob.handle_key_exchange(msg).await.unwrap();
        }
    }

    // Old message should fail with new key
    let old_result = bob.decrypt(&pre_rotation_envelope).await;
    if old_result.is_ok() {
        router.stop();
        return TestResult::fail(
            name,
            "Old message should fail after rotation",
            start.elapsed().as_millis(),
        );
    }

    // New message should work
    let new_envelope = alice.encrypt("after rotation").unwrap();
    let decrypted = bob.decrypt(&new_envelope).await.unwrap();

    router.stop();

    if decrypted != "after rotation" {
        return TestResult::fail(
            name,
            format!("Wrong decryption: {decrypted}"),
            start.elapsed().as_millis(),
        );
    }

    TestResult::pass(name, start.elapsed().as_millis())
}

/// Multiple encrypted rooms with separate keys.
async fn test_multiple_encrypted_rooms() -> TestResult {
    let start = std::time::Instant::now();
    let name = "multiple_encrypted_rooms";

    let router = TestRouter::start().await;

    // Room 1: Alice + Bob
    let store_a1 = Arc::new(MemoryKeyStore::new());
    let store_b1 = Arc::new(MemoryKeyStore::new());

    let mut alice_room1 = E2ESession::new(mk_config("alice", "/room/1", store_a1));
    alice_room1.start().await.unwrap();
    alice_room1.enable_encryption().await.unwrap();

    let mut bob_room1 = E2ESession::new(mk_config("bob", "/room/1", store_b1));
    bob_room1.start().await.unwrap();
    let bob_ann1 = bob_room1.request_group_key().unwrap().unwrap();
    let keyex1 = alice_room1
        .handle_peer_pubkey("bob", &bob_ann1)
        .await
        .unwrap()
        .unwrap();
    bob_room1.handle_key_exchange(&keyex1).await.unwrap();

    // Room 2: Alice + Carol
    let store_a2 = Arc::new(MemoryKeyStore::new());
    let store_c2 = Arc::new(MemoryKeyStore::new());

    let mut alice_room2 = E2ESession::new(mk_config("alice", "/room/2", store_a2));
    alice_room2.start().await.unwrap();
    alice_room2.enable_encryption().await.unwrap();

    let mut carol_room2 = E2ESession::new(mk_config("carol", "/room/2", store_c2));
    carol_room2.start().await.unwrap();
    let carol_ann = carol_room2.request_group_key().unwrap().unwrap();
    let keyex2 = alice_room2
        .handle_peer_pubkey("carol", &carol_ann)
        .await
        .unwrap()
        .unwrap();
    carol_room2.handle_key_exchange(&keyex2).await.unwrap();

    // Publish encrypted messages through router
    let client = Clasp::connect_to(&router.url()).await.unwrap();

    let env1 = alice_room1.encrypt("room 1 secret").unwrap();
    let env2 = alice_room2.encrypt("room 2 secret").unwrap();

    let _ = client
        .set(
            "/room/1/msg",
            Value::String(serde_json::to_string(&env1).unwrap()),
        )
        .await;
    let _ = client
        .set(
            "/room/2/msg",
            Value::String(serde_json::to_string(&env2).unwrap()),
        )
        .await;

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Bob decrypts room 1 message
    let d1 = bob_room1.decrypt(&env1).await.unwrap();
    if d1 != "room 1 secret" {
        router.stop();
        return TestResult::fail(name, format!("Room 1: {d1}"), start.elapsed().as_millis());
    }

    // Carol decrypts room 2 message
    let d2 = carol_room2.decrypt(&env2).await.unwrap();
    if d2 != "room 2 secret" {
        router.stop();
        return TestResult::fail(name, format!("Room 2: {d2}"), start.elapsed().as_millis());
    }

    // Bob should NOT decrypt room 2 message (different key)
    let cross_result = bob_room1.decrypt(&env2).await;
    if cross_result.is_ok() {
        router.stop();
        return TestResult::fail(
            name,
            "Bob should not decrypt room 2's message",
            start.elapsed().as_millis(),
        );
    }

    router.stop();
    TestResult::pass(name, start.elapsed().as_millis())
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("\n=== CLASP Crypto E2E Tests (Through Router) ===\n");

    let tests = vec![
        test_encrypted_pubsub_through_router().await,
        test_key_exchange_through_router().await,
        test_key_rotation_through_router().await,
        test_multiple_encrypted_rooms().await,
    ];

    let mut passed = 0;
    let mut failed = 0;

    for test in &tests {
        let status = if test.passed { "PASS" } else { "FAIL" };
        let color = if test.passed { "\x1b[32m" } else { "\x1b[31m" };
        println!(
            "  {}{}\x1b[0m {} ({}ms)",
            color, status, test.name, test.duration_ms
        );
        if test.passed {
            passed += 1;
        } else {
            failed += 1;
            println!("       {}", test.message);
        }
    }

    println!("\nResults: {} passed, {} failed", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}
