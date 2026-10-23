use crate::{ApiError, AppState};
use attendance_common::TimeHelper;
use attendance_domain::*;
use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DateFilterQuery {
    pub date: Option<String>,
    pub employee_code: Option<String>,
}

#[derive(Deserialize)]
pub struct DateRangeQuery {
    pub from: String,
    pub to: String,
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct SevenDayHistoryQuery {
    pub username: Option<String>,
}

#[derive(Deserialize)]
pub struct ForcedLoginRequest {
    pub username: String,
}

#[derive(Deserialize)]
pub struct ApiPunchPayload {
    #[serde(flatten)]
    pub punch: PunchRequest,
    pub image_b64: Option<String>,
}

pub fn attendance_router() -> Router<AppState> {
    Router::new()
        .route("/punch", post(handle_punch))
        .route("/forced-login", post(handle_forced_login))
        .route("/batch-punch", post(handle_batch_punch))
        .route("/today", get(get_today_attendance))
        .route("/records", get(get_attendance_records))
        .route("/range", get(get_range_attendance))
        .route("/summary", get(get_attendance_summary))
        .route("/history-7days", get(get_seven_day_history))
        .route("/history/:username", get(get_seven_day_history_by_path))
        .route("/user/:username", get(get_user_punch_history))
        .route("/user-stats/:username", get(get_user_monthly_stats))
        .route("/adjust", post(handle_manual_adjustment))
}

async fn handle_punch(
    State(state): State<AppState>,
    Json(api_payload): Json<ApiPunchPayload>,
) -> std::result::Result<Json<PunchResponse>, ApiError> {
    let mut payload = api_payload.punch;
    let mut decoded_image_bytes: Option<Vec<u8>> = None;

    if let Some(ref b64) = api_payload.image_b64 {
        if !b64.trim().is_empty() {
            use base64::{engine::general_purpose, Engine as _};
            let clean = b64
                .trim()
                .split_once(',')
                .map(|(_, d)| d)
                .unwrap_or_else(|| b64.trim());
            let bytes = general_purpose::STANDARD.decode(clean).map_err(|e| {
                attendance_common::AppError::Validation(format!("Invalid base64 image: {}", e))
            })?;
            if payload.face_embedding.is_none() {
                let df_state = state.get_deepface().ok_or_else(|| {
                    attendance_common::AppError::Validation(
                        "DeepFace ONNX models not loaded in ./models/".to_string(),
                    )
                })?;
                let emb = attendance_face::extract_face_embedding_vec(&df_state, &bytes)?;
                payload.face_embedding = Some(emb);
            }
            decoded_image_bytes = Some(bytes);
        }
    }

    if let Some(ref emb) = payload.face_embedding {
        let matched_user = state
            .face_index
            .search_tiered(emb.clone())
            .ok()
            .flatten()
            .filter(|m| m.is_match)
            .map(|m| m.username);

        if payload.username.trim().is_empty() {
            if let Some(uname) = matched_user {
                payload.username = uname;
            } else {
                return Err(attendance_common::AppError::Unauthorized(
                    "Face verification failed: no confident match found in enrolled profiles".to_string(),
                )
                .into());
            }
        } else if decoded_image_bytes.is_some() {
            let claimed = payload.username.trim().to_uppercase();
            match matched_user {
                Some(ref uname) if uname.eq_ignore_ascii_case(&claimed) => {}
                Some(other_user) => {
                    return Err(attendance_common::AppError::Unauthorized(format!(
                        "Face verification mismatch: camera image matched '{}' instead of claimed user '{}'",
                        other_user, claimed
                    ))
                    .into());
                }
                None => {
                    return Err(attendance_common::AppError::Unauthorized(
                        "Face verification failed: camera image did not match claimed user".to_string(),
                    )
                    .into());
                }
            }
        }
    } else if payload.username.trim().is_empty() {
        return Err(attendance_common::AppError::Validation(
            "Either username or a valid camera face image/embedding must be provided".to_string(),
        )
        .into());
    }

    if let Some(bytes) = decoded_image_bytes {
        if !payload.username.trim().is_empty() {
            let captures_dir = std::path::Path::new("data/captures");
            let _ = std::fs::create_dir_all(captures_dir);
            let upper = attendance_common::sanitize_filename(&payload.username.trim().to_uppercase())?;
            let date_str = TimeHelper::current_date_dhaka();
            let time_str = TimeHelper::current_time_dhaka().replace(':', "");
            let filename = format!("{}_{}_{}.jpg", upper, date_str, time_str);
            if let Ok(disk_path) = attendance_common::validate_filepath(
                &filename,
                Some(captures_dir),
                &["jpg", "jpeg", "png"],
            ) {
                if std::fs::write(&disk_path, &bytes).is_ok() {
                    payload.video_path = Some(format!("/api/v1/captures/{}", filename));
                }
            }
        }
    }

    let res = state.engine.process_punch(payload)?;
    Ok(Json(res))
}

async fn handle_batch_punch(
    State(state): State<AppState>,
    Json(batch): Json<BatchPunchRequest>,
) -> std::result::Result<Json<BatchPunchResponse>, ApiError> {
    let total = batch.punches.len();
    let mut successful = 0;
    let mut failed = 0;
    let mut results = Vec::new();
    let mut errors = Vec::new();

    for mut p in batch.punches {
        if p.device_id.is_none() {
            p.device_id = Some(batch.device_id.clone());
        }
        match state.engine.process_punch(p) {
            Ok(resp) => {
                successful += 1;
                results.push(resp);
            }
            Err(e) => {
                failed += 1;
                errors.push(e.to_string());
            }
        }
    }

    Ok(Json(BatchPunchResponse {
        total,
        successful,
        failed,
        results,
        errors,
    }))
}

async fn get_today_attendance(
    State(state): State<AppState>,
) -> std::result::Result<Json<Vec<AttendanceRecord>>, ApiError> {
    let today = TimeHelper::current_date_dhaka();
    let records = state.db.get_attendance_by_date(&today)?;
    Ok(Json(records))
}

async fn get_attendance_records(
    State(state): State<AppState>,
    Query(query): Query<DateFilterQuery>,
) -> std::result::Result<Json<Vec<AttendanceRecord>>, ApiError> {
    let target_date = query.date.unwrap_or_else(TimeHelper::current_date_dhaka);
    let records = state.db.get_attendance_by_date(&target_date)?;
    Ok(Json(records))
}

async fn get_range_attendance(
    State(state): State<AppState>,
    Query(query): Query<DateRangeQuery>,
) -> std::result::Result<Json<Vec<AttendanceRecord>>, ApiError> {
    let records = state.db.get_attendance_records_by_range(
        &query.from,
        &query.to,
        query.username.as_deref(),
    )?;
    Ok(Json(records))
}

async fn get_attendance_summary(
    State(state): State<AppState>,
    Query(query): Query<DateFilterQuery>,
) -> std::result::Result<Json<AttendanceSummaryResponse>, ApiError> {
    let summary = state.reporting.get_daily_summary(query.date.as_deref())?;
    Ok(Json(summary))
}

async fn get_seven_day_history_by_path(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    get_seven_day_history(
        State(state),
        Query(SevenDayHistoryQuery {
            username: Some(username),
        }),
    )
    .await
}

async fn get_seven_day_history(
    State(state): State<AppState>,
    Query(query): Query<SevenDayHistoryQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let target_user = match query.username {
        Some(u) if !u.trim().is_empty() => u.to_uppercase(),
        _ => state
            .db
            .get_last_updated_attendance_username()?
            .unwrap_or_else(|| "UNKNOWN".to_string())
            .to_uppercase(),
    };

    let today_str = TimeHelper::current_date_dhaka();
    let today_date = TimeHelper::parse_date(&today_str)
        .unwrap_or_else(|_| chrono::NaiveDate::from_ymd_opt(2026, 10, 14).unwrap());
    let seven_days_ago = today_date - chrono::Duration::days(7);

    let records = state.db.get_attendance_records_by_range(
        &seven_days_ago.format("%Y-%m-%d").to_string(),
        &today_date.format("%Y-%m-%d").to_string(),
        Some(&target_user),
    )?;

    let mut rec_by_date = std::collections::HashMap::new();
    for r in records {
        let norm_date = TimeHelper::parse_date(&r.date)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|_| r.date.clone());
        rec_by_date.insert(norm_date, r);
    }

    let mut rows = Vec::new();
    for i in 0..7 {
        let d = today_date - chrono::Duration::days(i);
        let iso_date = d.format("%Y-%m-%d").to_string();
        let display_date = if i == 0 {
            format!("{} (Today)", d.format("%d %b %Y"))
        } else {
            d.format("%d %b %Y").to_string()
        };

        let (in_time_str, out_time_str, duration, status_color) = if let Some(rec) = rec_by_date.get(&iso_date) {
            let in_fmt = rec
                .in_time
                .as_deref()
                .map(TimeHelper::format_12hour)
                .unwrap_or_else(|| "---".to_string());
            let out_fmt = rec
                .out_time
                .as_deref()
                .map(TimeHelper::format_12hour)
                .unwrap_or_else(|| "---".to_string());

            let dur = if let (Some(in_raw), Some(out_raw)) = (rec.in_time.as_deref(), rec.out_time.as_deref()) {
                if let (Ok(t_in), Ok(t_out)) = (TimeHelper::parse_time(in_raw), TimeHelper::parse_time(out_raw)) {
                    let mut diff_secs = (t_out - t_in).num_seconds();
                    if diff_secs < 0 {
                        diff_secs += 86400;
                    }
                    let hours = diff_secs / 3600;
                    let minutes = (diff_secs % 3600) / 60;
                    format!("{}h {}m", hours, minutes)
                } else {
                    "---".to_string()
                }
            } else {
                "---".to_string()
            };

            let color = if in_fmt != "---" && out_fmt != "---" {
                "#e8f5e9"
            } else if in_fmt != "---" {
                "#fff3e0"
            } else if (i + 1) % 2 == 0 {
                "#f8f9fa"
            } else {
                "white"
            };
            (in_fmt, out_fmt, dur, color)
        } else {
            let row_bg = if (i + 1) % 2 == 0 { "#f8f9fa" } else { "white" };
            ("---".to_string(), "---".to_string(), "---".to_string(), row_bg)
        };

        rows.push(serde_json::json!({
            "date": display_date,
            "iso_date": iso_date,
            "in_time": in_time_str,
            "out_time": out_time_str,
            "duration": duration,
            "status_color": status_color
        }));
    }

    Ok(Json(serde_json::json!({
        "username": target_user,
        "title": format!("{} - 7 Day Attendance History", target_user),
        "generated_on": today_date.format("%d %b %Y").to_string(),
        "history": rows,
        "records": rows
    })))
}

async fn get_user_punch_history(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<Vec<PunchLog>>, ApiError> {
    let logs = state.db.get_user_punch_logs(&username, 100)?;
    Ok(Json(logs))
}

async fn handle_manual_adjustment(
    State(state): State<AppState>,
    Json(payload): Json<ManualAttendanceAdjustmentRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let now = TimeHelper::now_utc_sqlite_string();
    let cu_uuid = state
        .db
        .get_user_by_username(&payload.username)
        .ok()
        .flatten()
        .map(|u| u.company_user_uuid)
        .unwrap_or_else(|| payload.username.to_uppercase());

    let record_id = payload
        .record_id
        .unwrap_or_else(|| TimeHelper::generate_record_id(&cu_uuid, &payload.date));

    let rec = AttendanceRecord {
        record_id,
        company_user_uuid: Some(cu_uuid),
        username: payload.username.to_uppercase(),
        date: payload.date.clone(),
        in_time: payload.in_time,
        out_time: payload.out_time,
        video_path_in: None,
        video_path_out: None,
        in_time_with_camera: Some("N".to_string()),
        out_time_with_camera: Some("N".to_string()),
        in_time_with_qr: Some("N".to_string()),
        out_time_with_qr: Some("N".to_string()),
        status: AttendanceStatus::Present,
        work_hours: 8.0,
        overtime_hours: 0.0,
        late_minutes: 0,
        early_exit_minutes: 0,
        ip_address: None,
        device_id: Some("MANUAL_ADMIN".to_string()),
        remarks: Some(payload.reason.clone()),
        created_at: Some(now.clone()),
    };

    state.db.save_in_time(&rec)?;

    let audit_log = SecurityAuditLog {
        id: None,
        event_type: "MANUAL_PUNCH_ADJUSTMENT".to_string(),
        actor_username: Some(payload.approved_by),
        target_resource: Some(format!("attendance:{}:{}", payload.username, payload.date)),
        ip_address: None,
        severity: AuditSeverity::Warning,
        details_json: Some(format!("Reason: {}", payload.reason)),
        created_at: now,
    };
    let _ = state.db.log_security_event(&audit_log);

    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Attendance record manually adjusted successfully"
    })))
}

async fn handle_forced_login(
    State(state): State<AppState>,
    Json(payload): Json<ForcedLoginRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let trimmed = payload.username.trim();
    if trimmed.is_empty() {
        return Err(attendance_common::AppError::Validation("Username is required".to_string()).into());
    }
    let punch_req = PunchRequest {
        username: trimmed.to_uppercase(),
        pin: None,
        android_id: None,
        method: Some("MANUAL".to_string()),
        device_id: Some("FORCED_LOGIN".to_string()),
        latitude: None,
        longitude: None,
        video_path: None,
        face_embedding: None,
        remarks: Some("Forced login without camera".to_string()),
    };
    let resp = state.engine.process_punch(punch_req)?;
    let time_12h = TimeHelper::format_12hour(&resp.timestamp);
    Ok(Json(serde_json::json!({
        "success": true,
        "username": resp.username,
        "login_type": resp.login_type,
        "record_id": resp.record_id,
        "timestamp": resp.timestamp,
        "message": format!("{}, YOUR {} IS RECORDED AT {}", resp.username, resp.login_type, time_12h)
    })))
}

async fn get_user_monthly_stats(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let upper = username.trim().to_uppercase();
    let today = TimeHelper::current_date_dhaka();
    let year_month = if today.len() >= 7 { &today[..7] } else { "2026-10" };
    let from = format!("{}-01", year_month);
    let to = format!("{}-31", year_month);

    let records = state
        .db
        .get_attendance_records_by_range(&from, &to, Some(&upper))?;

    let mut lates = 0usize;
    let mut late_absents = 0usize;
    let mut early_leaves = 0usize;

    let nine_am = chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap();
    let ten_am = chrono::NaiveTime::from_hms_opt(10, 0, 0).unwrap();
    let five_pm = chrono::NaiveTime::from_hms_opt(17, 0, 0).unwrap();

    for r in records {
        if let Some(ref in_t) = r.in_time {
            if let Ok(parsed_in) = TimeHelper::parse_time(in_t) {
                if parsed_in >= nine_am {
                    lates += 1;
                }
                if parsed_in >= ten_am {
                    late_absents += 1;
                }
            }
        }
        if let Some(ref out_t) = r.out_time {
            if let Ok(parsed_out) = TimeHelper::parse_time(out_t) {
                if parsed_out < five_pm {
                    early_leaves += 1;
                }
            }
        }
    }

    Ok(Json(serde_json::json!({
        "username": upper,
        "month": year_month,
        "total_lates": lates,
        "total_late_absents": late_absents,
        "total_early_leaves": early_leaves
    })))
}

pub async fn serve_capture_image(
    Path(filename): Path<String>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;

    let safe_name = match attendance_common::sanitize_filename(&filename) {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid filename").into_response(),
    };
    let captures_dir = std::path::Path::new("data/captures");
    let validated = match attendance_common::validate_filepath(
        &safe_name,
        Some(captures_dir),
        &["jpg", "jpeg", "png"],
    ) {
        Ok(p) => p,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid filename").into_response(),
    };
    match std::fs::read(&validated) {
        Ok(bytes) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "image/jpeg")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Capture image not found").into_response(),
    }
}

