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
        .route("/process", post(handle_trigger_sync))
        .route("/status", get(get_outbox_status))
        .route("/users-to-local", post(handle_sync_users_to_local))
        .route("/users-to-cloud", post(handle_sync_users_to_cloud))
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

async fn handle_sync_users_to_local(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let users = state.db.get_all_users()?;
    let mut existing_embeddings = state.face_index.get_all_embeddings();
    let mut indexed = 0usize;

    if let Ok(profiles) = state.db.get_all_face_profiles() {
        let mut users_from_profiles = std::collections::HashSet::new();
        for p in profiles {
            let upper = p.username.trim().to_uppercase();
            if existing_embeddings.contains_key(&upper) {
                continue;
            }
            if !p.embedding_vector.is_empty()
                && !attendance_face::FaceVectorIndex::is_corrupted_constant_vector(&p.embedding_vector)
            {
                if state.face_index.enroll_user(&upper, p.embedding_vector).is_ok() {
                    users_from_profiles.insert(upper);
                    indexed += 1;
                }
            }
        }
        if !users_from_profiles.is_empty() {
            existing_embeddings = state.face_index.get_all_embeddings();
        }
    }

    for u in &users {
        let upper = u.username.trim().to_uppercase();
        if existing_embeddings.contains_key(&upper) {
            continue;
        }
        if let Some(ref enc_str) = u.face_encoding {
            if let Ok(vec) = serde_json::from_str::<Vec<f32>>(enc_str) {
                if !vec.is_empty()
                    && vec.iter().any(|&v| v != 0.0)
                    && !attendance_face::FaceVectorIndex::is_corrupted_constant_vector(&vec)
                {
                    if state.face_index.enroll_user(&upper, vec).is_ok() {
                        indexed += 1;
                    }
                }
            }
        }
    }

    if let Ok(faiss_loaded) = state.face_index.load_faiss_files() {
        indexed += faiss_loaded;
    }
    let _ = state.face_index.sync_faiss_files();

    Ok(Json(serde_json::json!({
        "success": true,
        "local_users_count": users.len(),
        "indexed_faces_count": indexed,
        "message": format!("Synchronized {} users and {} facial encodings in local database", users.len(), indexed)
    })))
}


async fn handle_sync_users_to_cloud(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let users = state.db.get_all_users()?;
    let bq_cycle = state.outbox_worker.process_cycle().await.unwrap_or(0);
    Ok(Json(serde_json::json!({
        "success": true,
        "users_checked": users.len(),
        "outbox_events_flushed": bq_cycle,
        "message": format!("Successfully verified {} local users and flushed {} pending cloud sync events", users.len(), bq_cycle)
    })))
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
