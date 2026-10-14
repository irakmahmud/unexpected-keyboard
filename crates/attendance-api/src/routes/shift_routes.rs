use crate::{ApiError, AppState};
use attendance_domain::{CreateShiftRequest, Shift, ShiftType};
use axum::{
    extract::State,
    routing::get,
    Json, Router,
};

pub fn shift_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_shifts).post(create_shift))
}

async fn list_shifts(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<Shift>>, ApiError> {
    let shifts = state.db.list_shifts("TECHFIRM_01")?;
    Ok(Json(shifts))
}

async fn create_shift(
    State(state): State<AppState>,
    Json(payload): Json<CreateShiftRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let shift = Shift {
        id: None,
        company_id: payload.company_id,
        name: payload.name,
        code: payload.code,
        shift_type: ShiftType::RegularDay,
        start_time: payload.start_time,
        end_time: payload.end_time,
        grace_period_minutes: payload.grace_period_minutes,
        half_day_threshold_hours: payload.half_day_hours,
        min_work_hours_full_day: 8.0,
        break_duration_minutes: 60,
        is_overtime_enabled: true,
        overtime_starts_after_minutes: 30,
        is_active: true,
        created_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let id = state.db.create_shift(&shift)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "shift_id": id,
        "message": "Shift policy created successfully"
    })))
}
