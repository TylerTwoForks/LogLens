use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    Json,
};
use sqlx::FromRow;
use std::sync::Arc;

use crate::auth::authenticate;
use crate::error::{ApiError, ErrorResponse};
use crate::helpers::{fetch_org_role, require_permission};
use crate::models::*;
use crate::AppState;

use super::jobs::fetch_parse_job_for_org;

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
pub async fn list_job_benchmarks(
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
pub async fn list_job_events(
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

    let count_sql =
        format!("SELECT COUNT(*)::bigint FROM parsed_log_events WHERE {where_clause}");
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
pub async fn event_summary(
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
