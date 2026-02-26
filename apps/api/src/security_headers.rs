use axum::{
    http::{header::HeaderName, HeaderValue},
    Router,
};
use tower_http::set_header::SetResponseHeaderLayer;

/// Applies security headers to all API responses:
/// - X-Frame-Options: DENY
/// - X-Content-Type-Options: nosniff
/// - Referrer-Policy: strict-origin-when-cross-origin
/// - Content-Security-Policy: default-src 'none'; frame-ancestors 'none'
///
/// HSTS is only added when ENABLE_HSTS=true (production behind TLS).
pub fn apply_security_headers<S: Clone + Send + Sync + 'static>(router: Router<S>) -> Router<S> {
    let router = router
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-frame-options"),
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("referrer-policy"),
            HeaderValue::from_static("strict-origin-when-cross-origin"),
        ))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("content-security-policy"),
            HeaderValue::from_static("default-src 'none'; frame-ancestors 'none'"),
        ));

    // Add HSTS only in production (behind TLS termination)
    if std::env::var("ENABLE_HSTS").is_ok() {
        router.layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("strict-transport-security"),
            HeaderValue::from_static("max-age=63072000; includeSubDomains; preload"),
        ))
    } else {
        router
    }
}
