use crate::{ApiError, AppState};
use attendance_sync::DeadLetterQueueManager;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct OutboxStatusReport {
    pub pending_count: usize,
    pub cloud_target: String,
    pub bq_configured: bool,
    pub worker_active: bool,
}

pub fn sync_router() -> Router<AppState> {
    Router::new()
        .route("/trigger", post(handle_trigger_sync))
        .route("/status", get(get_outbox_status))
        .route("/dead-letters", get(get_dead_letters))
        .route("/dead-letters/:id/retry", post(retry_dead_letter))
}

async fn handle_trigger_sync(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let result = state.outbox_worker.process_cycle().await;
    match result {
        Ok(count) => Ok(Json(serde_json::json!({
            "success": true,
            "synced_events_count": count,
            "message": format!("Successfully processed {} events to Google BigQuery", count)
        }))),
        Err(e) => Ok(Json(serde_json::json!({
            "success": false,
            "error": e.to_string(),
            "message": "BigQuery sync failed or not configured. Events retained in durable SQLite outbox."
        }))),
    }
}

async fn get_outbox_status(
    State(state): State<AppState>,
) -> std::result::Result<Json<OutboxStatusReport>, ApiError> {
    let pending = state.db.fetch_pending_outbox_events(1000)?;
    let bq_configured = state.bq_client.is_configured();

    Ok(Json(OutboxStatusReport {
        pending_count: pending.len(),
        cloud_target: "Google BigQuery Streaming Ingestion".to_string(),
        bq_configured,
        worker_active: true,
    }))
}

async fn get_dead_letters(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<attendance_sync::DeadLetterItem>>, ApiError> {
    let dlq = DeadLetterQueueManager::new(state.db.clone());
    let items = dlq.list_dead_letters(50)?;
    Ok(Json(items))
}

async fn retry_dead_letter(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let dlq = DeadLetterQueueManager::new(state.db.clone());
    let success = dlq.retry_event(&id)?;
    Ok(Json(serde_json::json!({
        "success": success,
        "event_id": id,
        "message": "Dead letter event reset to PENDING status"
    })))
}
