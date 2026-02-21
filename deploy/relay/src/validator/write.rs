//! Chat-specific write validation.
//!
//! Enforces server-side authorization rules for room admin/ban/meta paths,
//! namespace metadata paths, DM inbox writes, and friend request writes.

use clasp_core::Value;
use clasp_router::{RouterState, Session, WriteValidator};

use super::helpers::{
    are_friends, extract_from_id, is_ns_creator, is_room_admin, is_room_creator, session_user_id,
};
use super::paths::{parse_dm_inbox_path, parse_friend_request_path, parse_ns_meta_path, parse_room_path};

/// Chat-specific write validator.
///
/// Enforces:
/// - `/chat/room/{rid}/admin/*` -- only room creator can write
/// - `/chat/room/{rid}/bans/*`  -- only room creator or admin can write
/// - `/chat/room/{rid}/meta`    -- only room creator can write (or initial creation)
/// - `/chat/registry/ns-meta/**` -- only namespace creator can write (or initial creation)
pub struct ChatWriteValidator;

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
                // No subject on session -- unauthenticated, let scope check handle it
                return Ok(());
            }
        };

        // Check room-level paths
        if let Some((room_id, sub_path)) = parse_room_path(address) {
            // /chat/room/{rid}/admin/{uid}
            if sub_path.starts_with("admin/") {
                if !is_room_creator(room_id, writer_id, state) {
                    return Err(format!(
                        "Only the room creator can modify admin roles in room {}",
                        room_id
                    ));
                }
                return Ok(());
            }

            // /chat/room/{rid}/bans/{uid}
            if sub_path.starts_with("bans/") {
                if !is_room_creator(room_id, writer_id, state)
                    && !is_room_admin(room_id, writer_id, state)
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
                if !is_room_creator(room_id, writer_id, state) {
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
            if !is_ns_creator(ns_path, writer_id, state) {
                return Err(format!(
                    "Only the namespace creator can modify namespace settings for {}",
                    ns_path
                ));
            }
            return Ok(());
        }

        // DM inbox writes: /chat/user/{target}/dms/{roomId}
        if let Some((target_id, _room_id)) = parse_dm_inbox_path(address) {
            // Allow null writes (cleanup/delete) without checks
            if !matches!(_value, Value::Null) {
                // Require fromId field (prevents anonymous/unattributed DMs)
                let from_id = match extract_from_id(_value) {
                    Some(id) => id,
                    None => {
                        return Err(
                            "DM notification must include a fromId field".to_string()
                        );
                    }
                };

                // Enforce fromId matches the session subject (prevents impersonation)
                if from_id != writer_id {
                    return Err(format!(
                        "DM notification fromId '{}' does not match session identity '{}'",
                        from_id, writer_id
                    ));
                }

                // Enforce friendship (prevents unsolicited DMs)
                if !are_friends(writer_id, target_id, state) {
                    return Err(format!(
                        "Cannot send DM to '{}': not friends",
                        target_id
                    ));
                }
            }
            return Ok(());
        }

        // Friend request writes: /chat/requests/{targetId}/{fromId}
        if let Some((_target_id, path_from_id)) = parse_friend_request_path(address) {
            // Path fromId segment must match session subject (prevents writing to others' keys)
            if path_from_id != writer_id {
                return Err(format!(
                    "Friend request path fromId '{}' does not match session identity '{}'",
                    path_from_id, writer_id
                ));
            }

            // Allow null writes (cleanup) without value checks
            if !matches!(_value, Value::Null) {
                // Require fromId field (prevents anonymous/unattributed requests)
                let from_id = match extract_from_id(_value) {
                    Some(id) => id,
                    None => {
                        return Err(
                            "Friend request must include a fromId field".to_string()
                        );
                    }
                };

                // Enforce value fromId matches the session subject (prevents impersonation)
                if from_id != writer_id {
                    return Err(format!(
                        "Friend request fromId '{}' does not match session identity '{}'",
                        from_id, writer_id
                    ));
                }
            }
            return Ok(());
        } else if address.starts_with("/chat/requests/") {
            // Reject old 1-segment paths or malformed request paths
            return Err(
                "Invalid friend request path: must be /chat/requests/{targetId}/{fromId}".to_string()
            );
        }

        // All other paths: pass through (existing scope check is sufficient)
        Ok(())
    }
}
