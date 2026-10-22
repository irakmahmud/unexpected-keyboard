use crate::{ApiError, AppState};
use attendance_auth::{AuthService, HrAuthService, SessionStore};
use attendance_domain::UserRole;
use axum::{
    extract::State,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LoginPayload {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub success: bool,
    pub token: String,
    pub username: String,
    pub role: UserRole,
    pub message: String,
}

#[derive(Deserialize)]
pub struct HrVerifyPayload {
    pub company_uuid: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct HrPasswordSetPayload {
    pub company_uuid: String,
    pub company_name: Option<String>,
    pub password: String,
}

#[derive(Deserialize)]
pub struct HrPasswordChangePayload {
    pub company_uuid: String,
    pub company_name: Option<String>,
    pub current_password: String,
    pub new_password: String,
}

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(handle_login))
        .route("/verify-hr", post(handle_hr_verify))
        .route("/set-hr-password", post(handle_set_hr_password))
        .route("/change-hr-password", post(handle_change_hr_password))
        .route("/sync-hr-password", post(handle_set_hr_password))
        .route("/forgot-password", axum::routing::get(handle_forgot_password).post(handle_forgot_password))
}

async fn handle_login(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> std::result::Result<Json<LoginResponse>, ApiError> {
    let user = state
        .db
        .get_user_by_username(&payload.username)?
        .ok_or_else(|| attendance_common::AppError::Unauthorized("User not found".to_string()))?;

    let stored_password = user.password.as_deref().unwrap_or("");
    let is_valid = AuthService::verify_password(&payload.password, stored_password);

    if !is_valid {
        return Err(attendance_common::AppError::Unauthorized("Invalid password".to_string()).into());
    }

    let sessions = SessionStore::default();
    let comp_id = user.company_uuid.as_deref().unwrap_or("TECHFIRM_01");
    let session = sessions.create_session(&user.username, user.role.clone(), comp_id);

    Ok(Json(LoginResponse {
        success: true,
        token: session.token,
        username: user.username,
        role: user.role,
        message: "Login successful".to_string(),
    }))
}

async fn handle_hr_verify(
    State(state): State<AppState>,
    Json(payload): Json<HrVerifyPayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let is_valid = HrAuthService::verify_hr_access(&state.db, &payload.company_uuid, &payload.password)?;

    Ok(Json(serde_json::json!({
        "success": is_valid,
        "authenticated": is_valid,
        "company_uuid": payload.company_uuid
    })))
}

async fn handle_set_hr_password(
    State(state): State<AppState>,
    Json(payload): Json<HrPasswordSetPayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let company_name = payload
        .company_name
        .unwrap_or_else(|| payload.company_uuid.clone());
    HrAuthService::set_hr_password(&state.db, &payload.company_uuid, &company_name, &payload.password)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "company_uuid": payload.company_uuid,
        "message": "HR password configured and synced to local database"
    })))
}

async fn handle_change_hr_password(
    State(state): State<AppState>,
    Json(payload): Json<HrPasswordChangePayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    if payload.new_password.trim().len() < 3 {
        return Err(attendance_common::AppError::Validation(
            "Password must be at least 3 characters".to_string(),
        )
        .into());
    }
    let company_name = payload
        .company_name
        .unwrap_or_else(|| payload.company_uuid.clone());
    HrAuthService::change_hr_password(
        &state.db,
        &payload.company_uuid,
        &company_name,
        payload.current_password.trim(),
        payload.new_password.trim(),
    )?;
    Ok(Json(serde_json::json!({
        "success": true,
        "company_uuid": payload.company_uuid,
        "message": "HR password updated successfully!"
    })))
}

async fn handle_forgot_password() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "title": "Password Recovery",
        "message": "Please contact your HR department or system administrator to reset your password."
    }))
}
