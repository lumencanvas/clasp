//! Tests for the declarative app config rule engine.
//!
//! Loads the chat.json config and tests the rule-based validators against
//! the same scenarios the old hardcoded validator tests covered.

use clasp_core::Value;
use clasp_relay::app_config::{AppConfig, RuleSnapshotFilter, RuleWriteValidator};
use clasp_router::{RouterState, Session, SnapshotFilter, WriteValidator};
use std::collections::HashMap;

fn load_chat_config() -> AppConfig {
    let json = include_str!("../config/chat.json");
    serde_json::from_str(json).expect("Failed to parse chat.json")
}

fn make_write_validator() -> RuleWriteValidator {
    let config = load_chat_config();
    RuleWriteValidator::new(config.write_rules)
}

fn make_snapshot_filter() -> RuleSnapshotFilter {
    let config = load_chat_config();
    RuleSnapshotFilter::new(config.snapshot_transforms, config.snapshot_visibility)
}

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
    state_set(
        state,
        &format!("/chat/user/{}/friends/{}", user_a, user_b),
        Value::String("friend".to_string()),
    );
    state_set(
        state,
        &format!("/chat/user/{}/friends/{}", user_b, user_a),
        Value::String("friend".to_string()),
    );
}

fn set_one_sided_friendship(state: &RouterState, user_a: &str, user_b: &str) {
    state_set(
        state,
        &format!("/chat/user/{}/friends/{}", user_a, user_b),
        Value::String("friend".to_string()),
    );
}

fn make_room_meta(creator_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "creatorId".to_string(),
        Value::String(creator_id.to_string()),
    );
    map.insert("name".to_string(), Value::String("Test Room".to_string()));
    Value::Map(map)
}

fn make_room_meta_with_password(creator_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "creatorId".to_string(),
        Value::String(creator_id.to_string()),
    );
    map.insert("name".to_string(), Value::String("Test Room".to_string()));
    map.insert(
        "passwordHash".to_string(),
        Value::String("secret_hash".to_string()),
    );
    map.insert(
        "passwordSalt".to_string(),
        Value::String("secret_salt".to_string()),
    );
    Value::Map(map)
}

fn make_ns_meta(created_by: &str) -> Value {
    let mut map = HashMap::new();
    map.insert(
        "createdBy".to_string(),
        Value::String(created_by.to_string()),
    );
    map.insert("name".to_string(), Value::String("Test NS".to_string()));
    Value::Map(map)
}

fn make_dm_notification(from_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("fromId".to_string(), Value::String(from_id.to_string()));
    map.insert(
        "fromName".to_string(),
        Value::String("Tester".to_string()),
    );
    map.insert(
        "roomId".to_string(),
        Value::String("dm-room1".to_string()),
    );
    map.insert("timestamp".to_string(), Value::Int(1000));
    Value::Map(map)
}

fn make_friend_request(from_id: &str) -> Value {
    let mut map = HashMap::new();
    map.insert("fromId".to_string(), Value::String(from_id.to_string()));
    map.insert(
        "fromName".to_string(),
        Value::String("Tester".to_string()),
    );
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

fn validate(
    address: &str,
    value: &Value,
    subject: &str,
    state: &RouterState,
) -> Result<(), String> {
    make_write_validator().validate_write(address, value, &make_session(subject), state)
}

fn filter(params: Vec<clasp_core::ParamValue>, subject: &str, state: &RouterState) -> Vec<String> {
    make_snapshot_filter()
        .filter_snapshot(params, &make_session(subject), state)
        .into_iter()
        .map(|pv| pv.address)
        .collect()
}

// ===========================================================
//  Config parsing
// ===========================================================

#[test]
fn test_chat_config_parses() {
    let config = load_chat_config();
    assert_eq!(config.scopes.len(), 22);
    assert!(!config.write_rules.is_empty());
    assert!(!config.snapshot_transforms.is_empty());
    assert!(!config.snapshot_visibility.is_empty());
    assert!(config.rate_limits.is_some());
}

#[test]
fn test_scope_templates() {
    let config = load_chat_config();
    let scopes: Vec<String> = config
        .scopes
        .iter()
        .map(|s| s.replace("{userId}", "alice"))
        .collect();
    assert!(scopes.contains(&"read:/chat/user/alice/**".to_string()));
    assert!(scopes.contains(&"write:/chat/user/alice/**".to_string()));
    assert!(scopes.contains(&"write:/chat/room/*/presence/alice".to_string()));
    assert!(scopes.contains(&"read:/chat/room/**".to_string()));
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
    assert!(
        err.contains("exists in state"),
        "expected friendship error, got: {}",
        err
    );
}

#[test]
fn test_dm_rejects_spoofed_from_id() {
    let state = RouterState::new();
    set_friendship(&state, "alice", "bob");
    let spoofed = make_dm_notification("charlie");
    let err = validate("/chat/user/bob/dms/dm-room1", &spoofed, "alice", &state).unwrap_err();
    assert!(
        err.contains("does not match session identity"),
        "got: {}",
        err
    );
}

#[test]
fn test_dm_null_write_skips_all_checks() {
    let state = RouterState::new();
    assert!(validate("/chat/user/bob/dms/dm-room1", &Value::Null, "alice", &state).is_ok());
}

#[test]
fn test_dm_allows_unilateral_friendship() {
    let state = RouterState::new();
    set_one_sided_friendship(&state, "bob", "alice");
    let val = make_dm_notification("alice");
    assert!(validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).is_ok());
}

#[test]
fn test_dm_without_from_id_rejected() {
    let state = RouterState::new();
    let mut map = HashMap::new();
    map.insert(
        "roomId".to_string(),
        Value::String("dm-room1".to_string()),
    );
    let val = Value::Map(map);
    let err = validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(
        err.contains("must include a fromId"),
        "got: {}",
        err
    );
}

#[test]
fn test_dm_without_from_id_rejected_even_when_friends() {
    let state = RouterState::new();
    set_friendship(&state, "alice", "bob");
    let mut map = HashMap::new();
    map.insert(
        "roomId".to_string(),
        Value::String("dm-room1".to_string()),
    );
    let val = Value::Map(map);
    let err = validate("/chat/user/bob/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(
        err.contains("must include a fromId"),
        "got: {}",
        err
    );
}

#[test]
fn test_dm_spoofed_from_id_when_not_friends() {
    let state = RouterState::new();
    let spoofed = make_dm_notification("charlie");
    let err = validate("/chat/user/bob/dms/dm-room1", &spoofed, "alice", &state).unwrap_err();
    // fromId check must fire BEFORE friendship check
    assert!(
        err.contains("does not match session identity"),
        "expected fromId error first, got: {}",
        err
    );
}

#[test]
fn test_dm_to_self_rejected() {
    let state = RouterState::new();
    let val = make_dm_notification("alice");
    let err = validate("/chat/user/alice/dms/dm-room1", &val, "alice", &state).unwrap_err();
    assert!(
        err.contains("exists in state"),
        "DM to self should fail: {}",
        err
    );
}

// ===========================================================
//  validate_write -- Friend requests
// ===========================================================

#[test]
fn test_friend_request_allows_valid_from_id() {
    let state = RouterState::new();
    let val = make_friend_request("alice");
    assert!(validate("/chat/requests/bob/alice", &val, "alice", &state).is_ok());
}

#[test]
fn test_friend_request_rejects_spoofed_from_id() {
    let state = RouterState::new();
    let spoofed = make_friend_request("charlie");
    let err = validate("/chat/requests/bob/alice", &spoofed, "alice", &state).unwrap_err();
    assert!(
        err.contains("does not match session identity"),
        "got: {}",
        err
    );
}

#[test]
fn test_friend_request_rejects_wrong_path_from_id() {
    let state = RouterState::new();
    let val = make_friend_request("alice");
    let err = validate("/chat/requests/bob/charlie", &val, "alice", &state).unwrap_err();
    assert!(
        err.contains("does not match session identity"),
        "got: {}",
        err
    );
}

#[test]
fn test_friend_request_null_write_allowed() {
    let state = RouterState::new();
    assert!(validate("/chat/requests/bob/alice", &Value::Null, "alice", &state).is_ok());
}

#[test]
fn test_friend_request_null_write_wrong_path_rejected() {
    let state = RouterState::new();
    let err = validate("/chat/requests/bob/charlie", &Value::Null, "alice", &state).unwrap_err();
    assert!(
        err.contains("does not match session identity"),
        "got: {}",
        err
    );
}

#[test]
fn test_friend_request_without_from_id_rejected() {
    let state = RouterState::new();
    let mut map = HashMap::new();
    map.insert(
        "message".to_string(),
        Value::String("hi".to_string()),
    );
    let val = Value::Map(map);
    let err = validate("/chat/requests/bob/alice", &val, "alice", &state).unwrap_err();
    assert!(
        err.contains("must include a fromId"),
        "got: {}",
        err
    );
}

#[test]
fn test_friend_request_rejects_1_segment_path() {
    let state = RouterState::new();
    let val = make_friend_request("alice");
    let err = validate("/chat/requests/bob", &val, "alice", &state).unwrap_err();
    assert!(
        err.contains("Invalid friend request path"),
        "got: {}",
        err
    );
}

#[test]
fn test_friend_request_rejects_1_segment_null() {
    let state = RouterState::new();
    let err = validate("/chat/requests/bob", &Value::Null, "alice", &state).unwrap_err();
    assert!(
        err.contains("Invalid friend request path"),
        "got: {}",
        err
    );
}

// ===========================================================
//  validate_write -- Room admin/bans/meta
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
    assert!(err.contains("does not match session identity"));
}

#[test]
fn test_room_admin_initial_creation_allowed() {
    let state = RouterState::new();
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
    state_set(
        &state,
        "/chat/room/r1/admin/bob",
        Value::String("admin".to_string()),
    );
    let val = Value::String("banned".to_string());
    assert!(validate("/chat/room/r1/bans/eve", &val, "bob", &state).is_ok());
}

#[test]
fn test_room_bans_regular_user_rejected() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    let val = Value::String("banned".to_string());
    assert!(validate("/chat/room/r1/bans/eve", &val, "eve", &state).is_err());
}

#[test]
fn test_room_meta_creator_allowed() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    assert!(
        validate("/chat/room/r1/meta", &make_room_meta("alice"), "alice", &state).is_ok()
    );
}

#[test]
fn test_room_meta_non_creator_rejected() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/meta", make_room_meta("alice"));
    assert!(
        validate("/chat/room/r1/meta", &make_room_meta("alice"), "eve", &state).is_err()
    );
}

// ===========================================================
//  validate_write -- Namespace meta
// ===========================================================

#[test]
fn test_ns_meta_creator_allowed() {
    let state = RouterState::new();
    state_set(
        &state,
        "/chat/registry/ns-meta/gaming",
        make_ns_meta("alice"),
    );
    assert!(
        validate(
            "/chat/registry/ns-meta/gaming",
            &make_ns_meta("alice"),
            "alice",
            &state
        )
        .is_ok()
    );
}

#[test]
fn test_ns_meta_non_creator_rejected() {
    let state = RouterState::new();
    state_set(
        &state,
        "/chat/registry/ns-meta/gaming",
        make_ns_meta("alice"),
    );
    assert!(
        validate(
            "/chat/registry/ns-meta/gaming",
            &make_ns_meta("alice"),
            "eve",
            &state
        )
        .is_err()
    );
}

#[test]
fn test_ns_meta_initial_creation_allowed() {
    let state = RouterState::new();
    assert!(
        validate(
            "/chat/registry/ns-meta/gaming",
            &make_ns_meta("alice"),
            "alice",
            &state
        )
        .is_ok()
    );
}

#[test]
fn test_ns_auth_path_checks_base_creator() {
    let state = RouterState::new();
    state_set(
        &state,
        "/chat/registry/ns-meta/gaming",
        make_ns_meta("alice"),
    );
    let val = Value::String("hash".to_string());
    assert!(
        validate("/chat/registry/ns-meta/gaming/__auth", &val, "alice", &state).is_ok()
    );
    assert!(
        validate("/chat/registry/ns-meta/gaming/__auth", &val, "eve", &state).is_err()
    );
}

// ===========================================================
//  validate_write -- Anonymous session
// ===========================================================

#[test]
fn test_anonymous_session_passes_through() {
    let state = RouterState::new();
    let session = make_anonymous_session();
    let val = Value::String("test".to_string());
    let v = make_write_validator();
    assert!(v
        .validate_write("/chat/user/x/dms/room1", &val, &session, &state)
        .is_ok());
    assert!(v
        .validate_write("/chat/room/r1/admin/x", &val, &session, &state)
        .is_ok());
}

// ===========================================================
//  validate_write -- Passthrough paths
// ===========================================================

#[test]
fn test_passthrough_paths_allowed() {
    let state = RouterState::new();
    let val = Value::String("test".to_string());
    assert!(validate("/chat/room/r1/messages", &val, "alice", &state).is_ok());
    assert!(validate("/chat/room/r1/presence/alice", &val, "alice", &state).is_ok());
    assert!(validate("/chat/room/r1/typing/alice", &val, "alice", &state).is_ok());
    assert!(
        validate("/chat/room/r1/reactions/msg1/alice", &val, "alice", &state).is_ok()
    );
    assert!(validate("/chat/user/alice/profile", &val, "alice", &state).is_ok());
    assert!(validate("/chat/registry/rooms/r1", &val, "alice", &state).is_ok());
}

// ===========================================================
//  Snapshot filter -- Password redaction
// ===========================================================

#[test]
fn test_snapshot_redacts_room_meta_passwords() {
    let state = RouterState::new();
    let params = vec![make_pv(
        "/chat/room/r1/meta",
        make_room_meta_with_password("alice"),
    )];
    let f = make_snapshot_filter();
    let result = f.filter_snapshot(params, &make_session("alice"), &state);
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
        make_pv(
            "/chat/registry/ns-meta/gaming/__auth",
            Value::String("hash".into()),
        ),
        make_pv("/chat/registry/ns-meta/gaming", make_ns_meta("alice")),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/registry/ns-meta/gaming"]);
}

#[test]
fn test_snapshot_auth_substring_in_path() {
    let state = RouterState::new();
    let params = vec![
        make_pv(
            "/chat/registry/ns-meta/test/__auth",
            Value::String("hash".into()),
        ),
        make_pv(
            "/chat/registry/ns-meta/test_auth_config",
            Value::String("ok".into()),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(
        !addresses.iter().any(|a| a.contains("/__auth")),
        "/__auth path leaked"
    );
    assert!(
        addresses.contains(&"/chat/registry/ns-meta/test_auth_config".to_string()),
        "non-__auth path was falsely stripped"
    );
}

// ===========================================================
//  Snapshot filter -- User privacy
// ===========================================================

#[test]
fn test_snapshot_allows_own_private_paths() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/alice/profile", Value::String("pub".into())),
        make_pv(
            "/chat/user/alice/friends/bob",
            Value::String("friend".into()),
        ),
        make_pv(
            "/chat/user/alice/dms/dm-room1",
            make_dm_notification("bob"),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(
        addresses.len(),
        3,
        "own paths should all pass: {:?}",
        addresses
    );
}

#[test]
fn test_snapshot_allows_other_user_profile() {
    let state = RouterState::new();
    let params = vec![make_pv(
        "/chat/user/bob/profile",
        Value::String("pub".into()),
    )];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/user/bob/profile"]);
}

#[test]
fn test_snapshot_strips_other_user_dms() {
    let state = RouterState::new();
    let params = vec![make_pv(
        "/chat/user/bob/dms/dm-room1",
        make_dm_notification("charlie"),
    )];
    let addresses = filter(params, "alice", &state);
    assert!(addresses.is_empty(), "victim DMs leaked: {:?}", addresses);
}

#[test]
fn test_snapshot_strips_other_user_friends() {
    let state = RouterState::new();
    let params = vec![make_pv(
        "/chat/user/bob/friends/charlie",
        Value::String("friend".into()),
    )];
    let addresses = filter(params, "alice", &state);
    assert!(
        addresses.is_empty(),
        "victim friends leaked: {:?}",
        addresses
    );
}

#[test]
fn test_snapshot_strips_other_user_all_private_paths() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
        make_pv(
            "/chat/user/bob/friends/charlie",
            Value::String("f".into()),
        ),
        make_pv(
            "/chat/user/bob/dms/dm1",
            make_dm_notification("charlie"),
        ),
        make_pv("/chat/user/bob/settings", Value::String("dark".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/user/bob/profile"]);
}

// ===========================================================
//  Snapshot filter -- Room membership gating
// ===========================================================

#[test]
fn test_snapshot_allows_room_internal_for_members() {
    let state = RouterState::new();
    state_set(
        &state,
        "/chat/room/r1/presence/alice",
        Value::String("online".into()),
    );
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        make_pv(
            "/chat/room/r1/crypto/pubkey/alice",
            Value::String("key".into()),
        ),
        make_pv(
            "/chat/room/r1/admin/alice",
            Value::String("admin".into()),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses.len(), 3);
}

#[test]
fn test_snapshot_strips_room_internal_for_non_members() {
    let state = RouterState::new();
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        make_pv(
            "/chat/room/r1/crypto/pubkey/bob",
            Value::String("key".into()),
        ),
        make_pv(
            "/chat/room/r1/admin/bob",
            Value::String("admin".into()),
        ),
        make_pv("/chat/room/r1/bans/eve", Value::String("ban".into())),
        make_pv(
            "/chat/room/r1/reactions/msg1/bob",
            Value::String(":+1:".into()),
        ),
        make_pv("/chat/room/r1/video/bob", Value::String("stream".into())),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(
        addresses.is_empty(),
        "non-member room data leaked: {:?}",
        addresses
    );
}

#[test]
fn test_snapshot_always_includes_room_meta() {
    let state = RouterState::new();
    let params = vec![make_pv("/chat/room/r1/meta", make_room_meta("bob"))];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/room/r1/meta"]);
}

#[test]
fn test_snapshot_allows_room_presence_and_typing_for_non_members() {
    let state = RouterState::new();
    let params = vec![
        make_pv(
            "/chat/room/r1/presence/bob",
            Value::String("online".into()),
        ),
        make_pv(
            "/chat/room/r1/typing/bob",
            Value::String("true".into()),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses.len(), 2);
}

#[test]
fn test_snapshot_null_presence_blocks_room_internal() {
    let state = RouterState::new();
    state_set(&state, "/chat/room/r1/presence/alice", Value::Null);
    let params = vec![make_pv(
        "/chat/room/r1/messages",
        Value::String("hello".into()),
    )];
    let addresses = filter(params, "alice", &state);
    assert!(
        addresses.is_empty(),
        "null presence should not grant room access"
    );
}

// ===========================================================
//  Snapshot filter -- Anonymous session
// ===========================================================

#[test]
fn test_snapshot_anonymous_session_strips_all_user_private_data() {
    let state = RouterState::new();
    let session = make_anonymous_session();
    let params = vec![
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
        make_pv(
            "/chat/user/bob/dms/dm1",
            make_dm_notification("charlie"),
        ),
        make_pv(
            "/chat/user/bob/friends/charlie",
            Value::String("f".into()),
        ),
    ];
    let f = make_snapshot_filter();
    let result = f.filter_snapshot(params, &session, &state);
    let addresses: Vec<&str> = result.iter().map(|pv| pv.address.as_str()).collect();
    assert!(addresses.contains(&"/chat/user/bob/profile"));
    assert!(
        !addresses.contains(&"/chat/user/bob/dms/dm1"),
        "anonymous sees DMs"
    );
    assert!(
        !addresses.contains(&"/chat/user/bob/friends/charlie"),
        "anonymous sees friends"
    );
}

#[test]
fn test_snapshot_anonymous_session_room_internal_filtered() {
    let state = RouterState::new();
    let session = make_anonymous_session();
    let params = vec![
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        make_pv(
            "/chat/room/r1/crypto/pubkey/bob",
            Value::String("key".into()),
        ),
    ];
    let f = make_snapshot_filter();
    let result = f.filter_snapshot(params, &session, &state);
    assert!(
        result.is_empty(),
        "anonymous sessions should not see room internals"
    );
}

// ===========================================================
//  Snapshot filter -- Friend request privacy
// ===========================================================

#[test]
fn test_snapshot_allows_own_friend_requests() {
    let state = RouterState::new();
    let params = vec![
        make_pv(
            "/chat/requests/alice/bob",
            make_friend_request("bob"),
        ),
        make_pv(
            "/chat/requests/alice/charlie",
            make_friend_request("charlie"),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(
        addresses.len(),
        2,
        "own requests should pass: {:?}",
        addresses
    );
}

#[test]
fn test_snapshot_strips_other_users_friend_requests() {
    let state = RouterState::new();
    let params = vec![
        make_pv(
            "/chat/requests/bob/alice",
            make_friend_request("alice"),
        ),
        make_pv(
            "/chat/requests/charlie/alice",
            make_friend_request("alice"),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert!(
        addresses.is_empty(),
        "other users' requests leaked: {:?}",
        addresses
    );
}

#[test]
fn test_snapshot_mixed_friend_requests() {
    let state = RouterState::new();
    let params = vec![
        make_pv(
            "/chat/requests/alice/bob",
            make_friend_request("bob"),
        ),
        make_pv(
            "/chat/requests/bob/alice",
            make_friend_request("alice"),
        ),
    ];
    let addresses = filter(params, "alice", &state);
    assert_eq!(addresses, vec!["/chat/requests/alice/bob"]);
}

#[test]
fn test_snapshot_anonymous_session_blocks_all_friend_requests() {
    let state = RouterState::new();
    let session = make_anonymous_session();
    let params = vec![
        make_pv(
            "/chat/requests/alice/bob",
            make_friend_request("bob"),
        ),
        make_pv(
            "/chat/requests/bob/alice",
            make_friend_request("alice"),
        ),
    ];
    let f = make_snapshot_filter();
    let result = f.filter_snapshot(params, &session, &state);
    assert!(
        result.is_empty(),
        "anonymous session should not see any friend requests"
    );
}

// ===========================================================
//  Snapshot filter -- Preserves values
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
    let f = make_snapshot_filter();
    let result = f.filter_snapshot(vec![pv], &make_session("alice"), &state);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].value, Value::String("best-friend".to_string()));
    assert_eq!(result[0].revision, 42);
    assert_eq!(result[0].writer, Some("session-xyz".to_string()));
    assert_eq!(result[0].timestamp, Some(12345));
}

// ===========================================================
//  Snapshot filter -- Full scenario
// ===========================================================

#[test]
fn test_snapshot_full_scenario() {
    let state = RouterState::new();
    state_set(
        &state,
        "/chat/room/r1/presence/alice",
        Value::String("online".into()),
    );

    let params = vec![
        // Own data -- keep all
        make_pv("/chat/user/alice/profile", Value::String("pub".into())),
        make_pv(
            "/chat/user/alice/friends/bob",
            Value::String("f".into()),
        ),
        make_pv(
            "/chat/user/alice/dms/dm1",
            make_dm_notification("bob"),
        ),
        // Other user -- keep only profile
        make_pv("/chat/user/bob/profile", Value::String("pub".into())),
        make_pv(
            "/chat/user/bob/friends/charlie",
            Value::String("f".into()),
        ),
        make_pv(
            "/chat/user/bob/dms/dm2",
            make_dm_notification("charlie"),
        ),
        // Room alice is in -- keep everything
        make_pv(
            "/chat/room/r1/meta",
            make_room_meta_with_password("alice"),
        ),
        make_pv("/chat/room/r1/messages", Value::String("hello".into())),
        // Room alice is NOT in -- keep only meta
        make_pv("/chat/room/r2/meta", make_room_meta("bob")),
        make_pv(
            "/chat/room/r2/messages",
            Value::String("secret".into()),
        ),
        // __auth -- strip
        make_pv(
            "/chat/registry/ns-meta/g/__auth",
            Value::String("hash".into()),
        ),
        // Registry -- keep
        make_pv(
            "/chat/registry/rooms/r1",
            Value::String("room".into()),
        ),
    ];

    let f = make_snapshot_filter();
    let result = f.filter_snapshot(params, &make_session("alice"), &state);
    let addresses: Vec<&str> = result.iter().map(|pv| pv.address.as_str()).collect();

    assert!(addresses.contains(&"/chat/user/alice/profile"));
    assert!(addresses.contains(&"/chat/user/alice/friends/bob"));
    assert!(addresses.contains(&"/chat/user/alice/dms/dm1"));
    assert!(addresses.contains(&"/chat/user/bob/profile"));
    assert!(
        !addresses.contains(&"/chat/user/bob/friends/charlie"),
        "victim friends leaked"
    );
    assert!(
        !addresses.contains(&"/chat/user/bob/dms/dm2"),
        "victim DMs leaked"
    );
    assert!(addresses.contains(&"/chat/room/r1/meta"));
    assert!(addresses.contains(&"/chat/room/r1/messages"));
    assert!(addresses.contains(&"/chat/room/r2/meta"));
    assert!(
        !addresses.contains(&"/chat/room/r2/messages"),
        "non-member room data leaked"
    );
    assert!(
        !addresses.contains(&"/chat/registry/ns-meta/g/__auth"),
        "__auth leaked"
    );
    assert!(addresses.contains(&"/chat/registry/rooms/r1"));

    // Verify password fields were redacted from room meta
    let r1_meta = result
        .iter()
        .find(|pv| pv.address == "/chat/room/r1/meta")
        .unwrap();
    match &r1_meta.value {
        Value::Map(map) => {
            assert!(
                !map.contains_key("passwordHash"),
                "passwordHash leaked in snapshot"
            );
            assert!(
                !map.contains_key("passwordSalt"),
                "passwordSalt leaked in snapshot"
            );
        }
        _ => panic!("Expected Map for room meta"),
    }
}

// ===========================================================
//  Corrupted room meta
// ===========================================================

#[test]
fn test_room_admin_with_corrupted_meta_rejects() {
    let state = RouterState::new();
    let mut map = HashMap::new();
    map.insert("name".to_string(), Value::String("broken room".to_string()));
    state_set(&state, "/chat/room/r1/meta", Value::Map(map));
    let val = Value::String("admin".to_string());
    assert!(
        validate("/chat/room/r1/admin/bob", &val, "eve", &state).is_err(),
        "corrupted meta should reject everyone"
    );
}

#[test]
fn test_room_admin_with_non_map_meta_rejects() {
    let state = RouterState::new();
    state_set(
        &state,
        "/chat/room/r1/meta",
        Value::String("corrupted".to_string()),
    );
    let val = Value::String("admin".to_string());
    assert!(validate("/chat/room/r1/admin/bob", &val, "eve", &state).is_err());
}

// ===========================================================
//  Pattern matching unit tests
// ===========================================================

#[test]
fn test_match_address_basic() {
    use clasp_relay::app_config::match_address;

    assert!(match_address("/a/b/c", "/a/b/c").is_some());
    assert!(match_address("/a/b/c", "/a/b/d").is_none());

    let c = match_address("/chat/room/{roomId}/meta", "/chat/room/r1/meta").unwrap();
    assert_eq!(c.get("roomId"), Some(&"r1"));

    assert!(match_address("/chat/**", "/chat/any/thing/here").is_some());
    assert!(match_address("/a/*/c", "/a/x/c").is_some());
    assert!(match_address("/a/*/c", "/a/x/d").is_none());
}

// ===========================================================
//  Rate limit config
// ===========================================================

#[test]
fn test_rate_limit_config_from_chat_json() {
    let config = load_chat_config();
    let rl = config.rate_limits.unwrap();
    assert_eq!(rl.login_max_attempts, 5);
    assert_eq!(rl.login_window_secs, 60);
    assert_eq!(rl.register_max_attempts, 10);
    assert_eq!(rl.register_window_secs, 60);
}
