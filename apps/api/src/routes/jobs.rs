use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use axum_extra::extract::Multipart;
use sqlx::PgPool;
use std::sync::Arc;
use tracing::error;

use crate::auth::authenticate;
use crate::error::{ApiError, ErrorResponse};
use crate::helpers::{fetch_org_role, require_permission};
use crate::models::*;
use crate::AppState;

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
pub async fn upload_logs(
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
pub async fn list_jobs(
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
pub async fn get_job(
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
pub async fn delete_job(
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

pub async fn run_parse_job(pool: PgPool, job_id: i64, content: String) {
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
            let global_idx = events
                .iter()
                .position(|e| std::ptr::eq(e, evt))
                .unwrap_or(chunk_idx) as i32;
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

pub async fn fetch_parse_job(pool: &PgPool, job_id: i64) -> Result<ParseJobRow, ApiError> {
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

pub async fn fetch_parse_job_for_org(
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
