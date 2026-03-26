//! Conversion helpers between CLASP types and DefraDB representations.

use std::collections::HashMap;

use clasp_core::{SignalType, Value};

/// Map a [`SignalType`] to its integer encoding for DefraDB storage.
pub fn signal_type_to_int(st: SignalType) -> i32 {
    match st {
        SignalType::Param => 0,
        SignalType::Event => 1,
        SignalType::Stream => 2,
        SignalType::Gesture => 3,
        SignalType::Timeline => 4,
    }
}

/// Recover a [`SignalType`] from its integer encoding.
///
/// Returns [`SignalType::Event`] for unrecognised values as a
/// safe fallback (events are ephemeral and carry no state).
pub fn int_to_signal_type(v: i32) -> SignalType {
    match v {
        0 => SignalType::Param,
        1 => SignalType::Event,
        2 => SignalType::Stream,
        3 => SignalType::Gesture,
        4 => SignalType::Timeline,
        _ => SignalType::Event,
    }
}

/// Serialize a CLASP [`Value`] to a JSON string for storage.
pub fn value_to_json(value: &Value) -> String {
    let json_value = value_to_serde(value);
    serde_json::to_string(&json_value).unwrap_or_else(|_| "null".to_string())
}

/// Deserialize a JSON string back into a CLASP [`Value`].
pub fn json_to_value(json: &str) -> Value {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(v) => serde_to_value(&v),
        Err(_) => Value::Null,
    }
}

/// Convert a CLASP glob pattern to a DefraDB `_like` pattern.
///
/// CLASP uses `/`-separated paths with `*` (single segment) and `**`
/// (multi-segment) wildcards. DefraDB's `_like` operator uses SQL LIKE
/// syntax with `%` for any sequence.
///
/// This is a simplification: both `*` and `**` map to `%` since DefraDB
/// does not support path-segment-aware matching. Queries may return
/// slightly broader results than a strict CLASP pattern would.
pub fn clasp_pattern_to_like(pattern: &str) -> String {
    let mut result = String::with_capacity(pattern.len());
    let mut chars = pattern.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '*' => {
                // Consume consecutive stars (** and * both become %)
                while chars.peek() == Some(&'*') {
                    chars.next();
                }
                result.push('%');
            }
            '%' => result.push_str("\\%"),
            '_' => result.push_str("\\_"),
            _ => result.push(ch),
        }
    }

    result
}

// -- Internal helpers -------------------------------------------------------

fn value_to_serde(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(arr) => serde_json::Value::Array(arr.iter().map(value_to_serde).collect()),
        Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), value_to_serde(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Bytes(bytes) => {
            // Encode as base64 string with a type tag so we can round-trip
            let encoded = base64_encode(bytes);
            serde_json::json!({ "__bytes": encoded })
        }
    }
}

fn serde_to_value(v: &serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                Value::Null
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => Value::Array(arr.iter().map(serde_to_value).collect()),
        serde_json::Value::Object(obj) => {
            // Check for encoded bytes
            if obj.len() == 1 {
                if let Some(encoded) = obj.get("__bytes").and_then(|v| v.as_str()) {
                    if let Some(bytes) = base64_decode(encoded) {
                        return Value::Bytes(bytes);
                    }
                }
            }
            let map: HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), serde_to_value(v)))
                .collect();
            Value::Map(map)
        }
    }
}

// Simple base64 without pulling in the base64 crate -- we already have
// serde_json so we use a minimal implementation via the format used by
// the existing clasp-core Bytes serialization.

fn base64_encode(bytes: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((bytes.len() + 2) / 3 * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;
        out.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        out.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

fn base64_decode(input: &str) -> Option<Vec<u8>> {
    fn char_val(c: u8) -> Option<u32> {
        match c {
            b'A'..=b'Z' => Some((c - b'A') as u32),
            b'a'..=b'z' => Some((c - b'a' + 26) as u32),
            b'0'..=b'9' => Some((c - b'0' + 52) as u32),
            b'+' => Some(62),
            b'/' => Some(63),
            b'=' => Some(0),
            _ => None,
        }
    }
    let bytes = input.as_bytes();
    if bytes.len() % 4 != 0 {
        return None;
    }
    let mut out = Vec::with_capacity(bytes.len() / 4 * 3);
    for chunk in bytes.chunks(4) {
        let a = char_val(chunk[0])?;
        let b = char_val(chunk[1])?;
        let c = char_val(chunk[2])?;
        let d = char_val(chunk[3])?;
        let triple = (a << 18) | (b << 12) | (c << 6) | d;
        out.push(((triple >> 16) & 0xFF) as u8);
        if chunk[2] != b'=' {
            out.push(((triple >> 8) & 0xFF) as u8);
        }
        if chunk[3] != b'=' {
            out.push((triple & 0xFF) as u8);
        }
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_signal_type_roundtrip() {
        let types = [
            SignalType::Param,
            SignalType::Event,
            SignalType::Stream,
            SignalType::Gesture,
            SignalType::Timeline,
        ];
        for st in types {
            assert_eq!(int_to_signal_type(signal_type_to_int(st)), st);
        }
    }

    #[test]
    fn convert_signal_type_unknown_falls_back() {
        assert_eq!(int_to_signal_type(99), SignalType::Event);
    }

    #[test]
    fn convert_value_roundtrip_null() {
        let json = value_to_json(&Value::Null);
        assert_eq!(json_to_value(&json), Value::Null);
    }

    #[test]
    fn convert_value_roundtrip_bool() {
        let json = value_to_json(&Value::Bool(true));
        assert_eq!(json_to_value(&json), Value::Bool(true));
    }

    #[test]
    fn convert_value_roundtrip_int() {
        let json = value_to_json(&Value::Int(42));
        assert_eq!(json_to_value(&json), Value::Int(42));
    }

    #[test]
    fn convert_value_roundtrip_float() {
        let val = Value::Float(3.14);
        let json = value_to_json(&val);
        match json_to_value(&json) {
            Value::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            other => panic!("expected Float, got {other:?}"),
        }
    }

    #[test]
    fn convert_value_roundtrip_string() {
        let val = Value::String("hello".into());
        let json = value_to_json(&val);
        assert_eq!(json_to_value(&json), val);
    }

    #[test]
    fn convert_value_roundtrip_array() {
        let val = Value::Array(vec![Value::Int(1), Value::Bool(false)]);
        let json = value_to_json(&val);
        assert_eq!(json_to_value(&json), val);
    }

    #[test]
    fn convert_value_roundtrip_map() {
        let mut map = HashMap::new();
        map.insert("key".into(), Value::String("val".into()));
        let val = Value::Map(map);
        let json = value_to_json(&val);
        assert_eq!(json_to_value(&json), val);
    }

    #[test]
    fn convert_value_roundtrip_bytes() {
        let val = Value::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let json = value_to_json(&val);
        assert_eq!(json_to_value(&json), val);
    }

    #[test]
    fn convert_value_invalid_json_returns_null() {
        assert_eq!(json_to_value("not valid json {{{"), Value::Null);
    }

    #[test]
    fn convert_pattern_literal() {
        assert_eq!(
            clasp_pattern_to_like("/synth/osc1/freq"),
            "/synth/osc1/freq"
        );
    }

    #[test]
    fn convert_pattern_single_wildcard() {
        assert_eq!(clasp_pattern_to_like("/synth/*/freq"), "/synth/%/freq");
    }

    #[test]
    fn convert_pattern_double_wildcard() {
        assert_eq!(clasp_pattern_to_like("/synth/**"), "/synth/%");
    }

    #[test]
    fn convert_pattern_mixed() {
        assert_eq!(clasp_pattern_to_like("/a/*/b/**/c"), "/a/%/b/%/c");
    }

    #[test]
    fn convert_pattern_escapes_sql_specials() {
        assert_eq!(clasp_pattern_to_like("100%_done"), "100\\%\\_done");
    }

    #[test]
    fn convert_pattern_empty() {
        assert_eq!(clasp_pattern_to_like(""), "");
    }
}
