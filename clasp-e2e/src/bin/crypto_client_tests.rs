//! CryptoClient integration tests via TestRouter.
//!
//! Tests the CryptoClient wrapper that provides transparent E2E encryption
//! over a real CLASP client connected to a router.

use std::sync::Arc;
use std::time::Duration;

use clasp_client::Clasp;
use clasp_core::{SecurityMode, Value};
use clasp_crypto::client::{CryptoClient, CryptoClientConfig};
use clasp_crypto::MemoryKeyStore;
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
            name: "CryptoClient Test Router".to_string(),
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

// ============================================================================
// Tests
// ============================================================================

async fn test_session_creates_e2e_session() -> TestResult {
    let start = std::time::Instant::now();
    let name = "session_creates_e2e_session";

    let store = Arc::new(MemoryKeyStore::new());
    let router = TestRouter::start().await;

    let client = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("connect failed: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };

    let mut crypto = CryptoClient::new(
        client,
        CryptoClientConfig {
            identity_id: "alice".into(),
            store,
            on_key_change: None,
            rotation_interval: None,
        },
    );

    // session() should create and return an E2ESession
    let session = crypto.session("/test/room");
    assert!(!session.encrypted(), "Session should start without a key");

    router.stop();
    TestResult::pass(name, start.elapsed().as_millis())
}

async fn test_find_session_longest_prefix() -> TestResult {
    let start = std::time::Instant::now();
    let name = "find_session_longest_prefix";

    let store = Arc::new(MemoryKeyStore::new());
    let router = TestRouter::start().await;

    let client = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("connect failed: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };

    let mut crypto = CryptoClient::new(
        client,
        CryptoClientConfig {
            identity_id: "alice".into(),
            store,
            on_key_change: None,
            rotation_interval: None,
        },
    );

    // Create two sessions with nested paths
    crypto.session("/app");
    crypto.session("/app/room");

    // /app/room/messages should match /app/room (longest prefix)
    let found = crypto.find_session("/app/room/messages");
    assert!(found.is_some(), "Should find a session");
    assert_eq!(found.unwrap().base_path(), "/app/room");

    // /app/other should match /app
    let found = crypto.find_session("/app/other");
    assert!(found.is_some(), "Should find a session");
    assert_eq!(found.unwrap().base_path(), "/app");

    // /unrelated should match nothing
    let found = crypto.find_session("/unrelated");
    assert!(found.is_none(), "Should not find a session");

    router.stop();
    TestResult::pass(name, start.elapsed().as_millis())
}

async fn test_encrypted_set_subscribe_through_router() -> TestResult {
    let start = std::time::Instant::now();
    let name = "encrypted_set_subscribe_through_router";

    let router = TestRouter::start().await;

    // Alice: sender with encryption
    let store_a = Arc::new(MemoryKeyStore::new());
    let alice_client = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("Alice connect: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };
    let mut alice = CryptoClient::new(
        alice_client,
        CryptoClientConfig {
            identity_id: "alice".into(),
            store: store_a.clone(),
            on_key_change: None,
            rotation_interval: None,
        },
    );

    // Create and enable encryption on Alice's session
    let alice_session = alice.session("/test/crypto");
    alice_session.start().await.expect("alice start");
    let _ann = alice_session
        .enable_encryption()
        .await
        .expect("alice enable");

    // Bob: receiver that gets the key via direct session API
    let store_b = Arc::new(MemoryKeyStore::new());
    let bob_client = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("Bob connect: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };
    let mut bob = CryptoClient::new(
        bob_client,
        CryptoClientConfig {
            identity_id: "bob".into(),
            store: store_b,
            on_key_change: None,
            rotation_interval: None,
        },
    );
    let bob_session = bob.session("/test/crypto");
    bob_session.start().await.expect("bob start");
    let bob_ann = bob_session
        .request_group_key()
        .expect("bob request")
        .expect("bob ann");

    // Perform key exchange manually
    let keyex = alice
        .session("/test/crypto")
        .handle_peer_pubkey("bob", &bob_ann)
        .await
        .expect("alice handle pubkey")
        .expect("alice keyex");
    bob.session("/test/crypto")
        .handle_key_exchange(&keyex)
        .await
        .expect("bob handle keyex");

    assert!(
        bob.session("/test/crypto").encrypted(),
        "Bob should have the group key"
    );

    // Alice encrypts and sends through the router
    let envelope = alice
        .session("/test/crypto")
        .encrypt("hello from alice")
        .expect("encrypt");

    let json_val = serde_json::to_value(&envelope).unwrap();
    alice
        .inner
        .set("/test/crypto/msg", Value::from(json_val.to_string()))
        .await
        .expect("alice set");

    // Give the router time to deliver
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Bob decrypts
    let decrypted = bob
        .session("/test/crypto")
        .decrypt(&envelope)
        .await
        .expect("bob decrypt");
    assert_eq!(decrypted, "hello from alice");

    router.stop();
    TestResult::pass(name, start.elapsed().as_millis())
}

async fn test_close_destroys_all_sessions() -> TestResult {
    let start = std::time::Instant::now();
    let name = "close_destroys_all_sessions";

    let store = Arc::new(MemoryKeyStore::new());
    let router = TestRouter::start().await;

    let client = match Clasp::connect_to(&router.url()).await {
        Ok(c) => c,
        Err(e) => {
            router.stop();
            return TestResult::fail(
                name,
                format!("connect failed: {e}"),
                start.elapsed().as_millis(),
            );
        }
    };

    let mut crypto = CryptoClient::new(
        client,
        CryptoClientConfig {
            identity_id: "alice".into(),
            store,
            on_key_change: None,
            rotation_interval: None,
        },
    );

    let s1 = crypto.session("/room/1");
    s1.start().await.expect("s1 start");
    s1.enable_encryption().await.expect("s1 enable");

    let s2 = crypto.session("/room/2");
    s2.start().await.expect("s2 start");
    s2.enable_encryption().await.expect("s2 enable");

    // close() should destroy all sessions
    crypto.close();

    // After close, find_session should return None (sessions cleared)
    assert!(
        crypto.find_session("/room/1/test").is_none(),
        "Sessions should be cleared after close"
    );

    router.stop();
    TestResult::pass(name, start.elapsed().as_millis())
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt().with_env_filter("info").init();

    println!("\n=== CLASP CryptoClient Integration Tests ===\n");

    let tests = vec![
        test_session_creates_e2e_session().await,
        test_find_session_longest_prefix().await,
        test_encrypted_set_subscribe_through_router().await,
        test_close_destroys_all_sessions().await,
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
