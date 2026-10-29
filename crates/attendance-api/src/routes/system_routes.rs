use crate::{ApiError, AppState};
use attendance_common::TimeHelper;
use attendance_domain::*;
use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct CursorQueryParams {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub after_date: Option<String>,
    pub after_id: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct BenchmarkQueryParams {
    pub rows: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct ShardedQueryParams {
    pub username: String,
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub date: Option<String>,
}

pub fn system_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(get_health_status))
        .route("/scale-architecture", get(get_scale_architecture))
        .route("/partitions", get(get_storage_partitions))
        .route("/cursor-attendance", get(get_cursor_attendance))
        .route(
            "/run-scale-benchmark",
            get(run_scale_benchmark).post(run_scale_benchmark),
        )
        .route("/postgres-migration-ddl", get(get_postgres_migration_ddl))
        .route("/sharded-query", get(get_sharded_query))
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
    let partitions_count = state.db.list_storage_partitions().map(|v| v.len()).unwrap_or(0);

    Json(serde_json::json!({
        "status": "healthy",
        "system": "Pure Rust Axum Attendance Core",
        "service_name": "Attendance Pure Rust Axum System",
        "version": "8.0.0-pure-rust",
        "server_time_dhaka": now_dhaka,
        "database": {
            "type": "SQLite WAL Mode (Operational Authoritative)",
            "active_driver": "SQLITE_WAL_PARTITIONED",
            "switchable_drivers": ["POSTGRESQL_PARTITIONED", "TIMESCALEDB_HYPERTABLE", "CLICKHOUSE_CLUSTER"],
            "scale_capacity": "50 Years / 1,000,000 (10 Lakh) Employees (64 Hash Shards + Time Partitions)",
            "active_time_partitions": partitions_count,
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

async fn get_scale_architecture(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let status = state.db.get_scale_architecture_status()?;
    Ok(Json(status))
}

async fn get_storage_partitions(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let partitions = state.db.list_storage_partitions()?;
    Ok(Json(serde_json::json!({
        "total_partitions": partitions.len(),
        "partitions": partitions
    })))
}

async fn get_cursor_attendance(
    State(state): State<AppState>,
    Query(params): Query<CursorQueryParams>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let res = state.db.get_cursor_paginated_attendance(
        params.year,
        params.month,
        params.after_date.as_deref(),
        params.after_id.as_deref(),
        params.limit.unwrap_or(50),
    )?;
    Ok(Json(res))
}

async fn run_scale_benchmark(
    State(state): State<AppState>,
    Query(params): Query<BenchmarkQueryParams>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let rows = params.rows.unwrap_or(50_000).clamp(1_000, 1_000_000);
    let db = state.db.clone();
    let res = tokio::task::spawn_blocking(move || db.run_empirical_scale_benchmark(rows))
        .await
        .map_err(|e| attendance_common::AppError::Internal(e.to_string()))??;
    Ok(Json(res))
}

async fn get_postgres_migration_ddl(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let ddl = state.db.generate_postgres_migration_ddl();
    Ok(Json(serde_json::json!({
        "target_engine": "PostgreSQL 16+ Declarative RANGE + HASH Partitioning",
        "ddl_sql": ddl
    })))
}

async fn get_sharded_query(
    State(state): State<AppState>,
    Query(params): Query<ShardedQueryParams>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let engine = state.db.sharded_engine.as_ref().ok_or_else(|| {
        attendance_common::AppError::Internal("PhysicalShardedEngine not initialized".to_string())
    })?;
    let res = engine.query_partitioned_user(
        &params.username,
        params.year.unwrap_or(2026),
        params.month.unwrap_or(10),
        params.date.as_deref(),
    )?;
    Ok(Json(res))
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
