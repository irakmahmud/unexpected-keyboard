use crate::{ApiError, AppState};
use attendance_domain::SecurityAuditLog;
use attendance_reporting::AuditLogExporter;
use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AuditQuery {
    pub limit: Option<usize>,
}

pub fn audit_router() -> Router<AppState> {
    Router::new()
        .route("/logs", get(get_recent_audit_logs))
        .route("/export.csv", get(export_audit_csv))
        .route("/export.json", get(export_audit_json))
}

async fn get_recent_audit_logs(
    State(state): State<AppState>,
    Query(query): Query<AuditQuery>,
) -> std::result::Result<Json<Vec<SecurityAuditLog>>, ApiError> {
    let limit = query.limit.unwrap_or(50).min(500);
    let logs = state.db.get_recent_audit_logs(limit)?;
    Ok(Json(logs))
}

async fn export_audit_csv(
    State(state): State<AppState>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let logs = state.db.get_recent_audit_logs(500)?;
    let csv = AuditLogExporter::to_csv(&logs);

    let headers = [
        (header::CONTENT_TYPE, "text/csv; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"security_audit_logs.csv\"",
        ),
    ];

    Ok((StatusCode::OK, headers, csv))
}

async fn export_audit_json(
    State(state): State<AppState>,
) -> std::result::Result<impl IntoResponse, ApiError> {
    let logs = state.db.get_recent_audit_logs(500)?;
    let json = AuditLogExporter::to_json(&logs)?;

    let headers = [
        (header::CONTENT_TYPE, "application/json; charset=utf-8"),
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"security_audit_logs.json\"",
        ),
    ];

    Ok((StatusCode::OK, headers, json))
}
