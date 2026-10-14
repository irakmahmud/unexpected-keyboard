use attendance_auth::JwtService;
use axum::{
    body::Body,
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};

pub async fn require_auth_token(
    jwt: JwtService,
    req: Request<Body>,
    next: Next,
) -> std::result::Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = &auth[7..];
            match jwt.verify(token) {
                Ok(_claims) => Ok(next.run(req).await),
                Err(e) => Err((
                    StatusCode::UNAUTHORIZED,
                    Json(serde_json::json!({
                        "error": format!("Invalid authorization token: {}", e),
                        "success": false
                    })),
                )),
            }
        }
        _ => Err((
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "error": "Missing or malformed Authorization header with Bearer token",
                "success": false
            })),
        )),
    }
}
