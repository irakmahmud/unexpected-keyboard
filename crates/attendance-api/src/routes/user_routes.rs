use crate::{ApiError, AppState};
use attendance_auth::PasswordHasher;
use attendance_common::CryptoHelper;
use attendance_domain::*;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/:username", get(get_user).put(update_user).delete(delete_user))
        .route("/:username/password", post(change_password))
}

async fn list_users(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<User>>, ApiError> {
    let users = state.db.get_all_users()?;
    Ok(Json(users))
}

async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<User>, ApiError> {
    let user = state.db.get_user_by_username(&username)?
        .ok_or_else(|| attendance_common::AppError::NotFound(format!("User {} not found", username)))?;
    Ok(Json(user))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let hashed_pw = payload
        .password
        .map(|p| PasswordHasher::hash_sha256(&p))
        .unwrap_or_else(|| PasswordHasher::hash_sha256("123456"));

    let role = match payload.role.as_deref() {
        Some("Admin") | Some("SuperAdmin") => UserRole::SuperAdmin,
        Some("CompanyAdmin") => UserRole::CompanyAdmin,
        Some("HrManager") => UserRole::HrManager,
        Some("SecurityGuard") => UserRole::SecurityGuard,
        Some("KioskTerminal") => UserRole::KioskTerminal,
        _ => UserRole::Employee,
    };

    let (face_encoding_str, emb_opt) = match payload.face_embedding {
        Some(emb) => {
            if let Some((dup_user, dist)) = state.face_index.find_duplicate_user(&emb, Some(&payload.username)) {
                return Err(attendance_common::AppError::Validation(format!(
                    "Duplicate face detected: face already registered to user {} (cosine distance {:.4} < 0.40)",
                    dup_user, dist
                ))
                .into());
            }
            let json_str = serde_json::to_string(&emb).unwrap_or_else(|_| "[]".to_string());
            (Some(json_str), Some(emb))
        }
        None => {
            // Python parity (lines 956 & 13760): serialize_face_embedding(np.zeros(512))
            let zero_vec = vec![0.0f32; 512];
            let json_str = serde_json::to_string(&zero_vec).unwrap_or_else(|_| "[]".to_string());
            (Some(json_str), None)
        }
    };

    let existing_cu_uuid = state
        .db
        .get_user_by_username(&payload.username)
        .ok()
        .flatten()
        .map(|u| u.company_user_uuid);

    let user = User {
        company_user_uuid: existing_cu_uuid
            .unwrap_or_else(|| format!("CU_{}", &CryptoHelper::new_uuid().replace('-', "")[..12])),
        company_name: payload.company_name,
        company_uuid: payload.company_uuid,
        username: payload.username.to_uppercase(),
        user_uuid: Some(CryptoHelper::new_uuid()),
        user_designation: payload.user_designation,
        user_android_id: payload.user_android_id,
        face_encoding: face_encoding_str,
        password: Some(hashed_pw),
        role,
        status: UserStatus::Active,
        email: payload.email,
        phone: payload.phone,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        last_updated: Some(chrono::Utc::now().to_rfc3339()),
    };

    state.db.insert_or_update_user(&user)?;

    if let Some(emb) = emb_opt {
        let _ = state.face_index.enroll_user(&user.username, emb);
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "username": user.username,
        "company_user_uuid": user.company_user_uuid,
        "message": "User registered successfully"
    })))
}

async fn update_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let mut user = state.db.get_user_by_username(&username)?
        .ok_or_else(|| attendance_common::AppError::NotFound("User not found".to_string()))?;

    if let Some(d) = payload.user_designation {
        user.user_designation = Some(d);
    }
    if let Some(aid) = payload.user_android_id {
        user.user_android_id = Some(aid);
    }
    if let Some(em) = payload.email {
        user.email = Some(em);
    }
    if let Some(ph) = payload.phone {
        user.phone = Some(ph);
    }
    if let Some(r) = payload.role {
        user.role = match r.as_str() {
            "Admin" | "SuperAdmin" => UserRole::SuperAdmin,
            "CompanyAdmin" => UserRole::CompanyAdmin,
            "HrManager" => UserRole::HrManager,
            _ => UserRole::Employee,
        };
    }
    if let Some(active) = payload.is_active {
        user.status = if active { UserStatus::Active } else { UserStatus::Inactive };
    }
    user.last_updated = Some(chrono::Utc::now().to_rfc3339());

    state.db.insert_or_update_user(&user)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "User updated successfully"
    })))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let deleted = state.db.delete_user(&username)?;
    Ok(Json(serde_json::json!({
        "success": deleted,
        "message": if deleted { "User deleted" } else { "User not found" }
    })))
}

async fn change_password(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<ChangePasswordRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let mut user = state.db.get_user_by_username(&username)?
        .ok_or_else(|| attendance_common::AppError::NotFound("User not found".to_string()))?;

    if let Some(old_pw) = &payload.old_password {
        if let Some(stored_pw) = &user.password {
            if !PasswordHasher::verify(old_pw, stored_pw) {
                return Err(attendance_common::AppError::Unauthorized("Invalid old password".to_string()).into());
            }
        }
    }

    user.password = Some(PasswordHasher::hash_sha256(&payload.new_password));
    user.last_updated = Some(chrono::Utc::now().to_rfc3339());
    state.db.insert_or_update_user(&user)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}
