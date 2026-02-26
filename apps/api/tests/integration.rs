use axum::{
    body::Body,
    http::{header, Method, Request, StatusCode},
};
use loglens_api::{build_router, AppState};
use serde_json::Value;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower::ServiceExt;

fn app_state(pool: PgPool) -> Arc<AppState> {
    Arc::new(AppState {
        pool,
        version: "test".to_owned(),
        parse_semaphore: Arc::new(Semaphore::new(4)),
    })
}

fn router(pool: PgPool) -> axum::Router {
    build_router(app_state(pool), "http://localhost:3000")
}

async fn json_body(response: axum::response::Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn json_request(method: Method, uri: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

fn authed_request(method: Method, uri: &str, sub: &str, email: &str) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-loglens-auth-sub", sub)
        .header("x-loglens-auth-email", email)
        .body(Body::empty())
        .unwrap()
}

fn authed_json_request(
    method: Method,
    uri: &str,
    sub: &str,
    email: &str,
    body: Value,
) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(uri)
        .header("x-loglens-auth-sub", sub)
        .header("x-loglens-auth-email", email)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap()
}

// ── Auth Flow Tests ─────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn register_creates_user(pool: PgPool) {
    let app = router(pool);
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "alice@example.com",
            "password": "securepass123"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = json_body(resp).await;
    assert_eq!(body["email"], "alice@example.com");
    assert!(body["user_id"].is_number());
    assert!(body["auth_subject"].is_string());
}

#[sqlx::test(migrations = "./migrations")]
async fn register_duplicate_email_returns_409(pool: PgPool) {
    let app = router(pool);

    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "dup@example.com",
            "password": "securepass123"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "dup@example.com",
            "password": "anotherpass456"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);
}

#[sqlx::test(migrations = "./migrations")]
async fn register_weak_password_returns_400(pool: PgPool) {
    let app = router(pool);
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "weak@example.com",
            "password": "short"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_with_correct_password(pool: PgPool) {
    let app = router(pool);

    // Register first
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "login@example.com",
            "password": "correctpassword"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Login
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "login@example.com",
            "password": "correctpassword"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = json_body(resp).await;
    assert_eq!(body["email"], "login@example.com");
}

#[sqlx::test(migrations = "./migrations")]
async fn login_wrong_password_returns_401(pool: PgPool) {
    let app = router(pool);

    // Register
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "wrong@example.com",
            "password": "correctpassword"
        }),
    );
    app.clone().oneshot(req).await.unwrap();

    // Login with wrong password
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "wrong@example.com",
            "password": "wrongpassword!"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn login_nonexistent_user_returns_401(pool: PgPool) {
    let app = router(pool);
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "ghost@example.com",
            "password": "somepassword!"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

// ── Password Reset Lifecycle ────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn password_reset_lifecycle(pool: PgPool) {
    let app = router(pool);

    // Register
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "reset@example.com",
            "password": "oldpassword1"
        }),
    );
    app.clone().oneshot(req).await.unwrap();

    // Request reset token
    let req = json_request(
        Method::POST,
        "/v1/auth/forgot-password",
        serde_json::json!({ "email": "reset@example.com" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = json_body(resp).await;
    let token = body["reset_token"].as_str().unwrap();
    assert!(!token.is_empty());

    // Reset password
    let req = json_request(
        Method::POST,
        "/v1/auth/reset-password",
        serde_json::json!({
            "token": token,
            "new_password": "newpassword2"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Login with new password
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "reset@example.com",
            "password": "newpassword2"
        }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Old password should fail
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "reset@example.com",
            "password": "oldpassword1"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[sqlx::test(migrations = "./migrations")]
async fn reset_with_invalid_token_returns_400(pool: PgPool) {
    let app = router(pool);
    let req = json_request(
        Method::POST,
        "/v1/auth/reset-password",
        serde_json::json!({
            "token": "bogus_token_value",
            "new_password": "newpassword2"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

// ── Tenant Boundary Tests ───────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn user_cannot_access_other_orgs_data(pool: PgPool) {
    let app = router(pool);

    // User A creates an org
    let req = authed_json_request(
        Method::POST,
        "/v1/orgs",
        "user_aaa",
        "a@example.com",
        serde_json::json!({ "name": "Org A" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = json_body(resp).await;
    let org_a_id = body["org_id"].as_i64().unwrap();

    // User B tries to access Org A
    let req = authed_request(
        Method::GET,
        &format!("/v1/orgs/{org_a_id}"),
        "user_bbb",
        "b@example.com",
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // User B tries to list Org A members
    let req = authed_request(
        Method::GET,
        &format!("/v1/orgs/{org_a_id}/members"),
        "user_bbb",
        "b@example.com",
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);

    // User B tries to list Org A jobs
    let req = authed_request(
        Method::GET,
        &format!("/v1/orgs/{org_a_id}/jobs"),
        "user_bbb",
        "b@example.com",
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── RBAC Tests ──────────────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn viewer_cannot_change_roles(pool: PgPool) {
    let app = router(pool.clone());

    // Owner creates org
    let req = authed_json_request(
        Method::POST,
        "/v1/orgs",
        "owner_sub",
        "owner@example.com",
        serde_json::json!({ "name": "RBAC Org" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    let body = json_body(resp).await;
    let org_id = body["org_id"].as_i64().unwrap();

    // Add viewer directly via SQL (simulating an invite)
    let viewer_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO app_users (auth_subject, email, updated_at) VALUES ('viewer_sub', 'viewer@example.com', NOW()) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO organization_memberships (org_id, user_id, role) VALUES ($1, $2, 'viewer')")
        .bind(org_id)
        .bind(viewer_id)
        .execute(&pool)
        .await
        .unwrap();

    // Viewer tries to change their own role
    let req = authed_json_request(
        Method::PATCH,
        &format!("/v1/orgs/{org_id}/members/{viewer_id}/role"),
        "viewer_sub",
        "viewer@example.com",
        serde_json::json!({ "role": "admin" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[sqlx::test(migrations = "./migrations")]
async fn member_cannot_change_billing(pool: PgPool) {
    let app = router(pool.clone());

    // Owner creates org
    let req = authed_json_request(
        Method::POST,
        "/v1/orgs",
        "billing_owner",
        "billing_owner@example.com",
        serde_json::json!({ "name": "Billing Org" }),
    );
    let resp = app.clone().oneshot(req).await.unwrap();
    let body = json_body(resp).await;
    let org_id = body["org_id"].as_i64().unwrap();

    // Add member directly via SQL
    let member_id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO app_users (auth_subject, email, updated_at) VALUES ('member_sub', 'member@example.com', NOW()) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO organization_memberships (org_id, user_id, role) VALUES ($1, $2, 'member')")
        .bind(org_id)
        .bind(member_id)
        .execute(&pool)
        .await
        .unwrap();

    // Member tries to change org license
    let req = authed_json_request(
        Method::PATCH,
        &format!("/v1/orgs/{org_id}/license"),
        "member_sub",
        "member@example.com",
        serde_json::json!({ "tier": "enterprise", "status": "active" }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

// ── Security Headers Tests ──────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn responses_include_security_headers(pool: PgPool) {
    let app = router(pool);
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    assert_eq!(
        resp.headers().get("x-frame-options").unwrap(),
        "DENY"
    );
    assert_eq!(
        resp.headers().get("x-content-type-options").unwrap(),
        "nosniff"
    );
    assert_eq!(
        resp.headers().get("referrer-policy").unwrap(),
        "strict-origin-when-cross-origin"
    );
    assert!(resp.headers().get("content-security-policy").is_some());
}

// ── Audit Event Tests ───────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn login_records_audit_event(pool: PgPool) {
    let app = router(pool.clone());

    // Register
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "audit@example.com",
            "password": "auditpass123"
        }),
    );
    app.clone().oneshot(req).await.unwrap();

    // Login
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "audit@example.com",
            "password": "auditpass123"
        }),
    );
    app.oneshot(req).await.unwrap();

    // Verify audit events
    let events: Vec<(String, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT event_type, metadata FROM audit_events ORDER BY id",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // Should have UserRegistered and LoginSuccess
    let types: Vec<&str> = events.iter().map(|(t, _)| t.as_str()).collect();
    assert!(types.contains(&"user_registered"));
    assert!(types.contains(&"login_success"));

    // Verify no password data in metadata
    for (_, meta) in &events {
        if let Some(meta) = meta {
            let s = meta.to_string();
            assert!(!s.contains("password"), "metadata must never contain passwords");
            assert!(!s.contains("hash"), "metadata must never contain hashes");
        }
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn failed_login_records_audit_event(pool: PgPool) {
    let app = router(pool.clone());

    // Register
    let req = json_request(
        Method::POST,
        "/v1/auth/register",
        serde_json::json!({
            "email": "failaudit@example.com",
            "password": "correctpass1"
        }),
    );
    app.clone().oneshot(req).await.unwrap();

    // Failed login
    let req = json_request(
        Method::POST,
        "/v1/auth/login",
        serde_json::json!({
            "email": "failaudit@example.com",
            "password": "wrongpassword"
        }),
    );
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Verify login_failed audit event
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM audit_events WHERE event_type = 'login_failed'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert!(count >= 1);
}

// ── Health and Version ──────────────────────────────────────────────

#[sqlx::test(migrations = "./migrations")]
async fn health_endpoint_works(pool: PgPool) {
    let app = router(pool);
    let req = Request::builder()
        .uri("/health")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[sqlx::test(migrations = "./migrations")]
async fn unauthenticated_request_to_me_returns_401(pool: PgPool) {
    let app = router(pool);
    let req = Request::builder()
        .uri("/v1/me")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
