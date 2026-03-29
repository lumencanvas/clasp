//! Integration tests for SignalTransform.
//!
//! Transforms run in the SET message handler (handlers/set.rs), NOT in state.set().
//! These tests verify the full message pipeline: client sends SET, router applies
//! transform, value in state reflects the transformation, client reads it back via GET.

#[cfg(feature = "websocket")]
mod websocket_transform_tests {
    use clasp_core::{
        codec, GetMessage, HelloMessage, Message, SecurityMode, SetMessage, Value,
    };
    use clasp_router::{Router, RouterConfig, SignalTransform};
    use std::sync::Arc;
    use std::time::Duration;
    use tokio::net::TcpListener;
    use tokio::time::timeout;

    use clasp_transport::{
        Transport, TransportEvent, TransportReceiver, TransportSender, WebSocketTransport,
    };

    /// Find an available port for testing.
    async fn find_available_port() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        listener.local_addr().unwrap().port()
    }

    /// Complete the CLASP handshake (HELLO -> WELCOME + SNAPSHOT).
    async fn complete_handshake<S: TransportSender, R: TransportReceiver>(
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

        // Drain WELCOME
        loop {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                let (msg, _) = codec::decode(&data).unwrap();
                if matches!(msg, Message::Welcome(_)) {
                    break;
                }
            }
        }
        // Drain initial SNAPSHOT
        loop {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                let (msg, _) = codec::decode(&data).unwrap();
                if matches!(msg, Message::Snapshot(_)) {
                    break;
                }
            }
        }
    }

    /// Read the next data message, skipping Connected events.
    async fn recv_msg<R: TransportReceiver>(receiver: &mut R) -> Message {
        loop {
            if let Some(TransportEvent::Data(data)) = receiver.recv().await {
                let (msg, _) = codec::decode(&data).unwrap();
                return msg;
            }
        }
    }

    // -- Test transforms --

    /// Doubles all Float values at any address.
    struct DoubleAllFloats;

    impl SignalTransform for DoubleAllFloats {
        fn transform(&self, _address: &str, value: &Value) -> Option<Value> {
            match value {
                Value::Float(f) => Some(Value::Float(f * 2.0)),
                _ => None,
            }
        }
    }

    /// Doubles Float values only for /sensors/** addresses.
    struct DoubleSensors;

    impl SignalTransform for DoubleSensors {
        fn transform(&self, address: &str, value: &Value) -> Option<Value> {
            if clasp_core::address::glob_match("/sensors/**", address) {
                match value {
                    Value::Float(f) => Some(Value::Float(f * 2.0)),
                    _ => None,
                }
            } else {
                None
            }
        }
    }

    /// Clamps Float values to [0.0, 100.0].
    struct ClampTransform;

    impl SignalTransform for ClampTransform {
        fn transform(&self, _address: &str, value: &Value) -> Option<Value> {
            match value {
                Value::Float(f) => {
                    let clamped = f.clamp(0.0, 100.0);
                    if (clamped - f).abs() > f64::EPSILON {
                        Some(Value::Float(clamped))
                    } else {
                        None
                    }
                }
                _ => None,
            }
        }
    }

    /// Helper: build a router with a transform and start it on a random port.
    /// Returns (url, router_handle).
    async fn start_router_with_transform(
        transform: Arc<dyn SignalTransform>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let port = find_available_port().await;
        let addr = format!("127.0.0.1:{}", port);
        let url = format!("ws://{}", addr);

        let config = RouterConfig {
            name: "transform-test".to_string(),
            security_mode: SecurityMode::Open,
            ..Default::default()
        };
        let router = Router::new(config).with_transforms(transform);

        let handle = tokio::spawn(async move {
            let _ = router.serve_websocket(&addr).await;
        });

        tokio::time::sleep(Duration::from_millis(100)).await;
        (url, handle)
    }

    /// Helper: send a SET message and wait for the ACK.
    async fn send_set_and_ack<S: TransportSender, R: TransportReceiver>(
        sender: &S,
        receiver: &mut R,
        address: &str,
        value: Value,
    ) {
        let set = Message::Set(SetMessage {
            address: address.to_string(),
            value,
            revision: None,
            lock: false,
            unlock: false,
            ttl: None,
        });
        sender.send(codec::encode(&set).unwrap()).await.unwrap();

        // Wait for ACK
        let result = timeout(Duration::from_secs(2), async {
            loop {
                let msg = recv_msg(receiver).await;
                if matches!(msg, Message::Ack(_)) {
                    return msg;
                }
            }
        })
        .await;
        assert!(result.is_ok(), "Should receive ACK for SET");
    }

    /// Helper: send GET and return the value from the SNAPSHOT response.
    async fn get_value<S: TransportSender, R: TransportReceiver>(
        sender: &S,
        receiver: &mut R,
        address: &str,
    ) -> Option<Value> {
        let get = Message::Get(GetMessage {
            address: address.to_string(),
        });
        sender.send(codec::encode(&get).unwrap()).await.unwrap();

        let result = timeout(Duration::from_secs(2), async {
            loop {
                let msg = recv_msg(receiver).await;
                if let Message::Snapshot(snap) = msg {
                    return snap;
                }
            }
        })
        .await;

        match result {
            Ok(snap) => snap.params.first().map(|p| p.value.clone()),
            Err(_) => panic!("Timed out waiting for SNAPSHOT response to GET"),
        }
    }

    #[tokio::test]
    async fn transform_doubles_float_value_end_to_end() {
        let (url, handle) =
            start_router_with_transform(Arc::new(DoubleAllFloats)).await;

        let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
        complete_handshake(&sender, &mut receiver, "test-client").await;

        // SET 22.5 -- the DoubleAllFloats transform should store 45.0
        send_set_and_ack(&sender, &mut receiver, "/test/value", Value::Float(22.5)).await;

        // GET should return the transformed value
        let stored = get_value(&sender, &mut receiver, "/test/value").await;
        assert_eq!(stored, Some(Value::Float(45.0)));

        handle.abort();
    }

    #[tokio::test]
    async fn transform_passes_through_non_float() {
        let (url, handle) =
            start_router_with_transform(Arc::new(DoubleAllFloats)).await;

        let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
        complete_handshake(&sender, &mut receiver, "test-client").await;

        // SET an Int -- DoubleAllFloats returns None for non-floats, so passthrough
        send_set_and_ack(&sender, &mut receiver, "/test/count", Value::Int(7)).await;

        let stored = get_value(&sender, &mut receiver, "/test/count").await;
        assert_eq!(stored, Some(Value::Int(7)));

        handle.abort();
    }

    #[tokio::test]
    async fn transform_pattern_matching_only_sensors() {
        let (url, handle) =
            start_router_with_transform(Arc::new(DoubleSensors)).await;

        let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
        complete_handshake(&sender, &mut receiver, "test-client").await;

        // SET to /sensors/temp -- should be doubled
        send_set_and_ack(
            &sender,
            &mut receiver,
            "/sensors/temp",
            Value::Float(10.0),
        )
        .await;
        let sensor_val = get_value(&sender, &mut receiver, "/sensors/temp").await;
        assert_eq!(sensor_val, Some(Value::Float(20.0)));

        // SET to /lights/dim -- should NOT be doubled
        send_set_and_ack(
            &sender,
            &mut receiver,
            "/lights/dim",
            Value::Float(10.0),
        )
        .await;
        let lights_val = get_value(&sender, &mut receiver, "/lights/dim").await;
        assert_eq!(lights_val, Some(Value::Float(10.0)));

        handle.abort();
    }

    #[tokio::test]
    async fn transform_clamps_out_of_range_value() {
        let (url, handle) =
            start_router_with_transform(Arc::new(ClampTransform)).await;

        let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
        complete_handshake(&sender, &mut receiver, "test-client").await;

        // SET 150.0 -- should be clamped to 100.0
        send_set_and_ack(&sender, &mut receiver, "/vol", Value::Float(150.0)).await;
        let val = get_value(&sender, &mut receiver, "/vol").await;
        assert_eq!(val, Some(Value::Float(100.0)));

        // SET -50.0 -- should be clamped to 0.0
        send_set_and_ack(&sender, &mut receiver, "/vol", Value::Float(-50.0)).await;
        let val = get_value(&sender, &mut receiver, "/vol").await;
        assert_eq!(val, Some(Value::Float(0.0)));

        // SET 42.0 -- in range, should pass through unchanged
        send_set_and_ack(&sender, &mut receiver, "/vol", Value::Float(42.0)).await;
        let val = get_value(&sender, &mut receiver, "/vol").await;
        assert_eq!(val, Some(Value::Float(42.0)));

        handle.abort();
    }

    #[tokio::test]
    async fn no_transform_passes_value_unchanged() {
        // Baseline: router without any transform
        let port = find_available_port().await;
        let addr = format!("127.0.0.1:{}", port);
        let url = format!("ws://{}", addr);

        let router = Router::default();
        let handle = tokio::spawn(async move {
            let _ = router.serve_websocket(&addr).await;
        });
        tokio::time::sleep(Duration::from_millis(100)).await;

        let (sender, mut receiver) = WebSocketTransport::connect(&url).await.unwrap();
        complete_handshake(&sender, &mut receiver, "test-client").await;

        send_set_and_ack(&sender, &mut receiver, "/test/raw", Value::Float(22.5)).await;
        let stored = get_value(&sender, &mut receiver, "/test/raw").await;
        assert_eq!(stored, Some(Value::Float(22.5)));

        handle.abort();
    }
}
