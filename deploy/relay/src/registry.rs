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
}

impl RegistryState {
    pub fn new(store: Arc<dyn EntityStore>, validator: Arc<CpskValidator>) -> Self {
        Self { store, validator }
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

/// Build the registry REST router.
pub fn registry_router(state: Arc<RegistryState>) -> Router {
    Router::new()
        .route("/api/entities", post(create_entity).get(list_entities))
        .route(
            "/api/entities/{id}",
            get(get_entity).delete(delete_entity),
        )
        .route("/api/entities/{id}/status", put(update_entity_status))
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
