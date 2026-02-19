use axum::body::Body;
use axum::http::{Request, StatusCode};
use clasp_core::security::CpskValidator;
use http_body_util::BodyExt;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

use clasp_relay::auth::{auth_router, AuthState};

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
