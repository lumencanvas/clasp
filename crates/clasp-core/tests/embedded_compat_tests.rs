//! Cross-Implementation Compatibility Tests
//!
//! These tests verify that clasp-embedded and clasp-core can interoperate correctly.
//! Messages encoded by one implementation should be decodable by the other.

use clasp_core::{codec, frame::FrameFlags, Frame, Message, SetMessage, Value};
use clasp_embedded as embedded;

/// Test that embedded SET messages can be decoded by core
#[test]
fn test_embedded_set_to_core_decode() {
    let mut buf = [0u8; 128];

    // Encode a SET message using embedded
    let len = embedded::encode_set_frame(&mut buf, "/test/value", &embedded::Value::Float(1.25));
    assert!(len > 0);

    // Decode using core
    let result = codec::decode(&buf[..len]);
    assert!(
        result.is_ok(),
        "Failed to decode embedded SET: {:?}",
        result
    );

    let (msg, _frame) = result.unwrap();
    match msg {
        Message::Set(set_msg) => {
            assert_eq!(set_msg.address, "/test/value");
            match set_msg.value {
                Value::Float(f) => assert!((f - 1.25).abs() < 0.001),
                _ => panic!("Expected Float value, got {:?}", set_msg.value),
            }
        }
        _ => panic!("Expected SET message, got {:?}", msg),
    }
}

/// Test that core SET messages can be decoded by embedded
#[test]
fn test_core_set_to_embedded_decode() {
    // Create a SET message using core
    let set_msg = SetMessage {
        address: "/sensor/temp".to_string(),
        value: Value::Float(25.5),
        revision: None,
        lock: false,
        unlock: false,
    };

    // Encode using core
    let frame = Frame {
        flags: FrameFlags::default(),
        timestamp: None,
        payload: codec::encode_message(&Message::Set(set_msg)).unwrap(),
    };
    let encoded = frame.encode().unwrap();

    // Decode using embedded
    let (_flags, payload_len) = embedded::decode_header(&encoded).expect("Failed to decode header");
    assert!(payload_len > 0);

    let payload = &encoded[embedded::HEADER_SIZE..embedded::HEADER_SIZE + payload_len];
    let msg = embedded::decode_message(payload).expect("Failed to decode message");

    match msg {
        embedded::Message::Set { address, value } => {
            assert_eq!(address, "/sensor/temp");
            assert!((value.as_float().unwrap() - 25.5).abs() < 0.001);
        }
        _ => panic!("Expected Set message, got {:?}", msg),
    }
}

/// Test all basic value types roundtrip: embedded -> core -> embedded
#[test]
fn test_value_types_roundtrip() {
    let test_cases = [
        ("/test/null", embedded::Value::Null),
        ("/test/bool_true", embedded::Value::Bool(true)),
        ("/test/bool_false", embedded::Value::Bool(false)),
        ("/test/int_pos", embedded::Value::Int(42)),
        ("/test/int_neg", embedded::Value::Int(-999)),
        ("/test/int_large", embedded::Value::Int(i64::MAX)),
        ("/test/float_pos", embedded::Value::Float(1.2345)),
        ("/test/float_neg", embedded::Value::Float(-273.15)),
        ("/test/float_zero", embedded::Value::Float(0.0)),
    ];

    for (address, embedded_value) in test_cases {
        // Encode with embedded
        let mut buf = [0u8; 128];
        let len = embedded::encode_set_frame(&mut buf, address, &embedded_value);
        assert!(len > 0, "Failed to encode {}", address);

        // Decode with core
        let (msg, _) = codec::decode(&buf[..len])
            .unwrap_or_else(|_| panic!("Core failed to decode {}", address));

        // Verify the message
        match msg {
            Message::Set(set_msg) => {
                assert_eq!(set_msg.address, address);

                // Verify value matches
                match (&embedded_value, &set_msg.value) {
                    (embedded::Value::Null, Value::Null) => {}
                    (embedded::Value::Bool(e), Value::Bool(c)) => assert_eq!(*e, *c),
                    (embedded::Value::Int(e), Value::Int(c)) => assert_eq!(*e, *c),
                    (embedded::Value::Float(e), Value::Float(c)) => {
                        assert!(
                            (e - c).abs() < 0.0001,
                            "Float mismatch for {}: {} vs {}",
                            address,
                            e,
                            c
                        );
                    }
                    _ => panic!(
                        "Value type mismatch for {}: {:?} vs {:?}",
                        address, embedded_value, set_msg.value
                    ),
                }
            }
            _ => panic!("Expected SET message for {}", address),
        }
    }
}

/// Test HELLO/WELCOME handshake compatibility
#[test]
fn test_hello_welcome_handshake() {
    // Embedded sends HELLO
    let mut hello_buf = [0u8; 64];
    let hello_len = embedded::encode_hello_frame(&mut hello_buf, "ESP32-Device");
    assert!(hello_len > 0);

    // Core decodes HELLO
    let (msg, _) = codec::decode(&hello_buf[..hello_len]).expect("Core failed to decode HELLO");
    match msg {
        Message::Hello(hello) => {
            assert_eq!(hello.name, "ESP32-Device");
            assert_eq!(hello.version, embedded::VERSION);
        }
        _ => panic!("Expected HELLO message, got {:?}", msg),
    }
}

/// Test PING/PONG compatibility
#[test]
fn test_ping_pong_compatibility() {
    // Embedded sends PING
    let mut ping_buf = [0u8; 16];
    let ping_len = embedded::encode_ping_frame(&mut ping_buf);
    assert!(ping_len > 0);

    // Core decodes PING
    let (msg, _) = codec::decode(&ping_buf[..ping_len]).expect("Core failed to decode PING");
    match msg {
        Message::Ping => {}
        _ => panic!("Expected PING message, got {:?}", msg),
    }

    // Embedded sends PONG
    let mut pong_buf = [0u8; 16];
    let pong_len = embedded::encode_pong_frame(&mut pong_buf);
    assert!(pong_len > 0);

    // Core decodes PONG
    let (msg, _) = codec::decode(&pong_buf[..pong_len]).expect("Core failed to decode PONG");
    match msg {
        Message::Pong => {}
        _ => panic!("Expected PONG message, got {:?}", msg),
    }
}

/// Test SUBSCRIBE message compatibility
#[test]
fn test_subscribe_compatibility() {
    let mut buf = [0u8; 64];
    let len = embedded::encode_subscribe_frame(&mut buf, "/lumen/scene/*/layer/**");
    assert!(len > 0);

    let (msg, _) = codec::decode(&buf[..len]).expect("Core failed to decode SUBSCRIBE");
    match msg {
        Message::Subscribe(sub) => {
            assert_eq!(sub.pattern, "/lumen/scene/*/layer/**");
        }
        _ => panic!("Expected SUBSCRIBE message, got {:?}", msg),
    }
}

/// Test frame header encoding compatibility
#[test]
fn test_frame_header_compatibility() {
    // Embedded frame header
    let mut buf = [0u8; 8];
    let written = embedded::encode_header(&mut buf, 0, 100);
    assert_eq!(written, embedded::HEADER_SIZE);

    // Verify magic byte
    assert_eq!(buf[0], 0x53); // 'S'

    // Verify payload length
    let (flags, payload_len) = embedded::decode_header(&buf).unwrap();
    assert_eq!(payload_len, 100);
    assert_eq!(flags, embedded::FLAGS_BINARY);
}

/// Test that embedded can handle unknown message types gracefully
#[test]
fn test_unknown_message_handling() {
    // Create a message with an unknown type code
    let payload = [0xFF, 0x01, 0x02, 0x03]; // Unknown message type 0xFF

    let msg = embedded::decode_message(&payload);
    match msg {
        Some(embedded::Message::Unknown(0xFF)) => {}
        _ => panic!("Expected Unknown(0xFF), got {:?}", msg),
    }
}

/// Test large address strings
#[test]
fn test_long_address_compatibility() {
    let long_address = "/very/long/path/with/many/segments/to/test/encoding/and/decoding/behavior";

    let mut buf = [0u8; 256];
    let len = embedded::encode_set_frame(&mut buf, long_address, &embedded::Value::Int(1));
    assert!(len > 0);

    let (msg, _) = codec::decode(&buf[..len]).expect("Failed to decode long address");
    match msg {
        Message::Set(set_msg) => {
            assert_eq!(set_msg.address, long_address);
        }
        _ => panic!("Expected SET message"),
    }
}

/// Test edge case values
#[test]
fn test_edge_case_values() {
    let test_cases = [
        ("max_int", embedded::Value::Int(i64::MAX)),
        ("min_int", embedded::Value::Int(i64::MIN)),
        ("pos_infinity", embedded::Value::Float(f64::INFINITY)),
        ("neg_infinity", embedded::Value::Float(f64::NEG_INFINITY)),
        ("epsilon", embedded::Value::Float(f64::EPSILON)),
    ];

    for (name, value) in test_cases {
        let address = format!("/edge/{}", name);
        let mut buf = [0u8; 128];
        let len = embedded::encode_set_frame(&mut buf, &address, &value);

        if len > 0 {
            let result = codec::decode(&buf[..len]);
            match result {
                Ok((Message::Set(set_msg), _)) => {
                    assert_eq!(set_msg.address, address);
                }
                Ok(other) => panic!("Unexpected message for {}: {:?}", name, other),
                Err(e) => panic!("Decode failed for {}: {:?}", name, e),
            }
        }
    }
}
