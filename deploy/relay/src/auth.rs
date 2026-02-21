//! Authentication module for CLASP Chat.
//!
//! Provides user registration and login with argon2 password hashing,
//! SQLite user storage, and CPSK token generation with scoped permissions.

use anyhow::Result;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    extract::{ConnectInfo, State},
    http::StatusCode,
    routing::post,
    Json, Router,
};
use axum::http::{HeaderValue, Method};
use clasp_core::security::{CpskValidator, Scope, TokenInfo};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tower_http::cors::CorsLayer;

/// Per-key rate limiter for auth endpoints (H7, M6).
/// See pentest ADM-01: Brute-Force Login
struct RateLimiter {
    /// key -> (attempt_count, window_start)
    attempts: HashMap<String, (u32, Instant)>,
}

impl RateLimiter {
    fn new() -> Self {
        Self { attempts: HashMap::new() }
    }

    /// Check if the key is currently rate-limited. Returns true if blocked.
    fn is_blocked(&mut self, key: &str, max_attempts: u32, window: Duration) -> bool {
        let now = Instant::now();
        if let Some((count, start)) = self.attempts.get(key) {
            if now.duration_since(*start) > window {
                self.attempts.remove(key);
                return false;
            }
            *count >= max_attempts
        } else {
            false
        }
    }

    /// Record an attempt. Returns the current count.
    fn record(&mut self, key: &str, window: Duration) -> u32 {
        let now = Instant::now();
        let entry = self.attempts.entry(key.to_string()).or_insert((0, now));
        if now.duration_since(entry.1) > window {
            *entry = (1, now);
            1
        } else {
            entry.0 += 1;
            entry.0
        }
    }

    /// Clear attempts for a key (on success).
    fn clear(&mut self, key: &str) {
        self.attempts.remove(key);
    }

    /// Prune entries older than the window.
    fn prune(&mut self, window: Duration) {
        let now = Instant::now();
        self.attempts.retain(|_, (_, start)| now.duration_since(*start) <= window);
    }
}

/// Shared auth state
pub struct AuthState {
    db: Mutex<Connection>,
    validator: Arc<CpskValidator>,
    login_limiter: Mutex<RateLimiter>,
    register_limiter: Mutex<RateLimiter>,
    /// Scope templates from app config. `{userId}` is replaced at token issue time.
    /// If None, issues full read/write tokens.
    scope_templates: Option<Vec<String>>,
    /// Rate limit configuration (from app config or defaults).
    rate_config: crate::app_config::RateLimitConfig,
}

impl AuthState {
    pub fn new(
        db_path: &str,
        validator: Arc<CpskValidator>,
        scope_templates: Option<Vec<String>>,
        rate_config: crate::app_config::RateLimitConfig,
    ) -> Result<Self> {
        let conn = Connection::open(db_path)?;

        // Create users table
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS users (
                id TEXT PRIMARY KEY,
                username TEXT UNIQUE NOT NULL,
                password_hash TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );"
        )?;

        Ok(Self {
            db: Mutex::new(conn),
            validator,
            login_limiter: Mutex::new(RateLimiter::new()),
            register_limiter: Mutex::new(RateLimiter::new()),
            scope_templates,
            rate_config,
        })
    }

    /// Build scopes for a user by substituting `{userId}` in scope templates.
    fn build_scopes(&self, user_id: &str) -> Vec<String> {
        match &self.scope_templates {
            Some(templates) => templates
                .iter()
                .map(|s| s.replace("{userId}", user_id))
                .collect(),
            None => {
                tracing::warn!("No app config -- issuing full read/write token");
                vec!["read:/**".into(), "write:/**".into()]
            }
        }
    }
}

#[derive(Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
    user_id: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user_id: String,
    username: String,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    error: String,
}


/// Validate a client-supplied user_id (M2).
/// Allows alphanumeric, hyphens, and underscores. Max 64 chars.
fn is_valid_user_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Generate a unique user ID (uses CPSK token generator for randomness)
fn generate_user_id() -> String {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    // Use last 8 chars of a generated token as random suffix
    let rand_part = &CpskValidator::generate_token()[29..];
    format!("u-{}-{}", ts, rand_part)
}

/// Register a new user (with IP rate limiting — M6).
async fn register(
    State(state): State<Arc<AuthState>>,
    request: axum::extract::Request,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip = extract_ip(request.extensions());

    // Rate limit registration per IP
    let register_window = Duration::from_secs(state.rate_config.register_window_secs);
    let register_max = state.rate_config.register_max_attempts;
    {
        let mut limiter = state.register_limiter.lock().unwrap();
        limiter.prune(register_window);
        if limiter.is_blocked(&ip, register_max, register_window) {
            return Err((StatusCode::TOO_MANY_REQUESTS, Json(ErrorResponse {
                error: "Too many registration attempts. Please wait and try again.".into(),
            })));
        }
        limiter.record(&ip, register_window);
    }

    let bytes = axum::body::to_bytes(request.into_body(), 1024 * 16)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Invalid request body".into(),
        })))?;
    let req: AuthRequest = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Invalid JSON".into(),
        })))?;

    let username = req.username.trim().to_string();
    let password = req.password;

    // Validate
    if username.len() < 2 || username.len() > 32 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Username must be 2-32 characters".into(),
        })));
    }
    if password.len() < 6 {
        return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Password must be at least 6 characters".into(),
        })));
    }

    // Hash password with Argon2id (memory-hard, resists GPU/ASIC attacks better than bcrypt)
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Failed to hash password".into(),
        })))?
        .to_string();

    // Use client-provided user_id if present (inherits guest identity on upgrade).
    // Security (C3): reject existing IDs. (M2): validate format.
    let user_id = match req.user_id.filter(|id| !id.trim().is_empty()) {
        Some(id) => {
            if !is_valid_user_id(&id) {
                return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    error: "Invalid user_id format (alphanumeric, hyphens, underscores; max 64 chars)".into(),
                })));
            }
            id
        }
        None => generate_user_id(),
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Insert user (the PRIMARY KEY constraint on `id` also prevents collisions
    // when a client supplies an existing user_id)
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO users (id, username, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            (&user_id, &username, &hash, &now),
        ).map_err(|e| {
            let msg = e.to_string();
            if msg.contains("UNIQUE") {
                // Could be username OR id collision
                (StatusCode::CONFLICT, Json(ErrorResponse {
                    error: "Username or identity already taken".into(),
                }))
            } else {
                (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: "Database error".into(),
                }))
            }
        })?;
    }

    // Generate token and register with validator
    let token = CpskValidator::generate_token();
    let scope_strings = state.build_scopes(&user_id);
    let scopes: Vec<Scope> = scope_strings
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();

    let info = TokenInfo::new(user_id.clone(), scopes)
        .with_subject(&user_id)
        .with_expires_in(Duration::from_secs(86400));

    state.validator.register(token.clone(), info);

    tracing::info!("Registered user: {} ({})", username, user_id);

    Ok(Json(AuthResponse {
        token,
        user_id,
        username,
    }))
}


/// Extract client IP from ConnectInfo extension (set by into_make_service_with_connect_info).
/// Falls back to "unknown" when not available (e.g., in tests).
fn extract_ip(extensions: &axum::http::Extensions) -> String {
    extensions
        .get::<ConnectInfo<SocketAddr>>()
        .map(|ci| ci.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Login an existing user (with brute-force protection — H7).
async fn login(
    State(state): State<Arc<AuthState>>,
    request: axum::extract::Request,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip = extract_ip(request.extensions());

    // Parse body
    let bytes = axum::body::to_bytes(request.into_body(), 1024 * 16)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Invalid request body".into(),
        })))?;
    let req: AuthRequest = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Invalid JSON".into(),
        })))?;

    let username = req.username.trim().to_string();
    let password = req.password;
    let ip_key = format!("ip:{}", ip);
    let user_key = format!("user:{}", username.to_lowercase());
    let login_window = Duration::from_secs(state.rate_config.login_window_secs);
    let login_max = state.rate_config.login_max_attempts;

    // Check rate limits before doing any work
    {
        let mut limiter = state.login_limiter.lock().unwrap();
        // Periodically prune stale entries
        limiter.prune(login_window);

        if limiter.is_blocked(&ip_key, login_max, login_window)
            || limiter.is_blocked(&user_key, login_max, login_window)
        {
            tracing::warn!("Login rate-limited: {} / {}", username, ip);
            return Err((StatusCode::TOO_MANY_REQUESTS, Json(ErrorResponse {
                error: "Too many login attempts. Please wait a minute and try again.".into(),
            })));
        }
    }

    // Look up user
    let (user_id, hash) = {
        let db = state.db.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT id, password_hash FROM users WHERE username = ?1")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Database error".into(),
            })))?;

        stmt.query_row([&username], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        }).map_err(|_| {
            // Record failed attempt (username not found)
            let mut limiter = state.login_limiter.lock().unwrap();
            limiter.record(&ip_key, login_window);
            limiter.record(&user_key, login_window);
            (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid username or password".into(),
            }))
        })?
    };

    // Verify password
    let parsed_hash = PasswordHash::new(&hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Internal error".into(),
        })))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| {
            // Record failed attempt (wrong password)
            let mut limiter = state.login_limiter.lock().unwrap();
            limiter.record(&ip_key, login_window);
            limiter.record(&user_key, login_window);
            (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
                error: "Invalid username or password".into(),
            }))
        })?;

    // Success — clear rate limit counters for this user/IP
    {
        let mut limiter = state.login_limiter.lock().unwrap();
        limiter.clear(&ip_key);
        limiter.clear(&user_key);
    }

    // Generate new token
    let token = CpskValidator::generate_token();
    let scope_strings = state.build_scopes(&user_id);
    let scopes: Vec<Scope> = scope_strings
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();

    let info = TokenInfo::new(user_id.clone(), scopes)
        .with_subject(&user_id)
        .with_expires_in(Duration::from_secs(86400));

    state.validator.register(token.clone(), info);

    tracing::info!("Login: {} ({})", username, user_id);

    Ok(Json(AuthResponse {
        token,
        user_id,
        username,
    }))
}

#[derive(Deserialize)]
pub struct GuestRequest {
    name: Option<String>,
    user_id: Option<String>,
}

/// Issue a guest token (anonymous access, with IP rate limiting — M6).
async fn guest(
    State(state): State<Arc<AuthState>>,
    request: axum::extract::Request,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let ip = extract_ip(request.extensions());

    // Rate limit guest creation per IP
    let register_window = Duration::from_secs(state.rate_config.register_window_secs);
    let register_max = state.rate_config.register_max_attempts;
    {
        let mut limiter = state.register_limiter.lock().unwrap();
        limiter.prune(register_window);
        if limiter.is_blocked(&ip, register_max, register_window) {
            return Err((StatusCode::TOO_MANY_REQUESTS, Json(ErrorResponse {
                error: "Too many requests. Please wait and try again.".into(),
            })));
        }
        limiter.record(&ip, register_window);
    }

    let bytes = axum::body::to_bytes(request.into_body(), 1024 * 16)
        .await
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Invalid request body".into(),
        })))?;
    let req: GuestRequest = serde_json::from_slice(&bytes)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse {
            error: "Invalid JSON".into(),
        })))?;

    // Use client-provided user_id if present (preserves frontend identity for scope matching).
    // Security (C3): reject registered IDs. (M2): validate format.
    let user_id = match req.user_id.filter(|id| !id.trim().is_empty()) {
        Some(id) => {
            if !is_valid_user_id(&id) {
                return Err((StatusCode::BAD_REQUEST, Json(ErrorResponse {
                    error: "Invalid user_id format".into(),
                })));
            }
            id
        }
        None => generate_user_id(),
    };

    // Reject if this user_id belongs to a registered user
    {
        let db = state.db.lock().unwrap();
        let mut stmt = db
            .prepare("SELECT 1 FROM users WHERE id = ?1")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                error: "Database error".into(),
            })))?;
        let id_taken = stmt.exists([&user_id]).unwrap_or(false);
        if id_taken {
            return Err((StatusCode::CONFLICT, Json(ErrorResponse {
                error: "This identity belongs to a registered user. Please sign in.".into(),
            })));
        }
    }

    // Check if requested name conflicts with a registered username
    if let Some(ref name) = req.name {
        let name_lower = name.trim().to_lowercase();
        let taken = {
            let db = state.db.lock().unwrap();
            let mut stmt = db
                .prepare("SELECT 1 FROM users WHERE LOWER(username) = ?1")
                .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
                    error: "Database error".into(),
                })))?;
            stmt.exists([&name_lower]).unwrap_or(false)
        };
        if taken {
            return Err((StatusCode::CONFLICT, Json(ErrorResponse {
                error: "That name belongs to a registered user. Sign in or pick a different name.".into(),
            })));
        }
    }

    let guest_name = req.name
        .filter(|n| !n.trim().is_empty())
        .map(|n| n.trim().to_string())
        .unwrap_or_else(|| format!("guest-{}", &user_id[user_id.len()-6..]));

    let token = CpskValidator::generate_token();
    let scope_strings = state.build_scopes(&user_id);
    let scopes: Vec<Scope> = scope_strings
        .iter()
        .filter_map(|s| Scope::parse(s).ok())
        .collect();

    let info = TokenInfo::new(user_id.clone(), scopes)
        .with_subject(&user_id)
        .with_expires_in(Duration::from_secs(86400));

    state.validator.register(token.clone(), info);

    tracing::info!("Guest joined: {} ({})", guest_name, user_id);

    Ok(Json(AuthResponse {
        token,
        user_id,
        username: guest_name,
    }))
}

/// Build the auth HTTP router.
/// `cors_origins`: comma-separated allowed origins, or empty/None for permissive (dev only).
pub fn auth_router(state: Arc<AuthState>, cors_origins: Option<&str>) -> Router {
    let cors = match cors_origins {
        Some(origins) if !origins.trim().is_empty() => {
            let allowed: Vec<HeaderValue> = origins
                .split(',')
                .filter_map(|o| o.trim().parse().ok())
                .collect();
            CorsLayer::new()
                .allow_origin(allowed)
                .allow_methods([Method::POST, Method::OPTIONS])
                .allow_headers(tower_http::cors::Any)
        }
        _ => {
            tracing::warn!("CORS: permissive mode (set --cors-origin for production)");
            CorsLayer::permissive()
        }
    };

    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/guest", post(guest))
        .layer(cors)
        .with_state(state)
}
