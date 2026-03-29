//! LensVM transport buffer codec.
//!
//! Handles serialization and deserialization of values crossing the WASM
//! boundary. The format is compact and designed for minimal allocation.

use crate::error::{LensError, Result};

/// Type IDs used in the transport buffer.
pub const TYPE_ERROR: i8 = -1;
pub const TYPE_NIL: i8 = 0;
pub const TYPE_JSON: i8 = 1;
pub const TYPE_EOS: i8 = 127;

/// A decoded transport buffer item.
#[derive(Debug, Clone, PartialEq)]
pub enum TransportItem {
    /// A JSON value (the common case for signal data).
    Json(serde_json::Value),
    /// Nil / no value.
    Nil,
    /// End of stream marker.
    EndOfStream,
}

/// Encode a JSON value into a transport buffer.
///
/// Format: [TypeId: i8] [Length: u32 LE] [Payload: bytes]
pub fn encode_json(value: &serde_json::Value) -> Vec<u8> {
    let payload = serde_json::to_vec(value).expect("JSON serialization should not fail");
    let mut buf = Vec::with_capacity(1 + 4 + payload.len());
    buf.push(TYPE_JSON as u8);
    buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    buf.extend_from_slice(&payload);
    buf
}

/// Encode an end-of-stream marker.
pub fn encode_eos() -> Vec<u8> {
    vec![TYPE_EOS as u8]
}

/// Decode a transport buffer from raw bytes.
///
/// Returns the decoded item and the number of bytes consumed.
pub fn decode(buf: &[u8]) -> Result<(TransportItem, usize)> {
    if buf.is_empty() {
        return Err(LensError::Decode("empty transport buffer".into()));
    }

    let type_id = buf[0] as i8;
    match type_id {
        TYPE_NIL => Ok((TransportItem::Nil, 1)),

        TYPE_EOS => Ok((TransportItem::EndOfStream, 1)),

        TYPE_JSON => {
            if buf.len() < 5 {
                return Err(LensError::Decode(
                    "JSON item too short for length header".into(),
                ));
            }
            let len = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
            if buf.len() < 5 + len {
                return Err(LensError::Decode(format!(
                    "JSON payload truncated: expected {} bytes, got {}",
                    len,
                    buf.len() - 5
                )));
            }
            let value: serde_json::Value = serde_json::from_slice(&buf[5..5 + len])?;
            Ok((TransportItem::Json(value), 5 + len))
        }

        TYPE_ERROR => {
            if buf.len() < 5 {
                return Err(LensError::Decode(
                    "error item too short for length header".into(),
                ));
            }
            let len = u32::from_le_bytes([buf[1], buf[2], buf[3], buf[4]]) as usize;
            if buf.len() < 5 + len {
                return Err(LensError::Decode("error payload truncated".into()));
            }
            let msg = String::from_utf8_lossy(&buf[5..5 + len]).into_owned();
            Err(LensError::LensError(msg))
        }

        other => Err(LensError::Decode(format!("unknown type ID: {}", other))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn round_trip_json() {
        let original = json!({"value": 0.5});
        let encoded = encode_json(&original);
        let (decoded, consumed) = decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn round_trip_eos() {
        let encoded = encode_eos();
        let (decoded, consumed) = decode(&encoded).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(decoded, TransportItem::EndOfStream);
    }

    #[test]
    fn decode_nil() {
        let buf = [TYPE_NIL as u8];
        let (decoded, consumed) = decode(&buf).unwrap();
        assert_eq!(consumed, 1);
        assert_eq!(decoded, TransportItem::Nil);
    }

    #[test]
    fn decode_error() {
        let msg = b"something went wrong";
        let mut buf = vec![TYPE_ERROR as u8];
        buf.extend_from_slice(&(msg.len() as u32).to_le_bytes());
        buf.extend_from_slice(msg);

        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, LensError::LensError(ref s) if s == "something went wrong"));
    }

    #[test]
    fn decode_empty_is_error() {
        let err = decode(&[]).unwrap_err();
        assert!(matches!(err, LensError::Decode(_)));
    }

    #[test]
    fn decode_truncated_json_is_error() {
        let mut buf = vec![TYPE_JSON as u8];
        buf.extend_from_slice(&100u32.to_le_bytes()); // claims 100 bytes
        buf.extend_from_slice(b"{}"); // only 2 bytes
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, LensError::Decode(_)));
    }

    #[test]
    fn decode_unknown_type_is_error() {
        let buf = [42u8];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, LensError::Decode(_)));
    }

    #[test]
    fn round_trip_complex_json() {
        let original = json!({
            "value": 0.123456789,
            "extra": "metadata",
            "nested": {"a": 1, "b": [2, 3]}
        });
        let encoded = encode_json(&original);
        let (decoded, consumed) = decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn round_trip_json_integer() {
        let original = json!({"value": 42});
        let encoded = encode_json(&original);
        let (decoded, _) = decode(&encoded).unwrap();
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn round_trip_json_null() {
        let original = json!(null);
        let encoded = encode_json(&original);
        let (decoded, _) = decode(&encoded).unwrap();
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn round_trip_json_string() {
        let original = json!("hello world");
        let encoded = encode_json(&original);
        let (decoded, _) = decode(&encoded).unwrap();
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn round_trip_json_array() {
        let original = json!([1.0, 2.0, 3.0]);
        let encoded = encode_json(&original);
        let (decoded, _) = decode(&encoded).unwrap();
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn round_trip_empty_json_object() {
        let original = json!({});
        let encoded = encode_json(&original);
        let (decoded, _) = decode(&encoded).unwrap();
        assert_eq!(decoded, TransportItem::Json(original));
    }

    #[test]
    fn decode_error_with_empty_message() {
        let mut buf = vec![TYPE_ERROR as u8];
        buf.extend_from_slice(&0u32.to_le_bytes());
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, LensError::LensError(ref s) if s.is_empty()));
    }

    #[test]
    fn decode_json_missing_length_header() {
        // Type ID for JSON but only 2 bytes after (need 4 for length)
        let buf = [TYPE_JSON as u8, 0x00, 0x00];
        let err = decode(&buf).unwrap_err();
        assert!(matches!(err, LensError::Decode(_)));
    }

    #[test]
    fn encode_json_large_payload() {
        // Large JSON to test length encoding
        let large = json!({"data": "x".repeat(10000)});
        let encoded = encode_json(&large);
        let (decoded, consumed) = decode(&encoded).unwrap();
        assert_eq!(consumed, encoded.len());
        assert_eq!(decoded, TransportItem::Json(large));
    }
}
