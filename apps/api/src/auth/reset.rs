use axum::{extract::State, Json};
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use utoipa::ToSchema;

use crate::audit::{record_audit_event, AuditEventType};
use crate::error::ApiError;
use crate::models::MutationResponse;
use crate::AppState;

use super::password::{hash_password, validate_password};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ForgotPasswordRequest {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ForgotPasswordResponse {
    pub message: String,
    /// Only returned in dev mode for testing. In production, the token would
    /// be sent via email.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reset_token: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResetPasswordRequest {
    pub token: String,
    pub new_password: String,
}

fn generate_reset_token() -> String {
    let bytes: [u8; 32] = rand::rng().random();
    hex::encode(bytes)
}

fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    hex::encode(digest)
}

#[utoipa::path(
    post,
    path = "/v1/auth/forgot-password",
    tag = "auth",
    request_body = ForgotPasswordRequest,
    responses(
        (status = 200, description = "Reset token generated (if user exists)", body = ForgotPasswordResponse)
    )
)]
pub async fn forgot_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ForgotPasswordRequest>,
) -> Result<Json<ForgotPasswordResponse>, ApiError> {
    let email = request.email.trim().to_lowercase();

    // Look up user — always return 200 to avoid email enumeration
    let user_id = sqlx::query_scalar::<_, i64>(
        "SELECT id FROM app_users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to look up user: {e}")))?;

    let Some(user_id) = user_id else {
        // User not found — return generic success message
        return Ok(Json(ForgotPasswordResponse {
            message: "If that email is registered, a reset token has been generated.".to_owned(),
            reset_token: None,
        }));
    };

    // Invalidate any existing unused tokens for this user
    sqlx::query(
        "UPDATE password_reset_tokens SET used_at = NOW() WHERE user_id = $1 AND used_at IS NULL",
    )
    .bind(user_id)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to invalidate old tokens: {e}")))?;

    // Generate and store new token (1-hour expiry)
    let raw_token = generate_reset_token();
    let token_hash = hash_token(&raw_token);

    sqlx::query(
        r#"
        INSERT INTO password_reset_tokens (user_id, token_hash, expires_at)
        VALUES ($1, $2, NOW() + INTERVAL '1 hour')
        "#,
    )
    .bind(user_id)
    .bind(&token_hash)
    .execute(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to store reset token: {e}")))?;

    record_audit_event(
        &state.pool,
        AuditEventType::PasswordResetRequested,
        Some(user_id),
        None,
        Some(user_id),
        None,
        serde_json::json!({}),
    )
    .await;

    // Return raw token in response for dev/testing (no email delivery yet)
    Ok(Json(ForgotPasswordResponse {
        message: "If that email is registered, a reset token has been generated.".to_owned(),
        reset_token: Some(raw_token),
    }))
}

#[utoipa::path(
    post,
    path = "/v1/auth/reset-password",
    tag = "auth",
    request_body = ResetPasswordRequest,
    responses(
        (status = 200, description = "Password reset successful", body = MutationResponse),
        (status = 400, description = "Invalid or expired token", body = crate::error::ErrorResponse)
    )
)]
pub async fn reset_password(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ResetPasswordRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    validate_password(&request.new_password)?;

    let token_hash = hash_token(&request.token);

    // Look up valid (unused, not expired) token
    #[derive(sqlx::FromRow)]
    struct TokenRow {
        id: i64,
        user_id: i64,
    }

    let token_row = sqlx::query_as::<_, TokenRow>(
        r#"
        SELECT id, user_id FROM password_reset_tokens
        WHERE token_hash = $1 AND used_at IS NULL AND expires_at > NOW()
        "#,
    )
    .bind(&token_hash)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to look up reset token: {e}")))?;

    let token_row = token_row
        .ok_or_else(|| ApiError::bad_request("invalid or expired reset token"))?;

    // Hash new password and update user
    let new_hash = hash_password(&request.new_password)?;

    sqlx::query("UPDATE app_users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(&new_hash)
        .bind(token_row.user_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal(format!("failed to update password: {e}")))?;

    // Mark token as used
    sqlx::query("UPDATE password_reset_tokens SET used_at = NOW() WHERE id = $1")
        .bind(token_row.id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal(format!("failed to mark token used: {e}")))?;

    record_audit_event(
        &state.pool,
        AuditEventType::PasswordResetCompleted,
        Some(token_row.user_id),
        None,
        Some(token_row.user_id),
        None,
        serde_json::json!({}),
    )
    .await;

    Ok(Json(MutationResponse {
        message: "password reset successful".to_owned(),
    }))
}
