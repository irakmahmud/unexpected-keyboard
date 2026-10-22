use crate::{ApiError, AppState};
use attendance_common::TimeHelper;
use attendance_reporting::{AnalyticsEngine, HtmlReportRenderer, MusterRollGenerator, ReportMetadata};
use axum::{
    extract::{Query, State},
    http::{header, HeaderValue},
    response::{Html, IntoResponse, Response},
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct ReportQuery {
    pub date: Option<String>,
    pub year: Option<i32>,
    pub month: Option<u32>,
}

#[derive(Deserialize)]
pub struct CustomizedReportQuery {
    pub username: Option<String>,
    pub year: Option<i32>,
    pub month: Option<String>,
    #[serde(alias = "from_date")]
    pub from_day: Option<u32>,
    #[serde(alias = "to_date")]
    pub to_day: Option<u32>,
    #[serde(alias = "set_in_time")]
    pub in_time: Option<String>,
    #[serde(alias = "set_out_time")]
    pub out_time: Option<String>,
}

pub fn report_router() -> Router<AppState> {
    Router::new()
        .route("/daily-summary", get(get_daily_summary))
        .route("/muster-roll", get(get_muster_roll))
        .route("/muster-roll/html", get(get_muster_roll_html))
        .route("/export.csv", get(export_csv_report))
        .route("/csv", get(export_csv_report))
        .route("/customized", get(export_customized_report))
        .route("/customized.csv", get(export_customized_report))
        .route("/xlsx", get(export_customized_xlsx_report))
        .route("/excel", get(export_customized_xlsx_report))
        .route("/export.xlsx", get(export_customized_xlsx_report))
        .route("/customized.xlsx", get(export_customized_xlsx_report))
        .route("/verify-images", get(get_verify_images))
        .route("/department-breakdown", get(get_department_breakdown))
}

async fn get_daily_summary(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let summary = state.reporting.get_daily_summary(query.date.as_deref())?;
    Ok(Json(serde_json::to_value(summary).unwrap()))
}

fn resolve_active_company(state: &AppState) -> (String, String) {
    if let Ok(mut companies) = state.db.list_companies() {
        companies.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
        if let Some(c) = companies.into_iter().next() {
            return (c.company_id, c.company_name);
        }
    }
    ("HQ_001".to_string(), "Smart Attendance HQ".to_string())
}

async fn get_muster_roll(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let year = query.year.unwrap_or(2026);
    let month = query.month.unwrap_or(10);
    let (comp_id, comp_name) = resolve_active_company(&state);
    let generator = MusterRollGenerator::new(state.db.clone());

    let roll = generator.generate_monthly_muster_roll(year, month, &comp_id)?;

    Ok(Json(serde_json::json!({
        "year": year,
        "month": month,
        "company_id": comp_id,
        "company_name": comp_name,
        "rows": roll
    })))
}

async fn get_muster_roll_html(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> std::result::Result<Html<String>, ApiError> {
    let year = query.year.unwrap_or(2026);
    let month = query.month.unwrap_or(10);
    let (comp_id, comp_name) = resolve_active_company(&state);
    let generator = MusterRollGenerator::new(state.db.clone());
    let roll = generator.generate_monthly_muster_roll(year, month, &comp_id)?;

    let meta = ReportMetadata {
        company_name: comp_name,
        title: "MONTHLY MUSTER ROLL & ATTENDANCE REGISTER".to_string(),
        month_year: format!("{:04}-{:02}", year, month),
        generated_at: TimeHelper::current_datetime_dhaka(),
        generated_by: "System Admin (Pure Rust Engine)".to_string(),
    };

    let days_in_month = TimeHelper::days_in_month(year, month);
    let html = HtmlReportRenderer::render_muster_roll_html(&meta, &roll, days_in_month);
    Ok(Html(html))
}

async fn export_csv_report(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> std::result::Result<Response, ApiError> {
    let target_date = query.date.unwrap_or_else(TimeHelper::current_date_dhaka);
    let csv = state.reporting.generate_csv_report(&target_date)?;

    let mut response = csv.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"attendance_{}.csv\"",
            target_date
        ))
        .unwrap(),
    );
    Ok(response)
}

async fn export_customized_report(
    State(state): State<AppState>,
    Query(query): Query<CustomizedReportQuery>,
) -> std::result::Result<Response, ApiError> {
    let year = query.year.unwrap_or(2026);
    let month_str = query.month.as_deref().unwrap_or("10");
    let month = TimeHelper::parse_month(month_str).unwrap_or(10);
    let max_days = TimeHelper::days_in_month(year, month);
    let from_day = query.from_day.unwrap_or(1).clamp(1, max_days);
    let to_day = query.to_day.unwrap_or(max_days).clamp(from_day, max_days);
    let in_time = query.in_time.as_deref().unwrap_or("09:00:00");
    let out_time = query.out_time.as_deref().unwrap_or("17:00:00");

    let csv = state.reporting.generate_customized_csv_report(
        query.username.as_deref(),
        year,
        month,
        from_day,
        to_day,
        in_time,
        out_time,
    )?;

    let mut response = csv.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"attendance_customized_{}_{:02}.csv\"",
            year, month
        ))
        .unwrap(),
    );
    Ok(response)
}

async fn export_customized_xlsx_report(
    State(state): State<AppState>,
    Query(query): Query<CustomizedReportQuery>,
) -> std::result::Result<Response, ApiError> {
    let year = query.year.unwrap_or(2026);
    let month_str = query.month.as_deref().unwrap_or("10");
    let month = TimeHelper::parse_month(month_str).unwrap_or(10);
    let max_days = TimeHelper::days_in_month(year, month);
    let from_day = query.from_day.unwrap_or(1).clamp(1, max_days);
    let to_day = query.to_day.unwrap_or(max_days).clamp(from_day, max_days);
    let in_time = query.in_time.as_deref().unwrap_or("09:00:00");
    let out_time = query.out_time.as_deref().unwrap_or("17:00:00");

    let xlsx_bytes = state.reporting.generate_customized_xlsx_report(
        query.username.as_deref(),
        year,
        month,
        from_day,
        to_day,
        in_time,
        out_time,
    )?;

    let mut response = xlsx_bytes.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&format!(
            "attachment; filename=\"attendance_customized_{}_{:02}.xlsx\"",
            year, month
        ))
        .unwrap(),
    );
    Ok(response)
}


async fn get_verify_images(
    State(state): State<AppState>,
    Query(query): Query<CustomizedReportQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let year = query.year.unwrap_or(2026);
    let month_str = query.month.as_deref().unwrap_or("10");
    let month = TimeHelper::parse_month(month_str).unwrap_or(10);
    let max_days = TimeHelper::days_in_month(year, month);
    let from_day = query.from_day.unwrap_or(1).clamp(1, max_days);
    let to_day = query.to_day.unwrap_or(max_days).clamp(1, max_days);

    if from_day > to_day {
        return Err(attendance_common::AppError::Validation(
            "From date cannot be greater than To date".to_string(),
        )
        .into());
    }

    let from_date_str = format!("{:04}-{:02}-{:02}", year, month, from_day);
    let to_date_str = format!("{:04}-{:02}-{:02}", year, month, to_day);
    let user_filter = match query.username.as_deref() {
        Some(u) if !u.trim().is_empty() && !u.eq_ignore_ascii_case("All Users") => Some(u),
        _ => None,
    };

    let records = state
        .db
        .get_attendance_records_by_range(&from_date_str, &to_date_str, user_filter)?;

    let mut images = Vec::new();
    for r in records {
        if r.video_path_in.is_some() || r.video_path_out.is_some() || r.in_time.is_some() || r.out_time.is_some() {
            images.push(serde_json::json!({
                "record_id": r.record_id,
                "username": r.username,
                "date": r.date,
                "in_time": r.in_time,
                "out_time": r.out_time,
                "in_time_with_camera": r.in_time_with_camera,
                "out_time_with_camera": r.out_time_with_camera,
                "video_path_in": r.video_path_in,
                "video_path_out": r.video_path_out
            }));
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "username": query.username.unwrap_or_else(|| "All Users".to_string()),
        "year": year,
        "month": month,
        "from_date": from_day,
        "to_date": to_day,
        "total_found": images.len(),
        "items": images
    })))
}

async fn get_department_breakdown(
    State(state): State<AppState>,
    Query(query): Query<ReportQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let target_date = query.date.unwrap_or_else(TimeHelper::current_date_dhaka);
    let analytics = AnalyticsEngine::new(state.db.clone());
    let breakdown = analytics.department_attendance_rate(&target_date)?;

    Ok(Json(serde_json::json!({
        "date": target_date,
        "departments": breakdown
    })))
}
