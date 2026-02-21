//! Health check HTTP server for liveness and readiness probes.
//!
//! Exposes:
//! - `GET /healthz` — Liveness: returns 200 if the process is running
//! - `GET /readyz`  — Readiness: returns 200 if the router is accepting connections

use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Shared health state, checked by readiness probes and set during shutdown.
pub struct HealthState {
    /// Set to `true` once the router is ready to accept connections.
    /// Set back to `false` during graceful shutdown.
    pub ready: AtomicBool,
}

impl HealthState {
    pub fn new() -> Self {
        Self {
            ready: AtomicBool::new(false),
        }
    }
}

/// Start the health check HTTP server. Runs until the listener is dropped.
pub async fn start_health_server(addr: std::net::SocketAddr, state: Arc<HealthState>) {
    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .with_state(state);

    match tokio::net::TcpListener::bind(addr).await {
        Ok(listener) => {
            tracing::info!("Health: http://{}/healthz", addr);
            if let Err(e) = axum::serve(listener, app).await {
                tracing::error!("Health server error: {}", e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to bind health server on {}: {}", addr, e);
        }
    }
}

async fn healthz() -> (StatusCode, &'static str) {
    (StatusCode::OK, "ok\n")
}

async fn readyz(State(state): State<Arc<HealthState>>) -> (StatusCode, &'static str) {
    if state.ready.load(Ordering::Relaxed) {
        (StatusCode::OK, "ready\n")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "not ready\n")
    }
}
