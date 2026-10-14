use crate::{ApiError, AppState};
use attendance_domain::*;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn company_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_all_companies).post(setup_company))
        .route("/list", get(list_all_companies))
        .route("/profile", get(get_default_company).post(update_company))
        .route("/:id", get(get_company_by_id).put(update_company_by_id))
        .route("/setup", post(setup_company))
}

async fn list_all_companies(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<Company>>, ApiError> {
    let list = state.db.list_companies()?;
    Ok(Json(list))
}

async fn get_default_company(
    State(state): State<AppState>,
) -> std::result::Result<Json<Option<Company>>, ApiError> {
    let comp = state.db.get_company("TECHFIRM_01")?;
    Ok(Json(comp))
}

async fn get_company_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<Company>, ApiError> {
    let comp = state.db.get_company(&id)?
        .ok_or_else(|| attendance_common::AppError::NotFound(format!("Company {} not found", id)))?;
    Ok(Json(comp))
}

async fn setup_company(
    State(state): State<AppState>,
    Json(payload): Json<CompanySetupRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let comp = Company {
        company_id: "TECHFIRM_01".to_string(),
        company_name: payload.company_name,
        password: payload.password,
        address_road: payload.address_road,
        address_city: payload.address_city,
        contact_person_name: None,
        contact_person_designation: None,
        contact_person_number: payload.phone,
        website: payload.website,
        email: payload.email,
        tax_id: None,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
        last_updated: Some(chrono::Utc::now().to_rfc3339()),
    };

    state.db.save_company(&comp)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "company_id": comp.company_id,
        "message": "Company information configured successfully"
    })))
}

async fn update_company(
    State(state): State<AppState>,
    Json(comp): Json<Company>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    state.db.save_company(&comp)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Company updated successfully"
    })))
}

async fn update_company_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(mut comp): Json<Company>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    comp.company_id = id;
    state.db.save_company(&comp)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "company_id": comp.company_id,
        "message": "Company updated successfully"
    })))
}
