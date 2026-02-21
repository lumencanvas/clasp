//! Federation security integration tests
//!
//! These tests exercise the full message-handling path for federation operations,
//! proving that namespace restrictions, resource limits, and scope enforcement
//! actually work end-to-end through the router.

#![cfg(all(feature = "websocket", feature = "federation"))]

use clasp_core::{
    codec, FederationOp, FederationSyncMessage, HelloMessage, Message,
};
use clasp_router::{Router, RouterConfig};
use clasp_transport::{Transport, TransportEvent, TransportReceiver, TransportSender, WebSocketTransport};
use std::collections::HashMap;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::time::timeout;

async fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

/// Receive the next data message from the transport, with timeout.
async fn recv_msg<R: TransportReceiver>(receiver: &mut R) -> Option<Message> {
    timeout(Duration::from_secs(2), async {
        loop {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    return msg;
                }
            }
        }
    })
    .await
    .ok()
}

/// Complete the HELLO/WELCOME/SNAPSHOT handshake for a federation peer.
async fn federation_handshake<S: TransportSender, R: TransportReceiver>(
    sender: &S,
    receiver: &mut R,
    name: &str,
) {
    let hello = Message::Hello(HelloMessage {
        version: 2,
        name: name.to_string(),
        features: vec!["param".to_string(), "federation".to_string()],
        capabilities: None,
        token: None,
    });
    sender.send(codec::encode(&hello).unwrap()).await.unwrap();

    // Drain WELCOME and SNAPSHOT
    let mut got_welcome = false;
    let mut got_snapshot = false;
    let deadline = timeout(Duration::from_secs(3), async {
        while !got_welcome || !got_snapshot {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    match msg {
                        Message::Welcome(_) => got_welcome = true,
                        Message::Snapshot(_) => got_snapshot = true,
                        _ => {}
                    }
                }
            }
        }
    })
    .await;
    assert!(deadline.is_ok(), "federation handshake timed out");
}

/// Complete a non-federation HELLO handshake.
async fn normal_handshake<S: TransportSender, R: TransportReceiver>(
    sender: &S,
    receiver: &mut R,
    name: &str,
) {
    let hello = Message::Hello(HelloMessage {
        version: 2,
        name: name.to_string(),
        features: vec!["param".to_string()],
        capabilities: None,
        token: None,
    });
    sender.send(codec::encode(&hello).unwrap()).await.unwrap();

    let mut got_welcome = false;
    let mut got_snapshot = false;
    let deadline = timeout(Duration::from_secs(3), async {
        while !got_welcome || !got_snapshot {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    match msg {
                        Message::Welcome(_) => got_welcome = true,
                        Message::Snapshot(_) => got_snapshot = true,
                        _ => {}
                    }
                }
            }
        }
    })
    .await;
    assert!(deadline.is_ok(), "normal handshake timed out");
}

async fn setup_router() -> (String, tokio::task::JoinHandle<()>) {
    let port = find_available_port().await;
    let addr = format!("127.0.0.1:{}", port);
    let router = Router::new(RouterConfig {
        features: vec!["param".to_string(), "federation".to_string()],
        ..Default::default()
    });

    let handle = {
        let addr = addr.clone();
        tokio::spawn(async move {
            let _ = router.serve_websocket(&addr).await;
        })
    };

    tokio::time::sleep(Duration::from_millis(100)).await;
    (format!("ws://{}", addr), handle)
}

/// Helper: declare namespaces and assert ACK
async fn declare_and_ack<S: TransportSender, R: TransportReceiver>(
    sender: &S,
    receiver: &mut R,
    patterns: Vec<String>,
) {
    let declare = Message::FederationSync(FederationSyncMessage {
        op: FederationOp::DeclareNamespaces,
        patterns,
        revisions: HashMap::new(),
        since_revision: None,
        origin: Some("peer".to_string()),
    });
    sender.send(codec::encode(&declare).unwrap()).await.unwrap();

    let response = recv_msg(receiver).await;
    assert!(response.is_some(), "should receive response for DeclareNamespaces");
    let msg = response.unwrap();
    assert!(
        matches!(msg, Message::Ack(_)),
        "expected ACK for DeclareNamespaces, got: {:?}", msg
    );
}

// ==========================================================================
// Test: Non-federation session cannot send FederationSync
// ==========================================================================

#[tokio::test]
async fn test_non_federation_session_rejected() {
    let (url, handle) = setup_router().await;
    let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
    normal_handshake(&sender, &mut receiver, "Normal Client").await;

    let fed_msg = Message::FederationSync(FederationSyncMessage {
        op: FederationOp::DeclareNamespaces,
        patterns: vec!["/sensors/**".to_string()],
        revisions: HashMap::new(),
        since_revision: None,
        origin: Some("rogue-peer".to_string()),
    });
    sender.send(codec::encode(&fed_msg).unwrap()).await.unwrap();

    let response = recv_msg(&mut receiver).await;
    assert!(response.is_some(), "should receive error response");
    match response.unwrap() {
        Message::Error(err) => {
            assert_eq!(err.code, 403);
            assert!(err.message.contains("federation"));
        }
        other => panic!("expected Error, got: {:?}", other),
    }

    handle.abort();
}

// ==========================================================================
// Test: DeclareNamespaces with too many patterns is rejected
// ==========================================================================

#[tokio::test]
async fn test_declare_namespaces_too_many_patterns() {
    let (url, handle) = setup_router().await;
    let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&sender, &mut receiver, "Peer").await;

    // 1001 patterns exceeds MAX_FEDERATION_PATTERNS = 1000
    let patterns: Vec<String> = (0..1001).map(|i| format!("/ns{}", i)).collect();

    let fed_msg = Message::FederationSync(FederationSyncMessage {
        op: FederationOp::DeclareNamespaces,
        patterns,
        revisions: HashMap::new(),
        since_revision: None,
        origin: Some("peer".to_string()),
    });
    sender.send(codec::encode(&fed_msg).unwrap()).await.unwrap();

    let response = recv_msg(&mut receiver).await;
    assert!(response.is_some(), "should receive error response");
    match response.unwrap() {
        Message::Error(err) => {
            assert_eq!(err.code, 400);
            assert!(err.message.contains("too many"));
        }
        other => panic!("expected Error(400), got: {:?}", other),
    }

    handle.abort();
}

// ==========================================================================
// Test: DeclareNamespaces + ACK happy path
// ==========================================================================

#[tokio::test]
async fn test_declare_namespaces_success() {
    let (url, handle) = setup_router().await;
    let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&sender, &mut receiver, "Peer").await;

    declare_and_ack(&sender, &mut receiver, vec!["/sensors/**".to_string()]).await;

    handle.abort();
}

// ==========================================================================
// Test: RequestSync outside declared namespaces is rejected
// ==========================================================================

#[tokio::test]
async fn test_request_sync_outside_declared_namespaces() {
    let (url, handle) = setup_router().await;
    let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&sender, &mut receiver, "Peer").await;

    // Declare only /sensors/**
    declare_and_ack(&sender, &mut receiver, vec!["/sensors/**".to_string()]).await;

    // Request sync for /audio/** — NOT in declared namespaces
    let sync = Message::FederationSync(FederationSyncMessage {
        op: FederationOp::RequestSync,
        patterns: vec!["/audio/**".to_string()],
        revisions: HashMap::new(),
        since_revision: None,
        origin: None,
    });
    sender.send(codec::encode(&sync).unwrap()).await.unwrap();

    let response = recv_msg(&mut receiver).await;
    assert!(response.is_some(), "should receive error response");
    match response.unwrap() {
        Message::Error(err) => {
            assert_eq!(err.code, 403);
            assert!(err.message.contains("not covered"));
        }
        other => panic!("expected Error(403), got: {:?}", other),
    }

    handle.abort();
}

// ==========================================================================
// Test: RequestSync within declared namespaces gets SyncComplete
// ==========================================================================

#[tokio::test]
async fn test_request_sync_within_declared_namespaces() {
    let (url, handle) = setup_router().await;
    let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&sender, &mut receiver, "Peer").await;

    declare_and_ack(&sender, &mut receiver, vec!["/sensors/**".to_string()]).await;

    // Request sync for /sensors/temp/** — within declared namespace
    let sync = Message::FederationSync(FederationSyncMessage {
        op: FederationOp::RequestSync,
        patterns: vec!["/sensors/temp/**".to_string()],
        revisions: HashMap::new(),
        since_revision: None,
        origin: None,
    });
    sender.send(codec::encode(&sync).unwrap()).await.unwrap();

    // Should receive SyncComplete (possibly preceded by empty Snapshot)
    let response = timeout(Duration::from_secs(3), async {
        loop {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    if let Message::FederationSync(ref fed) = msg {
                        if fed.op == FederationOp::SyncComplete {
                            return msg;
                        }
                    }
                    // Also accept Snapshot (empty state) but keep waiting for SyncComplete
                }
            }
        }
    })
    .await;

    assert!(response.is_ok(), "should receive SyncComplete for valid sync request");

    handle.abort();
}

// ==========================================================================
// Test: RevisionVector for addresses outside declared namespaces are silently skipped
// ==========================================================================

#[tokio::test]
async fn test_revision_vector_filters_out_of_scope_addresses() {
    let (url, handle) = setup_router().await;

    // Set some state via a normal client
    let (setter, mut setter_rx) = WebSocketTransport::connect(&url).await.unwrap();
    normal_handshake(&setter, &mut setter_rx, "Setter").await;

    let set = Message::Set(clasp_core::SetMessage {
        address: "/audio/mixer/volume".to_string(),
        value: clasp_core::Value::Float(0.8),
        revision: None,
        lock: false,
        unlock: false,
    });
    setter.send(codec::encode(&set).unwrap()).await.unwrap();
    // Wait for ACK to ensure state is written
    let ack = recv_msg(&mut setter_rx).await;
    assert!(matches!(ack, Some(Message::Ack(_))));

    // Federation peer declares only /sensors/**
    let (fed, mut fed_rx) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&fed, &mut fed_rx, "Fed Peer").await;
    declare_and_ack(&fed, &mut fed_rx, vec!["/sensors/**".to_string()]).await;

    // Send RevisionVector claiming peer has revision 0 for /audio/mixer/volume
    // Since /audio/** is NOT in declared namespaces, server should NOT send delta
    let mut revisions = HashMap::new();
    revisions.insert("/audio/mixer/volume".to_string(), 0u64);
    let rev_vec = Message::FederationSync(FederationSyncMessage {
        op: FederationOp::RevisionVector,
        patterns: vec![],
        revisions,
        since_revision: None,
        origin: None,
    });
    fed.send(codec::encode(&rev_vec).unwrap()).await.unwrap();

    // Wait briefly — server should NOT send any Snapshot data
    let snapshot = timeout(Duration::from_millis(500), async {
        loop {
            if let Some(TransportEvent::Data(data)) = fed_rx.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    if matches!(msg, Message::Snapshot(_)) {
                        return msg;
                    }
                }
            }
        }
    })
    .await;

    assert!(snapshot.is_err(), "should NOT receive snapshot for out-of-scope addresses");

    handle.abort();
}

// ==========================================================================
// Test: Re-declaring namespaces cleans up old subscriptions
// ==========================================================================

#[tokio::test]
async fn test_redeclare_namespaces_cleanup() {
    let (url, handle) = setup_router().await;

    // Federation peer
    let (fed, mut fed_rx) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&fed, &mut fed_rx, "Fed Peer").await;

    // Declare /sensors/**
    declare_and_ack(&fed, &mut fed_rx, vec!["/sensors/**".to_string()]).await;

    // Re-declare with /audio/** only (replaces /sensors/**)
    declare_and_ack(&fed, &mut fed_rx, vec!["/audio/**".to_string()]).await;

    // Normal client sets a value under /sensors/**
    let (setter, mut setter_rx) = WebSocketTransport::connect(&url).await.unwrap();
    normal_handshake(&setter, &mut setter_rx, "Setter").await;

    let set = Message::Set(clasp_core::SetMessage {
        address: "/sensors/temp/1".to_string(),
        value: clasp_core::Value::Float(22.5),
        revision: None,
        lock: false,
        unlock: false,
    });
    setter.send(codec::encode(&set).unwrap()).await.unwrap();
    let ack = recv_msg(&mut setter_rx).await;
    assert!(matches!(ack, Some(Message::Ack(_))));

    // Federation peer should NOT receive /sensors/temp/1 because the old
    // subscription was cleaned up during re-declare
    let stale = timeout(Duration::from_millis(500), async {
        loop {
            if let Some(TransportEvent::Data(data)) = fed_rx.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    if let Message::Set(s) = &msg {
                        if s.address.starts_with("/sensors/") {
                            return msg;
                        }
                    }
                }
            }
        }
    })
    .await;

    assert!(stale.is_err(), "should NOT receive /sensors data after re-declaring to /audio only");

    handle.abort();
}

// ==========================================================================
// Test: Federation peer DOES receive data for currently-declared namespace
// (proves subscriptions are working, not just that tests timeout)
// ==========================================================================

#[tokio::test]
async fn test_federation_peer_receives_declared_namespace_data() {
    let (url, handle) = setup_router().await;

    // Federation peer declares /sensors/**
    let (fed, mut fed_rx) = WebSocketTransport::connect(&url).await.unwrap();
    federation_handshake(&fed, &mut fed_rx, "Fed Peer").await;
    declare_and_ack(&fed, &mut fed_rx, vec!["/sensors/**".to_string()]).await;

    // Normal client sets a value under /sensors/**
    let (setter, mut setter_rx) = WebSocketTransport::connect(&url).await.unwrap();
    normal_handshake(&setter, &mut setter_rx, "Setter").await;

    let set = Message::Set(clasp_core::SetMessage {
        address: "/sensors/temp/1".to_string(),
        value: clasp_core::Value::Float(22.5),
        revision: None,
        lock: false,
        unlock: false,
    });
    setter.send(codec::encode(&set).unwrap()).await.unwrap();

    // Federation peer SHOULD receive this SET
    let received = timeout(Duration::from_secs(2), async {
        loop {
            if let Some(TransportEvent::Data(data)) = fed_rx.recv().await {
                if let Ok((msg, _)) = codec::decode(&data) {
                    if let Message::Set(ref s) = msg {
                        if s.address == "/sensors/temp/1" {
                            return msg;
                        }
                    }
                }
            }
        }
    })
    .await;

    assert!(received.is_ok(), "federation peer should receive data for declared namespace");

    handle.abort();
}
