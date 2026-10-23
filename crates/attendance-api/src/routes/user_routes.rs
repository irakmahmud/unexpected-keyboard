use crate::{ApiError, AppState};
use attendance_auth::PasswordHasher;
use attendance_common::CryptoHelper;
use attendance_domain::*;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose, Engine as _};

pub fn user_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users).post(create_user))
        .route("/check/:username", get(check_username_exists))
        .route("/check-device/:android_id", get(check_device_exists))
        .route("/:username", get(get_user).put(update_user).delete(delete_user))
        .route("/:username/password", post(change_password))
}

#[derive(serde::Deserialize, Default)]
pub struct UserListQuery {
    pub company_uuid: Option<String>,
    pub search: Option<String>,
}

async fn list_users(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<UserListQuery>,
) -> std::result::Result<Json<Vec<User>>, ApiError> {
    let mut users = state.db.get_all_users()?;
    if let Some(ref cid) = query.company_uuid {
        let c = cid.trim();
        if !c.is_empty() {
            users.retain(|u| u.company_uuid.as_deref() == Some(c));
        }
    }
    if let Some(ref s) = query.search {
        let term = s.trim().to_lowercase();
        if !term.is_empty() {
            users.retain(|u| {
                u.username.to_lowercase().contains(&term)
                    || u.user_designation
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&term)
            });
        }
    }
    Ok(Json(users))
}

async fn check_username_exists(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let upper = username.trim().to_uppercase();
    let user_opt = state.db.get_user_by_username(&upper)?;
    Ok(Json(serde_json::json!({
        "exists": user_opt.is_some(),
        "username": upper,
        "user": user_opt
    })))
}

async fn check_device_exists(
    State(state): State<AppState>,
    Path(android_id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let aid = android_id.trim();
    let user_opt = state.db.get_user_by_android_id(aid)?;
    Ok(Json(serde_json::json!({
        "exists": user_opt.is_some(),
        "android_id": aid,
        "user": user_opt
    })))
}

async fn get_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<User>, ApiError> {
    let user = state
        .db
        .get_user_by_username(&username.to_uppercase())?
        .ok_or_else(|| attendance_common::AppError::NotFound(format!("User {} not found", username)))?;
    Ok(Json(user))
}

fn decode_b64_image(raw_b64: &str) -> std::result::Result<Vec<u8>, attendance_common::AppError> {
    let clean = raw_b64
        .trim()
        .split_once(',')
        .map(|(_, b)| b)
        .unwrap_or_else(|| raw_b64.trim());
    general_purpose::STANDARD
        .decode(clean)
        .map_err(|e| attendance_common::AppError::Validation(format!("Invalid base64 image frame: {}", e)))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<CreateUserRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let upper_username = payload.username.trim().to_uppercase();
    if upper_username.is_empty() {
        return Err(attendance_common::AppError::Validation("Username is required".to_string()).into());
    }

    let raw_pw = payload.password.clone().unwrap_or_else(|| "123456".to_string());
    let hashed_pw = PasswordHasher::hash_sha256(&raw_pw);

    let role = match payload.role.as_deref() {
        Some("Admin") | Some("SuperAdmin") => UserRole::SuperAdmin,
        Some("CompanyAdmin") => UserRole::CompanyAdmin,
        Some("HrManager") => UserRole::HrManager,
        Some("SecurityGuard") => UserRole::SecurityGuard,
        Some("KioskTerminal") => UserRole::KioskTerminal,
        _ => UserRole::Employee,
    };

    // Collect camera frames sent for ONNX FaceNet512 + MTCNN extraction (do not duplicate image_b64 if images_b64 is already populated)
    let mut frames_b64: Vec<String> = Vec::new();
    if let Some(list) = payload.images_b64 {
        frames_b64.extend(list.into_iter().filter(|s| !s.trim().is_empty()));
    }
    if frames_b64.is_empty() {
        if let Some(single) = payload.image_b64 {
            if !single.trim().is_empty() {
                frames_b64.push(single);
            }
        }
    }

    let mut extracted_embeddings: Vec<Vec<f32>> = Vec::new();
    if !frames_b64.is_empty() {
        let df_state = state.get_deepface().ok_or_else(|| {
            attendance_common::AppError::Validation(
                "DeepFace ONNX models not loaded in ./models/".to_string(),
            )
        })?;

        let _ = std::fs::create_dir_all("data/captures");

        for (idx, frame_str) in frames_b64.iter().enumerate() {
            let img_bytes = decode_b64_image(frame_str)?;
            if idx == 0 {
                let save_path = format!("data/captures/{}_REGISTERED.jpg", upper_username);
                let _ = std::fs::write(&save_path, &img_bytes);
            }
            let emb = attendance_face::extract_face_embedding_vec(&df_state, &img_bytes)?;
            extracted_embeddings.push(emb);
        }
    } else if let Some(emb) = payload.face_embedding {
        if !emb.is_empty() && !attendance_face::FaceVectorIndex::is_corrupted_constant_vector(&emb) {
            extracted_embeddings.push(emb);
        }
    }

    let (face_encoding_str, has_camera_face) = if !extracted_embeddings.is_empty() {
        // Check duplicate user face across all existing users (cosine distance < 0.30, matching Python _is_duplicate_user)
        for emb in &extracted_embeddings {
            if let Some((dup_user, dist)) = state.face_index.find_duplicate_user(emb, Some(&upper_username)) {
                return Err(attendance_common::AppError::Validation(format!(
                    "Duplicate face detected: this face is already registered to user '{}' (cosine distance {:.4})",
                    dup_user, dist
                ))
                .into());
            }
        }
        let first_json = serde_json::to_string(&extracted_embeddings[0]).unwrap_or_else(|_| "[]".to_string());
        (Some(first_json), true)
    } else {
        // Python parity (lines 956 & 13760): serialize_face_embedding(np.zeros(512))
        let zero_vec = vec![0.0f32; 512];
        let json_str = serde_json::to_string(&zero_vec).unwrap_or_else(|_| "[]".to_string());
        (Some(json_str), false)
    };

    // Enforce duplicate Android Device ID check across other users (matching Python attendance_v8.py)
    if let Some(ref aid) = payload.user_android_id {
        let trimmed_aid = aid.trim();
        if !trimmed_aid.is_empty() {
            if let Ok(Some(existing_owner)) = state.db.get_user_by_android_id(trimmed_aid) {
                if !existing_owner.username.eq_ignore_ascii_case(&upper_username) {
                    return Err(attendance_common::AppError::Validation(format!(
                        "Device Android ID '{}' is already registered to user '{}'",
                        trimmed_aid, existing_owner.username
                    ))
                    .into());
                }
            }
        }
    }

    let existing_user = state.db.get_user_by_username(&upper_username).ok().flatten();
    let company_user_uuid = existing_user
        .as_ref()
        .map(|u| u.company_user_uuid.clone())
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let user_uuid = existing_user
        .as_ref()
        .and_then(|u| u.user_uuid.clone())
        .unwrap_or_else(CryptoHelper::new_uuid);

    // Resolve active company if not explicitly provided, or fill in the missing half (company_name / company_uuid)
    let (resolved_comp_name, resolved_comp_uuid) = match (payload.company_name.clone(), payload.company_uuid.clone()) {
        (Some(name), Some(id)) => (Some(name), Some(id)),
        (None, Some(id)) => {
            let name = state
                .db
                .get_company(&id)
                .ok()
                .flatten()
                .map(|c| c.company_name)
                .unwrap_or_else(|| id.clone());
            (Some(name), Some(id))
        }
        (Some(name), None) => {
            let id = state
                .db
                .list_companies()
                .ok()
                .and_then(|list| {
                    list.into_iter()
                        .find(|c| c.company_name.eq_ignore_ascii_case(&name))
                        .map(|c| c.company_id)
                })
                .unwrap_or_else(|| "TECHFIRM_01".to_string());
            (Some(name), Some(id))
        }
        (None, None) => {
            if let Ok(mut companies) = state.db.list_companies() {
                companies.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
                if let Some(c) = companies.into_iter().next() {
                    (Some(c.company_name), Some(c.company_id))
                } else {
                    (Some("Smart Attendance HQ".to_string()), Some("HQ_001".to_string()))
                }
            } else {
                (Some("Smart Attendance HQ".to_string()), Some("HQ_001".to_string()))
            }
        }
    };

    let now = attendance_common::TimeHelper::now_utc_sqlite_string();
    let user = User {
        company_user_uuid: company_user_uuid.clone(),
        company_name: resolved_comp_name.clone(),
        company_uuid: resolved_comp_uuid.clone(),
        username: upper_username.clone(),
        user_uuid: Some(user_uuid.clone()),
        user_designation: payload.user_designation.clone(),
        user_android_id: payload.user_android_id.clone(),
        face_encoding: face_encoding_str,
        register_with_camera: Some(if has_camera_face { "Y" } else { "N" }.to_string()),
        password: Some(hashed_pw),
        role,
        status: UserStatus::Active,
        email: payload.email.clone(),
        phone: payload.phone.clone(),
        created_at: Some(now.clone()),
        last_updated: Some(now.clone()),
    };

    state.db.insert_or_update_user(&user)?;

    // Also save/update corresponding Employee record so HR directory stays in sync
    let emp = Employee {
        id: None,
        employee_code: upper_username.clone(),
        user_uuid,
        company_id: resolved_comp_uuid.unwrap_or_else(|| "HQ_001".to_string()),
        first_name: upper_username.clone(),
        last_name: payload.user_designation.clone().unwrap_or_else(|| "Employee".to_string()),
        full_name: format!(
            "{} ({})",
            upper_username,
            payload.user_designation.as_deref().unwrap_or("Employee")
        ),
        email: payload.email,
        phone: payload.phone,
        department_id: None,
        designation_id: None,
        shift_id: None,
        rfid_card_uid: None,
        pin_code: Some(raw_pw),
        employment_type: EmploymentType::Permanent,
        join_date: attendance_common::TimeHelper::current_date_dhaka(),
        end_date: None,
        is_active: true,
        avatar_url: if has_camera_face {
            Some(format!("/api/v1/captures/{}_REGISTERED.jpg", upper_username))
        } else {
            None
        },
        emergency_contact_name: None,
        emergency_contact_phone: None,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };
    let _ = state.db.save_employee(&emp);

    // If camera embeddings were extracted, replace any old profiles in SQLite and RAM index
    if has_camera_face {
        let _ = state.db.delete_face_profiles(&upper_username);
        state.face_index.remove_user(&upper_username);
        // Restore face_encoding on users table after delete_face_profiles cleared it
        let _ = state.db.insert_or_update_user(&user);
        let conn = state.db.get_connection();
        let _ = conn.execute(
            "UPDATE users SET register_with_camera = 'Y' WHERE UPPER(username) = ?1",
            rusqlite::params![upper_username],
        );
        drop(conn);
        for (i, emb) in extracted_embeddings.iter().enumerate() {
            let pose_name = format!("ANGLE_{}", i + 1);
            let _ = state.db.save_face_profile(
                &upper_username,
                Some(&upper_username),
                emb,
                1.0,
                &pose_name,
            );
            let _ = state.face_index.enroll_user(&upper_username, emb.clone());
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "username": user.username,
        "company_user_uuid": user.company_user_uuid,
        "company_name": user.company_name,
        "company_uuid": user.company_uuid,
        "embeddings_enrolled": extracted_embeddings.len(),
        "with_camera": has_camera_face,
        "message": if has_camera_face {
            format!("User '{}' registered successfully with {} FaceNet512 ONNX embeddings!", user.username, extracted_embeddings.len())
        } else {
            format!("User '{}' registered successfully without camera (512-zero encoding saved)!", user.username)
        }
    })))
}

async fn update_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<UpdateUserRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let mut user = state
        .db
        .get_user_by_username(&username.to_uppercase())?
        .ok_or_else(|| attendance_common::AppError::NotFound("User not found".to_string()))?;

    if let Some(d) = payload.user_designation {
        user.user_designation = Some(d);
    }
    if let Some(aid) = payload.user_android_id {
        let trimmed_aid = aid.trim();
        if !trimmed_aid.is_empty() {
            if let Ok(Some(existing_owner)) = state.db.get_user_by_android_id(trimmed_aid) {
                if !existing_owner.username.eq_ignore_ascii_case(&user.username) {
                    return Err(attendance_common::AppError::Validation(format!(
                        "Device Android ID '{}' is already registered to user '{}'",
                        trimmed_aid, existing_owner.username
                    ))
                    .into());
                }
            }
        }
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
    user.last_updated = Some(attendance_common::TimeHelper::now_utc_sqlite_string());

    state.db.insert_or_update_user(&user)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "username": user.username,
        "user": user,
        "message": "User updated successfully"
    })))
}

async fn delete_user(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let upper = username.to_uppercase();
    let _ = state.db.delete_face_profiles(&upper);
    state.face_index.remove_user(&upper);
    let deleted = state.db.delete_user(&upper)?;
    Ok(Json(serde_json::json!({
        "success": deleted,
        "username": upper,
        "message": if deleted { "User and biometric profiles deleted" } else { "User not found" }
    })))
}

async fn change_password(
    State(state): State<AppState>,
    Path(username): Path<String>,
    Json(payload): Json<ChangePasswordRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let mut user = state
        .db
        .get_user_by_username(&username.to_uppercase())?
        .ok_or_else(|| attendance_common::AppError::NotFound("User not found".to_string()))?;

    if let Some(old_pw) = &payload.old_password {
        if !old_pw.trim().is_empty() {
            if let Some(stored_pw) = &user.password {
                if !PasswordHasher::verify(old_pw, stored_pw) && old_pw != stored_pw {
                    return Err(attendance_common::AppError::Unauthorized("Invalid old password".to_string()).into());
                }
            }
        }
    }

    user.password = Some(PasswordHasher::hash_sha256(&payload.new_password));
    user.last_updated = Some(attendance_common::TimeHelper::now_utc_sqlite_string());
    state.db.insert_or_update_user(&user)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Password changed successfully"
    })))
}
