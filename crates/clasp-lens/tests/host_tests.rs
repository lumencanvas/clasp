//! Integration tests for the LensVM WASM host.
//!
//! These tests use a real compiled WASM lens module (test-lens-scale)
//! to verify the full host protocol: alloc, next, set_param, transform, inverse.
//!
//! The test lens implements: output = input * scale_factor + offset
//! Default params (no set_param call): scale_factor=1.0, offset=0.0

use clasp_lens::{LensError, LensHost};
use serde_json::json;

const SCALE_LENS_WASM: &[u8] = include_bytes!("fixtures/scale_lens.wasm");

// ---------------------------------------------------------------------------
// Module loading
// ---------------------------------------------------------------------------

#[test]
fn load_valid_wasm_module() {
    let host = LensHost::new(SCALE_LENS_WASM);
    assert!(host.is_ok());
}

#[test]
fn load_invalid_bytes_returns_compile_error() {
    let result = LensHost::new(b"not a wasm module");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, LensError::Compile(_)),
        "expected Compile error, got: {:?}",
        err
    );
}

#[test]
fn load_empty_bytes_returns_error() {
    let result = LensHost::new(b"");
    assert!(result.is_err());
}

#[test]
fn load_truncated_wasm_header_returns_error() {
    // Valid WASM magic number but truncated
    let result = LensHost::new(&[0x00, 0x61, 0x73, 0x6D]);
    assert!(result.is_err());
}

// ---------------------------------------------------------------------------
// Module introspection
// ---------------------------------------------------------------------------

#[test]
fn has_inverse_returns_true_for_scale_lens() {
    let host = LensHost::new(SCALE_LENS_WASM).unwrap();
    assert!(host.has_inverse());
}

#[test]
fn debug_format_includes_capabilities() {
    let host = LensHost::new(SCALE_LENS_WASM).unwrap();
    let debug = format!("{:?}", host);
    assert!(debug.contains("has_inverse: true"));
    assert!(debug.contains("has_params: false"));
}

#[test]
fn debug_format_shows_params_after_set() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 2.0, "offset": 0.0}));
    let debug = format!("{:?}", host);
    assert!(debug.contains("has_params: true"));
}

// ---------------------------------------------------------------------------
// Forward transform: basic correctness
// ---------------------------------------------------------------------------

#[test]
fn transform_identity_without_params() {
    // Default: scale_factor=1.0, offset=0.0 (passthrough)
    let host = LensHost::new(SCALE_LENS_WASM).unwrap();
    let output = host.transform(&json!({"value": 0.5})).unwrap();
    assert_f64_eq(output["value"].as_f64().unwrap(), 0.5);
}

#[test]
fn transform_scale_and_offset() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 2.0, "offset": 0.5}));

    let output = host.transform(&json!({"value": 0.25})).unwrap();
    // 0.25 * 2.0 + 0.5 = 1.0
    assert_f64_eq(output["value"].as_f64().unwrap(), 1.0);
}

#[test]
fn transform_negative_scale() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": -1.0, "offset": 0.0}));

    let output = host.transform(&json!({"value": 0.7})).unwrap();
    assert_f64_eq(output["value"].as_f64().unwrap(), -0.7);
}

#[test]
fn transform_zero_input() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 100.0, "offset": 5.0}));

    let output = host.transform(&json!({"value": 0.0})).unwrap();
    assert_f64_eq(output["value"].as_f64().unwrap(), 5.0);
}

#[test]
fn transform_large_values() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 1000.0, "offset": 0.0}));

    let output = host.transform(&json!({"value": 999.999})).unwrap();
    let v = output["value"].as_f64().unwrap();
    assert!((v - 999999.0).abs() < 1.0, "expected ~999999, got {}", v);
}

#[test]
fn transform_very_small_values() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 1.0, "offset": 0.0}));

    let output = host.transform(&json!({"value": 1e-15})).unwrap();
    let v = output["value"].as_f64().unwrap();
    assert!(v > 0.0 && v < 1e-14, "expected tiny positive, got {}", v);
}

#[test]
fn transform_negative_offset() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 1.0, "offset": -10.0}));

    let output = host.transform(&json!({"value": 3.0})).unwrap();
    assert_f64_eq(output["value"].as_f64().unwrap(), -7.0);
}

// ---------------------------------------------------------------------------
// Inverse transform
// ---------------------------------------------------------------------------

#[test]
fn inverse_reverses_forward() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 2.0, "offset": 0.5}));

    let original = json!({"value": 0.7});
    let transformed = host.transform(&original).unwrap();
    assert_f64_eq(transformed["value"].as_f64().unwrap(), 1.9);

    let restored = host.inverse(&transformed).unwrap();
    assert_f64_eq(restored["value"].as_f64().unwrap(), 0.7);
}

#[test]
fn inverse_round_trip_many_values() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 3.7, "offset": -2.1}));

    for i in 0..20 {
        let v = (i as f64 - 10.0) * 0.37;
        let input = json!({"value": v});
        let fwd = host.transform(&input).unwrap();
        let rev = host.inverse(&fwd).unwrap();
        let restored = rev["value"].as_f64().unwrap();
        assert!(
            (restored - v).abs() < 1e-10,
            "round-trip failed for {}: got {}",
            v,
            restored
        );
    }
}

// ---------------------------------------------------------------------------
// Statefulness and isolation
// ---------------------------------------------------------------------------

#[test]
fn multiple_transforms_are_independent() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 3.0, "offset": 0.0}));

    for i in 0..10 {
        let v = i as f64 * 0.1;
        let output = host.transform(&json!({"value": v})).unwrap();
        assert_f64_eq(output["value"].as_f64().unwrap(), v * 3.0);
    }
}

#[test]
fn params_can_be_changed_between_calls() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    let input = json!({"value": 1.0});

    host.set_params(json!({"scale_factor": 2.0, "offset": 0.0}));
    let out1 = host.transform(&input).unwrap();
    assert_f64_eq(out1["value"].as_f64().unwrap(), 2.0);

    host.set_params(json!({"scale_factor": 5.0, "offset": 10.0}));
    let out2 = host.transform(&input).unwrap();
    assert_f64_eq(out2["value"].as_f64().unwrap(), 15.0);
}

#[test]
fn separate_hosts_from_same_bytes_are_independent() {
    let mut host_a = LensHost::new(SCALE_LENS_WASM).unwrap();
    let mut host_b = LensHost::new(SCALE_LENS_WASM).unwrap();

    host_a.set_params(json!({"scale_factor": 2.0, "offset": 0.0}));
    host_b.set_params(json!({"scale_factor": 10.0, "offset": 0.0}));

    let input = json!({"value": 1.0});
    let out_a = host_a.transform(&input).unwrap();
    let out_b = host_b.transform(&input).unwrap();

    assert_f64_eq(out_a["value"].as_f64().unwrap(), 2.0);
    assert_f64_eq(out_b["value"].as_f64().unwrap(), 10.0);
}

// ---------------------------------------------------------------------------
// Stress / throughput
// ---------------------------------------------------------------------------

#[test]
fn transform_1000_values_sequentially() {
    let mut host = LensHost::new(SCALE_LENS_WASM).unwrap();
    host.set_params(json!({"scale_factor": 2.0, "offset": 1.0}));

    for i in 0..1000 {
        let v = (i as f64) / 1000.0;
        let output = host.transform(&json!({"value": v})).unwrap();
        let expected = v * 2.0 + 1.0;
        let actual = output["value"].as_f64().unwrap();
        assert!(
            (actual - expected).abs() < 1e-10,
            "i={}: expected {}, got {}",
            i,
            expected,
            actual
        );
    }
}

// ---------------------------------------------------------------------------
// Edge cases in JSON structure
// ---------------------------------------------------------------------------

#[test]
fn transform_preserves_json_structure() {
    let host = LensHost::new(SCALE_LENS_WASM).unwrap();
    let output = host.transform(&json!({"value": 42.0})).unwrap();
    // Output should be a JSON object with a "value" key
    assert!(output.is_object());
    assert!(output.get("value").is_some());
}

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn assert_f64_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-10,
        "expected {}, got {}",
        expected,
        actual
    );
}
