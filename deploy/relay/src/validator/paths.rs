//! Path parsing utilities for chat addresses.
//!
//! Each function extracts structured components from CLASP address strings.

/// Parse a chat room path and extract components.
/// Returns (room_id, sub_path) for paths like `/chat/room/{rid}/admin/...`
pub(crate) fn parse_room_path(address: &str) -> Option<(&str, &str)> {
    let rest = address.strip_prefix("/chat/room/")?;
    let slash_pos = rest.find('/')?;
    let room_id = &rest[..slash_pos];
    let sub_path = &rest[slash_pos + 1..];
    Some((room_id, sub_path))
}

/// Parse a namespace meta path.
/// Returns the namespace path for paths like `/chat/registry/ns-meta/{path}`
pub(crate) fn parse_ns_meta_path(address: &str) -> Option<&str> {
    address.strip_prefix("/chat/registry/ns-meta/")
}

/// Parse DM inbox path: /chat/user/{targetId}/dms/{roomId}
/// Returns (target_id, room_id) if the path matches.
pub(crate) fn parse_dm_inbox_path(address: &str) -> Option<(&str, &str)> {
    let rest = address.strip_prefix("/chat/user/")?;
    let slash = rest.find('/')?;
    let target_id = &rest[..slash];
    let sub = &rest[slash + 1..];
    let room_id = sub.strip_prefix("dms/")?;
    if room_id.is_empty() || room_id.contains('/') {
        return None;
    }
    Some((target_id, room_id))
}

/// Parse friend request path: /chat/requests/{targetId}/{fromId}
/// Returns (target_id, from_id) for 2-segment paths.
pub(crate) fn parse_friend_request_path(address: &str) -> Option<(&str, &str)> {
    let rest = address.strip_prefix("/chat/requests/")?;
    if rest.is_empty() {
        return None;
    }
    let slash = rest.find('/')?;
    let target_id = &rest[..slash];
    let from_id = &rest[slash + 1..];
    if target_id.is_empty() || from_id.is_empty() || from_id.contains('/') {
        return None;
    }
    Some((target_id, from_id))
}
