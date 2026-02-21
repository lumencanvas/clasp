//! Chat-specific snapshot filtering.
//!
//! Strips sensitive fields from snapshots and enforces visibility rules
//! based on room membership and user privacy.

use clasp_core::Value;
use clasp_router::{RouterState, Session, SnapshotFilter};
use tracing::debug;

use super::helpers::user_in_room;
use super::paths::parse_room_path;

/// Chat-specific snapshot filter.
///
/// Strips sensitive fields from snapshots:
/// - `passwordHash` and `passwordSalt` from room meta values
/// - `__auth` namespace paths (contain password hashes)
/// - Room-internal paths for rooms the user hasn't joined
pub struct ChatSnapshotFilter;

impl ChatSnapshotFilter {
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

                // Filter other users' private paths (DMs, friends list)
                if let Some(rest) = pv.address.strip_prefix("/chat/user/") {
                    if let Some(slash) = rest.find('/') {
                        let path_user = &rest[..slash];
                        let sub = &rest[slash + 1..];
                        if path_user != user_id && sub != "profile" {
                            debug!(
                                "Filtering other user's private path from snapshot: {}",
                                pv.address
                            );
                            return None;
                        }
                    }
                }

                // Filter other users' friend requests
                if let Some(rest) = pv.address.strip_prefix("/chat/requests/") {
                    if let Some(slash) = rest.find('/') {
                        let target_id = &rest[..slash];
                        if target_id != user_id {
                            debug!(
                                "Filtering other user's friend request from snapshot: {}",
                                pv.address
                            );
                            return None;
                        }
                    }
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

                    if is_internal {
                        if !user_in_room(room_id, user_id, state) {
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
