use crate::{ApiError, AppState};
use attendance_domain::{
    ApplyLeaveRequest, LeaveApplication, LeaveDecisionRequest, LeaveStatus, LeaveType,
};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn leave_router() -> Router<AppState> {
    Router::new()
        .route("/apply", post(apply_leave))
        .route("/:app_num/decision", post(decide_leave))
        .route("/employee/:code", get(list_employee_leaves))
}

async fn apply_leave(
    State(state): State<AppState>,
    Json(payload): Json<ApplyLeaveRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let app_num = format!("LV_{}", uuid::Uuid::new_v4().to_string()[..8].to_uppercase());
    let leave = LeaveApplication {
        id: None,
        application_number: app_num.clone(),
        employee_code: payload.employee_code,
        username: payload.username,
        leave_type: LeaveType::Casual,
        start_date: payload.start_date,
        end_date: payload.end_date,
        total_days: 1.0,
        reason: payload.reason,
        status: LeaveStatus::Pending,
        approved_by_username: None,
        decision_remarks: None,
        applied_at: Some(chrono::Utc::now().to_rfc3339()),
        decided_at: None,
    };

    let id = state.db.apply_leave(&leave)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "leave_id": id,
        "application_number": app_num,
        "message": "Leave application submitted"
    })))
}

async fn decide_leave(
    State(state): State<AppState>,
    Path(app_num): Path<String>,
    Json(payload): Json<LeaveDecisionRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let status = if payload.status == "APPROVED" {
        LeaveStatus::Approved
    } else {
        LeaveStatus::Rejected
    };

    let approver = payload.approved_by.unwrap_or_else(|| "HR_ADMIN".to_string());
    state
        .db
        .update_leave_status(&app_num, status, &approver, payload.remarks.as_deref())?;

    Ok(Json(serde_json::json!({
        "success": true,
        "application_number": app_num,
        "message": "Leave decision recorded"
    })))
}

async fn list_employee_leaves(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> std::result::Result<Json<Vec<LeaveApplication>>, ApiError> {
    let leaves = state.db.list_employee_leaves(&code)?;
    Ok(Json(leaves))
}
