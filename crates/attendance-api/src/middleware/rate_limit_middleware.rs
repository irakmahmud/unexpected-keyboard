use attendance_common::RateLimiter;
use axum::{
    body::Body,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use std::sync::Arc;

pub async fn enforce_rate_limit(
    limiter: Arc<RateLimiter>,
    req: Request<Body>,
    next: Next,
) -> std::result::Result<Response, (StatusCode, Json<serde_json::Value>)> {
    // Extract client IP or default bucket
    let client_key = req
        .headers()
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("client_global");

    if !limiter.check_allowed(client_key) {
        return Err((
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "error": "Rate limit exceeded. Please wait before making more requests.",
                "success": false
            })),
        ));
    }

    Ok(next.run(req).await)
}
