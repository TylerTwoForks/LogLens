use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use std::sync::Arc;

use crate::audit::{record_audit_event, AuditEventType};
use crate::auth::authenticate;
use crate::error::{ApiError, ErrorResponse};
use crate::helpers::{fetch_org_role, fetch_org_summary, require_permission};
use crate::models::*;
use crate::AppState;

#[utoipa::path(
    get,
    path = "/v1/orgs",
    tag = "org",
    responses(
        (status = 200, description = "Organizations for authenticated user", body = ListOrgsResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse)
    )
)]
pub async fn list_orgs(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListOrgsResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;

    let rows = sqlx::query_as::<_, OrgRow>(
        r#"
        SELECT
          o.id AS org_id,
          o.name,
          m.role,
          COALESCE(ol.tier, 'free') AS tier,
          COALESCE(ol.status, 'active') AS status
        FROM organization_memberships m
        JOIN organizations o
          ON o.id = m.org_id
        LEFT JOIN organization_licenses ol
          ON ol.org_id = o.id
        WHERE m.user_id = $1
        ORDER BY o.id
        "#,
    )
    .bind(user.user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| ApiError::internal(format!("failed to list organizations: {error}")))?;

    let orgs = rows
        .into_iter()
        .map(to_org_summary)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ListOrgsResponse { orgs }))
}

#[utoipa::path(
    post,
    path = "/v1/orgs",
    tag = "org",
    request_body = CreateOrgRequest,
    responses(
        (status = 200, description = "Created organization with owner membership", body = OrgSummary),
        (status = 400, description = "Invalid organization payload", body = ErrorResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse)
    )
)]
pub async fn create_org(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateOrgRequest>,
) -> Result<Json<OrgSummary>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let org_name = request.name.trim();

    if org_name.is_empty() {
        return Err(ApiError::bad_request("organization name cannot be empty"));
    }

    let mut transaction = state.pool.begin().await.map_err(|error| {
        ApiError::internal(format!("failed to begin create-org transaction: {error}"))
    })?;

    let org_id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO organizations (name)
        VALUES ($1)
        RETURNING id
        "#,
    )
    .bind(org_name)
    .fetch_one(&mut *transaction)
    .await
    .map_err(|error| ApiError::internal(format!("failed to create organization: {error}")))?;

    sqlx::query(
        r#"
        INSERT INTO organization_memberships (org_id, user_id, role)
        VALUES ($1, $2, 'owner')
        "#,
    )
    .bind(org_id)
    .bind(user.user_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to create owner membership: {error}"))
    })?;

    sqlx::query(
        r#"
        INSERT INTO organization_licenses (org_id, tier, status, updated_at)
        VALUES ($1, 'free', 'active', NOW())
        "#,
    )
    .bind(org_id)
    .execute(&mut *transaction)
    .await
    .map_err(|error| {
        ApiError::internal(format!(
            "failed to initialize organization license: {error}"
        ))
    })?;

    transaction.commit().await.map_err(|error| {
        ApiError::internal(format!("failed to commit create-org transaction: {error}"))
    })?;

    record_audit_event(
        &state.pool,
        AuditEventType::OrgCreated,
        Some(user.user_id),
        None,
        None,
        Some(org_id),
        serde_json::json!({ "org_name": org_name }),
    )
    .await;

    let summary = fetch_org_summary(&state.pool, org_id, user.user_id).await?;
    Ok(Json(summary))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}",
    tag = "org",
    params(
        ("org_id" = i64, Path, description = "Organization identifier")
    ),
    responses(
        (status = 200, description = "Organization summary for current member", body = OrgSummary),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse)
    )
)]
pub async fn get_org(
    Path(org_id): Path<i64>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<OrgSummary>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let summary = fetch_org_summary(&state.pool, org_id, user.user_id).await?;
    Ok(Json(summary))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}/members",
    tag = "org",
    params(
        ("org_id" = i64, Path, description = "Organization identifier")
    ),
    responses(
        (status = 200, description = "Organization members", body = OrgMembersResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse)
    )
)]
pub async fn list_org_members(
    Path(org_id): Path<i64>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<OrgMembersResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    let rows = sqlx::query_as::<_, OrgMemberRow>(
        r#"
        SELECT
          u.id AS user_id,
          u.email,
          m.role
        FROM organization_memberships m
        JOIN app_users u
          ON u.id = m.user_id
        WHERE m.org_id = $1
        ORDER BY u.id
        "#,
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to load organization members: {error}"))
    })?;

    let members = rows
        .into_iter()
        .map(|row| {
            Ok(OrgMember {
                user_id: row.user_id,
                email: row.email,
                role: OrgRole::from_db(&row.role)?,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;

    Ok(Json(OrgMembersResponse { members }))
}

#[utoipa::path(
    patch,
    path = "/v1/orgs/{org_id}/license",
    tag = "billing",
    request_body = UpdateLicenseRequest,
    params(
        ("org_id" = i64, Path, description = "Organization identifier")
    ),
    responses(
        (status = 200, description = "Updated organization license", body = LicenseSnapshot),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Role is not allowed to manage billing", body = ErrorResponse)
    )
)]
pub async fn update_org_license(
    Path(org_id): Path<i64>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateLicenseRequest>,
) -> Result<Json<LicenseSnapshot>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::ManageBilling)?;

    sqlx::query(
        r#"
        INSERT INTO organization_licenses (org_id, tier, status, updated_at)
        VALUES ($1, $2, $3, NOW())
        ON CONFLICT (org_id) DO UPDATE
        SET tier = EXCLUDED.tier,
            status = EXCLUDED.status,
            updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .bind(request.tier.as_str())
    .bind(request.status.as_str())
    .execute(&state.pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to update organization license: {error}"))
    })?;

    record_audit_event(
        &state.pool,
        AuditEventType::OrgLicenseChanged,
        Some(user.user_id),
        None,
        None,
        Some(org_id),
        serde_json::json!({ "tier": request.tier.as_str(), "status": request.status.as_str() }),
    )
    .await;

    Ok(Json(license_snapshot(request.tier, request.status)))
}

#[utoipa::path(
    patch,
    path = "/v1/orgs/{org_id}/members/{member_user_id}/role",
    tag = "org",
    request_body = UpdateMemberRoleRequest,
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("member_user_id" = i64, Path, description = "Target member user identifier")
    ),
    responses(
        (status = 200, description = "Updated member role", body = MutationResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Role is not allowed to manage members", body = ErrorResponse),
        (status = 404, description = "Target member not found in organization", body = ErrorResponse)
    )
)]
pub async fn update_org_member_role(
    Path((org_id, member_user_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(request): Json<UpdateMemberRoleRequest>,
) -> Result<Json<MutationResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::ManageMembers)?;

    let result = sqlx::query(
        r#"
        UPDATE organization_memberships
        SET role = $3
        WHERE org_id = $1
          AND user_id = $2
        "#,
    )
    .bind(org_id)
    .bind(member_user_id)
    .bind(request.role.as_str())
    .execute(&state.pool)
    .await
    .map_err(|error| ApiError::internal(format!("failed to update member role: {error}")))?;

    if result.rows_affected() == 0 {
        return Err(ApiError::not_found(
            "target user is not a member of this organization",
        ));
    }

    record_audit_event(
        &state.pool,
        AuditEventType::OrgMemberRoleChanged,
        Some(user.user_id),
        None,
        Some(member_user_id),
        Some(org_id),
        serde_json::json!({ "new_role": request.role.as_str() }),
    )
    .await;

    Ok(Json(MutationResponse {
        message: "role updated".to_owned(),
    }))
}
