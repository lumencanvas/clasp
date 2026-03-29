//! Tests for the LensVM WASM transform pipeline.
//!
//! Gated behind `#[cfg(feature = "lens")]` since the lens module is optional.
//! Run with: cargo test --features lens

#[cfg(feature = "lens")]
mod lens_tests {
    use clasp_relay::lens::{LensConfigEntry, LensTransformPipeline};
    use std::path::PathBuf;

    #[test]
    fn lens_config_deserialize_full() {
        let json = r#"[
            {
                "address": "/sensors/**",
                "wasm": "/tmp/test.wasm",
                "params": {"alpha": 0.3}
            }
        ]"#;
        let configs: Vec<LensConfigEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].address, "/sensors/**");
        assert_eq!(configs[0].wasm, PathBuf::from("/tmp/test.wasm"));
        assert!(configs[0].params.is_some());
        let alpha = configs[0].params.as_ref().unwrap()["alpha"].as_f64().unwrap();
        assert!((alpha - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn lens_config_deserialize_no_params() {
        let json = r#"[{"address": "/audio/*", "wasm": "/opt/lenses/gain.wasm"}]"#;
        let configs: Vec<LensConfigEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].address, "/audio/*");
        assert!(configs[0].params.is_none());
    }

    #[test]
    fn lens_config_deserialize_multiple() {
        let json = r#"[
            {"address": "/a", "wasm": "/x.wasm"},
            {"address": "/b/**", "wasm": "/y.wasm", "params": {"k": 1}},
            {"address": "/c/d", "wasm": "/z.wasm", "params": null}
        ]"#;
        let configs: Vec<LensConfigEntry> = serde_json::from_str(json).unwrap();
        assert_eq!(configs.len(), 3);
        assert_eq!(configs[0].address, "/a");
        assert_eq!(configs[1].address, "/b/**");
        assert_eq!(configs[2].address, "/c/d");
        // null params deserializes as None
        assert!(configs[2].params.is_none());
    }

    #[test]
    fn lens_config_deserialize_empty_array() {
        let json = "[]";
        let configs: Vec<LensConfigEntry> = serde_json::from_str(json).unwrap();
        assert!(configs.is_empty());
    }

    #[test]
    fn lens_config_missing_required_field_errors() {
        // Missing "wasm" field
        let json = r#"[{"address": "/test"}]"#;
        let result = serde_json::from_str::<Vec<LensConfigEntry>>(json);
        assert!(result.is_err());

        // Missing "address" field
        let json = r#"[{"wasm": "/test.wasm"}]"#;
        let result = serde_json::from_str::<Vec<LensConfigEntry>>(json);
        assert!(result.is_err());
    }

    #[test]
    fn lens_pipeline_invalid_path_errors() {
        let configs = vec![LensConfigEntry {
            address: "/test".to_string(),
            wasm: PathBuf::from("/nonexistent/path/to/module.wasm"),
            params: None,
        }];
        let result = LensTransformPipeline::from_configs(&configs);
        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Failed to read WASM lens"),
                    "Error message should mention reading failure, got: {}",
                    err_msg
                );
            }
            Ok(_) => panic!("Expected error for nonexistent WASM path"),
        }
    }

    #[test]
    fn lens_pipeline_empty_configs_succeeds() {
        let configs: Vec<LensConfigEntry> = vec![];
        let pipeline = LensTransformPipeline::from_configs(&configs);
        assert!(pipeline.is_ok());
    }

    #[test]
    fn lens_pipeline_invalid_wasm_bytes_errors() {
        // Write garbage to a temp file -- should fail to instantiate as WASM
        let dir = tempfile::tempdir().unwrap();
        let wasm_path = dir.path().join("garbage.wasm");
        std::fs::write(&wasm_path, b"this is not valid wasm").unwrap();

        let configs = vec![LensConfigEntry {
            address: "/test".to_string(),
            wasm: wasm_path,
            params: None,
        }];
        let result = LensTransformPipeline::from_configs(&configs);
        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Failed to load WASM lens"),
                    "Error message should mention load failure, got: {}",
                    err_msg
                );
            }
            Ok(_) => panic!("Expected error for invalid WASM bytes"),
        }
    }

    /// If the lowpass.wasm has been built, test loading and transforming with it.
    /// Skip gracefully if the binary is not present.
    #[test]
    fn lens_pipeline_loads_real_wasm_if_available() {
        use clasp_core::Value;
        use clasp_router::SignalTransform;

        let wasm_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../lenses/lowpass/target/wasm32-unknown-unknown/release/lowpass.wasm");

        if !wasm_path.exists() {
            eprintln!(
                "Skipping real WASM test: lowpass.wasm not built. \
                 Run: cd lenses/lowpass && cargo build --target wasm32-unknown-unknown --release"
            );
            return;
        }

        let configs = vec![LensConfigEntry {
            address: "/sensors/**".to_string(),
            wasm: wasm_path,
            params: Some(serde_json::json!({"alpha": 0.3})),
        }];
        let pipeline = LensTransformPipeline::from_configs(&configs)
            .expect("Should load valid lowpass.wasm");

        // Matching address should produce a transformed value
        let result = pipeline.transform("/sensors/temp", &Value::Float(22.5));
        assert!(
            result.is_some(),
            "Expected transform to produce a value for matching address"
        );

        // Non-matching address should return None (no lens bound to this pattern)
        let result = pipeline.transform("/lights/brightness", &Value::Float(0.5));
        assert!(
            result.is_none(),
            "Expected None for non-matching address"
        );
    }
}
