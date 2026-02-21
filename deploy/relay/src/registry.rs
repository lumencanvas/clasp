//! Entity registry REST API for CLASP relay.
//!
//! Provides CRUD endpoints for entity management, protected by admin CPSK scope.
//! Follows the same Axum + shared state pattern as `auth.rs`.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post, put},
    Json, Router,
};
use clasp_core::security::{Action, CpskValidator, TokenValidator, ValidationResult};
use clasp_registry::{Entity, EntityId, EntityStatus, EntityStore};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct RegistryState {
    store: Arc<dyn EntityStore>,
    validator: Arc<CpskValidator>,
    /// Trust anchor public keys (hex-encoded) for capability tokens
    trust_anchors: Vec<String>,
    /// Max delegation chain depth for capability tokens
    cap_max_depth: usize,
}

impl RegistryState {
    pub fn new(store: Arc<dyn EntityStore>, validator: Arc<CpskValidator>) -> Self {
        Self {
            store,
            validator,
            trust_anchors: Vec::new(),
            cap_max_depth: 5,
        }
    }

    /// Set trust anchor info for the /api/trust-anchors endpoint
    pub fn with_trust_anchors(mut self, anchors: Vec<String>, max_depth: usize) -> Self {
        self.trust_anchors = anchors;
        self.cap_max_depth = max_depth;
        self
    }
}

/// Extractor that validates a Bearer token with admin scope.
struct AdminToken;

impl axum::extract::FromRequestParts<Arc<RegistryState>> for AdminToken {
    type Rejection = (StatusCode, Json<ErrorResponse>);

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &Arc<RegistryState>,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(ErrorResponse {
                        error: "missing Authorization header".into(),
                    }),
                )
            })?;

        let token = auth_header.strip_prefix("Bearer ").ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "expected Bearer token".into(),
                }),
            )
        })?;

        match state.validator.validate(token) {
            ValidationResult::Valid(info) => {
                if info.has_scope(Action::Admin, "/**") {
                    Ok(AdminToken)
                } else {
                    Err((
                        StatusCode::FORBIDDEN,
                        Json(ErrorResponse {
                            error: "admin scope required".into(),
                        }),
                    ))
                }
            }
            ValidationResult::Expired => Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "token expired".into(),
                }),
            )),
            _ => Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "invalid token".into(),
                }),
            )),
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct EntityResponse {
    id: String,
    entity_type: clasp_registry::EntityType,
    name: String,
    status: EntityStatus,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    namespaces: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    scopes: Vec<String>,
    #[serde(skip_serializing_if = "std::collections::HashMap::is_empty")]
    metadata: std::collections::HashMap<String, String>,
}

impl From<Entity> for EntityResponse {
    fn from(e: Entity) -> Self {
        Self {
            id: e.id.as_str().to_string(),
            entity_type: e.entity_type,
            name: e.name,
            status: e.status,
            tags: e.tags,
            namespaces: e.namespaces,
            scopes: e.scopes,
            metadata: e.metadata,
        }
    }
}

#[derive(Deserialize)]
struct CreateEntityRequest {
    entity_type: clasp_registry::EntityType,
    name: String,
    public_key: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    namespaces: Vec<String>,
    #[serde(default)]
    scopes: Vec<String>,
    #[serde(default)]
    metadata: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
struct UpdateStatusRequest {
    status: EntityStatus,
}

#[derive(Deserialize)]
struct ListQuery {
    #[serde(default)]
    offset: Option<usize>,
    #[serde(default = "default_limit")]
    limit: Option<usize>,
}

fn default_limit() -> Option<usize> {
    Some(100)
}

async fn create_entity(
    State(state): State<Arc<RegistryState>>,
    _admin: AdminToken,
    Json(req): Json<CreateEntityRequest>,
) -> Result<(StatusCode, Json<EntityResponse>), (StatusCode, Json<ErrorResponse>)> {
    let public_key = hex::decode(&req.public_key).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "public_key must be hex-encoded Ed25519 public key (64 hex chars)".into(),
            }),
        )
    })?;

    let id = EntityId::from_public_key(&public_key).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid public key: {}", e),
            }),
        )
    })?;

    let entity = Entity {
        id,
        entity_type: req.entity_type,
        name: req.name,
        public_key,
        created_at: std::time::SystemTime::now(),
        metadata: req.metadata,
        tags: req.tags,
        namespaces: req.namespaces,
        scopes: req.scopes,
        status: EntityStatus::Active,
    };

    state.store.create(&entity).await.map_err(|e| {
        (
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: format!("failed to create entity: {}", e),
            }),
        )
    })?;

    tracing::info!("Entity created: {} ({})", entity.name, entity.id);
    Ok((StatusCode::CREATED, Json(EntityResponse::from(entity))))
}

async fn list_entities(
    State(state): State<Arc<RegistryState>>,
    _admin: AdminToken,
    axum::extract::Query(query): axum::extract::Query<ListQuery>,
) -> Result<Json<Vec<EntityResponse>>, (StatusCode, Json<ErrorResponse>)> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(100);

    let entities = state.store.list(offset, limit).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("failed to list entities: {}", e),
            }),
        )
    })?;

    Ok(Json(entities.into_iter().map(EntityResponse::from).collect()))
}

async fn get_entity(
    State(state): State<Arc<RegistryState>>,
    _admin: AdminToken,
    Path(id): Path<String>,
) -> Result<Json<EntityResponse>, (StatusCode, Json<ErrorResponse>)> {
    let entity_id = EntityId::parse(&id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid entity ID: {}", e),
            }),
        )
    })?;

    let entity = state
        .store
        .get(&entity_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("store error: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "entity not found".into(),
                }),
            )
        })?;

    Ok(Json(EntityResponse::from(entity)))
}

async fn delete_entity(
    State(state): State<Arc<RegistryState>>,
    _admin: AdminToken,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let entity_id = EntityId::parse(&id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid entity ID: {}", e),
            }),
        )
    })?;

    let deleted = state.store.delete(&entity_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("failed to delete entity: {}", e),
            }),
        )
    })?;

    if deleted {
        tracing::info!("Entity deleted: {}", id);
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "entity not found".into(),
            }),
        ))
    }
}

async fn update_entity_status(
    State(state): State<Arc<RegistryState>>,
    _admin: AdminToken,
    Path(id): Path<String>,
    Json(req): Json<UpdateStatusRequest>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let entity_id = EntityId::parse(&id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid entity ID: {}", e),
            }),
        )
    })?;

    state
        .store
        .update_status(&entity_id, req.status)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("failed to update status: {}", e),
                }),
            )
        })?;

    tracing::info!("Entity {} status -> {}", id, req.status);
    Ok(StatusCode::OK)
}

// =========================================================================
// Entity token minting
// =========================================================================

#[derive(Deserialize)]
struct MintTokenRequest {
    /// Hex-encoded 32-byte Ed25519 signing key
    keypair_hex: String,
}

#[derive(Serialize)]
struct MintTokenResponse {
    token: String,
    entity_id: String,
}

/// Mint an entity authentication token.
///
/// # Security
///
/// This endpoint accepts a signing key (private key) in the request body.
/// The key is used transiently to mint a token and is never stored, but it
/// **does traverse the network**. This endpoint MUST be served behind TLS
/// in production. For offline token minting without network exposure, use:
///
/// ```bash
/// clasp token entity mint --key ./device.key
/// ```
async fn mint_entity_token(
    State(state): State<Arc<RegistryState>>,
    _admin: AdminToken,
    Path(id): Path<String>,
    Json(req): Json<MintTokenRequest>,
) -> Result<Json<MintTokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    let entity_id = EntityId::parse(&id).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("invalid entity ID: {}", e),
            }),
        )
    })?;

    // Look up entity and verify it exists and is active
    let entity = state
        .store
        .get(&entity_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("store error: {}", e),
                }),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "entity not found".into(),
                }),
            )
        })?;

    if !entity.is_active() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: format!("entity is {}, must be active to mint tokens", entity.status),
            }),
        ));
    }

    // Decode signing key from hex
    let key_bytes = hex::decode(&req.keypair_hex).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "keypair_hex must be 64 hex chars (32-byte Ed25519 signing key)".into(),
            }),
        )
    })?;

    if key_bytes.len() != 32 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("expected 32-byte signing key, got {} bytes", key_bytes.len()),
            }),
        ));
    }

    let key_array: [u8; 32] = key_bytes.try_into().unwrap();
    let signing_key = ed25519_dalek::SigningKey::from_bytes(&key_array);

    // Verify the derived public key matches the entity's stored public key
    let derived_pub = signing_key.verifying_key().to_bytes().to_vec();
    if derived_pub != entity.public_key {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "signing key does not match entity's registered public key".into(),
            }),
        ));
    }

    // Reconstruct EntityKeypair and generate token
    let keypair = clasp_registry::EntityKeypair::from_signing_key(signing_key)
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("failed to create keypair: {}", e),
                }),
            )
        })?;

    let token = clasp_registry::generate_token(&keypair).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("failed to generate token: {}", e),
            }),
        )
    })?;

    tracing::warn!(
        "Entity token minted for {} -- signing key was transmitted over the network. \
         Ensure this endpoint is served behind TLS in production.",
        entity_id
    );
    Ok(Json(MintTokenResponse {
        token,
        entity_id: entity_id.as_str().to_string(),
    }))
}

// =========================================================================
// Trust anchors info
// =========================================================================

#[derive(Serialize)]
struct TrustAnchorsResponse {
    anchors: Vec<String>,
    cap_max_depth: usize,
}

async fn get_trust_anchors(
    State(state): State<Arc<RegistryState>>,
) -> Json<TrustAnchorsResponse> {
    Json(TrustAnchorsResponse {
        anchors: state.trust_anchors.clone(),
        cap_max_depth: state.cap_max_depth,
    })
}

/// Build the registry REST router.
pub fn registry_router(state: Arc<RegistryState>) -> Router {
    Router::new()
        .route("/api/entities", post(create_entity).get(list_entities))
        .route(
            "/api/entities/{id}",
            get(get_entity).delete(delete_entity),
        )
        .route("/api/entities/{id}/status", put(update_entity_status))
        .route("/api/entities/{id}/token", post(mint_entity_token))
        .route("/api/trust-anchors", get(get_trust_anchors))
        .with_state(state)
}

/// Hex decode/encode helper for public keys in JSON
mod hex {
    pub fn decode(s: &str) -> Result<Vec<u8>, String> {
        if s.len() % 2 != 0 {
            return Err("odd-length hex string".into());
        }
        (0..s.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&s[i..i + 2], 16).map_err(|e| e.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use clasp_core::security::{Scope, TokenInfo};
    use clasp_registry::MemoryEntityStore;
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn hex_encode(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    fn make_test_state() -> (Arc<RegistryState>, String) {
        let store: Arc<dyn EntityStore> = Arc::new(MemoryEntityStore::new());
        let validator = Arc::new(CpskValidator::new());

        // Register an admin token
        let admin_token = CpskValidator::generate_token();
        let scopes = vec![Scope::new(Action::Admin, "/**").unwrap()];
        let info = TokenInfo::new(admin_token.clone(), scopes)
            .with_subject("test-admin".to_string());
        validator.register(admin_token.clone(), info);

        let state = Arc::new(
            RegistryState::new(store, validator)
                .with_trust_anchors(vec!["deadbeef".to_string()], 5),
        );

        (state, admin_token)
    }

    fn make_app(state: Arc<RegistryState>) -> axum::Router {
        registry_router(state)
    }

    #[tokio::test]
    async fn test_mint_token_without_auth_returns_401() {
        let (state, _) = make_test_state();
        let app = make_app(state);

        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/entities/test-id/token")
            .header("content-type", "application/json")
            .body(Body::from(r#"{"keypair_hex":"aa"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_mint_token_with_non_admin_returns_403() {
        let (state, _admin_token) = make_test_state();

        // Register a non-admin token with only read scope
        let read_token = CpskValidator::generate_token();
        let scopes = vec![Scope::new(clasp_core::security::Action::Read, "/**").unwrap()];
        let info = TokenInfo::new(read_token.clone(), scopes);
        state.validator.register(read_token.clone(), info);

        let app = make_app(state);

        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/entities/test-id/token")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", read_token))
            .body(Body::from(r#"{"keypair_hex":"aa"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_mint_token_with_invalid_keypair_returns_400() {
        let (state, admin_token) = make_test_state();

        // Create an entity first
        let keypair = clasp_registry::EntityKeypair::generate().unwrap();
        let entity = keypair.to_entity(
            clasp_registry::EntityType::Device,
            "test-device".to_string(),
        );
        state.store.create(&entity).await.unwrap();

        let app = make_app(state);

        let req = axum::http::Request::builder()
            .method("POST")
            .uri(&format!("/api/entities/{}/token", entity.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(r#"{"keypair_hex":"not-valid-hex"}"#))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_get_trust_anchors() {
        let (state, _) = make_test_state();
        let app = make_app(state);

        let req = axum::http::Request::builder()
            .method("GET")
            .uri("/api/trust-anchors")
            .body(Body::empty())
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json["anchors"][0], "deadbeef");
        assert_eq!(json["cap_max_depth"], 5);
    }

    #[tokio::test]
    async fn test_mint_token_happy_path() {
        let (state, admin_token) = make_test_state();

        // Generate a keypair and register the entity
        let keypair = clasp_registry::EntityKeypair::generate().unwrap();
        let entity = keypair.to_entity(
            clasp_registry::EntityType::Device,
            "mint-test-device".to_string(),
        );
        state.store.create(&entity).await.unwrap();

        let app = make_app(state);

        // Mint a token using the actual signing key
        let signing_key_hex = hex_encode(&keypair.signing_key.to_bytes());
        let body = serde_json::json!({ "keypair_hex": signing_key_hex });

        let req = axum::http::Request::builder()
            .method("POST")
            .uri(&format!("/api/entities/{}/token", entity.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::OK);

        let body = resp.into_body().collect().await.unwrap().to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        // Verify the response contains a token and correct entity ID
        assert!(json["token"].as_str().is_some_and(|t| !t.is_empty()));
        assert_eq!(json["entity_id"].as_str().unwrap(), entity.id.as_str());

        // Verify the token is actually valid by parsing and verifying it
        let token_str = json["token"].as_str().unwrap();
        let parsed = clasp_registry::parse_token(token_str);
        assert!(parsed.is_ok(), "minted token should be parseable");

        let parsed = parsed.unwrap();
        assert_eq!(parsed.entity_id, entity.id.as_str());
        let verify = clasp_registry::token::verify_token_signature(&parsed, keypair.public_key_bytes());
        assert!(verify.is_ok(), "minted token should verify against the entity's public key");
    }

    #[tokio::test]
    async fn test_mint_token_wrong_key_returns_403() {
        let (state, admin_token) = make_test_state();

        // Register entity with one keypair
        let keypair = clasp_registry::EntityKeypair::generate().unwrap();
        let entity = keypair.to_entity(
            clasp_registry::EntityType::Device,
            "wrong-key-device".to_string(),
        );
        state.store.create(&entity).await.unwrap();

        // Try to mint with a DIFFERENT signing key
        let wrong_keypair = clasp_registry::EntityKeypair::generate().unwrap();
        let wrong_key_hex = hex_encode(&wrong_keypair.signing_key.to_bytes());

        let app = make_app(state);
        let body = serde_json::json!({ "keypair_hex": wrong_key_hex });

        let req = axum::http::Request::builder()
            .method("POST")
            .uri(&format!("/api/entities/{}/token", entity.id))
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn test_create_and_get_entity() {
        let (state, admin_token) = make_test_state();

        // Generate a real keypair to get a valid public key
        let keypair = clasp_registry::EntityKeypair::generate().unwrap();
        let pub_hex = hex_encode(keypair.public_key_bytes());

        let app = make_app(state);

        let body = serde_json::json!({
            "entity_type": "device",
            "name": "test-device",
            "public_key": pub_hex,
        });

        let req = axum::http::Request::builder()
            .method("POST")
            .uri("/api/entities")
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", admin_token))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}
