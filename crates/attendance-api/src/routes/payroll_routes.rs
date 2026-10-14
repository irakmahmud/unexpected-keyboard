use crate::{ApiError, AppState};
use attendance_core::PayrollCalculator;
use attendance_domain::*;
use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PayslipCalculateQuery {
    pub employee_code: String,
    pub month_year: String, // "2026-10"
}

pub fn payroll_router() -> Router<AppState> {
    Router::new()
        .route("/salary/:code", get(get_salary_structure).post(save_salary_structure))
        .route("/payslip", get(get_or_calculate_payslip))
}

async fn get_salary_structure(
    State(state): State<AppState>,
    Path(code): Path<String>,
) -> std::result::Result<Json<Option<SalaryStructure>>, ApiError> {
    let salary = state.db.get_salary_structure(&code)?;
    Ok(Json(salary))
}

async fn save_salary_structure(
    State(state): State<AppState>,
    Path(code): Path<String>,
    Json(mut salary): Json<SalaryStructure>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    salary.employee_code = code;
    state.db.save_salary_structure(&salary)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Salary structure saved successfully"
    })))
}

async fn get_or_calculate_payslip(
    State(state): State<AppState>,
    Query(query): Query<PayslipCalculateQuery>,
) -> std::result::Result<Json<PaySlip>, ApiError> {
    // Check if already calculated
    if let Some(existing) = state.db.get_payslip(&query.employee_code, &query.month_year)? {
        return Ok(Json(existing));
    }

    // Retrieve salary structure
    let salary = state.db.get_salary_structure(&query.employee_code)?
        .unwrap_or_else(|| {
            let mut s = SalaryStructure::default();
            s.employee_code = query.employee_code.clone();
            s
        });

    let emp = state.db.get_employee_by_code(&query.employee_code)?
        .ok_or_else(|| attendance_common::AppError::NotFound("Employee not found".to_string()))?;

    let ot_config = OvertimeConfig::default();

    let payslip = PayrollCalculator::calculate_monthly_payslip(
        &salary,
        &ot_config,
        &emp.full_name,
        &query.month_year,
        22,
        21,
        1,
        0,
        4,
        8.0,
        1,
    )?;

    state.db.save_payslip(&payslip)?;

    Ok(Json(payslip))
}
