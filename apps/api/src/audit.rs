use serde_json::Value as JsonValue;
use sqlx::PgPool;
use tracing::error;

/// Audit event types recorded by the system.
///
/// PRIVACY RULE: metadata must NEVER contain passwords, password hashes,
/// session tokens, or raw log content.
#[derive(Debug, Clone, Copy)]
pub enum AuditEventType {
    UserRegistered,
    LoginSuccess,
    LoginFailed,
    Logout,
    PasswordResetRequested,
    PasswordResetCompleted,
    OrgCreated,
    OrgMemberRoleChanged,
    OrgLicenseChanged,
    UserLicenseChanged,
}

impl AuditEventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::UserRegistered => "user_registered",
            Self::LoginSuccess => "login_success",
            Self::LoginFailed => "login_failed",
            Self::Logout => "logout",
            Self::PasswordResetRequested => "password_reset_requested",
            Self::PasswordResetCompleted => "password_reset_completed",
            Self::OrgCreated => "org_created",
            Self::OrgMemberRoleChanged => "org_member_role_changed",
            Self::OrgLicenseChanged => "org_license_changed",
            Self::UserLicenseChanged => "user_license_changed",
        }
    }
}

/// Records an audit event. Failures are logged but never propagated —
/// an audit insert failure must not break the request that triggered it.
pub async fn record_audit_event(
    pool: &PgPool,
    event_type: AuditEventType,
    actor_user_id: Option<i64>,
    actor_ip: Option<&str>,
    target_user_id: Option<i64>,
    org_id: Option<i64>,
    metadata: JsonValue,
) {
    let result = sqlx::query(
        r#"
        INSERT INTO audit_events (event_type, actor_user_id, actor_ip, target_user_id, org_id, metadata)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#,
    )
    .bind(event_type.as_str())
    .bind(actor_user_id)
    .bind(actor_ip)
    .bind(target_user_id)
    .bind(org_id)
    .bind(&metadata)
    .execute(pool)
    .await;

    if let Err(e) = result {
        error!(
            event_type = event_type.as_str(),
            "failed to record audit event: {e}"
        );
    }
}
