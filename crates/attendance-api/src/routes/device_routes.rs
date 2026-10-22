use crate::{ApiError, AppState};
use attendance_common::CryptoHelper;
use attendance_domain::*;
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};

pub fn device_router() -> Router<AppState> {
    Router::new()
        .route("/register", post(register_device))
        .route("/heartbeat", post(device_heartbeat))
        .route("/session", post(create_device_session))
        .route("/session/:id", get(get_device_session))
        .route(
            "/receive_android_id",
            post(receive_android_id).get(get_receive_android_id_fallback),
        )
}

async fn register_device(
    State(state): State<AppState>,
    Json(payload): Json<DeviceRegisterRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let dev = RegisteredDevice {
        device_id: payload.device_id.clone(),
        device_name: payload.device_name,
        android_id: None,
        mac_address: payload.mac_address,
        ip_address: payload.ip_address,
        device_type: payload.device_type,
        location_name: payload.location_tag,
        status: DeviceStatus::Online,
        last_heartbeat: Some(chrono::Utc::now().to_rfc3339()),
        app_version: Some("v8.0.0".to_string()),
        created_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    state.db.register_device(&dev)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "device_id": payload.device_id,
        "message": "Device registered successfully"
    })))
}

async fn device_heartbeat(
    State(state): State<AppState>,
    Json(payload): Json<DeviceHeartbeatRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    state.devices.record_heartbeat(&payload.device_id);

    Ok(Json(serde_json::json!({
        "success": true,
        "status": "ACK",
        "device_id": payload.device_id,
        "server_time": chrono::Utc::now().to_rfc3339()
    })))
}

async fn create_device_session(
    State(state): State<AppState>,
) -> std::result::Result<Json<DeviceSession>, ApiError> {
    let session = DeviceSession {
        session_id: CryptoHelper::new_short_token(),
        created_at: chrono::Utc::now().to_rfc3339(),
        registered_android_id: None,
        status: "PENDING".to_string(),
    };

    state.db.save_device_session(&session)?;
    Ok(Json(session))
}

pub async fn get_device_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    if let Ok(Some(session)) = state.db.get_device_session(&id) {
        Ok(Json(serde_json::json!({
            "status": "ready",
            "session_id": session.session_id,
            "registered_android_id": session.registered_android_id,
            "session_status": session.status,
            "created_at": session.created_at
        })))
    } else {
        // Fallback response matching Python AndroidIDReceiver line 5829
        Ok(Json(serde_json::json!({
            "status": "ready",
            "session_id": id
        })))
    }
}

pub async fn get_receive_android_id_fallback() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ready",
        "session_id": "receive_android_id"
    }))
}

pub async fn receive_android_id(
    State(state): State<AppState>,
    Json(payload): Json<AndroidIdRegistrationRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let aid = payload.android_id.trim();
    if aid.is_empty() {
        return Ok(Json(serde_json::json!({
            "status": "error",
            "message": "No ID provided"
        })));
    }

    // 1. Update or create session status if session_id is provided
    if let Some(sid) = &payload.session_id {
        let mut session = state.db.get_device_session(sid)?.unwrap_or_else(|| DeviceSession {
            session_id: sid.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            registered_android_id: None,
            status: "PENDING".to_string(),
        });
        session.registered_android_id = Some(aid.to_string());
        session.status = "CONNECTED".to_string();
        state.db.save_device_session(&session)?;
    }

    // 2. Check if user already exists with this Android ID for QR Attendance punch (matching Python handle_android_id)
    let existing_user = state.db.get_user_by_android_id(aid)?;
    let mut attendance_response = None;

    if let Some(u) = existing_user {
        let punch_req = PunchRequest {
            username: u.username.clone(),
            method: Some("QR".to_string()),
            pin: None,
            android_id: Some(aid.to_string()),
            video_path: None,
            face_embedding: None,
            latitude: None,
            longitude: None,
            device_id: payload.session_id.clone().or_else(|| Some("QR_TERMINAL".to_string())),
            remarks: Some("QR Scanned via Mobile Device".to_string()),
        };

        match state.engine.process_punch(punch_req) {
            Ok(resp) => {
                attendance_response = Some(serde_json::to_value(resp).unwrap());
            }
            Err(e) => {
                tracing::warn!("Auto-punch on QR receive error for user {}: {}", u.username, e);
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "status": "success",
        "android_id": aid,
        "message": "ID received!",
        "attendance": attendance_response
    })))
}
