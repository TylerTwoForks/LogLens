use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::models::*;
use crate::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        version,
        openapi_json,
        super::me::me,
        super::me::update_me_license,
        super::orgs::list_orgs,
        super::orgs::create_org,
        super::orgs::get_org,
        super::orgs::list_org_members,
        super::orgs::update_org_license,
        super::orgs::update_org_member_role,
        super::jobs::upload_logs,
        super::jobs::list_jobs,
        super::jobs::get_job,
        super::jobs::delete_job,
        super::events::list_job_benchmarks,
        super::events::list_job_events,
        super::events::event_summary,
        crate::auth::register,
        crate::auth::login,
        crate::auth::logout,
        crate::auth::reset::forgot_password,
        crate::auth::reset::reset_password
    ),
    components(schemas(
        crate::auth::RegisterRequest,
        crate::auth::LoginRequest,
        crate::auth::AuthResponse,
        crate::auth::reset::ForgotPasswordRequest,
        crate::auth::reset::ForgotPasswordResponse,
        crate::auth::reset::ResetPasswordRequest,
        HealthResponse,
        VersionResponse,
        ErrorResponse,
        OrgRole,
        LicenseTier,
        LicenseStatus,
        LicenseSnapshot,
        MeResponse,
        OrgSummary,
        ListOrgsResponse,
        CreateOrgRequest,
        OrgMembersResponse,
        OrgMember,
        UpdateLicenseRequest,
        UpdateMemberRoleRequest,
        MutationResponse,
        JobStatus,
        ParseJobResponse,
        UploadResponse,
        ListJobsQuery,
        ListJobsResponse,
        BenchmarkSnapshotResponse,
        ListBenchmarksResponse,
        LogEventResponse,
        ListEventsQuery,
        ListEventsResponse,
        EventSummaryQuery,
        EventSummaryResponse,
        EventTypeBucket,
        TimelineBucket
    )),
    tags(
        (name = "service", description = "LogLens API foundation endpoints"),
        (name = "auth", description = "Authentication and identity endpoints"),
        (name = "org", description = "Organization membership and role-gated endpoints"),
        (name = "billing", description = "License and entitlement endpoints"),
        (name = "ingest", description = "Log upload, parsing, and job management endpoints")
    )
)]
pub struct ApiDoc;

use crate::error::ErrorResponse;

#[utoipa::path(
    get,
    path = "/health",
    tag = "service",
    responses(
        (status = 200, description = "Service healthy", body = HealthResponse),
        (status = 503, description = "Database unavailable", body = HealthResponse)
    )
)]
pub async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthResponse>) {
    let status = match sqlx::query_scalar::<_, i64>("SELECT 1::bigint")
        .fetch_one(&state.pool)
        .await
    {
        Ok(_) => (StatusCode::OK, "up"),
        Err(_) => (StatusCode::SERVICE_UNAVAILABLE, "degraded"),
    };

    (
        status.0,
        Json(HealthResponse {
            status: status.1.to_owned(),
            database: status.1.to_owned(),
        }),
    )
}

#[utoipa::path(
    get,
    path = "/version",
    tag = "service",
    responses(
        (status = 200, description = "Service version", body = VersionResponse)
    )
)]
pub async fn version(State(state): State<Arc<AppState>>) -> Json<VersionResponse> {
    Json(VersionResponse {
        version: state.version.clone(),
    })
}

#[utoipa::path(
    get,
    path = "/openapi.json",
    tag = "service",
    responses(
        (status = 200, description = "OpenAPI contract")
    )
)]
pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}
