use clasp_core::types::{SnapshotMessage, Value};
use clasp_router::{Router, RouterConfig, RouterStateConfig};
use std::collections::HashMap;

fn test_router() -> Router {
    let config = RouterConfig {
        name: "test".to_string(),
        security_mode: clasp_core::SecurityMode::Open,
        max_sessions: 10,
        session_timeout: 60,
        features: vec!["param".to_string()],
        max_subscriptions_per_session: 10,
        gesture_coalescing: false,
        gesture_coalesce_interval_ms: 16,
        max_messages_per_second: 0,
        rate_limiting_enabled: false,
        state_config: RouterStateConfig::unlimited(),
    };
    Router::new(config)
}

#[test]
fn snapshot_empty_state_produces_empty_params() {
    let router = test_router();
    let snapshot = router.state().full_snapshot();
    assert!(snapshot.params.is_empty());
}

#[test]
fn snapshot_contains_set_values() {
    let router = test_router();
    let writer = "writer-1".to_string();

    router
        .state()
        .set("/test/a", Value::String("hello".into()), &writer, None, false, false)
        .unwrap();
    router
        .state()
        .set("/test/b", Value::Int(42), &writer, None, false, false)
        .unwrap();

    let snapshot = router.state().full_snapshot();
    assert_eq!(snapshot.params.len(), 2);

    let a = snapshot.params.iter().find(|p| p.address == "/test/a").unwrap();
    assert_eq!(a.value, Value::String("hello".into()));

    let b = snapshot.params.iter().find(|p| p.address == "/test/b").unwrap();
    assert_eq!(b.value, Value::Int(42));
}

#[test]
fn snapshot_serializes_to_valid_json() {
    let router = test_router();
    let writer = "w".to_string();

    router
        .state()
        .set("/json/test", Value::String("data".into()), &writer, None, false, false)
        .unwrap();

    let snapshot = router.state().full_snapshot();
    let json = serde_json::to_string(&snapshot).unwrap();

    // Verify it parses back
    let parsed: SnapshotMessage = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.params.len(), 1);
    assert_eq!(parsed.params[0].address, "/json/test");
}

#[test]
fn snapshot_restore_into_fresh_router() {
    // Create and populate source router
    let source = test_router();
    let admin = "admin".to_string();
    let system = "system".to_string();

    source
        .state()
        .set("/room/1/name", Value::String("General".into()), &admin, None, false, false)
        .unwrap();

    let mut meta = HashMap::new();
    meta.insert("encrypted".to_string(), Value::Bool(true));
    source
        .state()
        .set("/room/1/meta", Value::Map(meta), &admin, None, false, false)
        .unwrap();

    source
        .state()
        .set("/user/count", Value::Int(5), &system, None, false, false)
        .unwrap();

    // Take snapshot and serialize
    let snapshot = source.state().full_snapshot();
    let json = serde_json::to_string(&snapshot).unwrap();

    // Restore into fresh router
    let dest = test_router();
    let restored: SnapshotMessage = serde_json::from_str(&json).unwrap();
    let restore_writer = "restore".to_string();

    for pv in &restored.params {
        dest.state()
            .set(&pv.address, pv.value.clone(), &restore_writer, Some(pv.revision), false, false)
            .unwrap();
    }

    // Verify values match
    let dest_snapshot = dest.state().full_snapshot();
    assert_eq!(dest_snapshot.params.len(), 3);

    let name = dest_snapshot.params.iter().find(|p| p.address == "/room/1/name").unwrap();
    assert_eq!(name.value, Value::String("General".into()));

    let meta = dest_snapshot.params.iter().find(|p| p.address == "/room/1/meta").unwrap();
    match &meta.value {
        Value::Map(m) => {
            assert_eq!(m.get("encrypted"), Some(&Value::Bool(true)));
        }
        _ => panic!("Expected Map value for meta"),
    }

    let count = dest_snapshot.params.iter().find(|p| p.address == "/user/count").unwrap();
    assert_eq!(count.value, Value::Int(5));
}
