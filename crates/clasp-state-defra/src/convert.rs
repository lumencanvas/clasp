//! Conversion helpers between ParamState and DefraDB document representations.

use std::collections::HashMap;

use clasp_core::state::ParamState;
use clasp_core::{ConflictStrategy, Ttl, Value};
use serde_json::json;

/// Convert a ParamState into a DefraDB document (JSON object).
pub fn param_to_defra(address: &str, state: &ParamState) -> serde_json::Value {
    json!({
        "address": address,
        "value": value_to_json(&state.value),
        "valueType": value_type_tag(&state.value),
        "revision": state.revision as i64,
        "writer": state.writer,
        "timestamp": state.timestamp as i64,
        "lastAccessed": state.last_accessed as i64,
        "strategy": strategy_to_str(state.strategy),
        "lockHolder": state.lock_holder.as_deref().unwrap_or(""),
        "origin": state.origin.as_deref().unwrap_or(""),
        "ttlMode": ttl_mode_str(state.ttl.as_ref()),
        "ttlSecs": ttl_secs(state.ttl.as_ref()),
    })
}

/// Convert a DefraDB document back into (address, ParamState).
pub fn defra_to_param(doc: &serde_json::Value) -> crate::Result<(String, ParamState)> {
    let address = doc["address"]
        .as_str()
        .ok_or_else(|| crate::DefraStateError::Deserialization("missing address".into()))?
        .to_string();

    let value_json = doc["value"].as_str().unwrap_or("null");
    let value_type = doc["valueType"].as_str().unwrap_or("null");
    let value = json_to_value(value_json, value_type);

    let revision = doc["revision"].as_i64().unwrap_or(1) as u64;
    let writer = doc["writer"].as_str().unwrap_or("").to_string();
    let timestamp = doc["timestamp"].as_i64().unwrap_or(0) as u64;
    let last_accessed = doc["lastAccessed"].as_i64().unwrap_or(0) as u64;
    let strategy = str_to_strategy(doc["strategy"].as_str().unwrap_or("lww"));

    let lock_holder = doc["lockHolder"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from);

    let origin = doc["origin"]
        .as_str()
        .filter(|s| !s.is_empty())
        .map(String::from);

    let ttl_mode = doc["ttlMode"].as_str().unwrap_or("none");
    let ttl_secs_val = doc["ttlSecs"].as_i64().unwrap_or(0) as u32;
    let ttl = str_to_ttl(ttl_mode, ttl_secs_val);

    let state = ParamState {
        value,
        revision,
        writer,
        timestamp,
        last_accessed,
        strategy,
        lock_holder,
        meta: None, // ParamMeta is not persisted to DefraDB
        origin,
        ttl,
    };

    Ok((address, state))
}

// -- Strategy helpers --------------------------------------------------------

pub fn strategy_to_str(s: ConflictStrategy) -> &'static str {
    match s {
        ConflictStrategy::Lww => "lww",
        ConflictStrategy::Max => "max",
        ConflictStrategy::Min => "min",
        ConflictStrategy::Lock => "lock",
        ConflictStrategy::Merge => "merge",
    }
}

pub fn str_to_strategy(s: &str) -> ConflictStrategy {
    match s {
        "lww" => ConflictStrategy::Lww,
        "max" => ConflictStrategy::Max,
        "min" => ConflictStrategy::Min,
        "lock" => ConflictStrategy::Lock,
        "merge" => ConflictStrategy::Merge,
        _ => ConflictStrategy::Lww,
    }
}

// -- TTL helpers -------------------------------------------------------------

fn ttl_mode_str(ttl: Option<&Ttl>) -> &'static str {
    match ttl {
        Some(Ttl::Sliding(_)) => "sliding",
        Some(Ttl::Absolute(_)) => "absolute",
        Some(Ttl::Never) => "never",
        None => "none",
    }
}

fn ttl_secs(ttl: Option<&Ttl>) -> i64 {
    match ttl {
        Some(Ttl::Sliding(s)) => *s as i64,
        Some(Ttl::Absolute(s)) => *s as i64,
        _ => 0,
    }
}

pub fn str_to_ttl(mode: &str, secs: u32) -> Option<Ttl> {
    match mode {
        "sliding" => Some(Ttl::Sliding(secs)),
        "absolute" => Some(Ttl::Absolute(secs)),
        "never" => Some(Ttl::Never),
        _ => None,
    }
}

// -- Value serialization -----------------------------------------------------

/// Tag string for the CLASP Value variant, used to disambiguate on deserialize.
fn value_type_tag(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Map(_) => "map",
        Value::Bytes(_) => "bytes",
    }
}

/// Serialize a CLASP Value to a JSON string for storage.
fn value_to_json(value: &Value) -> String {
    let json_value = value_to_serde(value);
    serde_json::to_string(&json_value).unwrap_or_else(|_| "null".to_string())
}

/// Deserialize a JSON string back into a CLASP Value, using the type tag
/// to resolve ambiguities (e.g. Int vs Float for whole numbers).
fn json_to_value(json: &str, type_tag: &str) -> Value {
    match serde_json::from_str::<serde_json::Value>(json) {
        Ok(v) => serde_to_value(&v, type_tag),
        Err(_) => Value::Null,
    }
}

fn value_to_serde(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => json!(i),
        Value::Float(f) => json!(f),
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
            let encoded = base64_encode(bytes);
            json!({ "__bytes": encoded })
        }
    }
}

fn serde_to_value(v: &serde_json::Value, type_tag: &str) -> Value {
    match v {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            // Use type tag to disambiguate Int vs Float
            match type_tag {
                "int" => Value::Int(n.as_i64().unwrap_or(0)),
                "float" => Value::Float(n.as_f64().unwrap_or(0.0)),
                _ => {
                    if let Some(i) = n.as_i64() {
                        Value::Int(i)
                    } else if let Some(f) = n.as_f64() {
                        Value::Float(f)
                    } else {
                        Value::Null
                    }
                }
            }
        }
        serde_json::Value::String(s) => {
            if type_tag == "bytes" {
                // Should not normally happen (bytes use __bytes object), but handle gracefully
                Value::String(s.clone())
            } else {
                Value::String(s.clone())
            }
        }
        serde_json::Value::Array(arr) => {
            Value::Array(arr.iter().map(|v| serde_to_value(v, "")).collect())
        }
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
                .map(|(k, v)| (k.clone(), serde_to_value(v, "")))
                .collect();
            Value::Map(map)
        }
    }
}

// -- Base64 (minimal, matching clasp-journal-defra) --------------------------

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
    use clasp_core::state::ParamState;
    use clasp_core::{ConflictStrategy, Ttl, Value};

    #[test]
    fn param_convert_roundtrip() {
        let state = ParamState {
            value: Value::Float(0.75),
            revision: 5,
            writer: "session-abc".to_string(),
            timestamp: 1_700_000_000_000_000,
            last_accessed: 1_700_000_001_000_000,
            strategy: ConflictStrategy::Max,
            lock_holder: Some("session-abc".to_string()),
            meta: None,
            origin: Some("router-1".to_string()),
            ttl: Some(Ttl::Sliding(300)),
        };

        let doc = param_to_defra("/synth/osc1/freq", &state);
        let (addr, recovered) = defra_to_param(&doc).unwrap();

        assert_eq!(addr, "/synth/osc1/freq");
        assert_eq!(recovered.value, Value::Float(0.75));
        assert_eq!(recovered.revision, 5);
        assert_eq!(recovered.writer, "session-abc");
        assert_eq!(recovered.timestamp, 1_700_000_000_000_000);
        assert_eq!(recovered.last_accessed, 1_700_000_001_000_000);
        assert_eq!(recovered.strategy, ConflictStrategy::Max);
        assert_eq!(
            recovered.lock_holder,
            Some("session-abc".to_string())
        );
        assert_eq!(recovered.origin, Some("router-1".to_string()));
        assert_eq!(recovered.ttl, Some(Ttl::Sliding(300)));
    }

    #[test]
    fn ttl_modes_convert() {
        // Sliding
        assert_eq!(str_to_ttl("sliding", 60), Some(Ttl::Sliding(60)));
        // Absolute
        assert_eq!(str_to_ttl("absolute", 3600), Some(Ttl::Absolute(3600)));
        // Never
        assert_eq!(str_to_ttl("never", 0), Some(Ttl::Never));
        // None
        assert_eq!(str_to_ttl("none", 0), None);
        // Unknown
        assert_eq!(str_to_ttl("bogus", 0), None);
    }

    #[test]
    fn strategy_convert() {
        let strategies = [
            (ConflictStrategy::Lww, "lww"),
            (ConflictStrategy::Max, "max"),
            (ConflictStrategy::Min, "min"),
            (ConflictStrategy::Lock, "lock"),
            (ConflictStrategy::Merge, "merge"),
        ];
        for (strat, s) in strategies {
            assert_eq!(strategy_to_str(strat), s);
            assert_eq!(str_to_strategy(s), strat);
        }
        // Unknown falls back to Lww
        assert_eq!(str_to_strategy("unknown"), ConflictStrategy::Lww);
    }

    #[test]
    fn value_roundtrip_null() {
        let json = value_to_json(&Value::Null);
        assert_eq!(json_to_value(&json, "null"), Value::Null);
    }

    #[test]
    fn value_roundtrip_int() {
        let json = value_to_json(&Value::Int(42));
        assert_eq!(json_to_value(&json, "int"), Value::Int(42));
    }

    #[test]
    fn value_roundtrip_float() {
        let json = value_to_json(&Value::Float(3.14));
        match json_to_value(&json, "float") {
            Value::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            other => panic!("expected Float, got {other:?}"),
        }
    }

    #[test]
    fn value_roundtrip_bytes() {
        let val = Value::Bytes(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        let json = value_to_json(&val);
        assert_eq!(json_to_value(&json, "bytes"), val);
    }

    #[test]
    fn empty_lock_holder_becomes_none() {
        let doc = json!({
            "address": "/test",
            "value": "null",
            "valueType": "null",
            "revision": 1,
            "writer": "s1",
            "timestamp": 0,
            "lastAccessed": 0,
            "strategy": "lww",
            "lockHolder": "",
            "origin": "",
            "ttlMode": "none",
            "ttlSecs": 0,
        });
        let (_, state) = defra_to_param(&doc).unwrap();
        assert_eq!(state.lock_holder, None);
        assert_eq!(state.origin, None);
    }
}
