//! Helper functions for chat validation.
//!
//! Shared utilities for looking up room creators, admins, friendships,
//! and extracting fields from CLASP values.

use clasp_core::Value;
use clasp_router::{RouterState, Session};

/// Extract the user_id from a session's subject field (set during HELLO from the token).
pub(crate) fn session_user_id(session: &Session) -> Option<&str> {
    session.subject.as_deref()
}

/// Extract the `creatorId` field from a Value::Map.
pub(crate) fn extract_creator_id(value: &Value) -> Option<&str> {
    match value {
        Value::Map(map) => map.get("creatorId").and_then(|v| v.as_str()),
        _ => None,
    }
}

/// Extract the `fromId` field from a Value::Map.
pub(crate) fn extract_from_id(value: &Value) -> Option<&str> {
    match value {
        Value::Map(map) => map.get("fromId").and_then(|v| v.as_str()),
        _ => None,
    }
}

/// Check if the writer is the room creator by looking up the room meta.
pub(crate) fn is_room_creator(
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
            // No meta exists -- this is initial room creation, allow it
            true
        }
    }
}

/// Check if the writer is an admin of the room.
pub(crate) fn is_room_admin(
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

/// Check if two users are friends (bidirectional -- OR logic).
pub(crate) fn are_friends(user_a: &str, user_b: &str, state: &RouterState) -> bool {
    let path_a = format!("/chat/user/{}/friends/{}", user_a, user_b);
    let path_b = format!("/chat/user/{}/friends/{}", user_b, user_a);
    matches!(state.get(&path_a), Some(v) if !matches!(v, Value::Null))
        || matches!(state.get(&path_b), Some(v) if !matches!(v, Value::Null))
}

/// Check if the writer is the namespace creator.
pub(crate) fn is_ns_creator(
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
            // No meta exists -- this is initial namespace creation, allow it
            true
        }
    }
}

/// Check if a user has joined a room (has presence or an active subscription).
pub(crate) fn user_in_room(
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
