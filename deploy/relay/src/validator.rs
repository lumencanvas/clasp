//! Chat-specific write validation and snapshot filtering.
//!
//! Enforces server-side authorization rules for room admin/ban/meta paths
//! and namespace metadata paths. Also filters sensitive fields from snapshots.

use clasp_core::Value;
use clasp_router::{RouterState, Session, SnapshotFilter, WriteValidator};
use tracing::debug;

/// Extract the user_id from a session's subject field (set during HELLO from the token).
fn session_user_id(session: &Session) -> Option<&str> {
    session.subject.as_deref()
}

/// Extract the `creatorId` field from a Value::Map.
fn extract_creator_id(value: &Value) -> Option<&str> {
    match value {
        Value::Map(map) => map.get("creatorId").and_then(|v| v.as_str()),
        _ => None,
    }
}

/// Parse a chat room path and extract components.
/// Returns (room_id, sub_path) for paths like `/chat/room/{rid}/admin/...`
fn parse_room_path(address: &str) -> Option<(&str, &str)> {
    let rest = address.strip_prefix("/chat/room/")?;
    let slash_pos = rest.find('/')?;
    let room_id = &rest[..slash_pos];
    let sub_path = &rest[slash_pos + 1..];
    Some((room_id, sub_path))
}

/// Parse a namespace meta path.
/// Returns the namespace path for paths like `/chat/registry/ns-meta/{path}`
fn parse_ns_meta_path(address: &str) -> Option<&str> {
    address.strip_prefix("/chat/registry/ns-meta/")
}

/// Chat-specific write validator.
///
/// Enforces:
/// - `/chat/room/{rid}/admin/*` — only room creator can write
/// - `/chat/room/{rid}/bans/*`  — only room creator or admin can write
/// - `/chat/room/{rid}/meta`    — only room creator can write (or initial creation)
/// - `/chat/registry/ns-meta/**` — only namespace creator can write (or initial creation)
pub struct ChatWriteValidator;

impl ChatWriteValidator {
    /// Check if the writer is the room creator by looking up the room meta.
    fn is_room_creator(
        room_id: &str,
        writer_id: &str,
        state: &RouterState,
    ) -> bool {
        let meta_address = format!("/chat/room/{}/meta", room_id);
        match state.get(&meta_address) {
            Some(meta_value) => {
                extract_creator_id(&meta_value) == Some(writer_id)
            }
            None => {
                // No meta exists — this is initial room creation, allow it
                true
            }
        }
    }

    /// Check if the writer is an admin of the room.
    fn is_room_admin(
        room_id: &str,
        writer_id: &str,
        state: &RouterState,
    ) -> bool {
        let admin_address = format!("/chat/room/{}/admin/{}", room_id, writer_id);
        match state.get(&admin_address) {
            Some(Value::Null) => false,
            Some(_) => true,
            None => false,
        }
    }

    /// Check if the writer is the namespace creator.
    fn is_ns_creator(
        ns_path: &str,
        writer_id: &str,
        state: &RouterState,
    ) -> bool {
        // Strip __auth suffix if present to find the base meta path
        let base_path = ns_path
            .strip_suffix("/__auth")
            .unwrap_or(ns_path);
        let meta_address = format!("/chat/registry/ns-meta/{}", base_path);
        match state.get(&meta_address) {
            Some(meta_value) => {
                // Check createdBy field
                match &meta_value {
                    Value::Map(map) => {
                        map.get("createdBy")
                            .and_then(|v| v.as_str())
                            == Some(writer_id)
                    }
                    _ => false,
                }
            }
            None => {
                // No meta exists — this is initial namespace creation, allow it
                true
            }
        }
    }
}

impl WriteValidator for ChatWriteValidator {
    fn validate_write(
        &self,
        address: &str,
        _value: &Value,
        session: &Session,
        state: &RouterState,
    ) -> Result<(), String> {
        let writer_id = match session_user_id(session) {
            Some(id) => id,
            None => {
                // No subject on session — unauthenticated, let scope check handle it
                return Ok(());
            }
        };

        // Check room-level paths
        if let Some((room_id, sub_path)) = parse_room_path(address) {
            // /chat/room/{rid}/admin/{uid}
            if sub_path.starts_with("admin/") {
                if !Self::is_room_creator(room_id, writer_id, state) {
                    return Err(format!(
                        "Only the room creator can modify admin roles in room {}",
                        room_id
                    ));
                }
                return Ok(());
            }

            // /chat/room/{rid}/bans/{uid}
            if sub_path.starts_with("bans/") {
                if !Self::is_room_creator(room_id, writer_id, state)
                    && !Self::is_room_admin(room_id, writer_id, state)
                {
                    return Err(format!(
                        "Only the room creator or an admin can manage bans in room {}",
                        room_id
                    ));
                }
                return Ok(());
            }

            // /chat/room/{rid}/meta
            if sub_path == "meta" {
                if !Self::is_room_creator(room_id, writer_id, state) {
                    return Err(format!(
                        "Only the room creator can modify room settings for room {}",
                        room_id
                    ));
                }
                return Ok(());
            }
        }

        // Check namespace meta paths: /chat/registry/ns-meta/{path}
        if let Some(ns_path) = parse_ns_meta_path(address) {
            if !Self::is_ns_creator(ns_path, writer_id, state) {
                return Err(format!(
                    "Only the namespace creator can modify namespace settings for {}",
                    ns_path
                ));
            }
            return Ok(());
        }

        // All other paths: pass through (existing scope check is sufficient)
        Ok(())
    }
}

/// Chat-specific snapshot filter.
///
/// Strips sensitive fields from snapshots:
/// - `passwordHash` and `passwordSalt` from room meta values
/// - `__auth` namespace paths (contain password hashes)
/// - Room-internal paths for rooms the user hasn't joined
pub struct ChatSnapshotFilter;

impl ChatSnapshotFilter {
    /// Check if a user has joined a room (has presence or an active subscription).
    fn user_in_room(
        room_id: &str,
        user_id: &str,
        state: &RouterState,
    ) -> bool {
        let presence_address = format!("/chat/room/{}/presence/{}", room_id, user_id);
        match state.get(&presence_address) {
            Some(Value::Null) => false,
            Some(_) => true,
            None => false,
        }
    }

    /// Strip sensitive fields from a room meta Value.
    fn redact_room_meta(value: clasp_core::ParamValue) -> clasp_core::ParamValue {
        match value.value {
            Value::Map(mut map) => {
                map.remove("passwordHash");
                map.remove("passwordSalt");
                clasp_core::ParamValue {
                    value: Value::Map(map),
                    ..value
                }
            }
            _ => value,
        }
    }
}

impl SnapshotFilter for ChatSnapshotFilter {
    fn filter_snapshot(
        &self,
        params: Vec<clasp_core::ParamValue>,
        session: &Session,
        state: &RouterState,
    ) -> Vec<clasp_core::ParamValue> {
        let user_id = session.subject.as_deref().unwrap_or("");

        params
            .into_iter()
            .filter_map(|pv| {
                // Strip __auth namespace paths entirely
                if pv.address.contains("/__auth") {
                    debug!("Filtering __auth path from snapshot: {}", pv.address);
                    return None;
                }

                // Check room-level paths
                if let Some((room_id, sub_path)) = parse_room_path(&pv.address) {
                    // Always redact password fields from room meta
                    if sub_path == "meta" {
                        return Some(Self::redact_room_meta(pv));
                    }

                    // For room-internal paths (not user identity, not registry),
                    // check if the user is in the room
                    let is_internal = sub_path.starts_with("messages")
                        || sub_path.starts_with("crypto/")
                        || sub_path.starts_with("admin/")
                        || sub_path.starts_with("bans/")
                        || sub_path.starts_with("reactions/")
                        || sub_path.starts_with("video/");

                    if is_internal && !user_id.is_empty() {
                        if !Self::user_in_room(room_id, user_id, state) {
                            debug!(
                                "Filtering non-member room path from snapshot: {}",
                                pv.address
                            );
                            return None;
                        }
                    }
                }

                Some(pv)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_meta_value(creator_id: &str) -> Value {
        let mut map = HashMap::new();
        map.insert("creatorId".to_string(), Value::String(creator_id.to_string()));
        map.insert("name".to_string(), Value::String("Test Room".to_string()));
        Value::Map(map)
    }

    fn make_meta_with_password(creator_id: &str) -> Value {
        let mut map = HashMap::new();
        map.insert("creatorId".to_string(), Value::String(creator_id.to_string()));
        map.insert("name".to_string(), Value::String("Test Room".to_string()));
        map.insert("passwordHash".to_string(), Value::String("secret_hash".to_string()));
        map.insert("passwordSalt".to_string(), Value::String("secret_salt".to_string()));
        Value::Map(map)
    }

    fn make_ns_meta_value(created_by: &str) -> Value {
        let mut map = HashMap::new();
        map.insert("createdBy".to_string(), Value::String(created_by.to_string()));
        map.insert("name".to_string(), Value::String("Test NS".to_string()));
        Value::Map(map)
    }

    #[test]
    fn test_parse_room_path() {
        assert_eq!(
            parse_room_path("/chat/room/abc123/admin/user1"),
            Some(("abc123", "admin/user1"))
        );
        assert_eq!(
            parse_room_path("/chat/room/abc123/meta"),
            Some(("abc123", "meta"))
        );
        assert_eq!(parse_room_path("/chat/user/abc/profile"), None);
    }

    #[test]
    fn test_parse_ns_meta_path() {
        assert_eq!(
            parse_ns_meta_path("/chat/registry/ns-meta/gaming"),
            Some("gaming")
        );
        assert_eq!(
            parse_ns_meta_path("/chat/registry/ns-meta/gaming/__auth"),
            Some("gaming/__auth")
        );
        assert_eq!(parse_ns_meta_path("/chat/room/abc/meta"), None);
    }

    #[test]
    fn test_redact_room_meta() {
        let pv = clasp_core::ParamValue {
            address: "/chat/room/abc/meta".to_string(),
            value: make_meta_with_password("alice"),
            revision: 1,
            writer: Some("session1".to_string()),
            timestamp: Some(0),
        };

        let redacted = ChatSnapshotFilter::redact_room_meta(pv);
        match redacted.value {
            Value::Map(map) => {
                assert!(map.contains_key("creatorId"));
                assert!(map.contains_key("name"));
                assert!(!map.contains_key("passwordHash"));
                assert!(!map.contains_key("passwordSalt"));
            }
            _ => panic!("Expected Map"),
        }
    }
}
