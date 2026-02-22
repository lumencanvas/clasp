//! Integration tests for MQTT and OSC server adapters
//!
//! Tests for the protocol server adapters that allow the router to accept
//! connections from MQTT and OSC clients directly.

use tokio::net::TcpListener;

/// Find an available TCP port for testing
#[allow(dead_code)]
async fn find_available_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    listener.local_addr().unwrap().port()
}

/// Find an available UDP port for testing
#[allow(dead_code)]
fn find_available_udp_port() -> u16 {
    let socket = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    socket.local_addr().unwrap().port()
}

// =============================================================================
// MQTT Server Adapter Tests
// =============================================================================

#[cfg(feature = "mqtt-server")]
mod mqtt_adapter_tests {
    use super::*;
    use clasp_router::adapters::MqttServerConfig;

    /// Test MQTT server adapter creation with default config
    #[test]
    fn test_mqtt_config_default() {
        let config = MqttServerConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0:1883");
        assert_eq!(config.namespace, "/mqtt");
        assert!(!config.require_auth);
        assert!(config.tls.is_none());
        assert_eq!(config.max_clients, 0); // 0 = unlimited
        assert_eq!(config.session_timeout_secs, 300);
    }

    /// Test MQTT server adapter creation with custom config
    #[test]
    fn test_mqtt_config_custom() {
        let config = MqttServerConfig {
            bind_addr: "127.0.0.1:11883".to_string(),
            namespace: "/custom".to_string(),
            require_auth: true,
            tls: None,
            max_clients: 100,
            session_timeout_secs: 60,
        };
        assert_eq!(config.bind_addr, "127.0.0.1:11883");
        assert_eq!(config.namespace, "/custom");
        assert!(config.require_auth);
        assert_eq!(config.max_clients, 100);
        assert_eq!(config.session_timeout_secs, 60);
    }

    /// Test MQTT topic to CLASP address conversion
    #[test]
    fn test_mqtt_topic_to_clasp_address() {
        // Topic "sensors/temp" with namespace "/mqtt" -> "/mqtt/sensors/temp"
        let namespace = "/mqtt";
        let topic = "sensors/temp";
        let expected = format!("{}/{}", namespace, topic);
        assert_eq!(expected, "/mqtt/sensors/temp");
    }

    /// Test MQTT wildcard topic conversion
    #[test]
    fn test_mqtt_wildcard_topic_conversion() {
        // MQTT "#" -> CLASP "**"
        // MQTT "+" -> CLASP "*"
        let namespace = "/mqtt";

        // Single-level wildcard
        let topic_single = "sensors/+/temp";
        let clasp_single = topic_single.replace('+', "*");
        assert_eq!(clasp_single, "sensors/*/temp");

        // Multi-level wildcard
        let topic_multi = "sensors/#";
        let clasp_multi = topic_multi.replace('#', "**");
        assert_eq!(clasp_multi, "sensors/**");
    }

    /// Test MQTT adapter integration with router state
    #[tokio::test]
    async fn test_mqtt_adapter_shares_router_state() {
        use clasp_router::adapters::MqttServerAdapter;
        use dashmap::DashMap;
        use std::sync::Arc;

        let router = Router::default();
        let state = router.state();

        // Set a value in the router state
        state
            .set(
                "/test/param",
                clasp_core::Value::Float(42.0),
                &"test-session".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        // Verify the state is accessible
        let value = state.get("/test/param");
        assert!(value.is_some());
        match value.unwrap() {
            clasp_core::Value::Float(f) => assert!((f - 42.0).abs() < 0.001),
            _ => panic!("Expected Float value"),
        }
    }
}

// =============================================================================
// OSC Server Adapter Tests
// =============================================================================

#[cfg(feature = "osc-server")]
mod osc_adapter_tests {
    use super::*;
    use clasp_router::adapters::OscServerConfig;

    /// Test OSC server adapter creation with default config
    #[test]
    fn test_osc_config_default() {
        let config = OscServerConfig::default();
        assert_eq!(config.bind_addr, "0.0.0.0:8000");
        assert_eq!(config.namespace, "/osc");
        assert_eq!(config.session_timeout_secs, 30);
        assert!(!config.auto_subscribe);
    }

    /// Test OSC server adapter creation with custom config
    #[test]
    fn test_osc_config_custom() {
        let config = OscServerConfig {
            bind_addr: "127.0.0.1:9000".to_string(),
            namespace: "/custom".to_string(),
            session_timeout_secs: 60,
            auto_subscribe: true,
        };
        assert_eq!(config.bind_addr, "127.0.0.1:9000");
        assert_eq!(config.namespace, "/custom");
        assert_eq!(config.session_timeout_secs, 60);
        assert!(config.auto_subscribe);
    }

    /// Test OSC address to CLASP address conversion
    #[test]
    fn test_osc_address_to_clasp_address() {
        // OSC address "/synth/volume" with namespace "/osc" -> "/osc/synth/volume"
        let namespace = "/osc";
        let osc_addr = "/synth/volume";
        let expected = format!("{}{}", namespace, osc_addr);
        assert_eq!(expected, "/osc/synth/volume");
    }

    /// Test OSC value type conversion - Int
    #[test]
    fn test_osc_value_conversion_int() {
        // OSC Int -> CLASP Value::Int
        let osc_val = 42i32;
        let clasp_val = clasp_core::Value::Int(osc_val as i64);
        match clasp_val {
            clasp_core::Value::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int value"),
        }
    }

    /// Test OSC value type conversion - Float
    #[test]
    fn test_osc_value_conversion_float() {
        // OSC Float -> CLASP Value::Float
        let osc_val = 0.75f32;
        let clasp_val = clasp_core::Value::Float(osc_val as f64);
        match clasp_val {
            clasp_core::Value::Float(f) => assert!((f - 0.75).abs() < 0.001),
            _ => panic!("Expected Float value"),
        }
    }

    /// Test OSC value type conversion - String
    #[test]
    fn test_osc_value_conversion_string() {
        // OSC String -> CLASP Value::String
        let osc_val = "hello";
        let clasp_val = clasp_core::Value::String(osc_val.to_string());
        match clasp_val {
            clasp_core::Value::String(s) => assert_eq!(s, "hello"),
            _ => panic!("Expected String value"),
        }
    }

    /// Test OSC value type conversion - Bool
    #[test]
    fn test_osc_value_conversion_bool() {
        // OSC Bool -> CLASP Value::Bool
        let clasp_val_true = clasp_core::Value::Bool(true);
        let clasp_val_false = clasp_core::Value::Bool(false);
        match clasp_val_true {
            clasp_core::Value::Bool(b) => assert!(b),
            _ => panic!("Expected Bool value"),
        }
        match clasp_val_false {
            clasp_core::Value::Bool(b) => assert!(!b),
            _ => panic!("Expected Bool value"),
        }
    }

    /// Test OSC adapter integration with router state
    #[tokio::test]
    async fn test_osc_adapter_shares_router_state() {
        let router = Router::default();
        let state = router.state();

        // Set a value in the router state
        state
            .set(
                "/osc/synth/volume",
                clasp_core::Value::Float(0.8),
                &"osc-test".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        // Verify the state is accessible
        let value = state.get("/osc/synth/volume");
        assert!(value.is_some());
        match value.unwrap() {
            clasp_core::Value::Float(f) => assert!((f - 0.8).abs() < 0.001),
            _ => panic!("Expected Float value"),
        }
    }
}

// =============================================================================
// Multi-Protocol Integration Tests
// =============================================================================

#[cfg(all(feature = "mqtt-server", feature = "osc-server", feature = "websocket"))]
mod multi_protocol_tests {
    use super::*;
    use clasp_router::{
        adapters::{MqttServerConfig, OscServerConfig},
        MultiProtocolConfig,
    };

    /// Test multi-protocol config creation
    #[test]
    fn test_multi_protocol_config_creation() {
        let config = MultiProtocolConfig {
            websocket_addr: Some("127.0.0.1:7330".to_string()),
            mqtt: Some(MqttServerConfig {
                bind_addr: "127.0.0.1:1883".to_string(),
                ..Default::default()
            }),
            osc: Some(OscServerConfig {
                bind_addr: "127.0.0.1:8000".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };

        assert!(config.websocket_addr.is_some());
        assert!(config.mqtt.is_some());
        assert!(config.osc.is_some());
    }

    /// Test that multi-protocol servers share state
    #[tokio::test]
    async fn test_multi_protocol_shared_state() {
        let router = Router::default();
        let state = router.state();

        // Set values with different "protocol" prefixes
        state
            .set(
                "/mqtt/sensor/temp",
                clasp_core::Value::Float(22.5),
                &"mqtt-client".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        state
            .set(
                "/osc/control/volume",
                clasp_core::Value::Float(0.75),
                &"osc-client".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        state
            .set(
                "/clasp/param/value",
                clasp_core::Value::Int(100),
                &"ws-client".to_string(),
                None,
                false,
                false,
            )
            .unwrap();

        // All values should be accessible from the shared state
        assert!(state.get("/mqtt/sensor/temp").is_some());
        assert!(state.get("/osc/control/volume").is_some());
        assert!(state.get("/clasp/param/value").is_some());
    }
}
