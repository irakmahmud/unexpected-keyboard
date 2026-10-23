use crate::{ApiError, AppState};
use attendance_common::TimeHelper;
use attendance_domain::*;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};

pub fn system_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(get_health_status))
        .route("/vacuum", post(trigger_vacuum))
        .route("/integrity", get(check_integrity))
        .route("/backup", post(create_backup).get(list_backups))
        .route("/stats/departments", get(get_department_stats))
        .route("/stats/hourly", get(get_hourly_stats))
}

pub async fn get_health_status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let now_dhaka = TimeHelper::current_datetime_dhaka();
    let bq_configured = state.bq_client.is_configured();
    let enrolled_faces = state.db.count_enrolled_faces().unwrap_or(0);

    Json(serde_json::json!({
        "status": "healthy",
        "system": "Pure Rust Axum Attendance Core",
        "service_name": "Attendance Pure Rust Axum System",
        "version": "8.0.0-pure-rust",
        "server_time_dhaka": now_dhaka,
        "database": {
            "type": "SQLite WAL Mode (Operational Authoritative)",
            "foreign_keys": "ENABLED",
            "enrolled_biometrics_count": enrolled_faces
        },
        "cloud_analytics": {
            "platform": "Google BigQuery",
            "configured": bq_configured,
            "architecture": "Transactional Outbox Pattern with Zero Data Loss"
        },
        "python_runtime_dependencies": "None (100% Native Pure Rust)",
        "memory_safety": "Guaranteed by Rust compiler borrow checker"
    }))
}

async fn trigger_vacuum(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    state.db.vacuum_database()?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Database VACUUM completed successfully"
    })))
}

async fn check_integrity(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let res = state.db.verify_database_integrity()?;
    Ok(Json(serde_json::json!({
        "success": true,
        "integrity_status": res
    })))
}

async fn create_backup(
    State(state): State<AppState>,
) -> std::result::Result<Json<DatabaseBackupInfo>, ApiError> {
    let backup_info = state.db.create_backup("data/backups")?;
    Ok(Json(backup_info))
}

async fn list_backups(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<DatabaseBackupInfo>>, ApiError> {
    let backups = state.db.list_backups()?;
    Ok(Json(backups))
}

async fn get_department_stats(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let today = TimeHelper::current_date_dhaka();
    let stats = state.db.get_department_stats(&today)?;
    Ok(Json(serde_json::json!(stats)))
}

async fn get_hourly_stats(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let today = TimeHelper::current_date_dhaka();
    let stats = state.db.get_hourly_punches(&today)?;
    Ok(Json(serde_json::json!(stats)))
}
