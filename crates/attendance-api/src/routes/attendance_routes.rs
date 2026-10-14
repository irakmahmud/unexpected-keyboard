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
        .route("/user/:username", get(get_user_punch_history))
        .route("/adjust", post(handle_manual_adjustment))
}

async fn handle_punch(
    State(state): State<AppState>,
    Json(mut payload): Json<PunchRequest>,
) -> std::result::Result<Json<PunchResponse>, ApiError> {
    if payload.username.trim().is_empty() {
        if let Some(ref emb) = payload.face_embedding {
            if let Ok(Some(match_res)) = state.face_index.search_tiered(emb.clone()) {
                if match_res.is_match {
                    payload.username = match_res.username;
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
        rec_by_date.insert(r.date.clone(), r);
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
        "history": rows
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
