//! Journal query REST API for the relay server.
//!
//! Provides read-only query endpoints for the journal, protected by admin CPSK scope.
//! Follows the same Axum + shared state pattern as `registry.rs`.

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use clasp_core::security::{Action, CpskValidator, TokenValidator, ValidationResult};
use clasp_journal::{entry::ParamSnapshot, Journal, JournalEntry};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct JournalApiState {
    pub journal: Arc<dyn Journal>,
    pub validator: Arc<CpskValidator>,
}

/// Query parameters for the `/api/journal/query` endpoint.
#[derive(Deserialize)]
pub struct JournalQueryParams {
    /// Address glob pattern (e.g. "/mixer/**")
    pub pattern: Option<String>,
    /// Start timestamp (microseconds since epoch)
    pub from: Option<u64>,
    /// End timestamp (microseconds since epoch)
    pub to: Option<u64>,
    /// Maximum number of entries to return
    pub limit: Option<u32>,
    /// Comma-separated signal types: param, event, stream, gesture, timeline
    #[serde(default)]
    pub types: Option<String>,
}

/// Query parameters for the `/api/journal/since` endpoint.
#[derive(Deserialize)]
pub struct SinceParams {
    /// Sequence number to start from (exclusive)
    pub seq: u64,
    /// Maximum number of entries to return
    pub limit: Option<u32>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct LatestSeqResponse {
    seq: u64,
}

type ApiError = (StatusCode, Json<ErrorResponse>);

fn err(status: StatusCode, msg: impl Into<String>) -> ApiError {
    (status, Json(ErrorResponse { error: msg.into() }))
}

/// Validate admin Bearer token from request headers.
fn validate_admin(headers: &HeaderMap, validator: &CpskValidator) -> Result<(), ApiError> {
    let auth_header = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "missing Authorization header"))?;

    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| err(StatusCode::UNAUTHORIZED, "expected Bearer token"))?;

    match validator.validate(token) {
        ValidationResult::Valid(info) => {
            if info.has_scope(Action::Admin, "/**") {
                Ok(())
            } else {
                Err(err(StatusCode::FORBIDDEN, "admin scope required"))
            }
        }
        ValidationResult::Expired => Err(err(StatusCode::UNAUTHORIZED, "token expired")),
        _ => Err(err(StatusCode::UNAUTHORIZED, "invalid token")),
    }
}

fn parse_signal_types(types_str: &str) -> Vec<clasp_core::SignalType> {
    types_str
        .split(',')
        .filter_map(|t| match t.trim().to_lowercase().as_str() {
            "param" => Some(clasp_core::SignalType::Param),
            "event" => Some(clasp_core::SignalType::Event),
            "stream" => Some(clasp_core::SignalType::Stream),
            "gesture" => Some(clasp_core::SignalType::Gesture),
            "timeline" => Some(clasp_core::SignalType::Timeline),
            _ => None,
        })
        .collect()
}

async fn query_journal(
    State(state): State<Arc<JournalApiState>>,
    headers: HeaderMap,
    Query(params): Query<JournalQueryParams>,
) -> Result<Json<Vec<JournalEntry>>, ApiError> {
    validate_admin(&headers, &state.validator)?;

    let pattern = params.pattern.as_deref().unwrap_or("/**");
    let types: Vec<clasp_core::SignalType> = params
        .types
        .as_deref()
        .map(parse_signal_types)
        .unwrap_or_default();

    let entries = state
        .journal
        .query(pattern, params.from, params.to, params.limit, &types)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("journal query failed: {}", e)))?;

    Ok(Json(entries))
}

async fn since_journal(
    State(state): State<Arc<JournalApiState>>,
    headers: HeaderMap,
    Query(params): Query<SinceParams>,
) -> Result<Json<Vec<JournalEntry>>, ApiError> {
    validate_admin(&headers, &state.validator)?;

    let entries = state
        .journal
        .since(params.seq, params.limit)
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("journal since failed: {}", e)))?;

    Ok(Json(entries))
}

async fn latest_seq(
    State(state): State<Arc<JournalApiState>>,
    headers: HeaderMap,
) -> Result<Json<LatestSeqResponse>, ApiError> {
    validate_admin(&headers, &state.validator)?;

    let seq = state
        .journal
        .latest_seq()
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("journal latest_seq failed: {}", e)))?;

    Ok(Json(LatestSeqResponse { seq }))
}

async fn load_snapshot(
    State(state): State<Arc<JournalApiState>>,
    headers: HeaderMap,
) -> Result<Json<Vec<ParamSnapshot>>, ApiError> {
    validate_admin(&headers, &state.validator)?;

    let snapshot = state
        .journal
        .load_snapshot()
        .await
        .map_err(|e| err(StatusCode::INTERNAL_SERVER_ERROR, format!("journal load_snapshot failed: {}", e)))?;

    Ok(Json(snapshot.unwrap_or_default()))
}

/// Build the journal query REST router.
pub fn journal_router(state: Arc<JournalApiState>) -> Router {
    Router::new()
        .route("/api/journal/query", get(query_journal))
        .route("/api/journal/since", get(since_journal))
        .route("/api/journal/latest", get(latest_seq))
        .route("/api/journal/snapshot", get(load_snapshot))
        .with_state(state)
}
