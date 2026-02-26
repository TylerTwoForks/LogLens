use axum::{extract::State, http::HeaderMap, Json};
use std::sync::Arc;

use crate::audit::{record_audit_event, AuditEventType};
use crate::auth::authenticate;
use crate::error::ApiError;
use crate::helpers::fetch_individual_license;
use crate::models::*;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/v1/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current authenticated user context", body = MeResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse)
    )
)]
pub async fn me(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<MeResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let license = fetch_individual_license(&state.pool, user.user_id).await?;

    Ok(Json(MeResponse {
        user_id: user.user_id,
        auth_subject: user.auth_subject,
        email: user.email,
        individual_license: license,
    }))
}

use crate::error::ErrorResponse;

#[utoipa::path(
    patch,
    path = "/v1/me/license",
    tag = "billing",
    request_body = UpdateLicenseRequest,
    responses(
        (status = 200, description = "Updated individual license", body = LicenseSnapshot),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse)
    )
)]
pub async fn update_me_license(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateLicenseRequest>,
) -> Result<Json<LicenseSnapshot>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;

    sqlx::query(
        r#"
        INSERT INTO individual_licenses (user_id, tier, status, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (user_id) DO UPDATE
        SET tier = EXCLUDED.tier,
            status = EXCLUDED.status,
            updated_at = NOW()
        "#,
    )
    .bind(user.user_id)
    .bind(request.tier.as_str())
    .bind(request.status.as_str())
    .execute(&state.pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to update individual license: {error}"))
    })?;

    record_audit_event(
        &state.pool,
        AuditEventType::UserLicenseChanged,
        Some(user.user_id),
        None,
        Some(user.user_id),
        None,
        serde_json::json!({ "tier": request.tier.as_str(), "status": request.status.as_str() }),
    )
    .await;

    Ok(Json(license_snapshot(request.tier, request.status)))
}
