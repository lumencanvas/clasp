//! Value conversion between CLASP Values and DefraDB JSON.

use std::collections::HashMap;

use clasp_core::Value;

/// Convert a CLASP [`Value`] to a [`serde_json::Value`].
pub fn clasp_value_to_json(value: &Value) -> serde_json::Value {
    match value {
        Value::Null => serde_json::Value::Null,
        Value::Bool(b) => serde_json::Value::Bool(*b),
        Value::Int(i) => serde_json::json!(i),
        Value::Float(f) => serde_json::json!(f),
        Value::String(s) => serde_json::Value::String(s.clone()),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(clasp_value_to_json).collect())
        }
        Value::Map(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map
                .iter()
                .map(|(k, v)| (k.clone(), clasp_value_to_json(v)))
                .collect();
            serde_json::Value::Object(obj)
        }
        Value::Bytes(bytes) => {
            // Encode as array of integers for JSON storage
            serde_json::Value::Array(bytes.iter().map(|b| serde_json::json!(b)).collect())
        }
    }
}

/// Convert a [`serde_json::Value`] to a CLASP [`Value`].
pub fn json_to_clasp_value(json: &serde_json::Value) -> Value {
    match json {
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
        serde_json::Value::Array(arr) => {
            Value::Array(arr.iter().map(json_to_clasp_value).collect())
        }
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Value> = obj
                .iter()
                .map(|(k, v)| (k.clone(), json_to_clasp_value(v)))
                .collect();
            Value::Map(map)
        }
    }
}

/// Convert a full DefraDB document JSON object to a CLASP Map value.
///
/// Strips internal DefraDB fields (those starting with `_`) from the
/// resulting map so only user-defined fields are exposed as signals.
pub fn json_doc_to_clasp_map(doc: &serde_json::Value) -> Value {
    match doc {
        serde_json::Value::Object(obj) => {
            let map: HashMap<String, Value> = obj
                .iter()
                .filter(|(k, _)| !k.starts_with('_'))
                .map(|(k, v)| (k.clone(), json_to_clasp_value(v)))
                .collect();
            Value::Map(map)
        }
        other => json_to_clasp_value(other),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_conversion_roundtrip_null() {
        let v = Value::Null;
        let json = clasp_value_to_json(&v);
        assert_eq!(json_to_clasp_value(&json), v);
    }

    #[test]
    fn value_conversion_roundtrip_bool() {
        let v = Value::Bool(true);
        let json = clasp_value_to_json(&v);
        assert_eq!(json_to_clasp_value(&json), v);
    }

    #[test]
    fn value_conversion_roundtrip_int() {
        let v = Value::Int(42);
        let json = clasp_value_to_json(&v);
        assert_eq!(json_to_clasp_value(&json), v);
    }

    #[test]
    fn value_conversion_roundtrip_float() {
        let v = Value::Float(3.14);
        let json = clasp_value_to_json(&v);
        match json_to_clasp_value(&json) {
            Value::Float(f) => assert!((f - 3.14).abs() < 1e-10),
            other => panic!("expected Float, got {other:?}"),
        }
    }

    #[test]
    fn value_conversion_roundtrip_string() {
        let v = Value::String("hello".into());
        let json = clasp_value_to_json(&v);
        assert_eq!(json_to_clasp_value(&json), v);
    }

    #[test]
    fn value_conversion_roundtrip_array() {
        let v = Value::Array(vec![Value::Int(1), Value::Bool(false)]);
        let json = clasp_value_to_json(&v);
        assert_eq!(json_to_clasp_value(&json), v);
    }

    #[test]
    fn value_conversion_roundtrip_map() {
        let mut map = HashMap::new();
        map.insert("key".into(), Value::String("val".into()));
        let v = Value::Map(map);
        let json = clasp_value_to_json(&v);
        assert_eq!(json_to_clasp_value(&json), v);
    }

    #[test]
    fn json_doc_strips_internal_fields() {
        let doc = serde_json::json!({
            "_docID": "bae-abc",
            "_key": "internal",
            "name": "Alice",
            "age": 30
        });
        let map = json_doc_to_clasp_map(&doc);
        match &map {
            Value::Map(m) => {
                assert!(!m.contains_key("_docID"));
                assert!(!m.contains_key("_key"));
                assert_eq!(m.get("name"), Some(&Value::String("Alice".into())));
                assert_eq!(m.get("age"), Some(&Value::Int(30)));
            }
            other => panic!("expected Map, got {other:?}"),
        }
    }
}
