use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use axum_extra::extract::Multipart;
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::{net::SocketAddr, sync::Arc};
use tokio::sync::Semaphore;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use utoipa::{OpenApi, ToSchema};

const MAX_CONCURRENT_PARSE_JOBS: usize = 4;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
    version: String,
    parse_semaphore: Arc<Semaphore>,
}

#[derive(Debug, Serialize, ToSchema)]
struct ErrorResponse {
    error: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct HealthResponse {
    status: String,
    database: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct VersionResponse {
    version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum OrgRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl OrgRole {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            "viewer" => Ok(Self::Viewer),
            _ => Err(ApiError::internal("invalid role persisted in database")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LicenseTier {
    Free,
    Pro,
    Enterprise,
}

impl LicenseTier {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Pro => "pro",
            Self::Enterprise => "enterprise",
        }
    }

    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "free" => Ok(Self::Free),
            "pro" => Ok(Self::Pro),
            "enterprise" => Ok(Self::Enterprise),
            _ => Err(ApiError::internal(
                "invalid license tier persisted in database",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum LicenseStatus {
    Active,
    PastDue,
    Canceled,
}

impl LicenseStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::PastDue => "past_due",
            Self::Canceled => "canceled",
        }
    }

    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "active" => Ok(Self::Active),
            "past_due" => Ok(Self::PastDue),
            "canceled" => Ok(Self::Canceled),
            _ => Err(ApiError::internal(
                "invalid license status persisted in database",
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, ToSchema, PartialEq, Eq)]
struct LicenseSnapshot {
    tier: LicenseTier,
    status: LicenseStatus,
    features: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct MeResponse {
    user_id: i64,
    auth_subject: String,
    email: String,
    individual_license: LicenseSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
struct OrgSummary {
    org_id: i64,
    name: String,
    role: OrgRole,
    license: LicenseSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
struct ListOrgsResponse {
    orgs: Vec<OrgSummary>,
}

#[derive(Debug, Deserialize, ToSchema)]
struct CreateOrgRequest {
    name: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct OrgMembersResponse {
    members: Vec<OrgMember>,
}

#[derive(Debug, Serialize, ToSchema)]
struct OrgMember {
    user_id: i64,
    email: String,
    role: OrgRole,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateLicenseRequest {
    tier: LicenseTier,
    status: LicenseStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateMemberRoleRequest {
    role: OrgRole,
}

#[derive(Debug, Serialize, ToSchema)]
struct MutationResponse {
    message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
}

impl JobStatus {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }

    fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            _ => Err(ApiError::internal("invalid job status persisted in database")),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct ParseJobResponse {
    job_id: i64,
    org_id: i64,
    file_name: String,
    status: JobStatus,
    total_lines: i64,
    parsed_lines: i64,
    benchmark_count: i32,
    error_message: Option<String>,
    created_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    expires_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct UploadResponse {
    jobs: Vec<ParseJobResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
struct BenchmarkSnapshotResponse {
    sequence: i32,
    label: String,
    query_rows: i64,
    query_rows_limit: i64,
    query_rows_delta: i64,
    heap_size_pct: f64,
    heap_size_bytes_limit: i64,
    heap_size_delta: f64,
    cpu_time_ms: i64,
    cpu_time_limit: i64,
    cpu_time_delta: i64,
    dml_statements: i64,
    dml_statements_limit: i64,
    soql_queries: i64,
    soql_queries_limit: i64,
}

#[derive(Debug, Serialize, ToSchema)]
struct ListBenchmarksResponse {
    benchmarks: Vec<BenchmarkSnapshotResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
struct LogEventResponse {
    line_index: i32,
    timestamp: String,
    nanos: Option<i64>,
    event_type: String,
    line_number: Option<i32>,
    log_level: Option<String>,
    class_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
struct ListJobsQuery {
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct ListJobsResponse {
    jobs: Vec<ParseJobResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
struct ListEventsQuery {
    #[serde(default)]
    offset: Option<i64>,
    #[serde(default)]
    limit: Option<i64>,
    #[serde(default)]
    event_type: Option<String>,
    #[serde(default)]
    log_level: Option<String>,
    /// Case-insensitive partial match on event_type
    #[serde(default)]
    search: Option<String>,
    #[serde(default)]
    class_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
struct ListEventsResponse {
    events: Vec<LogEventResponse>,
    total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
struct EventTypeBucket {
    event_type: String,
    count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
struct TimelineBucket {
    nanos_start: i64,
    nanos_end: i64,
    count: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
struct EventSummaryQuery {
    #[serde(default = "default_timeline_buckets")]
    buckets: i64,
}

fn default_timeline_buckets() -> i64 {
    50
}

#[derive(Debug, Serialize, ToSchema)]
struct EventSummaryResponse {
    event_type_counts: Vec<EventTypeBucket>,
    timeline: Vec<TimelineBucket>,
    total_events: i64,
    class_names: Vec<String>,
}

#[derive(Debug, FromRow)]
struct ParseJobRow {
    id: i64,
    org_id: i64,
    file_name: String,
    status: String,
    total_lines: i64,
    parsed_lines: i64,
    benchmark_count: i32,
    error_message: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
struct BenchmarkRow {
    sequence: i32,
    label: String,
    query_rows: i64,
    query_rows_limit: i64,
    query_rows_delta: i64,
    heap_size_pct: f64,
    heap_size_bytes_limit: i64,
    heap_size_delta: f64,
    cpu_time_ms: i64,
    cpu_time_limit: i64,
    cpu_time_delta: i64,
    dml_statements: i64,
    dml_statements_limit: i64,
    soql_queries: i64,
    soql_queries_limit: i64,
}

#[derive(Debug, FromRow)]
struct LogEventRow {
    line_index: i32,
    timestamp: String,
    nanos: Option<i64>,
    event_type: String,
    line_number: Option<i32>,
    log_level: Option<String>,
    class_name: Option<String>,
}

#[derive(Debug)]
struct AuthenticatedUser {
    user_id: i64,
    auth_subject: String,
    email: String,
}

#[derive(Debug, Clone, Copy)]
enum OrgPermission {
    View,
    ManageMembers,
    ManageBilling,
}

#[derive(Debug, FromRow)]
struct UserRow {
    id: i64,
    email: String,
}

#[derive(Debug, FromRow)]
struct LicenseRow {
    tier: String,
    status: String,
}

#[derive(Debug, FromRow)]
struct OrgRow {
    org_id: i64,
    name: String,
    role: String,
    tier: String,
    status: String,
}

#[derive(Debug, FromRow)]
struct OrgMemberRow {
    user_id: i64,
    email: String,
    role: String,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        version,
        openapi_json,
        me,
        update_me_license,
        list_orgs,
        create_org,
        get_org,
        list_org_members,
        update_org_license,
        update_org_member_role,
        upload_logs,
        list_jobs,
        get_job,
        delete_job,
        list_job_benchmarks,
        list_job_events,
        event_summary
    ),
    components(schemas(
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
struct ApiDoc;

#[derive(Debug, Parser)]
#[command(name = "loglens-api")]
#[command(about = "LogLens Rust API service")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    #[arg(
        long,
        env = "DATABASE_URL",
        default_value = "postgres://loglens:loglens@localhost:5432/loglens"
    )]
    database_url: String,

    #[arg(long, env = "APP_HOST", default_value = "0.0.0.0")]
    host: String,

    #[arg(long, env = "APP_PORT", default_value_t = 8080)]
    port: u16,

    #[arg(
        long,
        env = "CORS_ALLOWED_ORIGIN",
        default_value = "http://localhost:3000"
    )]
    cors_allowed_origin: String,
}

#[derive(Debug, Clone, Copy, Subcommand)]
enum Command {
    Serve,
    Migrate,
    PrintOpenapi,
}

#[utoipa::path(
    get,
    path = "/health",
    tag = "service",
    responses(
        (status = 200, description = "Service healthy", body = HealthResponse),
        (status = 503, description = "Database unavailable", body = HealthResponse)
    )
)]
async fn health(State(state): State<Arc<AppState>>) -> (StatusCode, Json<HealthResponse>) {
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
async fn version(State(state): State<Arc<AppState>>) -> Json<VersionResponse> {
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
async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[utoipa::path(
    get,
    path = "/v1/me",
    tag = "auth",
    responses(
        (status = 200, description = "Current authenticated user context", body = MeResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse)
    )
)]
async fn me(
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
async fn update_me_license(
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
    .map_err(|error| ApiError::internal(format!("failed to update individual license: {error}")))?;

    Ok(Json(license_snapshot(request.tier, request.status)))
}

#[utoipa::path(
    get,
    path = "/v1/orgs",
    tag = "org",
    responses(
        (status = 200, description = "Organizations for authenticated user", body = ListOrgsResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse)
    )
)]
async fn list_orgs(
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
async fn create_org(
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
    .map_err(|error| ApiError::internal(format!("failed to create owner membership: {error}")))?;

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
async fn get_org(
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
async fn list_org_members(
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
    .map_err(|error| ApiError::internal(format!("failed to load organization members: {error}")))?;

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
async fn update_org_license(
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
async fn update_org_member_role(
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

    Ok(Json(MutationResponse {
        message: "role updated".to_owned(),
    }))
}

#[utoipa::path(
    post,
    path = "/v1/orgs/{org_id}/uploads",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier")
    ),
    responses(
        (status = 200, description = "Log files accepted and parse jobs created", body = UploadResponse),
        (status = 400, description = "No valid log files in upload", body = ErrorResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse)
    )
)]
async fn upload_logs(
    Path(org_id): Path<i64>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    let mut jobs = Vec::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::bad_request(format!("failed to read multipart field: {e}")))?
    {
        let file_name = field
            .file_name()
            .unwrap_or("upload.log")
            .to_owned();

        let content = field
            .text()
            .await
            .map_err(|e| ApiError::bad_request(format!("failed to read file content: {e}")))?;

        if content.trim().is_empty() {
            continue;
        }

        let job_id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO parse_jobs (org_id, file_name, status)
            VALUES ($1, $2, 'queued')
            RETURNING id
            "#,
        )
        .bind(org_id)
        .bind(&file_name)
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::internal(format!("failed to create parse job: {e}")))?;

        let pool = state.pool.clone();
        let semaphore = state.parse_semaphore.clone();
        tokio::spawn(async move {
            let _permit = semaphore.acquire().await;
            run_parse_job(pool, job_id, content).await;
        });

        let job_row = fetch_parse_job(&state.pool, job_id).await?;
        jobs.push(to_parse_job_response(job_row)?);
    }

    if jobs.is_empty() {
        return Err(ApiError::bad_request("no valid log files in upload"));
    }

    Ok(Json(UploadResponse { jobs }))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}/jobs",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("status" = Option<String>, Query, description = "Filter by job status")
    ),
    responses(
        (status = 200, description = "Parse jobs for this organization", body = ListJobsResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse)
    )
)]
async fn list_jobs(
    Path(org_id): Path<i64>,
    Query(query): Query<ListJobsQuery>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListJobsResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    let rows = if let Some(ref status) = query.status {
        sqlx::query_as::<_, ParseJobRow>(
            r#"
            SELECT id, org_id, file_name, status, total_lines, parsed_lines,
                   benchmark_count, error_message, created_at, started_at, finished_at, expires_at
            FROM parse_jobs WHERE org_id = $1 AND status = $2
            ORDER BY id DESC
            "#,
        )
        .bind(org_id)
        .bind(status)
        .fetch_all(&state.pool)
        .await
    } else {
        sqlx::query_as::<_, ParseJobRow>(
            r#"
            SELECT id, org_id, file_name, status, total_lines, parsed_lines,
                   benchmark_count, error_message, created_at, started_at, finished_at, expires_at
            FROM parse_jobs WHERE org_id = $1
            ORDER BY id DESC
            "#,
        )
        .bind(org_id)
        .fetch_all(&state.pool)
        .await
    }
    .map_err(|e| ApiError::internal(format!("failed to list parse jobs: {e}")))?;

    let jobs = rows
        .into_iter()
        .map(to_parse_job_response)
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(ListJobsResponse { jobs }))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}/jobs/{job_id}",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("job_id" = i64, Path, description = "Parse job identifier")
    ),
    responses(
        (status = 200, description = "Parse job status", body = ParseJobResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse),
        (status = 404, description = "Job not found", body = ErrorResponse)
    )
)]
async fn get_job(
    Path((org_id, job_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ParseJobResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    let row = fetch_parse_job_for_org(&state.pool, job_id, org_id).await?;
    Ok(Json(to_parse_job_response(row)?))
}

#[utoipa::path(
    delete,
    path = "/v1/orgs/{org_id}/jobs/{job_id}",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("job_id" = i64, Path, description = "Parse job identifier")
    ),
    responses(
        (status = 200, description = "Job deleted", body = MutationResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse),
        (status = 404, description = "Job not found", body = ErrorResponse)
    )
)]
async fn delete_job(
    Path((org_id, job_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<MutationResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    let _ = fetch_parse_job_for_org(&state.pool, job_id, org_id).await?;

    sqlx::query("DELETE FROM parse_jobs WHERE id = $1 AND org_id = $2")
        .bind(job_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(|e| ApiError::internal(format!("failed to delete parse job: {e}")))?;

    Ok(Json(MutationResponse {
        message: format!("Job {job_id} deleted"),
    }))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}/jobs/{job_id}/benchmarks",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("job_id" = i64, Path, description = "Parse job identifier")
    ),
    responses(
        (status = 200, description = "Benchmark snapshots for completed parse job", body = ListBenchmarksResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse),
        (status = 404, description = "Job not found", body = ErrorResponse)
    )
)]
async fn list_job_benchmarks(
    Path((org_id, job_id)): Path<(i64, i64)>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListBenchmarksResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    // Verify job belongs to org
    let _ = fetch_parse_job_for_org(&state.pool, job_id, org_id).await?;

    let rows = sqlx::query_as::<_, BenchmarkRow>(
        r#"
        SELECT
          sequence, label,
          query_rows, query_rows_limit, query_rows_delta,
          heap_size_pct, heap_size_bytes_limit, heap_size_delta,
          cpu_time_ms, cpu_time_limit, cpu_time_delta,
          dml_statements, dml_statements_limit,
          soql_queries, soql_queries_limit
        FROM benchmark_snapshots
        WHERE job_id = $1
        ORDER BY sequence
        "#,
    )
    .bind(job_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to load benchmark snapshots: {e}")))?;

    let benchmarks = rows.into_iter().map(to_benchmark_response).collect();
    Ok(Json(ListBenchmarksResponse { benchmarks }))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}/jobs/{job_id}/events",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("job_id" = i64, Path, description = "Parse job identifier"),
        ("offset" = Option<i64>, Query, description = "Pagination offset"),
        ("limit" = Option<i64>, Query, description = "Pagination limit (max 500)"),
        ("event_type" = Option<String>, Query, description = "Filter by event type"),
        ("log_level" = Option<String>, Query, description = "Filter by log level"),
        ("search" = Option<String>, Query, description = "Case-insensitive partial match on event_type"),
        ("class_name" = Option<String>, Query, description = "Filter by Apex class/trigger name")
    ),
    responses(
        (status = 200, description = "Parsed log events", body = ListEventsResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse),
        (status = 404, description = "Job not found", body = ErrorResponse)
    )
)]
async fn list_job_events(
    Path((org_id, job_id)): Path<(i64, i64)>,
    Query(query): Query<ListEventsQuery>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ListEventsResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;

    let _ = fetch_parse_job_for_org(&state.pool, job_id, org_id).await?;

    let page_limit = query.limit.unwrap_or(100).min(500);
    let page_offset = query.offset.unwrap_or(0);

    let mut where_clauses = vec!["job_id = $1".to_owned()];
    let mut bind_idx = 2u32;

    if query.event_type.is_some() {
        where_clauses.push(format!("event_type = ${bind_idx}"));
        bind_idx += 1;
    }
    if query.log_level.is_some() {
        where_clauses.push(format!("log_level = ${bind_idx}"));
        bind_idx += 1;
    }

    let search_pattern = query.search.as_ref().map(|s| format!("%{s}%"));
    if search_pattern.is_some() {
        where_clauses.push(format!("event_type ILIKE ${bind_idx}"));
        bind_idx += 1;
    }
    if query.class_name.is_some() {
        where_clauses.push(format!("class_name = ${bind_idx}"));
        bind_idx += 1;
    }

    let where_clause = where_clauses.join(" AND ");

    let count_sql = format!("SELECT COUNT(*)::bigint FROM parsed_log_events WHERE {where_clause}");
    let select_sql = format!(
        "SELECT line_index, timestamp, nanos, event_type, line_number, log_level, class_name \
         FROM parsed_log_events WHERE {where_clause} \
         ORDER BY line_index LIMIT ${bind_idx} OFFSET ${}",
        bind_idx + 1
    );

    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql).bind(job_id);
    if let Some(ref et) = query.event_type {
        count_q = count_q.bind(et);
    }
    if let Some(ref ll) = query.log_level {
        count_q = count_q.bind(ll);
    }
    if let Some(ref sp) = search_pattern {
        count_q = count_q.bind(sp);
    }
    if let Some(ref cn) = query.class_name {
        count_q = count_q.bind(cn);
    }

    let total = count_q
        .fetch_one(&state.pool)
        .await
        .map_err(|e| ApiError::internal(format!("failed to count events: {e}")))?;

    let mut select_q = sqlx::query_as::<_, LogEventRow>(&select_sql).bind(job_id);
    if let Some(ref et) = query.event_type {
        select_q = select_q.bind(et);
    }
    if let Some(ref ll) = query.log_level {
        select_q = select_q.bind(ll);
    }
    if let Some(ref sp) = search_pattern {
        select_q = select_q.bind(sp);
    }
    if let Some(ref cn) = query.class_name {
        select_q = select_q.bind(cn);
    }
    select_q = select_q.bind(page_limit).bind(page_offset);

    let rows = select_q
        .fetch_all(&state.pool)
        .await
        .map_err(|e| ApiError::internal(format!("failed to load events: {e}")))?;

    let events = rows.into_iter().map(to_event_response).collect();
    Ok(Json(ListEventsResponse { events, total }))
}

#[utoipa::path(
    get,
    path = "/v1/orgs/{org_id}/jobs/{job_id}/event-summary",
    tag = "ingest",
    params(
        ("org_id" = i64, Path, description = "Organization identifier"),
        ("job_id" = i64, Path, description = "Parse job identifier"),
        ("buckets" = Option<i64>, Query, description = "Number of timeline buckets (default 50, max 200)")
    ),
    responses(
        (status = 200, description = "Aggregated event summary for charts", body = EventSummaryResponse),
        (status = 401, description = "Missing or invalid auth context", body = ErrorResponse),
        (status = 403, description = "Cross-org access denied", body = ErrorResponse),
        (status = 404, description = "Job not found", body = ErrorResponse)
    )
)]
async fn event_summary(
    Path((org_id, job_id)): Path<(i64, i64)>,
    Query(query): Query<EventSummaryQuery>,
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
) -> Result<Json<EventSummaryResponse>, ApiError> {
    let user = authenticate(&headers, &state.pool).await?;
    let caller_role = fetch_org_role(&state.pool, org_id, user.user_id).await?;
    let _ = require_permission(caller_role, OrgPermission::View)?;
    let _ = fetch_parse_job_for_org(&state.pool, job_id, org_id).await?;

    let bucket_count = query.buckets.clamp(1, 200);

    #[derive(FromRow)]
    struct TypeCount {
        event_type: String,
        count: i64,
    }

    let type_counts: Vec<TypeCount> = sqlx::query_as(
        "SELECT event_type, COUNT(*)::bigint AS count \
         FROM parsed_log_events WHERE job_id = $1 \
         GROUP BY event_type ORDER BY count DESC",
    )
    .bind(job_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to aggregate event types: {e}")))?;

    let total_events: i64 = type_counts.iter().map(|tc| tc.count).sum();

    let event_type_counts: Vec<EventTypeBucket> = type_counts
        .into_iter()
        .map(|tc| EventTypeBucket {
            event_type: tc.event_type,
            count: tc.count,
        })
        .collect();

    #[derive(FromRow)]
    struct NanosRange {
        min_nanos: Option<i64>,
        max_nanos: Option<i64>,
    }

    let range: NanosRange = sqlx::query_as(
        "SELECT MIN(nanos)::bigint AS min_nanos, MAX(nanos)::bigint AS max_nanos \
         FROM parsed_log_events WHERE job_id = $1 AND nanos IS NOT NULL",
    )
    .bind(job_id)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to get nanos range: {e}")))?;

    let timeline = match (range.min_nanos, range.max_nanos) {
        (Some(min_n), Some(max_n)) if max_n > min_n => {
            let span = max_n - min_n;
            let bucket_width = (span + bucket_count - 1) / bucket_count;

            #[derive(FromRow)]
            struct BucketCount {
                bucket: Option<i64>,
                count: i64,
            }

            let rows: Vec<BucketCount> = sqlx::query_as(
                "SELECT ((nanos - $2) / $3)::bigint AS bucket, COUNT(*)::bigint AS count \
                 FROM parsed_log_events \
                 WHERE job_id = $1 AND nanos IS NOT NULL \
                 GROUP BY bucket ORDER BY bucket",
            )
            .bind(job_id)
            .bind(min_n)
            .bind(bucket_width)
            .fetch_all(&state.pool)
            .await
            .map_err(|e| ApiError::internal(format!("failed to bucket timeline: {e}")))?;

            rows.into_iter()
                .filter_map(|r| {
                    r.bucket.map(|b| TimelineBucket {
                        nanos_start: min_n + b * bucket_width,
                        nanos_end: min_n + (b + 1) * bucket_width,
                        count: r.count,
                    })
                })
                .collect()
        }
        _ => vec![],
    };

    let class_names: Vec<String> = sqlx::query_scalar(
        "SELECT DISTINCT class_name FROM parsed_log_events \
         WHERE job_id = $1 AND class_name IS NOT NULL \
         ORDER BY class_name",
    )
    .bind(job_id)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to list class names: {e}")))?;

    Ok(Json(EventSummaryResponse {
        event_type_counts,
        timeline,
        total_events,
        class_names,
    }))
}

async fn run_parse_job(pool: PgPool, job_id: i64, content: String) {
    let set_running = sqlx::query(
        "UPDATE parse_jobs SET status = 'running', started_at = NOW() WHERE id = $1",
    )
    .bind(job_id)
    .execute(&pool)
    .await;

    if let Err(e) = set_running {
        error!("failed to mark job {job_id} running: {e}");
        return;
    }

    let events = parser_core::parse_log(&content);
    let total_lines = content.lines().count() as i64;
    let parsed_lines = events.len() as i64;

    let benchmarks = parser_sfdc_benchmarks::extract_benchmarks(&events);
    let benchmark_count = benchmarks.len() as i32;

    // Insert parsed events in batches (message and raw_line are deliberately
    // NOT persisted to satisfy the no-durable-raw-log-retention requirement).
    let batch_size = 500;
    for chunk in events.chunks(batch_size) {
        let mut qb = String::from(
            "INSERT INTO parsed_log_events (job_id, line_index, timestamp, nanos, event_type, line_number, log_level, class_name) VALUES ",
        );
        let mut params_idx = 1u32;

        for (i, _) in chunk.iter().enumerate() {
            if i > 0 {
                qb.push_str(", ");
            }
            qb.push_str(&format!(
                "(${}, ${}, ${}, ${}, ${}, ${}, ${}, ${})",
                params_idx,
                params_idx + 1,
                params_idx + 2,
                params_idx + 3,
                params_idx + 4,
                params_idx + 5,
                params_idx + 6,
                params_idx + 7
            ));
            params_idx += 8;
        }

        let mut q = sqlx::query(&qb);
        for (chunk_idx, evt) in chunk.iter().enumerate() {
            let global_idx = events.iter().position(|e| std::ptr::eq(e, evt)).unwrap_or(chunk_idx) as i32;
            let nanos = evt.nanos.map(|n| n as i64);
            let line_number = evt.line_number.map(|n| n as i32);
            let log_level = evt.level.as_ref().map(|l| l.to_string());
            q = q
                .bind(job_id)
                .bind(global_idx)
                .bind(&evt.timestamp)
                .bind(nanos)
                .bind(evt.event_type.to_string())
                .bind(line_number)
                .bind(log_level)
                .bind(&evt.class_name);
        }

        if let Err(e) = q.execute(&pool).await {
            error!("failed to insert parsed events for job {job_id}: {e}");
            let _ = sqlx::query(
                "UPDATE parse_jobs SET status = 'failed', error_message = $2, finished_at = NOW() WHERE id = $1",
            )
            .bind(job_id)
            .bind(format!("event insertion failed: {e}"))
            .execute(&pool)
            .await;
            return;
        }
    }

    // Insert benchmark snapshots
    for snap in &benchmarks {
        let result = sqlx::query(
            r#"
            INSERT INTO benchmark_snapshots (
                job_id, sequence, label,
                query_rows, query_rows_limit, query_rows_delta,
                heap_size_pct, heap_size_bytes_limit, heap_size_delta,
                cpu_time_ms, cpu_time_limit, cpu_time_delta,
                dml_statements, dml_statements_limit,
                soql_queries, soql_queries_limit
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
        )
        .bind(job_id)
        .bind(snap.sequence as i32)
        .bind(&snap.label)
        .bind(snap.query_rows)
        .bind(snap.query_rows_limit)
        .bind(snap.query_rows_delta)
        .bind(snap.heap_size_pct)
        .bind(snap.heap_size_bytes_limit)
        .bind(snap.heap_size_delta)
        .bind(snap.cpu_time_ms)
        .bind(snap.cpu_time_limit)
        .bind(snap.cpu_time_delta)
        .bind(snap.dml_statements)
        .bind(snap.dml_statements_limit)
        .bind(snap.soql_queries)
        .bind(snap.soql_queries_limit)
        .execute(&pool)
        .await;

        if let Err(e) = result {
            error!("failed to insert benchmark snapshot for job {job_id}: {e}");
            let _ = sqlx::query(
                "UPDATE parse_jobs SET status = 'failed', error_message = $2, finished_at = NOW() WHERE id = $1",
            )
            .bind(job_id)
            .bind(format!("benchmark insertion failed: {e}"))
            .execute(&pool)
            .await;
            return;
        }
    }

    let _ = sqlx::query(
        r#"
        UPDATE parse_jobs
        SET status = 'done',
            total_lines = $2,
            parsed_lines = $3,
            benchmark_count = $4,
            finished_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(job_id)
    .bind(total_lines)
    .bind(parsed_lines)
    .bind(benchmark_count)
    .execute(&pool)
    .await;
}

async fn fetch_parse_job(pool: &PgPool, job_id: i64) -> Result<ParseJobRow, ApiError> {
    sqlx::query_as::<_, ParseJobRow>(
        r#"
        SELECT id, org_id, file_name, status, total_lines, parsed_lines,
               benchmark_count, error_message, created_at, started_at, finished_at, expires_at
        FROM parse_jobs WHERE id = $1
        "#,
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to fetch parse job: {e}")))?
    .ok_or_else(|| ApiError::not_found("parse job not found"))
}

async fn fetch_parse_job_for_org(
    pool: &PgPool,
    job_id: i64,
    org_id: i64,
) -> Result<ParseJobRow, ApiError> {
    sqlx::query_as::<_, ParseJobRow>(
        r#"
        SELECT id, org_id, file_name, status, total_lines, parsed_lines,
               benchmark_count, error_message, created_at, started_at, finished_at, expires_at
        FROM parse_jobs WHERE id = $1 AND org_id = $2
        "#,
    )
    .bind(job_id)
    .bind(org_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| ApiError::internal(format!("failed to fetch parse job: {e}")))?
    .ok_or_else(|| ApiError::not_found("parse job not found in this organization"))
}

fn to_parse_job_response(row: ParseJobRow) -> Result<ParseJobResponse, ApiError> {
    Ok(ParseJobResponse {
        job_id: row.id,
        org_id: row.org_id,
        file_name: row.file_name,
        status: JobStatus::from_db(&row.status)?,
        total_lines: row.total_lines,
        parsed_lines: row.parsed_lines,
        benchmark_count: row.benchmark_count,
        error_message: row.error_message,
        created_at: row.created_at.to_rfc3339(),
        started_at: row.started_at.map(|t| t.to_rfc3339()),
        finished_at: row.finished_at.map(|t| t.to_rfc3339()),
        expires_at: row.expires_at.to_rfc3339(),
    })
}

fn to_benchmark_response(row: BenchmarkRow) -> BenchmarkSnapshotResponse {
    BenchmarkSnapshotResponse {
        sequence: row.sequence,
        label: row.label,
        query_rows: row.query_rows,
        query_rows_limit: row.query_rows_limit,
        query_rows_delta: row.query_rows_delta,
        heap_size_pct: row.heap_size_pct,
        heap_size_bytes_limit: row.heap_size_bytes_limit,
        heap_size_delta: row.heap_size_delta,
        cpu_time_ms: row.cpu_time_ms,
        cpu_time_limit: row.cpu_time_limit,
        cpu_time_delta: row.cpu_time_delta,
        dml_statements: row.dml_statements,
        dml_statements_limit: row.dml_statements_limit,
        soql_queries: row.soql_queries,
        soql_queries_limit: row.soql_queries_limit,
    }
}

fn to_event_response(row: LogEventRow) -> LogEventResponse {
    LogEventResponse {
        line_index: row.line_index,
        timestamp: row.timestamp,
        nanos: row.nanos,
        event_type: row.event_type,
        line_number: row.line_number,
        log_level: row.log_level,
        class_name: row.class_name,
    }
}

fn to_org_summary(row: OrgRow) -> Result<OrgSummary, ApiError> {
    let tier = LicenseTier::from_db(&row.tier)?;
    let status = LicenseStatus::from_db(&row.status)?;
    Ok(OrgSummary {
        org_id: row.org_id,
        name: row.name,
        role: OrgRole::from_db(&row.role)?,
        license: license_snapshot(tier, status),
    })
}

async fn authenticate(headers: &HeaderMap, pool: &PgPool) -> Result<AuthenticatedUser, ApiError> {
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

async fn fetch_individual_license(
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
    .map_err(|error| ApiError::internal(format!("failed to fetch individual license: {error}")))?;

    let tier = LicenseTier::from_db(&row.tier)?;
    let status = LicenseStatus::from_db(&row.status)?;
    Ok(license_snapshot(tier, status))
}

async fn fetch_org_role(
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
    .map_err(|error| ApiError::internal(format!("failed to resolve organization role: {error}")))?;

    role.map(|value| OrgRole::from_db(&value)).transpose()
}

async fn fetch_org_summary(
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

fn required_header<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, ApiError> {
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

fn optional_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn role_allows(role: &OrgRole, permission: OrgPermission) -> bool {
    match permission {
        OrgPermission::View => true,
        OrgPermission::ManageMembers => matches!(role, OrgRole::Owner | OrgRole::Admin),
        OrgPermission::ManageBilling => matches!(role, OrgRole::Owner | OrgRole::Admin),
    }
}

fn require_permission(
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

fn license_snapshot(tier: LicenseTier, status: LicenseStatus) -> LicenseSnapshot {
    LicenseSnapshot {
        features: entitlement_keys(&tier, &status),
        tier,
        status,
    }
}

fn entitlement_keys(tier: &LicenseTier, status: &LicenseStatus) -> Vec<String> {
    if !matches!(status, LicenseStatus::Active) {
        return Vec::new();
    }

    let values = match tier {
        LicenseTier::Free => vec!["single_log_upload", "log_search"],
        LicenseTier::Pro => vec![
            "single_log_upload",
            "multi_log_upload",
            "organization_workspaces",
            "log_search",
        ],
        LicenseTier::Enterprise => vec![
            "single_log_upload",
            "multi_log_upload",
            "organization_workspaces",
            "advanced_limits_insights",
            "priority_support",
            "log_search",
        ],
    };

    values.into_iter().map(str::to_owned).collect()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let cli = Cli::parse();
    let command = cli.command.unwrap_or(Command::Serve);

    match command {
        Command::Serve => serve(&cli).await,
        Command::Migrate => migrate(&cli.database_url).await,
        Command::PrintOpenapi => {
            let json = serde_json::to_string_pretty(&ApiDoc::openapi())
                .context("failed to serialize OpenAPI contract")?;
            println!("{json}");
            Ok(())
        }
    }
}

async fn serve(cli: &Cli) -> anyhow::Result<()> {
    let pool = connect_pool(&cli.database_url).await?;
    run_migrations(&pool).await?;

    let version =
        std::env::var("APP_VERSION").unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_owned());
    let state = Arc::new(AppState {
        pool: pool.clone(),
        version,
        parse_semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_PARSE_JOBS)),
    });
    let app = build_router(state, &cli.cors_allowed_origin);

    tokio::spawn(expired_job_reaper(pool));

    let addr: SocketAddr = format!("{}:{}", cli.host, cli.port)
        .parse()
        .context("failed to parse listen address")?;

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("failed to bind TCP listener")?;

    info!("API listening on http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("axum server error")?;

    Ok(())
}

/// Background task that periodically deletes parse_jobs whose 30-day
/// retention window has elapsed. CASCADE foreign keys on parsed_log_events
/// and benchmark_snapshots handle child-row cleanup automatically.
async fn expired_job_reaper(pool: PgPool) {
    let interval = std::time::Duration::from_secs(60 * 60); // hourly
    loop {
        match sqlx::query_scalar::<_, i64>(
            "DELETE FROM parse_jobs WHERE expires_at <= NOW() RETURNING id",
        )
        .fetch_all(&pool)
        .await
        {
            Ok(ids) if !ids.is_empty() => {
                info!("expired-job reaper deleted {} job(s): {:?}", ids.len(), ids);
            }
            Ok(_) => {}
            Err(e) => {
                error!("expired-job reaper failed: {e}");
            }
        }
        tokio::time::sleep(interval).await;
    }
}

async fn migrate(database_url: &str) -> anyhow::Result<()> {
    let pool = connect_pool(database_url).await?;
    run_migrations(&pool).await?;
    info!("migrations completed");
    Ok(())
}

async fn connect_pool(database_url: &str) -> anyhow::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(8)
        .connect(database_url)
        .await
        .context("failed to connect to PostgreSQL")
}

async fn run_migrations(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .context("failed to run migrations")
}

fn build_router(state: Arc<AppState>, cors_allowed_origin: &str) -> Router {
    let allowed_origins: Vec<HeaderValue> = cors_allowed_origin
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::OPTIONS])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::HeaderName::from_static("x-loglens-auth-sub"),
            header::HeaderName::from_static("x-loglens-auth-email"),
        ]);

    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/openapi.json", get(openapi_json))
        .route("/v1/me", get(me))
        .route("/v1/me/license", patch(update_me_license))
        .route("/v1/orgs", get(list_orgs).post(create_org))
        .route("/v1/orgs/{org_id}", get(get_org))
        .route("/v1/orgs/{org_id}/members", get(list_org_members))
        .route("/v1/orgs/{org_id}/license", patch(update_org_license))
        .route(
            "/v1/orgs/{org_id}/members/{member_user_id}/role",
            patch(update_org_member_role),
        )
        .route("/v1/orgs/{org_id}/uploads", post(upload_logs))
        .route("/v1/orgs/{org_id}/jobs", get(list_jobs))
        .route("/v1/orgs/{org_id}/jobs/{job_id}", get(get_job).delete(delete_job))
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}/benchmarks",
            get(list_job_benchmarks),
        )
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}/events",
            get(list_job_events),
        )
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}/event-summary",
            get(event_summary),
        )
        .layer(cors)
        .with_state(state)
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };

    #[cfg(unix)]
    let terminate = async {
        use tokio::signal::unix::{signal, SignalKind};

        if let Ok(mut stream) = signal(SignalKind::terminate()) {
            let _ = stream.recv().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_policy_enforces_expected_permissions() {
        assert!(role_allows(&OrgRole::Owner, OrgPermission::ManageBilling));
        assert!(role_allows(&OrgRole::Admin, OrgPermission::ManageMembers));
        assert!(!role_allows(&OrgRole::Member, OrgPermission::ManageBilling));
        assert!(!role_allows(&OrgRole::Viewer, OrgPermission::ManageMembers));
    }

    #[test]
    fn cross_org_access_is_denied_without_membership() {
        let result = require_permission(None, OrgPermission::View);
        assert!(matches!(
            result,
            Err(ApiError {
                status: StatusCode::FORBIDDEN,
                ..
            })
        ));
    }

    #[test]
    fn license_status_controls_entitlements() {
        let active = entitlement_keys(&LicenseTier::Enterprise, &LicenseStatus::Active);
        assert!(active
            .iter()
            .any(|feature| feature == "advanced_limits_insights"));

        let suspended = entitlement_keys(&LicenseTier::Enterprise, &LicenseStatus::PastDue);
        assert!(suspended.is_empty());
    }
}
