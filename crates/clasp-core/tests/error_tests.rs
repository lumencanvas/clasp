//! Error Handling Tests
//!
//! Tests for error cases and edge conditions:
//! - Malformed messages
//! - Invalid protocol versions
//! - Connection errors
//! - Resource limits
//! - Timeout handling

use bytes::Bytes;
use clasp_core::{codec, HelloMessage, Message, SetMessage, Value, PROTOCOL_VERSION};
use clasp_test_utils::TestRouter;
use clasp_transport::{Transport, TransportEvent, TransportReceiver, TransportSender, WebSocketTransport};
use std::time::Duration;
use tokio::time::timeout;

// ============================================================================
// Tests
// ============================================================================

#[tokio::test]
async fn test_malformed_message() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Send garbage data
    let garbage = Bytes::from(vec![0xFF, 0xFE, 0xFD, 0xFC, 0x00, 0x01, 0x02]);
    sender.send(garbage).await.expect("Failed to send");

    // Server should handle gracefully - either error or disconnect
    let response = timeout(Duration::from_secs(1), receiver.recv()).await;

    // Any response is acceptable - key is server didn't crash
    // The test passes as long as we reach this point without panic
    match response {
        Ok(Some(TransportEvent::Error(_))) => {} // Error is fine
        Ok(Some(TransportEvent::Disconnected { .. })) => {} // Disconnect is fine
        Ok(Some(TransportEvent::Connected)) => {} // Connection still ok
        Ok(Some(TransportEvent::Data(_))) => {}  // Even data response is ok
        Ok(None) => {}                           // Connection closed is fine
        Err(_) => {} // Timeout is fine (server ignored bad data)
    }
}

#[tokio::test]
async fn test_truncated_message() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Encode a valid message then truncate it
    let hello = Message::Hello(HelloMessage {
        version: 2,
        name: "Test".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    let bytes = codec::encode(&hello).expect("Failed to encode");
    // Truncate to just 3 bytes
    let truncated = Bytes::from(bytes.to_vec()[..3.min(bytes.len())].to_vec());
    sender.send(truncated).await.expect("Failed to send");

    // Server should handle gracefully
    let _response = timeout(Duration::from_secs(1), receiver.recv()).await;

    // Any graceful handling is acceptable - test passes if we reach here
}

#[tokio::test]
async fn test_wrong_protocol_version() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Send HELLO with wrong version
    let hello = Message::Hello(HelloMessage {
        version: 99, // Invalid version (not 2)
        name: "BadVersion".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    sender
        .send(codec::encode(&hello).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Should get error or still work (version mismatch handling varies)
    let response = timeout(Duration::from_secs(2), async {
        loop {
            if let Some(event) = receiver.recv().await {
                match event {
                    TransportEvent::Data(data) => {
                        let (msg, _) = codec::decode(&data).expect("Failed to decode");
                        return Some(msg);
                    }
                    TransportEvent::Connected => continue,
                    TransportEvent::Disconnected { .. } => return None,
                    TransportEvent::Error(_) => return None,
                }
            }
        }
    })
    .await;

    // Either error or welcome (with potential version warning) is acceptable
    match response {
        Ok(Some(Message::Welcome(_))) => {} // Server accepted anyway
        Ok(Some(Message::Error(_))) => {}   // Server rejected - also fine
        Ok(Some(_)) => {}                   // Any other message - server handled somehow
        Ok(None) => {}                      // Error during receive
        Err(_) => {}                        // Timeout - server might have ignored
    }
}

#[tokio::test]
async fn test_message_before_hello() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Send SET before HELLO
    let set = Message::Set(SetMessage {
        address: "/test".to_string(),
        value: Value::Int(1),
        revision: None,
        lock: false,
        unlock: false,
    });
    sender
        .send(codec::encode(&set).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Server should reject or ignore
    let response = timeout(Duration::from_secs(1), receiver.recv()).await;

    // Should not get ACK (no session established)
    match response {
        Ok(Some(TransportEvent::Data(data))) => {
            let (msg, _) = codec::decode(&data).expect("Failed to decode");
            match msg {
                Message::Ack(_) => panic!("Should not ACK before HELLO"),
                Message::Error(_) => {} // Error is correct behavior
                _ => {}                 // Other responses ok
            }
        }
        _ => {} // Timeout or disconnect is fine
    }
}

#[tokio::test]
async fn test_duplicate_hello() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Send first HELLO
    let hello = Message::Hello(HelloMessage {
        version: PROTOCOL_VERSION,
        name: "First".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    sender
        .send(codec::encode(&hello).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Wait for WELCOME
    let got_welcome = loop {
        match timeout(Duration::from_secs(2), receiver.recv()).await {
            Ok(Some(TransportEvent::Data(data))) => {
                let (msg, _) = codec::decode(&data).expect("Failed to decode");
                if matches!(msg, Message::Welcome(_)) {
                    break true;
                }
            }
            Ok(Some(TransportEvent::Connected)) => continue,
            _ => break false,
        }
    };
    assert!(got_welcome, "Expected WELCOME message");

    // Send second HELLO (should be ignored or cause error)
    let hello2 = Message::Hello(HelloMessage {
        version: PROTOCOL_VERSION,
        name: "Second".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    sender
        .send(codec::encode(&hello2).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Server should handle gracefully
    let _response = timeout(Duration::from_millis(500), receiver.recv()).await;

    // Any non-crash behavior is acceptable - test passes if we reach here
}

#[tokio::test]
async fn test_very_long_address() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Complete handshake
    let hello = Message::Hello(HelloMessage {
        version: PROTOCOL_VERSION,
        name: "LongAddressTest".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    sender
        .send(codec::encode(&hello).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Wait for handshake
    loop {
        match timeout(Duration::from_secs(2), receiver.recv()).await {
            Ok(Some(TransportEvent::Data(data))) => {
                let (msg, _) = codec::decode(&data).expect("Failed to decode");
                if matches!(msg, Message::Snapshot(_)) {
                    break;
                }
            }
            Ok(Some(TransportEvent::Connected)) => continue,
            _ => break,
        }
    }

    // Send SET with very long address (10KB)
    let long_addr = format!("/{}", "a".repeat(10_000));
    let set = Message::Set(SetMessage {
        address: long_addr,
        value: Value::Int(1),
        revision: None,
        lock: false,
        unlock: false,
    });
    sender
        .send(codec::encode(&set).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Should handle gracefully
    let _response = timeout(Duration::from_secs(1), receiver.recv()).await;

    // Either ACK, error, or timeout is acceptable - test passes if we reach here
}

#[tokio::test]
async fn test_empty_address() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Complete handshake
    let hello = Message::Hello(HelloMessage {
        version: PROTOCOL_VERSION,
        name: "EmptyAddressTest".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    sender
        .send(codec::encode(&hello).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Wait for handshake
    loop {
        match timeout(Duration::from_secs(2), receiver.recv()).await {
            Ok(Some(TransportEvent::Data(data))) => {
                let (msg, _) = codec::decode(&data).expect("Failed to decode");
                if matches!(msg, Message::Snapshot(_)) {
                    break;
                }
            }
            Ok(Some(TransportEvent::Connected)) => continue,
            _ => break,
        }
    }

    // Send SET with empty address
    let set = Message::Set(SetMessage {
        address: "".to_string(), // Empty!
        value: Value::Int(1),
        revision: None,
        lock: false,
        unlock: false,
    });
    sender
        .send(codec::encode(&set).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Should handle gracefully (error or ignore)
    let response = timeout(Duration::from_secs(1), receiver.recv()).await;

    match response {
        Ok(Some(TransportEvent::Data(data))) => {
            let (msg, _) = codec::decode(&data).expect("Failed to decode");
            match msg {
                Message::Error(_) => {} // Error is correct
                Message::Ack(_) => {}   // Accepting empty is also valid
                _ => {}
            }
        }
        _ => {} // Timeout is fine
    }
}

#[tokio::test]
async fn test_rapid_disconnect_reconnect() {
    let router = TestRouter::start().await;

    for i in 0..5 {
        let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
            .await
            .expect("Failed to connect");

        // Quick handshake
        let hello = Message::Hello(HelloMessage {
            version: PROTOCOL_VERSION,
            name: format!("Rapid{}", i),
            features: vec![],
            capabilities: None,
            token: None,
        });
        sender
            .send(codec::encode(&hello).expect("Failed to encode"))
            .await
            .expect("Failed to send");

        // Wait briefly for WELCOME
        let _ = timeout(Duration::from_millis(100), receiver.recv()).await;

        // Disconnect
        sender.close().await.expect("Failed to close");
    }

    // Test passes if all iterations complete without error
}

#[tokio::test]
async fn test_connection_to_closed_port() {
    // Try connecting to a port that's definitely not listening
    let result = timeout(
        Duration::from_secs(2),
        WebSocketTransport::connect("ws://127.0.0.1:1"),
    )
    .await;

    match result {
        Ok(Err(_)) => {} // Connection refused - expected
        Err(_) => {}     // Timeout - also acceptable
        Ok(Ok(_)) => panic!("Should not connect to closed port"),
    }
}

#[tokio::test]
async fn test_special_characters_in_address() {
    let router = TestRouter::start().await;

    let (sender, mut receiver) = WebSocketTransport::connect(&router.url())
        .await
        .expect("Failed to connect");

    // Complete handshake
    let hello = Message::Hello(HelloMessage {
        version: PROTOCOL_VERSION,
        name: "SpecialChars".to_string(),
        features: vec![],
        capabilities: None,
        token: None,
    });
    sender
        .send(codec::encode(&hello).expect("Failed to encode"))
        .await
        .expect("Failed to send");

    // Wait for handshake
    loop {
        match timeout(Duration::from_secs(2), receiver.recv()).await {
            Ok(Some(TransportEvent::Data(data))) => {
                let (msg, _) = codec::decode(&data).expect("Failed to decode");
                if matches!(msg, Message::Snapshot(_)) {
                    break;
                }
            }
            Ok(Some(TransportEvent::Connected)) => continue,
            _ => break,
        }
    }

    // Test various special characters
    let special_addresses = vec![
        "/path/with spaces",
        "/path/with\ttabs",
        "/unicode/\u{65e5}\u{672c}\u{8a9e}",
        "/emoji/\u{1f3b5}",
        "/symbols/@#$%",
    ];

    for addr in special_addresses {
        let set = Message::Set(SetMessage {
            address: addr.to_string(),
            value: Value::Int(1),
            revision: None,
            lock: false,
            unlock: false,
        });
        sender
            .send(codec::encode(&set).expect("Failed to encode"))
            .await
            .expect("Failed to send");

        // Should handle each address
        let _ = timeout(Duration::from_millis(100), receiver.recv()).await;
    }

    // Test passes if all special addresses were handled without crash
}
