use crate::{ApiError, AppState};
use attendance_domain::{Employee, EmploymentType};
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateEmployeePayload {
    pub employee_code: String,
    pub first_name: String,
    pub last_name: String,
    pub company_id: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: Option<i64>,
    pub shift_id: Option<i64>,
    pub rfid_card_uid: Option<String>,
    pub pin_code: Option<String>,
    pub join_date: String,
}

pub fn employee_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_employees).post(create_employee))
        .route("/:code", get(get_employee))
}

async fn list_employees(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<Employee>>, ApiError> {
    let employees = state.db.list_employees("TECHFIRM_01")?;
    Ok(Json(employees))
}

async fn get_employee(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> std::result::Result<Json<Employee>, ApiError> {
    let emp = state
        .db
        .get_employee_by_code(&code)?
        .ok_or_else(|| attendance_common::AppError::NotFound(format!("Employee {} not found", code)))?;
    Ok(Json(emp))
}

async fn create_employee(
    State(state): State<AppState>,
    Json(payload): Json<CreateEmployeePayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let full_name = format!("{} {}", payload.first_name, payload.last_name).trim().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let emp = Employee {
        id: None,
        employee_code: payload.employee_code.to_uppercase(),
        user_uuid: uuid::Uuid::new_v4().to_string(),
        company_id: payload.company_id,
        first_name: payload.first_name,
        last_name: payload.last_name,
        full_name,
        email: payload.email,
        phone: payload.phone,
        department_id: payload.department_id,
        designation_id: None,
        shift_id: payload.shift_id,
        rfid_card_uid: payload.rfid_card_uid,
        pin_code: payload.pin_code,
        employment_type: EmploymentType::Permanent,
        join_date: payload.join_date,
        end_date: None,
        is_active: true,
        avatar_url: None,
        emergency_contact_name: None,
        emergency_contact_phone: None,
        created_at: Some(now.clone()),
        updated_at: Some(now),
    };

    state.db.save_employee(&emp)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "employee_code": emp.employee_code,
        "message": "Employee created successfully"
    })))
}
