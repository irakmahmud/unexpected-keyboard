use crate::{ApiError, AppState};
use attendance_domain::*;
use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct HolidayCheckQuery {
    pub date: String,
    pub company_id: Option<String>,
}

pub fn holiday_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_holidays).post(add_holiday))
        .route("/check", get(check_is_holiday))
}

async fn list_holidays(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<Holiday>>, ApiError> {
    let holidays = state.db.list_holidays("TECHFIRM_01")?;
    Ok(Json(holidays))
}

async fn add_holiday(
    State(state): State<AppState>,
    Json(payload): Json<HolidayCreateRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let holiday_type = match payload.holiday_type.as_deref() {
        Some("Religious") => HolidayType::Religious,
        Some("CompanyDeclared") => HolidayType::CompanyDeclared,
        Some("OptionalObservance") => HolidayType::OptionalObservance,
        _ => HolidayType::NationalPublic,
    };

    let holiday = Holiday {
        id: None,
        company_id: payload.company_id,
        name: payload.name,
        holiday_date: payload.holiday_date,
        holiday_type,
        description: payload.description,
        is_recurring_yearly: payload.is_recurring.unwrap_or(false),
    };

    let id = state.db.add_holiday(&holiday)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "holiday_id": id,
        "message": "Holiday added successfully"
    })))
}

async fn check_is_holiday(
    State(state): State<AppState>,
    Query(query): Query<HolidayCheckQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let comp_id = query.company_id.unwrap_or_else(|| "TECHFIRM_01".to_string());
    let is_hld = state.db.is_holiday(&comp_id, &query.date)?;

    Ok(Json(serde_json::json!({
        "date": query.date,
        "is_holiday": is_hld
    })))
}
