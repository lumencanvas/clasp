//! Lock and Conflict Resolution Tests
//!
//! Tests for:
//! - Lock acquisition and denial for non-owners
//! - Basic last-write-wins (LWW) behavior

use clasp_core::Value;
use clasp_test_utils::{TestRouter, ValueCollector};
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_lock_acquisition_and_denial() {
    let router = TestRouter::start().await;

    // Watcher to observe final state
    let watcher = router
        .connect_client_named("Watcher")
        .await
        .expect("Watcher should connect");

    let collector = ValueCollector::new();
    watcher
        .subscribe("/locks/value", collector.callback_ref())
        .await
        .expect("Subscribe should succeed");

    // Owner takes lock and sets initial value
    let owner = router
        .connect_client_named("Owner")
        .await
        .expect("Owner should connect");

    owner
        .set_locked("/locks/value", Value::Int(1))
        .await
        .expect("set_locked should succeed");

    // Wait for initial value to arrive
    assert!(
        collector.wait_for_count(1, Duration::from_secs(2)).await,
        "Should receive initial locked value"
    );

    // Another client attempts to overwrite while locked
    let other = router
        .connect_client_named("Other")
        .await
        .expect("Other should connect");

    other
        .set("/locks/value", Value::Int(2))
        .await
        .expect("set should succeed (sent, but may be rejected)");

    // Give router time to process
    sleep(Duration::from_millis(200)).await;

    let values = collector.values();
    assert!(!values.is_empty(), "Should have observed values on /locks/value");

    let (_, last_val) = values.last().unwrap();
    match last_val {
        Value::Int(v) => assert_eq!(*v, 1, "Locked value should not be modified by non-owner"),
        _ => panic!("Unexpected value type for locked param"),
    }
}

#[tokio::test]
async fn test_lww_last_write_wins() {
    let router = TestRouter::start().await;

    let writer1 = router
        .connect_client_named("Writer1")
        .await
        .expect("Writer1 should connect");
    let writer2 = router
        .connect_client_named("Writer2")
        .await
        .expect("Writer2 should connect");

    // First write
    writer1
        .set("/lww/value", Value::Int(1))
        .await
        .expect("Writer1 set should succeed");

    // Slight delay to ensure different timestamps
    sleep(Duration::from_millis(50)).await;

    // Second write should win (LWW)
    writer2
        .set("/lww/value", Value::Int(2))
        .await
        .expect("Writer2 set should succeed");

    // Reader checks final value
    let reader = router
        .connect_client_named("Reader")
        .await
        .expect("Reader should connect");

    // Small wait to allow state to settle
    sleep(Duration::from_millis(100)).await;

    let value = reader
        .get("/lww/value")
        .await
        .expect("Reader get should succeed");

    match value {
        Value::Int(v) => assert_eq!(v, 2, "LWW: final value should be from last writer"),
        _ => panic!("Unexpected value type for LWW param"),
    }
}
