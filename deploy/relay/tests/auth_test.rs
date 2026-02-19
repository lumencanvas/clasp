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
    auth_router(state)
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
