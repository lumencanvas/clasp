//! Tests for chat write validation and snapshot filtering.

use super::filter::ChatSnapshotFilter;
use super::helpers::{are_friends, extract_from_id};
use super::paths::{
    parse_dm_inbox_path, parse_friend_request_path, parse_ns_meta_path, parse_room_path,
};
use super::write::ChatWriteValidator;
use clasp_core::Value;
use clasp_router::{RouterState, Session, SnapshotFilter, WriteValidator};
use std::collections::HashMap;

// ---- Test helpers ----

fn make_session(subject: &str) -> Session {
    Session::stub(Some(subject.to_string()))
}

fn make_anonymous_session() -> Session {
    Session::stub(None)
}

fn state_set(state: &RouterState, address: &str, value: Value) {
    let _ = state.set(address, value, &"test".to_string(), None, false, false);
}

fn set_friendship(state: &RouterState, user_a: &str, user_b: &str) {
    state_set(state, &format!("/chat/user/{}/friends/{}", user_a, user_b),
        Value::String("friend".to_string()));
    state_set(state, &format!("/chat/user/{}/friends/{}", user_b, user_a),
        Value::String("friend".to_string()));
}

fn set_one_sided_friendship(state: &RouterState, user_a: &str, user_b: &str) {
    state_set(state, &format!("/chat/user/{}/friends/{}", user_a, user_b),
        Value::String("friend".to_string()));
}

fn make_room_meta(creator_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("creatorId".to_string(), Value::String(creator_id.to_string()));
    map.insert("name".to_string(), Value::String("Test Room".to_string()));
    Value::Map(map)
}

fn make_room_meta_with_password(creator_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("creatorId".to_string(), Value::String(creator_id.to_string()));
    map.insert("name".to_string(), Value::String("Test Room".to_string()));
    map.insert("passwordHash".to_string(), Value::String("secret_hash".to_string()));
    map.insert("passwordSalt".to_string(), Value::String("secret_salt".to_string()));
    Value::Map(map)
}

fn make_ns_meta(created_by: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("createdBy".to_string(), Value::String(created_by.to_string()));
    map.insert("name".to_string(), Value::String("Test NS".to_string()));
    Value::Map(map)
}

fn make_dm_notification(from_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("fromId".to_string(), Value::String(from_id.to_string()));
    map.insert("fromName".to_string(), Value::String("Tester".to_string()));
    map.insert("roomId".to_string(), Value::String("dm-room1".to_string()));
    map.insert("timestamp".to_string(), Value::Int(1000));
    Value::Map(map)
}

fn make_friend_request(from_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("fromId".to_string(), Value::String(from_id.to_string()));
    map.insert("fromName".to_string(), Value::String("Tester".to_string()));
    Value::Map(map)
}

fn make_pv(address: &str, value: Value) -> clasp_core::ParamValue {
    clasp_core::ParamValue {
        address: address.to_string(),
        value,
        revision: 1,
        writer: Some("session1".to_string()),
        timestamp: Some(0),
    }
}

fn validate(address: &str, value: &Value, subject: &str, state: &RouterState) -> Result<(), String> {
    ChatWriteValidator.validate_write(address, value, &make_session(subject), state)
}

fn filter(params: Vec<clasp_core::ParamValue>, subject: &str, state: &RouterState) -> Vec<String> {
    ChatSnapshotFilter.filter_snapshot(params, &make_session(subject), state)
        .into_iter().map(|pv| pv.address).collect()
}

// ===========================================================
//  Path parsers
// ===========================================================

#[test]
fn test_parse_room_path() {
    assert_eq!(parse_room_path("/chat/room/abc123/admin/user1"), Some(("abc123", "admin/user1")));
    assert_eq!(parse_room_path("/chat/room/abc123/meta"), Some(("abc123", "meta")));
    assert_eq!(parse_room_path("/chat/room/r1/messages"), Some(("r1", "messages")));
    // Wrong prefix
    assert_eq!(parse_room_path("/chat/user/abc/profile"), None);
    // No sub-path
    assert_eq!(parse_room_path("/chat/room/abc123"), None);
}

#[test]
fn test_parse_ns_meta_path() {
    assert_eq!(parse_ns_meta_path("/chat/registry/ns-meta/gaming"), Some("gaming"));
    assert_eq!(parse_ns_meta_path("/chat/registry/ns-meta/gaming/__auth"), Some("gaming/__auth"));
    assert_eq!(parse_ns_meta_path("/chat/registry/ns-meta/a/b/c"), Some("a/b/c"));
    assert_eq!(parse_ns_meta_path("/chat/room/abc/meta"), None);
}

#[test]
fn test_parse_dm_inbox_path() {
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/dms/room123"), Some(("alice", "room123")));
    assert_eq!(parse_dm_inbox_path("/chat/user/u-123/dms/dm-abc-def"), Some(("u-123", "dm-abc-def")));
    // Sub-path after room ID
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/dms/room123/extra"), None);
    // Not a DM sub-path
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/friends/bob"), None);
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/profile"), None);
    // Wrong root prefix
    assert_eq!(parse_dm_inbox_path("/chat/room/abc/dms/room123"), None);
    // Empty room ID
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/dms/"), None);
    // Bare user path
    assert_eq!(parse_dm_inbox_path("/chat/user/alice"), None);
}

#[test]
fn test_parse_friend_request_path() {
    // 2-segment: /chat/requests/{targetId}/{fromId}
    assert_eq!(parse_friend_request_path("/chat/requests/alice/bob"), Some(("alice", "bob")));
    assert_eq!(parse_friend_request_path("/chat/requests/u-123/u-456"), Some(("u-123", "u-456")));
    // Missing fromId segment
    assert_eq!(parse_friend_request_path("/chat/requests/alice"), None);
    // Empty target or fromId
    assert_eq!(parse_friend_request_path("/chat/requests/"), None);
    assert_eq!(parse_friend_request_path("/chat/requests/alice/"), None);
    // Extra segment
    assert_eq!(parse_friend_request_path("/chat/requests/alice/bob/extra"), None);
    // Wrong prefix
    assert_eq!(parse_friend_request_path("/chat/user/alice"), None);
    assert_eq!(parse_friend_request_path("/chat/requests"), None);
}

// ===========================================================
//  Friendship logic
// ===========================================================

#[test]
fn test_are_friends_bidirectional() {
    let state = RouterState::new();
    set_friendship(&state, "alice", "bob");
    assert!(are_friends("alice", "bob", &state));
    assert!(are_friends("bob", "alice", &state));
}

#[test]
fn test_are_friends_unilateral_a_to_b() {
    let state = RouterState::new();
    set_one_sided_friendship(&state, "alice", "bob");
    // OR logic: either direction is sufficient
    assert!(are_friends("alice", "bob", &state));
    assert!(are_friends("bob", "alice", &state));
}

#[test]
fn test_are_friends_not_friends() {
    let state = RouterState::new();
    assert!(!are_friends("alice", "bob", &state));
}

#[test]
fn test_are_friends_null_entry_not_counted() {
    let state = RouterState::new();
    state_set(&state, "/chat/user/alice/friends/bob", Value::Null);
    assert!(!are_friends("alice", "bob", &state));
}

#[test]
fn test_are_friends_both_null_not_counted() {
    let state = RouterState::new();
    state_set(&state, "/chat/user/alice/friends/bob", Value::Null);
    state_set(&state, "/chat/user/bob/friends/alice", Value::Null);
    assert!(!are_friends("alice", "bob", &state));
}

// ===========================================================
//  fromId extraction
// ===========================================================

#[test]
fn test_extract_from_id_valid() {
    let val = make_dm_notification("alice");
    assert_eq!(extract_from_id(&val), Some("alice"));
}

#[test]
fn test_extract_from_id_missing_key() {
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String("test".to_string()));
    assert_eq!(extract_from_id(&Value::Map(map)), None);
}

#[test]
fn test_extract_from_id_non_string_value() {
    let mut map = HashMap::new();
    map.insert("fromId".to_string(), Value::Int(42));
    assert_eq!(extract_from_id(&Value::Map(map)), None);
}

#[test]
fn test_extract_from_id_non_map() {
    assert_eq!(extract_from_id(&Value::Null), None);
    assert_eq!(extract_from_id(&Value::String("x".into())), None);
    assert_eq!(extract_from_id(&Value::Int(1)), None);
}

// ===========================================================
//  validate_write -- DM inbox
// ===========================================================

#[test]
fn test_dm_allows_friends() {
    let state = RouterState::new();
    set_friendship(&state, "alice", "bob");
    let val = make_dm_notification("alice");
    assert!(validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).is_ok());
}

#[test]
fn test_dm_rejects_non_friends() {
    let state = RouterState::new();
    let val = make_dm_notification("alice");
    let err = validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(err.contains("not friends"), "expected 'not friends', got: {}", err);
}

#[test]
fn test_dm_rejects_spoofed_from_id() {
    let state = RouterState::new();
    set_friendship(&state, "alice", "bob");
    // alice sends but claims to be charlie
    let spoofed = make_dm_notification("charlie");
    let err = validate("/chat/user/bob/dms/dm-room1", &spoofed, "alice", &state).unwrap_err();
    assert!(err.contains("does not match session identity"), "got: {}", err);
}

#[test]
fn test_dm_null_write_skips_all_checks() {
    let state = RouterState::new();
    // No friendship, but null (cleanup) should pass
    assert!(validate("/chat/user/bob/dms/dm-room1", &Value::Null, "alice", &state).is_ok());
}

#[test]
fn test_dm_allows_unilateral_friendship() {
    let state = RouterState::new();
    set_one_sided_friendship(&state, "bob", "alice"); // only bob->alice exists
    let val = make_dm_notification("alice");
    assert!(validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).is_ok());
}

#[test]
fn test_dm_without_from_id_rejected() {
    let state = RouterState::new();
    // Value with no fromId field -- must be rejected regardless of friendship
    let mut map = HashMap::new();
    map.insert("roomId".to_string(), Value::String("dm-room1".to_string()));
    let val = Value::Map(map);
    let err = validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(err.contains("must include a fromId"), "got: {}", err);
}

// ===========================================================
//  validate_write -- Friend requests
// ===========================================================

#[test]
fn test_friend_request_allows_valid_from_id() {
    let state = RouterState::new();
    let val = make_friend_request("alice");
    // 2-segment path: /chat/requests/{targetId}/{fromId}
    assert!(validate("/chat/requests/bob/alice", &val, "alice", &state).is_ok());
}

#[test]
fn test_friend_request_rejects_spoofed_from_id() {
    let state = RouterState::new();
    let spoofed = make_friend_request("charlie");
    // Path fromId matches session, but value fromId doesn't
    let err = validate("/chat/requests/bob/alice", &spoofed, "alice", &state).unwrap_err();
    assert!(err.contains("does not match session identity"), "got: {}", err);
}

#[test]
fn test_friend_request_rejects_wrong_path_from_id() {
    let state = RouterState::new();
    let val = make_friend_request("alice");
    // Path fromId doesn't match session subject
    let err = validate("/chat/requests/bob/charlie", &val, "alice", &state).unwrap_err();
    assert!(err.contains("does not match session identity"), "got: {}", err);
}

#[test]
fn test_friend_request_null_write_allowed() {
    let state = RouterState::new();
    // Null cleanup: path fromId must still match session
    assert!(validate("/chat/requests/bob/alice", &Value::Null, "alice", &state).is_ok());
}

#[test]
fn test_friend_request_null_write_wrong_path_rejected() {
    let state = RouterState::new();
    // Null cleanup with wrong path fromId
    let err = validate("/chat/requests/bob/charlie", &Value::Null, "alice", &state).unwrap_err();
    assert!(err.contains("does not match session identity"), "got: {}", err);
}

#[test]
fn test_friend_request_without_from_id_rejected() {
    let state = RouterState::new();
    // A map without fromId -- must be rejected
    let mut map = HashMap::new();
    map.insert("message".to_string(), Value::String("hi".to_string()));
    let val = Value::Map(map);
    let err = validate("/chat/requests/bob/alice", &val, "alice", &state).unwrap_err();
    assert!(err.contains("must include a fromId"), "got: {}", err);
}

#[test]
fn test_friend_request_rejects_1_segment_path() {
    let state = RouterState::new();
    let val = make_friend_request("alice");
    let err = validate("/chat/requests/bob", &val, "alice", &state).unwrap_err();
    assert!(err.contains("Invalid friend request path"), "got: {}", err);
}

#[test]
fn test_friend_request_rejects_1_segment_null() {
    let state = RouterState::new();
    let err = validate("/chat/requests/bob", &Value::Null, "alice", &state).unwrap_err();
    assert!(err.contains("Invalid friend request path"), "got: {}", err);
}

// ===========================================================
//  validate_write -- Room admin/bans/meta (pre-existing logic)
// ===========================================================

#[test]
fn test_room_admin_creator_allowed() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    let val = Value::String("admin".to_string());
    assert!(validate("/chat/room/r1/admin/bob", &val, "alice", &state).is_ok());
}

#[test]
fn test_room_admin_non_creator_rejected() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    let val = Value::String("admin".to_string());
    let err = validate("/chat/room/r1/admin/bob", &val, "eve", &state).unwrap_err();
    assert!(err.contains("Only the room creator"));
}

#[test]
fn test_room_admin_initial_creation_allowed() {
    let state = RouterState::new();
    // No meta yet -- initial creation, any user allowed
    let val = Value::String("admin".to_string());
    assert!(validate("/chat/room/r1/admin/alice", &val, "alice", &state).is_ok());
}

#[test]
fn test_room_bans_creator_allowed() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    let val = Value::String("banned".to_string());
    assert!(validate("/chat/room/r1/bans/eve", &val, "alice", &state).is_ok());
}

#[test]
fn test_room_bans_admin_allowed() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    state_set(&state, "/chat/room/r1/admin/bob", Value::String("admin".to_string()));
    let val = Value::String("banned".to_string());
    assert!(validate("/chat/room/r1/bans/eve", &val, "bob", &state).is_ok());
}

#[test]
fn test_room_bans_regular_user_rejected() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    let val = Value::String("banned".to_string());
    let err = validate("/chat/room/r1/bans/eve", &val, "eve", &state).unwrap_err();
    assert!(err.contains("Only the room creator or an admin"));
}

#[test]
fn test_room_meta_creator_allowed() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    assert!(validate("/chat/room/r1/meta", &make_room_meta("alice"), "alice", &state).is_ok());
}

#[test]
fn test_room_meta_non_creator_rejected() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    let err = validate("/chat/room/r1/meta", &make_room_meta("alice"), "eve", &state).unwrap_err();
    assert!(err.contains("Only the room creator"));
}

// ===========================================================
//  validate_write -- Namespace meta (pre-existing logic)
// ===========================================================

#[test]
fn test_ns_meta_creator_allowed() {
    let state = RouterState::new();
    state_set(&state, "/chat/registry/ns-meta/gaming", make_ns_meta("alice"));
    assert!(validate("/chat/registry/ns-meta/gaming", &make_ns_meta("alice"), "alice", &state).is_ok());
}

#[test]
fn test_ns_meta_non_creator_rejected() {
    let state = RouterState::new();
    state_set(&state, "/chat/registry/ns-meta/gaming", make_ns_meta("alice"));
    let err = validate("/chat/registry/ns-meta/gaming", &make_ns_meta("alice"), "eve", &state).unwrap_err();
    assert!(err.contains("Only the namespace creator"));
}

#[test]
fn test_ns_meta_initial_creation_allowed() {
    let state = RouterState::new();
    assert!(validate("/chat/registry/ns-meta/gaming", &make_ns_meta("alice"), "alice", &state).is_ok());
}

#[test]
fn test_ns_auth_path_checks_base_creator() {
    let state = RouterState::new();
    state_set(&state, "/chat/registry/ns-meta/gaming", make_ns_meta("alice"));
    // Writing to gaming/__auth should check gaming's creator
    let val = Value::String("hash".to_string());
    assert!(validate("/chat/registry/ns-meta/gaming/__auth", &val, "alice", &state).is_ok());
    assert!(validate("/chat/registry/ns-meta/gaming/__auth", &val, "eve", &state).is_err());
}

// ===========================================================
//  validate_write -- Anonymous session (no subject)
// ===========================================================

#[test]
fn test_anonymous_session_passes_through() {
    let state = RouterState::new();
    let session = make_anonymous_session();
    // No subject => no writer_id => early return Ok
    let val = Value::String("test".to_string());
    assert!(ChatWriteValidator.validate_write("/chat/user/x/dms/room1", &val, &session, &state).is_ok());
    assert!(ChatWriteValidator.validate_write("/chat/room/r1/admin/x", &val, &session, &state).is_ok());
}

// ===========================================================
//  validate_write -- Passthrough paths
// ===========================================================

#[test]
fn test_passthrough_paths_allowed() {
    let state = RouterState::new();
    let val = Value::String("test".to_string());
    // Messages, presence, typing, reactions -- all pass through
    assert!(validate("/chat/room/r1/messages", &val, "alice", &state).is_ok());
    assert!(validate("/chat/room/r1/presence/alice", &val, "alice", &state).is_ok());
    assert!(validate("/chat/room/r1/typing/alice", &val, "alice", &state).is_ok());
    assert!(validate("/chat/room/r1/reactions/msg1/alice", &val, "alice", &state).is_ok());
    assert!(validate("/chat/user/alice/profile", &val, "alice", &state).is_ok());
    assert!(validate("/chat/registry/rooms/r1", &val, "alice", &state).is_ok());
}

// ===========================================================
//  Snapshot filter -- Password redaction
// ===========================================================

#[test]
fn test_snapshot_redacts_room_meta_passwords() {
    let state = RouterState::new();
    let params = vec![make_pv("/chat/room/r1/meta", make_room_meta_with_password("alice"))];
    let result = ChatSnapshotFilter.filter_snapshot(params, &make_session("alice"), &state);
    assert_eq!(result.len(), 1);
    match &result[0].value {
        Value::Map(map) => {
            assert!(map.contains_key("creatorId"));
            assert!(map.contains_key("name"));
            assert!(!map.contains_key("passwordHash"), "passwordHash leaked");
            assert!(!map.contains_key("passwordSalt"), "passwordSalt leaked");
        }
        _ => panic!("Expected Map"),
    }
}

// ===========================================================
//  Snapshot filter -- __auth stripping
// ===========================================================

#[test]
fn test_snapshot_strips_auth_paths() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/registry/ns-meta/gaming/__auth", Value::String("hash".into())),
        make_pv("/chat/registry/ns-meta/gaming", make_ns_meta("alice")),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/registry/ns-meta/gaming"]);
}

// ===========================================================
//  Snapshot filter -- User privacy
// ===========================================================

#[test]
fn test_snapshot_allows_own_private_paths() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/alice/profile", Value::String("pub".into())),
        make_pv("/chat/user/alice/friends/bob", Value::String("friend".into())),
        make_pv("/chat/user/alice/dms/dm-room1", make_dm_notification("bob")),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses.len(), 3, "own paths should all pass: {:?}", addresses);
}

#[test]
fn test_snapshot_allows_other_user_profile() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/user/bob/profile"]);
}

#[test]
fn test_snapshot_strips_other_user_dms() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/bob/dms/dm-room1", make_dm_notification("charlie")),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(addresses.is_empty(), "victim DMs leaked: {:?}", addresses);
}

#[test]
fn test_snapshot_strips_other_user_friends() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/bob/friends/charlie", Value::String("friend".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(addresses.is_empty(), "victim friends leaked: {:?}", addresses);
}

#[test]
fn test_snapshot_strips_other_user_all_private_paths() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
        make_pv("/chat/user/bob/friends/charlie", Value::String("f".into())),
        make_pv("/chat/user/bob/dms/dm1", make_dm_notification("charlie")),
        make_pv("/chat/user/bob/settings", Value::String("dark".into())),
    ];
    let addresses = filter(params, "alice", &state);
    // Only bob's profile should survive
    assert_eq!(addresses, vec!["/chat/user/bob/profile"]);
}

// ===========================================================
//  Snapshot filter -- Room membership gating
// ===========================================================

#[test]
fn test_snapshot_allows_room_internal_for_members() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/presence/alice", Value::String("online".into()));
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        make_pv("/chat/room/r1/crypto/pubkey/alice", Value::String("key".into())),
        make_pv("/chat/room/r1/admin/alice", Value::String("admin".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses.len(), 3);
}

#[test]
fn test_snapshot_strips_room_internal_for_non_members() {
    let state = RouterState::new();
    // alice has no presence in r1
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        make_pv("/chat/room/r1/crypto/pubkey/bob", Value::String("key".into())),
        make_pv("/chat/room/r1/admin/bob", Value::String("admin".into())),
        make_pv("/chat/room/r1/bans/eve", Value::String("ban".into())),
        make_pv("/chat/room/r1/reactions/msg1/bob", Value::String(":+1:".into())),
        make_pv("/chat/room/r1/video/bob", Value::String("stream".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(addresses.is_empty(), "non-member room data leaked: {:?}", addresses);
}

#[test]
fn test_snapshot_always_includes_room_meta() {
    let state = RouterState::new();
    // alice is NOT in the room, but meta should still be included (for discovery)
    let params = vec![
        make_pv("/chat/room/r1/meta", make_room_meta("bob")),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/room/r1/meta"]);
}

#[test]
fn test_snapshot_allows_room_presence_and_typing_for_non_members() {
    let state = RouterState::new();
    // presence and typing are not in the "is_internal" list, so they pass through
    let params = vec![
        make_pv("/chat/room/r1/presence/bob", Value::String("online".into())),
        make_pv("/chat/room/r1/typing/bob", Value::String("true".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses.len(), 2);
}

// ===========================================================
//  Snapshot filter -- Anonymous session (S2)
// ===========================================================

#[test]
fn test_snapshot_anonymous_session_strips_all_user_private_data() {
    let state = RouterState::new();
    let session = make_anonymous_session(); // subject = None -> user_id = ""
    let params = vec![
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
        make_pv("/chat/user/bob/dms/dm1", make_dm_notification("charlie")),
        make_pv("/chat/user/bob/friends/charlie", Value::String("f".into())),
    ];
    let result = ChatSnapshotFilter.filter_snapshot(params, &session, &state);
    let addresses: Vec<&str> = result.iter().map(|pv| pv.address.as_str()).collect();
    // Anonymous user_id is "", which != "bob", so non-profile paths are stripped
    assert!(addresses.contains(&"/chat/user/bob/profile"));
    assert!(!addresses.contains(&"/chat/user/bob/dms/dm1"), "anonymous sees DMs");
    assert!(!addresses.contains(&"/chat/user/bob/friends/charlie"), "anonymous sees friends");
}

#[test]
fn test_snapshot_anonymous_session_room_internal_filtered() {
    // Fixed: anonymous sessions now get room membership filtering applied
    let state = RouterState::new();
    let session = make_anonymous_session();
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        make_pv("/chat/room/r1/crypto/pubkey/bob", Value::String("key".into())),
    ];
    let result = ChatSnapshotFilter.filter_snapshot(params, &session, &state);
    // Anonymous user_id is "" which has no presence in any room
    assert!(result.is_empty(), "anonymous sessions should not see room internals");
}

// ===========================================================
//  DM -- Missing fromId with friendship (S3)
// ===========================================================

#[test]
fn test_dm_without_from_id_rejected_even_when_friends() {
    // Fixed: payload without fromId is now rejected regardless of friendship
    let state = RouterState::new();
    set_friendship(&state, "alice", "bob");
    let mut map = HashMap::new();
    map.insert("roomId".to_string(), Value::String("dm-room1".to_string()));
    // No fromId field at all
    let val = Value::Map(map);
    let err = validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(err.contains("must include a fromId"), "got: {}", err);
}

// ===========================================================
//  __auth false positive (S4)
// ===========================================================

#[test]
fn test_snapshot_auth_substring_in_path() {
    // S4: contains("/__auth") could false-positive on creative paths
    let state = RouterState::new();
    let params = vec![
        // This path contains /__auth as a segment -- should be stripped
        make_pv("/chat/registry/ns-meta/test/__auth", Value::String("hash".into())),
        // This does NOT contain /__auth -- should survive
        make_pv("/chat/registry/ns-meta/test_auth_config", Value::String("ok".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(!addresses.iter().any(|a| a.contains("/__auth")), "/__auth path leaked");
    assert!(addresses.contains(&"/chat/registry/ns-meta/test_auth_config".to_string()),
        "non-__auth path was falsely stripped");
}

// ===========================================================
//  fromId check ordering (S5)
// ===========================================================

#[test]
fn test_dm_spoofed_from_id_when_not_friends() {
    // S5: verify fromId check fires BEFORE friendship check
    // (don't leak friendship status to attacker)
    let state = RouterState::new();
    let spoofed = make_dm_notification("charlie"); // fromId=charlie, session=alice
    let err = validate("/chat/user/bob/dms/dm-room1", &spoofed, "alice", &state).unwrap_err();
    // Should get fromId error, not friendship error
    assert!(err.contains("does not match session identity"),
        "expected fromId error first, got: {}", err);
}

// ===========================================================
//  DM to self (B1)
// ===========================================================

#[test]
fn test_dm_to_self_rejected() {
    // B1: writing to own DM inbox -- are_friends("alice", "alice") is false
    let state = RouterState::new();
    let val = make_dm_notification("alice");
    let err = validate("/chat/user/alice/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(err.contains("not friends"), "DM to self should fail: {}", err);
}

// ===========================================================
//  Null presence blocks room data (B2)
// ===========================================================

#[test]
fn test_snapshot_null_presence_blocks_room_internal() {
    // B2: explicit Null presence should be treated as "not in room"
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/presence/alice", Value::Null);
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(addresses.is_empty(), "null presence should not grant room access");
}

// ===========================================================
//  Corrupted room meta (B3)
// ===========================================================

#[test]
fn test_room_admin_with_corrupted_meta_rejects() {
    // B3: meta exists but has no creatorId field
    let state = RouterState::new();
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String("broken room".to_string()));
    state_set(&state, "/chat/room/r1/meta", Value::Map(map));
    let val = Value::String("admin".to_string());
    let err = validate("/chat/room/r1/admin/bob", &val, "eve", &state).unwrap_err();
    assert!(err.contains("Only the room creator"),
        "corrupted meta should reject everyone: {}", err);
}

#[test]
fn test_room_admin_with_non_map_meta_rejects() {
    // B3: meta exists but is a string, not a map
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", Value::String("corrupted".to_string()));
    let val = Value::String("admin".to_string());
    let err = validate("/chat/room/r1/admin/bob", &val, "eve", &state).unwrap_err();
    assert!(err.contains("Only the room creator"));
}

// ===========================================================
//  are_friends with non-string values (Q2)
// ===========================================================

#[test]
fn test_are_friends_non_string_values_count() {
    let state = RouterState::new();
    // Int value -- not Null, so counts as friendship
    state_set(&state, "/chat/user/alice/friends/bob", Value::Int(1));
    assert!(are_friends("alice", "bob", &state));
}

#[test]
fn test_are_friends_false_bool_counts() {
    let state = RouterState::new();
    // Bool(false) -- not Null, so counts as friendship under current logic
    state_set(&state, "/chat/user/alice/friends/bob", Value::Bool(false));
    assert!(are_friends("alice", "bob", &state),
        "any non-Null value should count as friendship");
}

#[test]
fn test_are_friends_empty_string_counts() {
    let state = RouterState::new();
    state_set(&state, "/chat/user/alice/friends/bob", Value::String("".to_string()));
    assert!(are_friends("alice", "bob", &state),
        "empty string is not Null, counts as friendship");
}

// ===========================================================
//  Path traversal in DM paths (Q3)
// ===========================================================

#[test]
fn test_parse_dm_inbox_path_traversal_rejected() {
    // Attempt to break out of DM path
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/dms/../../admin"), None);
    assert_eq!(parse_dm_inbox_path("/chat/user/alice/dms/room1/../../../etc"), None);
}

// ===========================================================
//  Snapshot preserves values through filter (Q1)
// ===========================================================

#[test]
fn test_snapshot_preserves_values_and_metadata() {
    let state = RouterState::new();
    let pv = clasp_core::ParamValue {
        address: "/chat/user/alice/friends/bob".to_string(),
        value: Value::String("best-friend".to_string()),
        revision: 42,
        writer: Some("session-xyz".to_string()),
        timestamp: Some(12345),
    };
    let result = ChatSnapshotFilter.filter_snapshot(vec![pv], &make_session("alice"), &state);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, Value::String("best-friend".to_string()));
    assert_eq!(result[0].revision, 42);
    assert_eq!(result[0].writer, Some("session-xyz".to_string()));
    assert_eq!(result[0].timestamp, Some(12345));
}

// ===========================================================
//  Snapshot filter -- Friend request privacy
// ===========================================================

#[test]
fn test_snapshot_allows_own_friend_requests() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/requests/alice/bob", make_friend_request("bob")),
        make_pv("/chat/requests/alice/charlie", make_friend_request("charlie")),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses.len(), 2, "own requests should pass: {:?}", addresses);
}

#[test]
fn test_snapshot_strips_other_users_friend_requests() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/requests/bob/alice", make_friend_request("alice")),
        make_pv("/chat/requests/charlie/alice", make_friend_request("alice")),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(addresses.is_empty(), "other users' requests leaked: {:?}", addresses);
}

#[test]
fn test_snapshot_mixed_friend_requests() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/requests/alice/bob", make_friend_request("bob")),
        make_pv("/chat/requests/bob/alice", make_friend_request("alice")),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/requests/alice/bob"]);
}

#[test]
fn test_snapshot_anonymous_session_blocks_all_friend_requests() {
    let state = RouterState::new();
    let session = make_anonymous_session();
    let params = vec![
        make_pv("/chat/requests/alice/bob", make_friend_request("bob")),
        make_pv("/chat/requests/bob/alice", make_friend_request("alice")),
    ];
    let result = ChatSnapshotFilter.filter_snapshot(params, &session, &state);
    assert!(result.is_empty(), "anonymous session should not see any friend requests");
}

// ===========================================================
//  Snapshot filter -- Mixed scenario (full snapshot)
// ===========================================================

#[test]
fn test_snapshot_full_scenario() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/presence/alice", Value::String("online".into()));

    let params = vec![
        // Own data -- keep all
        make_pv("/chat/user/alice/profile", Value::String("pub".into())),
        make_pv("/chat/user/alice/friends/bob", Value::String("f".into())),
        make_pv("/chat/user/alice/dms/dm1", make_dm_notification("bob")),
        // Other user -- keep only profile
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
        make_pv("/chat/user/bob/friends/charlie", Value::String("f".into())),
        make_pv("/chat/user/bob/dms/dm2", make_dm_notification("charlie")),
        // Room alice is in -- keep everything
        make_pv("/chat/room/r1/meta", make_room_meta_with_password("alice")),
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        // Room alice is NOT in -- keep only meta
        make_pv("/chat/room/r2/meta", make_room_meta("bob")),
        make_pv("/chat/room/r2/messages", Value::String("secret".into())),
        // __auth -- strip
        make_pv("/chat/registry/ns-meta/g/__auth", Value::String("hash".into())),
        // Registry -- keep
        make_pv("/chat/registry/rooms/r1", Value::String("room".into())),
    ];

    let result = ChatSnapshotFilter.filter_snapshot(params, &make_session("alice"), &state);
    let addresses: Vec<&str> = result.iter().map(|pv| pv.address.as_str()).collect();

    assert!(addresses.contains(&"/chat/user/alice/profile"));
    assert!(addresses.contains(&"/chat/user/alice/friends/bob"));
    assert!(addresses.contains(&"/chat/user/alice/dms/dm1"));
    assert!(addresses.contains(&"/chat/user/bob/profile"));
    assert!(!addresses.contains(&"/chat/user/bob/friends/charlie"), "victim friends leaked");
    assert!(!addresses.contains(&"/chat/user/bob/dms/dm2"), "victim DMs leaked");
    assert!(addresses.contains(&"/chat/room/r1/meta"));
    assert!(addresses.contains(&"/chat/room/r1/messages"));
    assert!(addresses.contains(&"/chat/room/r2/meta"));
    assert!(!addresses.contains(&"/chat/room/r2/messages"), "non-member room data leaked");
    assert!(!addresses.contains(&"/chat/registry/ns-meta/g/__auth"), "__auth leaked");
    assert!(addresses.contains(&"/chat/registry/rooms/r1"));

    // Verify password fields were redacted from room meta
    let r1_meta = result.iter().find(|pv| pv.address == "/chat/room/r1/meta").unwrap();
    match &r1_meta.value {
        Value::Map(map) => {
            assert!(!map.contains_key("passwordHash"), "passwordHash leaked in snapshot");
            assert!(!map.contains_key("passwordSalt"), "passwordSalt leaked in snapshot");
        }
        _ => panic!("Expected Map for room meta"),
    }
}
