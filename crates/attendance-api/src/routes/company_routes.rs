use crate::{ApiError, AppState};
use attendance_auth::{HrAuthService, PasswordHasher};
use attendance_domain::*;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct CompanySearchQuery {
    pub search: Option<String>,
}

#[derive(Deserialize)]
pub struct CompanyVerifyPayload {
    pub company_id: String,
    pub password: String,
}

pub fn company_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_all_companies).post(setup_company))
        .route("/list", get(list_all_companies))
        .route("/profile", get(get_default_company).post(update_company))
        .route("/verify", post(verify_and_select_company))
        .route("/create", post(setup_company))
        .route("/:id", get(get_company_by_id).put(update_company_by_id))
        .route("/setup", post(setup_company))
}

async fn list_all_companies(
    State(state): State<AppState>,
    Query(query): Query<CompanySearchQuery>,
) -> std::result::Result<Json<Vec<Company>>, ApiError> {
    let mut list = state.db.list_companies()?;
    if let Some(ref term) = query.search {
        let t = term.trim().to_lowercase();
        if !t.is_empty() {
            list.retain(|c| c.company_name.to_lowercase().contains(&t));
        }
    }
    Ok(Json(list))
}

async fn get_default_company(
    State(state): State<AppState>,
) -> std::result::Result<Json<Option<Company>>, ApiError> {
    let mut list = state.db.list_companies()?;
    if list.is_empty() {
        return Ok(Json(None));
    }
    // Sort by last_updated descending so most recently selected/updated company is active
    list.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
    Ok(Json(list.into_iter().next()))
}

async fn get_company_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Company>, ApiError> {
    let comp = state
        .db
        .get_company(&id)?
        .ok_or_else(|| attendance_common::AppError::NotFound(format!("Company {} not found", id)))?;
    Ok(Json(comp))
}

async fn verify_and_select_company(
    State(state): State<AppState>,
    Json(payload): Json<CompanyVerifyPayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let mut comp = state
        .db
        .get_company(&payload.company_id)?
        .ok_or_else(|| attendance_common::AppError::NotFound("Company not found in database".to_string()))?;

    let entered = payload.password.trim();
    let is_valid = entered == comp.password || PasswordHasher::verify(entered, &comp.password);

    if !is_valid {
        return Err(attendance_common::AppError::Unauthorized(
            "Invalid password! Please try again.".to_string(),
        )
        .into());
    }

    // Touch last_updated so this company becomes the active selected company in local database
    comp.last_updated = Some(attendance_common::TimeHelper::now_utc_sqlite_string());
    state.db.save_company(&comp)?;

    // Ensure default HR password (HR@123) exists for this company if not yet configured
    if state.db.get_hr_password(&comp.company_id)?.is_none() {
        let _ = HrAuthService::set_hr_password(&state.db, &comp.company_id, &comp.company_name, "HR@123");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "company_id": comp.company_id,
        "company_name": comp.company_name,
        "company": comp,
        "message": format!("Company '{}' selected and synced to local database!", comp.company_name)
    })))
}

async fn setup_company(
    State(state): State<AppState>,
    Json(payload): Json<CompanySetupRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let company_name = payload.company_name.trim().to_string();
    let password = payload.password.trim().to_string();
    if company_name.is_empty() || password.is_empty() {
        return Err(attendance_common::AppError::Validation(
            "Company Name and Password are required".to_string(),
        )
        .into());
    }

    let existing = state.db.list_companies()?;
    if payload.company_id.is_none() {
        if existing
            .iter()
            .any(|c| c.company_name.eq_ignore_ascii_case(&company_name))
        {
            return Err(attendance_common::AppError::Conflict(
                "A company with this name already exists".to_string(),
            )
            .into());
        }
    }

    let company_id = payload
        .company_id
        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
    let now = attendance_common::TimeHelper::now_utc_sqlite_string();

    let comp = Company {
        company_id: company_id.clone(),
        company_name: company_name.clone(),
        password: PasswordHasher::hash_sha256(&password),
        address_road: payload.address_road,
        address_city: payload.address_city,
        contact_person_name: payload.contact_person_name,
        contact_person_designation: payload.contact_person_designation,
        contact_person_number: payload.contact_person_number.or(payload.phone),
        website: payload.website,
        email: payload.email,
        tax_id: None,
        created_at: Some(now.clone()),
        last_updated: Some(now),
    };

    state.db.save_company(&comp)?;

    // Initialize default HR password (HR@123) for the newly created company
    if state.db.get_hr_password(&company_id)?.is_none() {
        let _ = HrAuthService::set_hr_password(&state.db, &company_id, &company_name, "HR@123");
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "company_id": comp.company_id,
        "company_name": comp.company_name,
        "company": comp,
        "message": "Company created successfully!"
    })))
}

async fn update_company(
    State(state): State<AppState>,
    Json(mut comp): Json<Company>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    if comp.password.trim().is_empty() {
        if let Ok(Some(existing)) = state.db.get_company(&comp.company_id) {
            comp.password = existing.password;
        }
    } else if comp.password.len() != 64 {
        comp.password = PasswordHasher::hash_sha256(comp.password.trim());
    }
    comp.last_updated = Some(attendance_common::TimeHelper::now_utc_sqlite_string());
    state.db.save_company(&comp)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "company_id": comp.company_id,
        "company_name": comp.company_name,
        "company": comp,
        "message": "Company updated and synced to local database successfully!"
    })))
}

async fn update_company_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut comp): Json<Company>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let existing = state.db.list_companies()?;
    if existing
        .iter()
        .any(|c| c.company_id != id && c.company_name.eq_ignore_ascii_case(comp.company_name.trim()))
    {
        return Err(attendance_common::AppError::Conflict(
            "Another company with this name already exists".to_string(),
        )
        .into());
    }

    if comp.password.trim().is_empty() {
        if let Some(prev) = existing.iter().find(|c| c.company_id == id) {
            comp.password = prev.password.clone();
        }
    } else if comp.password.len() != 64 {
        comp.password = PasswordHasher::hash_sha256(comp.password.trim());
    }

    comp.company_id = id;
    comp.last_updated = Some(attendance_common::TimeHelper::now_utc_sqlite_string());
    state.db.save_company(&comp)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "company_id": comp.company_id,
        "company_name": comp.company_name,
        "company": comp,
        "message": "Company updated and synced to local database successfully!"
    })))
}
