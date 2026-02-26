pub mod audit;
pub mod auth;
pub mod error;
pub mod helpers;
pub mod models;
pub mod routes;
pub mod security_headers;

use axum::{
    http::{header, HeaderValue, Method},
    routing::{get, patch, post},
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::{cors::CorsLayer, limit::RequestBodyLimitLayer};

use routes::{events, jobs, me, orgs, service};

pub const MAX_CONCURRENT_PARSE_JOBS: usize = 4;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub version: String,
    pub parse_semaphore: Arc<Semaphore>,
}

pub fn build_router(state: Arc<AppState>, cors_allowed_origin: &str) -> Router {
    let allowed_origins: Vec<HeaderValue> = cors_allowed_origin
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::HeaderName::from_static("x-loglens-auth-sub"),
            header::HeaderName::from_static("x-loglens-auth-email"),
        ]);

    // Rate limiter for auth endpoints: 5 requests per minute per IP
    let auth_rate_limit = GovernorConfigBuilder::default()
        .per_second(12) // replenish 1 token every 12s = ~5/min
        .burst_size(5)
        .finish()
        .unwrap();

    // Rate limiter for general API: 100 requests per minute per IP
    let api_rate_limit = GovernorConfigBuilder::default()
        .per_second(1) // replenish every 1s
        .burst_size(100)
        .finish()
        .unwrap();

    // Auth routes: strict rate limit + 1 MB body
    let auth_routes = Router::new()
        .route("/v1/auth/register", post(auth::register))
        .route("/v1/auth/login", post(auth::login))
        .route("/v1/auth/logout", post(auth::logout))
        .route("/v1/auth/forgot-password", post(auth::reset::forgot_password))
        .route("/v1/auth/reset-password", post(auth::reset::reset_password))
        .layer(GovernorLayer::new(Arc::new(auth_rate_limit)))
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    // Upload routes: 50 MB body limit + general rate limit
    let upload_routes = Router::new()
        .route("/v1/orgs/{org_id}/uploads", post(jobs::upload_logs))
        .layer(RequestBodyLimitLayer::new(50 * 1024 * 1024));

    // All other API routes: 1 MB body limit + general rate limit
    let api_routes = Router::new()
        .route("/health", get(service::health))
        .route("/version", get(service::version))
        .route("/openapi.json", get(service::openapi_json))
        .route("/v1/me", get(me::me))
        .route("/v1/me/license", patch(me::update_me_license))
        .route("/v1/orgs", get(orgs::list_orgs).post(orgs::create_org))
        .route("/v1/orgs/{org_id}", get(orgs::get_org))
        .route("/v1/orgs/{org_id}/members", get(orgs::list_org_members))
        .route(
            "/v1/orgs/{org_id}/license",
            patch(orgs::update_org_license),
        )
        .route(
            "/v1/orgs/{org_id}/members/{member_user_id}/role",
            patch(orgs::update_org_member_role),
        )
        .route("/v1/orgs/{org_id}/jobs", get(jobs::list_jobs))
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}",
            get(jobs::get_job).delete(jobs::delete_job),
        )
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}/benchmarks",
            get(events::list_job_benchmarks),
        )
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}/events",
            get(events::list_job_events),
        )
        .route(
            "/v1/orgs/{org_id}/jobs/{job_id}/event-summary",
            get(events::event_summary),
        )
        .layer(RequestBodyLimitLayer::new(1024 * 1024));

    let router = auth_routes
        .merge(upload_routes)
        .merge(api_routes)
        .layer(GovernorLayer::new(Arc::new(api_rate_limit)))
        .layer(cors)
        .with_state(state);

    security_headers::apply_security_headers(router)
}
