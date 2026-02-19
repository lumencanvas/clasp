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
    extract::State,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use clasp_core::security::{CpskValidator, Scope, TokenInfo};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tower_http::cors::CorsLayer;

/// Shared auth state
pub struct AuthState {
    db: Mutex<Connection>,
    validator: Arc<CpskValidator>,
}

impl AuthState {
    pub fn new(db_path: &str, validator: Arc<CpskValidator>) -> Result<Self> {
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
        })
    }
}

#[derive(Deserialize)]
pub struct AuthRequest {
    username: String,
    password: String,
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

/// Build the 9 scopes for a user
fn build_scopes(user_id: &str) -> Vec<String> {
    vec![
        "read:/chat/**".to_string(),
        format!("write:/chat/user/{}/**", user_id),
        "write:/chat/requests/*".to_string(),
        "write:/chat/room/*/messages".to_string(),
        format!("write:/chat/room/*/presence/{}", user_id),
        format!("write:/chat/room/*/typing/{}", user_id),
        "write:/chat/room/*/reactions/**".to_string(),
        "write:/chat/room/*/crypto/**".to_string(),
        "write:/chat/room/*/admin/**".to_string(),
        "write:/chat/room/*/bans/**".to_string(),
        "write:/chat/room/*/meta".to_string(),
        "write:/chat/registry/rooms/*".to_string(),
    ]
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

/// Register a new user
async fn register(
    State(state): State<Arc<AuthState>>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
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

    // Hash password
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Failed to hash password".into(),
        })))?
        .to_string();

    let user_id = generate_user_id();
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Insert user
    {
        let db = state.db.lock().unwrap();
        db.execute(
            "INSERT INTO users (id, username, password_hash, created_at) VALUES (?1, ?2, ?3, ?4)",
            (&user_id, &username, &hash, &now),
        ).map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                (StatusCode::CONFLICT, Json(ErrorResponse {
                    error: "Username already taken".into(),
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
    let scope_strings = build_scopes(&user_id);
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

/// Login an existing user
async fn login(
    State(state): State<Arc<AuthState>>,
    Json(req): Json<AuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let username = req.username.trim().to_string();
    let password = req.password;

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
        }).map_err(|_| (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
            error: "Invalid username or password".into(),
        })))?
    };

    // Verify password
    let parsed_hash = PasswordHash::new(&hash)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: "Internal error".into(),
        })))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| (StatusCode::UNAUTHORIZED, Json(ErrorResponse {
            error: "Invalid username or password".into(),
        })))?;

    // Generate new token
    let token = CpskValidator::generate_token();
    let scope_strings = build_scopes(&user_id);
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
}

/// Issue a guest token (anonymous access)
async fn guest(
    State(state): State<Arc<AuthState>>,
    Json(req): Json<GuestRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, Json<ErrorResponse>)> {
    let user_id = generate_user_id();

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
    let scope_strings = build_scopes(&user_id);
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

/// Build the auth HTTP router
pub fn auth_router(state: Arc<AuthState>) -> Router {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/guest", post(guest))
        .layer(CorsLayer::permissive())
        .with_state(state)
}
