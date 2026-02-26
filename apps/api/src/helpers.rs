use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::*;

pub fn role_allows(role: &OrgRole, permission: OrgPermission) -> bool {
    match permission {
        OrgPermission::View => true,
        OrgPermission::ManageMembers => matches!(role, OrgRole::Owner | OrgRole::Admin),
        OrgPermission::ManageBilling => matches!(role, OrgRole::Owner | OrgRole::Admin),
    }
}

pub fn require_permission(
    role: Option<OrgRole>,
    permission: OrgPermission,
) -> Result<OrgRole, ApiError> {
    let role = role.ok_or_else(|| ApiError::forbidden("cross-organization access denied"))?;

    if role_allows(&role, permission) {
        Ok(role)
    } else {
        Err(ApiError::forbidden(
            "role does not have permission for this action",
        ))
    }
}

pub async fn fetch_org_role(
    pool: &PgPool,
    org_id: i64,
    user_id: i64,
) -> Result<Option<OrgRole>, ApiError> {
    let role = sqlx::query_scalar::<_, String>(
        r#"
        SELECT role
        FROM organization_memberships
        WHERE org_id = $1
          AND user_id = $2
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to resolve organization role: {error}"))
    })?;

    role.map(|value| OrgRole::from_db(&value)).transpose()
}

pub async fn fetch_org_summary(
    pool: &PgPool,
    org_id: i64,
    user_id: i64,
) -> Result<OrgSummary, ApiError> {
    let row = sqlx::query_as::<_, OrgRow>(
        r#"
        SELECT
          o.id AS org_id,
          o.name,
          m.role,
          COALESCE(ol.tier, 'free') AS tier,
          COALESCE(ol.status, 'active') AS status
        FROM organizations o
        JOIN organization_memberships m
          ON m.org_id = o.id
         AND m.user_id = $2
        LEFT JOIN organization_licenses ol
          ON ol.org_id = o.id
        WHERE o.id = $1
        "#,
    )
    .bind(org_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to fetch organization summary: {error}"))
    })?;

    let row = row.ok_or_else(|| ApiError::forbidden("cross-organization access denied"))?;
    to_org_summary(row)
}

pub async fn fetch_individual_license(
    pool: &PgPool,
    user_id: i64,
) -> Result<LicenseSnapshot, ApiError> {
    let row = sqlx::query_as::<_, LicenseRow>(
        r#"
        SELECT tier, status
        FROM individual_licenses
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map_err(|error| {
        ApiError::internal(format!("failed to fetch individual license: {error}"))
    })?;

    let tier = LicenseTier::from_db(&row.tier)?;
    let status = LicenseStatus::from_db(&row.status)?;
    Ok(license_snapshot(tier, status))
}
