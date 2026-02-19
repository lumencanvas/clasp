use axum::body::Body;
use axum::http::{Request, StatusCode};
use clasp_core::security::{CpskValidator, Scope};
use clasp_router::Session;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use clasp_relay::auth::{auth_router, build_scopes, AuthState};

fn setup_app() -> axum::Router {
    let validator = Arc::new(CpskValidator::new());
    let state = Arc::new(AuthState::new(":memory:", validator).unwrap());
    auth_router(state, None)
}

async fn post_json(app: axum::Router, uri: &str, body: Value) -> (StatusCode, Value) {
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let status = response.status();
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let json: Value = serde_json::from_slice(&body_bytes).unwrap_or(json!({}));
    (status, json)
}

#[tokio::test]
async fn register_returns_200_with_token_and_user_id() {
    let app = setup_app();
    let (status, body) = post_json(
        app,
        "/auth/register",
        json!({ "username": "alice", "password": "secret123" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
    assert!(body["user_id"].is_string());
    assert_eq!(body["username"], "alice");
}

#[tokio::test]
async fn register_duplicate_username_returns_409() {
    let app = setup_app();

    // First registration
    let (status, _) = post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "alice", "password": "secret123" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    // Duplicate
    let (status, _) = post_json(
        app,
        "/auth/register",
        json!({ "username": "alice", "password": "different456" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_short_username_returns_400() {
    let app = setup_app();
    let (status, body) = post_json(
        app,
        "/auth/register",
        json!({ "username": "a", "password": "secret123" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("Username"));
}

#[tokio::test]
async fn register_short_password_returns_400() {
    let app = setup_app();
    let (status, body) = post_json(
        app,
        "/auth/register",
        json!({ "username": "alice", "password": "12345" }),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().unwrap().contains("Password"));
}

#[tokio::test]
async fn login_correct_credentials_returns_200() {
    let app = setup_app();

    // Register
    post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "bob", "password": "mypassword" }),
    )
    .await;

    // Login
    let (status, body) = post_json(
        app,
        "/auth/login",
        json!({ "username": "bob", "password": "mypassword" }),
    )
    .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
    assert!(body["user_id"].is_string());
    assert_eq!(body["username"], "bob");
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let app = setup_app();

    post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "charlie", "password": "correctpass" }),
    )
    .await;

    let (status, _) = post_json(
        app,
        "/auth/login",
        json!({ "username": "charlie", "password": "wrongpass" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_nonexistent_user_returns_401() {
    let app = setup_app();
    let (status, _) = post_json(
        app,
        "/auth/login",
        json!({ "username": "nobody", "password": "whatever" }),
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn guest_returns_200_with_generated_user_id() {
    let app = setup_app();
    let (status, body) = post_json(app, "/auth/guest", json!({})).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["token"].is_string());
    assert!(body["user_id"].is_string());
    assert!(body["username"].as_str().unwrap().starts_with("guest-"));
}

#[tokio::test]
async fn guest_with_name_returns_that_name() {
    let app = setup_app();
    let (status, body) =
        post_json(app, "/auth/guest", json!({ "name": "Visitor" })).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["username"], "Visitor");
}

#[tokio::test]
async fn guest_with_conflicting_name_returns_409() {
    let app = setup_app();

    // Register a user
    post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "TakenName", "password": "secret123" }),
    )
    .await;

    // Guest with same name (case-insensitive)
    let (status, _) =
        post_json(app, "/auth/guest", json!({ "name": "takenname" })).await;

    assert_eq!(status, StatusCode::CONFLICT);
}

// === Security fix tests ===

/// C3: guest with a registered user's user_id should be rejected
#[tokio::test]
async fn guest_with_registered_user_id_returns_409() {
    let app = setup_app();

    // Register a user
    let (status, body) = post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "victim", "password": "secret123" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let victim_id = body["user_id"].as_str().unwrap().to_string();

    // Guest tries to claim the registered user's ID
    let (status, body) = post_json(
        app,
        "/auth/guest",
        json!({ "user_id": victim_id, "name": "attacker" }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(body["error"].as_str().unwrap().contains("registered"));
}

/// C3: register with an already-taken user_id should be rejected
#[tokio::test]
async fn register_with_existing_user_id_returns_409() {
    let app = setup_app();

    // Register first user
    let (status, body) = post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "first", "password": "secret123" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let first_id = body["user_id"].as_str().unwrap().to_string();

    // Register second user reusing the first user's ID
    let (status, _) = post_json(
        app,
        "/auth/register",
        json!({ "username": "second", "password": "secret456", "user_id": first_id }),
    )
    .await;
    assert_eq!(status, StatusCode::CONFLICT);
}

/// H7: login rate limiting — too many failed attempts should return 429
#[tokio::test]
async fn login_rate_limiting_blocks_after_max_attempts() {
    let app = setup_app();

    // Register a user
    post_json(
        app.clone(),
        "/auth/register",
        json!({ "username": "ratelimited", "password": "correctpass" }),
    )
    .await;

    // Make 5 failed login attempts (the max before rate limiting kicks in)
    for _ in 0..5 {
        let (status, _) = post_json(
            app.clone(),
            "/auth/login",
            json!({ "username": "ratelimited", "password": "wrongpass" }),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }

    // 6th attempt should be rate-limited
    let (status, body) = post_json(
        app.clone(),
        "/auth/login",
        json!({ "username": "ratelimited", "password": "wrongpass" }),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(body["error"].as_str().unwrap().contains("Too many"));

    // Even correct password should be blocked while rate-limited
    let (status, _) = post_json(
        app,
        "/auth/login",
        json!({ "username": "ratelimited", "password": "correctpass" }),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
}

/// M2: user_id with invalid characters should be rejected
#[tokio::test]
async fn guest_with_invalid_user_id_format_returns_400() {
    let app = setup_app();

    // Path traversal attempt
    let (status, _) = post_json(
        app.clone(),
        "/auth/guest",
        json!({ "user_id": "../../admin", "name": "hacker" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Wildcard injection
    let (status, _) = post_json(
        app.clone(),
        "/auth/guest",
        json!({ "user_id": "user-*-all", "name": "hacker" }),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Valid user_id should work
    let (status, body) = post_json(
        app,
        "/auth/guest",
        json!({ "user_id": "u-12345-abcdef", "name": "legit" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], "u-12345-abcdef");
}

/// M6: registration rate limiting — too many registrations from same IP
#[tokio::test]
async fn registration_rate_limiting_blocks_after_max_attempts() {
    let app = setup_app();

    // Make 10 guest requests (the max before rate limiting kicks in)
    for i in 0..10 {
        let (status, _) = post_json(
            app.clone(),
            "/auth/guest",
            json!({ "name": format!("guest-{}", i) }),
        )
        .await;
        assert_eq!(status, StatusCode::OK);
    }

    // 11th attempt should be rate-limited
    let (status, body) = post_json(
        app,
        "/auth/guest",
        json!({ "name": "one-too-many" }),
    )
    .await;
    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert!(body["error"].as_str().unwrap().contains("Too many"));
}

// === Scope restriction tests (S1) ===

/// Helper: parse scopes and check if any scope allows the given action+address
fn scopes_allow(scopes: &[Scope], action: clasp_core::Action, address: &str) -> bool {
    scopes.iter().any(|s| s.allows(action, address))
}

/// Helper: check that ONLY read-action scopes (not write-implies-read) allow a read
fn read_only_scopes_allow(scopes: &[Scope], address: &str) -> bool {
    scopes.iter().any(|s| {
        s.action() == clasp_core::Action::Read && s.allows(clasp_core::Action::Read, address)
    })
}

// -- Read scope restriction: verify no explicit read:/chat/** --

#[test]
fn scope_no_explicit_global_read() {
    let scope_strings = build_scopes("alice");
    // The old `read:/chat/**` should NOT be present as an explicit read scope
    assert!(
        !scope_strings.contains(&"read:/chat/**".to_string()),
        "read:/chat/** still present in scope strings"
    );
}

#[test]
fn scope_explicit_read_own_user_data() {
    let scopes: Vec<Scope> = build_scopes("alice")
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    // Explicit read scopes cover own user data
    assert!(read_only_scopes_allow(&scopes, "/chat/user/alice/profile"));
    assert!(read_only_scopes_allow(&scopes, "/chat/user/alice/friends/bob"));
    assert!(read_only_scopes_allow(&scopes, "/chat/user/alice/dms/room1"));
}

#[test]
fn scope_explicit_read_denies_other_user_private_data() {
    let scopes: Vec<Scope> = build_scopes("alice")
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    // Explicit read scopes should NOT cover other user's private data
    assert!(read_only_scopes_allow(&scopes, "/chat/user/bob/profile"),
        "should allow reading other user's profile");
    assert!(!read_only_scopes_allow(&scopes, "/chat/user/bob/dms/room1"),
        "explicit read scope covers other user's DMs");
    assert!(!read_only_scopes_allow(&scopes, "/chat/user/bob/friends/charlie"),
        "explicit read scope covers other user's friends");
    assert!(!read_only_scopes_allow(&scopes, "/chat/user/bob/settings"),
        "explicit read scope covers other user's settings");
}

#[test]
fn scope_write_implies_read_on_dm_paths_at_action_level() {
    // Action::Write.allows(Action::Read) is true in the core model,
    // so scopes_allow() sees write:/chat/user/*/dms/* as granting read.
    // However, the SUBSCRIBE handler uses has_strict_read_scope() which
    // only checks explicit read scopes, mitigating subscription-based attacks.
    let scopes: Vec<Scope> = build_scopes("alice")
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    let read = clasp_core::Action::Read;
    // Still true at the Action model level
    assert!(scopes_allow(&scopes, read, "/chat/user/bob/dms/room1"),
        "write-implies-read exists at Action level (mitigated by strict read in SUBSCRIBE)");
    // But strict read-only scopes do NOT cover it
    assert!(!read_only_scopes_allow(&scopes, "/chat/user/bob/dms/room1"),
        "strict read scopes should NOT cover other user's DMs");
}

#[test]
fn scope_can_read_rooms_and_registry() {
    let scopes: Vec<Scope> = build_scopes("alice")
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    let read = clasp_core::Action::Read;
    assert!(scopes_allow(&scopes, read, "/chat/room/r1/messages"));
    assert!(scopes_allow(&scopes, read, "/chat/room/r1/meta"));
    assert!(scopes_allow(&scopes, read, "/chat/registry/rooms/r1"));
    assert!(scopes_allow(&scopes, read, "/chat/registry/ns/gaming/room1"));
}

#[test]
fn scope_can_read_own_friend_requests() {
    let scopes: Vec<Scope> = build_scopes("alice")
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    // Explicit read only covers own inbox
    assert!(read_only_scopes_allow(&scopes, "/chat/requests/alice"));
    assert!(!read_only_scopes_allow(&scopes, "/chat/requests/bob"),
        "explicit read scope covers other user's friend request inbox");
}

#[test]
fn scope_write_implies_read_on_friend_requests_at_action_level() {
    // Same pattern: write:/chat/requests/* implies read at Action level,
    // but SUBSCRIBE uses strict read scopes which blocks it.
    let scopes: Vec<Scope> = build_scopes("alice")
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    let read = clasp_core::Action::Read;
    assert!(scopes_allow(&scopes, read, "/chat/requests/bob"),
        "write-implies-read exists at Action level");
    assert!(!read_only_scopes_allow(&scopes, "/chat/requests/bob"),
        "strict read scopes should NOT cover other user's request inbox");
}

/// H2: CORS — explicit origin should be set when provided
#[tokio::test]
async fn cors_explicit_origin_is_set() {
    let validator = Arc::new(CpskValidator::new());
    let state = Arc::new(AuthState::new(":memory:", validator).unwrap());
    let app = auth_router(state, Some("https://chat.example.com"));

    // Preflight request
    let req = Request::builder()
        .method("OPTIONS")
        .uri("/auth/login")
        .header("origin", "https://chat.example.com")
        .header("access-control-request-method", "POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(req).await.unwrap();
    let acao = response.headers().get("access-control-allow-origin");
    assert!(acao.is_some());
    assert_eq!(acao.unwrap().to_str().unwrap(), "https://chat.example.com");
}

// === has_strict_read_scope tests ===

/// Create an authenticated Session with the scopes from build_scopes()
fn make_authenticated_session(user_id: &str) -> Session {
    let scope_strings = build_scopes(user_id);
    let scopes: Vec<Scope> = scope_strings
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();
    let mut session = Session::stub(Some(user_id.to_string()));
    session.set_authenticated("test-token".to_string(), Some(user_id.to_string()), scopes);
    session
}

#[test]
fn strict_read_allows_own_user_data() {
    let session = make_authenticated_session("alice");
    assert!(session.has_strict_read_scope("/chat/user/alice/profile"));
    assert!(session.has_strict_read_scope("/chat/user/alice/friends/bob"));
    assert!(session.has_strict_read_scope("/chat/user/alice/dms/room1"));
}

#[test]
fn strict_read_allows_other_user_profile() {
    let session = make_authenticated_session("alice");
    assert!(session.has_strict_read_scope("/chat/user/bob/profile"));
}

#[test]
fn strict_read_denies_other_user_dms() {
    let session = make_authenticated_session("alice");
    // This is the critical fix: strict read denies subscribing to other users' DMs
    assert!(!session.has_strict_read_scope("/chat/user/bob/dms/room1"),
        "strict read should deny access to other user's DMs");
}

#[test]
fn strict_read_denies_other_user_friends() {
    let session = make_authenticated_session("alice");
    assert!(!session.has_strict_read_scope("/chat/user/bob/friends/charlie"),
        "strict read should deny access to other user's friends");
}

#[test]
fn strict_read_denies_other_user_request_inbox() {
    let session = make_authenticated_session("alice");
    assert!(!session.has_strict_read_scope("/chat/requests/bob"),
        "strict read should deny subscribing to other user's request inbox");
}

#[test]
fn strict_read_allows_own_request_inbox() {
    let session = make_authenticated_session("alice");
    assert!(session.has_strict_read_scope("/chat/requests/alice"));
}

#[test]
fn strict_read_allows_rooms_and_registry() {
    let session = make_authenticated_session("alice");
    assert!(session.has_strict_read_scope("/chat/room/r1/messages"));
    assert!(session.has_strict_read_scope("/chat/room/r1/meta"));
    assert!(session.has_strict_read_scope("/chat/registry/rooms/r1"));
    assert!(session.has_strict_read_scope("/chat/registry/ns/gaming/room1"));
}

#[test]
fn strict_read_vs_has_scope_on_victim_dms() {
    let session = make_authenticated_session("alice");
    // has_scope(Read) passes due to write-implies-read
    assert!(session.has_scope(clasp_core::Action::Read, "/chat/user/bob/dms/room1"),
        "has_scope should allow via write-implies-read");
    // has_strict_read_scope blocks it
    assert!(!session.has_strict_read_scope("/chat/user/bob/dms/room1"),
        "has_strict_read_scope should block write-implies-read");
}
