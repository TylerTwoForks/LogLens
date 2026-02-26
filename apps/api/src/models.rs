use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use utoipa::ToSchema;

use crate::error::ApiError;

// ── Roles & Permissions ─────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OrgRole {
    Owner,
    Admin,
    Member,
    Viewer,
}

impl OrgRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Admin => "admin",
            Self::Member => "member",
            Self::Viewer => "viewer",
        }
    }

    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "owner" => Ok(Self::Owner),
            "admin" => Ok(Self::Admin),
            "member" => Ok(Self::Member),
            "viewer" => Ok(Self::Viewer),
            _ => Err(ApiError::internal("invalid role persisted in database")),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OrgPermission {
    View,
    ManageMembers,
    ManageBilling,
}

// ── License ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LicenseTier {
    Free,
    Pro,
    Enterprise,
}

impl LicenseTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Free => "free",
            Self::Pro => "pro",
            Self::Enterprise => "enterprise",
        }
    }

    pub fn from_db(value: &str) -> Result<Self, ApiError> {
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
pub enum LicenseStatus {
    Active,
    PastDue,
    Canceled,
}

impl LicenseStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::PastDue => "past_due",
            Self::Canceled => "canceled",
        }
    }

    pub fn from_db(value: &str) -> Result<Self, ApiError> {
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
pub struct LicenseSnapshot {
    pub tier: LicenseTier,
    pub status: LicenseStatus,
    pub features: Vec<String>,
}

// ── Job Status ──────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
}

impl JobStatus {
    pub fn from_db(value: &str) -> Result<Self, ApiError> {
        match value {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "done" => Ok(Self::Done),
            "failed" => Ok(Self::Failed),
            _ => Err(ApiError::internal(
                "invalid job status persisted in database",
            )),
        }
    }
}

// ── API Request / Response Types ────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub database: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VersionResponse {
    pub version: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MeResponse {
    pub user_id: i64,
    pub auth_subject: String,
    pub email: String,
    pub individual_license: LicenseSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrgSummary {
    pub org_id: i64,
    pub name: String,
    pub role: OrgRole,
    pub license: LicenseSnapshot,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListOrgsResponse {
    pub orgs: Vec<OrgSummary>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrgRequest {
    pub name: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrgMembersResponse {
    pub members: Vec<OrgMember>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OrgMember {
    pub user_id: i64,
    pub email: String,
    pub role: OrgRole,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLicenseRequest {
    pub tier: LicenseTier,
    pub status: LicenseStatus,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMemberRoleRequest {
    pub role: OrgRole,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct MutationResponse {
    pub message: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ParseJobResponse {
    pub job_id: i64,
    pub org_id: i64,
    pub file_name: String,
    pub status: JobStatus,
    pub total_lines: i64,
    pub parsed_lines: i64,
    pub benchmark_count: i32,
    pub error_message: Option<String>,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub expires_at: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UploadResponse {
    pub jobs: Vec<ParseJobResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct BenchmarkSnapshotResponse {
    pub sequence: i32,
    pub label: String,
    pub query_rows: i64,
    pub query_rows_limit: i64,
    pub query_rows_delta: i64,
    pub heap_size_pct: f64,
    pub heap_size_bytes_limit: i64,
    pub heap_size_delta: f64,
    pub cpu_time_ms: i64,
    pub cpu_time_limit: i64,
    pub cpu_time_delta: i64,
    pub dml_statements: i64,
    pub dml_statements_limit: i64,
    pub soql_queries: i64,
    pub soql_queries_limit: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListBenchmarksResponse {
    pub benchmarks: Vec<BenchmarkSnapshotResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LogEventResponse {
    pub line_index: i32,
    pub timestamp: String,
    pub nanos: Option<i64>,
    pub event_type: String,
    pub line_number: Option<i32>,
    pub log_level: Option<String>,
    pub class_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListJobsQuery {
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListJobsResponse {
    pub jobs: Vec<ParseJobResponse>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ListEventsQuery {
    #[serde(default)]
    pub offset: Option<i64>,
    #[serde(default)]
    pub limit: Option<i64>,
    #[serde(default)]
    pub event_type: Option<String>,
    #[serde(default)]
    pub log_level: Option<String>,
    /// Case-insensitive partial match on event_type
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub class_name: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ListEventsResponse {
    pub events: Vec<LogEventResponse>,
    pub total: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventTypeBucket {
    pub event_type: String,
    pub count: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct TimelineBucket {
    pub nanos_start: i64,
    pub nanos_end: i64,
    pub count: i64,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EventSummaryQuery {
    #[serde(default = "default_timeline_buckets")]
    pub buckets: i64,
}

pub fn default_timeline_buckets() -> i64 {
    50
}

#[derive(Debug, Serialize, ToSchema)]
pub struct EventSummaryResponse {
    pub event_type_counts: Vec<EventTypeBucket>,
    pub timeline: Vec<TimelineBucket>,
    pub total_events: i64,
    pub class_names: Vec<String>,
}

// ── Database Row Types ──────────────────────────────────────────────

#[derive(Debug, FromRow)]
pub struct UserRow {
    pub id: i64,
    pub email: String,
}

#[derive(Debug, FromRow)]
pub struct LicenseRow {
    pub tier: String,
    pub status: String,
}

#[derive(Debug, FromRow)]
pub struct OrgRow {
    pub org_id: i64,
    pub name: String,
    pub role: String,
    pub tier: String,
    pub status: String,
}

#[derive(Debug, FromRow)]
pub struct OrgMemberRow {
    pub user_id: i64,
    pub email: String,
    pub role: String,
}

#[derive(Debug, FromRow)]
pub struct ParseJobRow {
    pub id: i64,
    pub org_id: i64,
    pub file_name: String,
    pub status: String,
    pub total_lines: i64,
    pub parsed_lines: i64,
    pub benchmark_count: i32,
    pub error_message: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub finished_at: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, FromRow)]
pub struct BenchmarkRow {
    pub sequence: i32,
    pub label: String,
    pub query_rows: i64,
    pub query_rows_limit: i64,
    pub query_rows_delta: i64,
    pub heap_size_pct: f64,
    pub heap_size_bytes_limit: i64,
    pub heap_size_delta: f64,
    pub cpu_time_ms: i64,
    pub cpu_time_limit: i64,
    pub cpu_time_delta: i64,
    pub dml_statements: i64,
    pub dml_statements_limit: i64,
    pub soql_queries: i64,
    pub soql_queries_limit: i64,
}

#[derive(Debug, FromRow)]
pub struct LogEventRow {
    pub line_index: i32,
    pub timestamp: String,
    pub nanos: Option<i64>,
    pub event_type: String,
    pub line_number: Option<i32>,
    pub log_level: Option<String>,
    pub class_name: Option<String>,
}

// ── Conversion Helpers ──────────────────────────────────────────────

pub fn to_org_summary(row: OrgRow) -> Result<OrgSummary, ApiError> {
    let tier = LicenseTier::from_db(&row.tier)?;
    let status = LicenseStatus::from_db(&row.status)?;
    Ok(OrgSummary {
        org_id: row.org_id,
        name: row.name,
        role: OrgRole::from_db(&row.role)?,
        license: license_snapshot(tier, status),
    })
}

pub fn to_parse_job_response(row: ParseJobRow) -> Result<ParseJobResponse, ApiError> {
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

pub fn to_benchmark_response(row: BenchmarkRow) -> BenchmarkSnapshotResponse {
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

pub fn to_event_response(row: LogEventRow) -> LogEventResponse {
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

// ── License Helpers ─────────────────────────────────────────────────

pub fn license_snapshot(tier: LicenseTier, status: LicenseStatus) -> LicenseSnapshot {
    LicenseSnapshot {
        features: entitlement_keys(&tier, &status),
        tier,
        status,
    }
}

pub fn entitlement_keys(tier: &LicenseTier, status: &LicenseStatus) -> Vec<String> {
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
