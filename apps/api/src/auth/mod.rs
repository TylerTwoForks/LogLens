pub mod password;
pub mod reset;

use axum::{extract::State, http::HeaderMap, Json};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::sync::Arc;
use utoipa::ToSchema;

use crate::audit::{record_audit_event, AuditEventType};
use crate::error::ApiError;
use crate::models::UserRow;
use crate::AppState;

use password::{hash_email_to_subject, hash_password, validate_email, validate_password, verify_password};

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub auth_subject: String,
    pub email: String,
}

// ── Request / Response types ────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct AuthResponse {
    pub user_id: i64,
    pub auth_subject: String,
    pub email: String,
}

// ── Auth endpoints ──────────────────────────────────────────────────

#[utoipa::path(
    post,
    path = "/v1/auth/register",
    tag = "auth",
    request_body = RegisterRequest,
    responses(
        (status = 200, description = "User registered", body = AuthResponse),
        (status = 400, description = "Invalid input", body = crate::error::ErrorResponse),
        (status = 409, description = "Email already registered", body = crate::error::ErrorResponse)
    )
)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(request): Json<RegisterRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let email = request.email.trim().to_lowercase();
    validate_email(&email)?;
    validate_password(&request.password)?;

    let password_hash = hash_password(&request.password)?;
    let auth_subject = hash_email_to_subject(&email);

    // Check if email already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM app_users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to check existing user: {e}")))?;

    if existing.is_some() {
        return Err(ApiError {
            status: axum::http::StatusCode::CONFLICT,
            message: "email already registered".to_owned(),
        });
    }

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO app_users (auth_subject, email, password_hash, updated_at)
        VALUES ($1, $2, $3, NOW())
        RETURNING id, email
        "#,
    )
    .bind(&auth_subject)
    .bind(&email)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to create user: {e}")))?;

    // Initialize free license
    sqlx::query(
        r#"
        INSERT INTO individual_licenses (user_id, tier, status, updated_at)
        VALUES ($1, 'free', 'active', NOW())
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(user.id)
    .execute(&state.pool)
    .await
    .map_err(|e| {
        ApiError::internal(format!("failed to initialize individual license: {e}"))
    })?;

    record_audit_event(
        &state.pool,
        AuditEventType::UserRegistered,
        Some(user.id),
        None,
        Some(user.id),
        None,
        serde_json::json!({ "email": &user.email }),
    )
    .await;

    Ok(Json(AuthResponse {
        user_id: user.id,
        auth_subject,
        email: user.email,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/auth/login",
    tag = "auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials", body = crate::error::ErrorResponse)
    )
)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let email = request.email.trim().to_lowercase();

    #[derive(sqlx::FromRow)]
    struct UserWithPassword {
        id: i64,
        auth_subject: String,
        email: String,
        password_hash: Option<String>,
    }

    let user = sqlx::query_as::<_, UserWithPassword>(
        "SELECT id, auth_subject, email, password_hash FROM app_users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to look up user: {e}")))?;

    let Some(user) = user else {
        record_audit_event(
            &state.pool,
            AuditEventType::LoginFailed,
            None,
            None,
            None,
            None,
            serde_json::json!({ "reason": "unknown_email" }),
        )
        .await;
        return Err(ApiError::unauthorized("invalid credentials"));
    };

    let password_hash = user
        .password_hash
        .as_deref()
        .ok_or_else(|| ApiError::unauthorized("invalid credentials"))?;

    if !verify_password(&request.password, password_hash)? {
        record_audit_event(
            &state.pool,
            AuditEventType::LoginFailed,
            Some(user.id),
            None,
            Some(user.id),
            None,
            serde_json::json!({ "reason": "wrong_password" }),
        )
        .await;
        return Err(ApiError::unauthorized("invalid credentials"));
    }

    record_audit_event(
        &state.pool,
        AuditEventType::LoginSuccess,
        Some(user.id),
        None,
        Some(user.id),
        None,
        serde_json::json!({}),
    )
    .await;

    Ok(Json(AuthResponse {
        user_id: user.id,
        auth_subject: user.auth_subject,
        email: user.email,
    }))
}

#[utoipa::path(
    post,
    path = "/v1/auth/logout",
    tag = "auth",
    responses(
        (status = 200, description = "Logged out", body = crate::models::MutationResponse)
    )
)]
pub async fn logout(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Json<crate::models::MutationResponse> {
    // Session invalidation is handled by the Next.js layer (cookie deletion).
    // This endpoint exists for API completeness and audit recording.
    let actor_id = if let Ok(user) = authenticate(&headers, &state.pool).await {
        Some(user.user_id)
    } else {
        None
    };

    record_audit_event(
        &state.pool,
        AuditEventType::Logout,
        actor_id,
        None,
        actor_id,
        None,
        serde_json::json!({}),
    )
    .await;

    Json(crate::models::MutationResponse {
        message: "logged out".to_owned(),
    })
}

// ── Header-based auth (existing mechanism, unchanged) ───────────────

pub async fn authenticate(
    headers: &HeaderMap,
    pool: &PgPool,
) -> Result<AuthenticatedUser, ApiError> {
    let auth_subject = required_header(headers, "x-loglens-auth-sub")?.to_owned();
    let email = optional_header(headers, "x-loglens-auth-email")
        .map_or_else(|| format!("{auth_subject}@loglens.local"), str::to_owned);

    let user = sqlx::query_as::<_, UserRow>(
        r#"
        INSERT INTO app_users (auth_subject, email, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (auth_subject) DO UPDATE
        SET email = EXCLUDED.email,
            updated_at = NOW()
        RETURNING id, email
        "#,
    )
    .bind(&auth_subject)
    .bind(&email)
    .fetch_one(pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to resolve authenticated user: {error}"))
    })?;

    sqlx::query(
        r#"
        INSERT INTO individual_licenses (user_id, tier, status, updated_at)
        VALUES ($1, 'free', 'active', NOW())
        ON CONFLICT (user_id) DO NOTHING
        "#,
    )
    .bind(user.id)
    .execute(pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to initialize individual license: {error}"))
    })?;

    Ok(AuthenticatedUser {
        user_id: user.id,
        auth_subject,
        email: user.email,
    })
}

pub fn required_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, ApiError> {
    let value = optional_header(headers, name).ok_or_else(|| {
        ApiError::unauthorized(format!("missing required authentication header: {name}"))
    })?;

    if value.is_empty() {
        return Err(ApiError::unauthorized(format!(
            "authentication header cannot be empty: {name}"
        )));
    }

    Ok(value)
}

pub fn optional_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}
