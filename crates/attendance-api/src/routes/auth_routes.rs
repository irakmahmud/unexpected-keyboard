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
    #[serde(alias = "company_id")]
    pub company_uuid: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct HrPasswordSetPayload {
    #[serde(alias = "company_id", default = "default_company_uuid")]
    pub company_uuid: String,
    pub company_name: Option<String>,
    pub password: Option<String>,
}

fn default_company_uuid() -> String {
    "HQ_001".to_string()
}

#[derive(Deserialize)]
pub struct HrPasswordChangePayload {
    #[serde(alias = "company_id")]
    pub company_uuid: String,
    pub company_name: Option<String>,
    pub current_password: String,
    pub new_password: String,
}

#[derive(Deserialize, Default)]
pub struct ForgotPasswordPayload {
    pub username: Option<String>,
    pub verification: Option<String>,
    pub hr_password: Option<String>,
    pub new_password: Option<String>,
}

pub fn auth_router() -> Router<AppState> {
    Router::new()
        .route("/login", post(handle_login))
        .route("/verify-hr", post(handle_hr_verify))
        .route("/set-hr-password", post(handle_set_hr_password))
        .route("/change-hr-password", post(handle_change_hr_password))
        .route("/sync-hr-password", post(handle_sync_hr_password))
        .route(
            "/forgot-password",
            axum::routing::get(handle_forgot_password_get).post(handle_forgot_password_post),
        )
        .route("/reset-password", post(handle_forgot_password_post))
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
    let comp_id = user.company_uuid.as_deref().unwrap_or("HQ_001");
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
    let pw = payload.password.as_deref().unwrap_or("HR@123");
    HrAuthService::set_hr_password(&state.db, &payload.company_uuid, &company_name, pw)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "company_uuid": payload.company_uuid,
        "message": "HR password configured and synced to local database"
    })))
}

async fn handle_sync_hr_password(
    State(state): State<AppState>,
    Json(payload): Json<HrPasswordSetPayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let company_name = payload
        .company_name
        .unwrap_or_else(|| payload.company_uuid.clone());
    if let Some(ref explicit_pw) = payload.password {
        if !explicit_pw.trim().is_empty() {
            HrAuthService::set_hr_password(&state.db, &payload.company_uuid, &company_name, explicit_pw.trim())?;
        }
    } else if state.db.get_hr_password(&payload.company_uuid)?.is_none() {
        HrAuthService::set_hr_password(&state.db, &payload.company_uuid, &company_name, "HR@123")?;
    }
    Ok(Json(serde_json::json!({
        "success": true,
        "company_uuid": payload.company_uuid,
        "message": "HR password synced to local database"
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

async fn handle_forgot_password_get() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "success": true,
        "title": "Password Recovery",
        "message": "Please enter your username and HR authorization password or registered Android ID to reset your password, or contact your HR administrator."
    }))
}

async fn handle_forgot_password_post(
    State(state): State<AppState>,
    payload_opt: Option<Json<ForgotPasswordPayload>>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    if let Some(Json(payload)) = payload_opt {
        if let (Some(ref uname), Some(ref new_pw)) = (&payload.username, &payload.new_password) {
            let trimmed_uname = uname.trim().to_uppercase();
            let trimmed_pw = new_pw.trim();
            if trimmed_uname.is_empty() || trimmed_pw.len() < 3 {
                return Err(attendance_common::AppError::Validation(
                    "Username and a new password (min 3 chars) are required".to_string(),
                )
                .into());
            }
            let mut user = state
                .db
                .get_user_by_username(&trimmed_uname)?
                .ok_or_else(|| attendance_common::AppError::NotFound(format!("User '{}' not found", trimmed_uname)))?;

            let comp_uuid = user.company_uuid.as_deref().unwrap_or("HQ_001");
            let ver = payload
                .hr_password
                .as_deref()
                .or(payload.verification.as_deref())
                .unwrap_or("")
                .trim();

            let verified_by_hr = !ver.is_empty() && HrAuthService::verify_hr_access(&state.db, comp_uuid, ver).unwrap_or(false);
            let verified_by_aid = !ver.is_empty()
                && user
                    .user_android_id
                    .as_deref()
                    .map(|aid| aid.eq_ignore_ascii_case(ver))
                    .unwrap_or(false);
            let verified_by_desig = !ver.is_empty()
                && user
                    .user_designation
                    .as_deref()
                    .map(|d| d.eq_ignore_ascii_case(ver))
                    .unwrap_or(false);

            if !verified_by_hr && !verified_by_aid && !verified_by_desig {
                return Err(attendance_common::AppError::Unauthorized(
                    "Verification failed: provide valid HR password, registered Android ID, or designation".to_string(),
                )
                .into());
            }

            user.password = Some(AuthService::hash_password(trimmed_pw));
            user.last_updated = Some(attendance_common::TimeHelper::now_utc_sqlite_string());
            state.db.insert_or_update_user(&user)?;

            return Ok(Json(serde_json::json!({
                "success": true,
                "username": trimmed_uname,
                "message": format!("Password for '{}' has been reset successfully!", trimmed_uname)
            })));
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "title": "Password Recovery",
        "message": "Please contact your HR department or system administrator to reset your password."
    })))
}
