use crate::{ApiError, AppState};
use attendance_common::{AppError, TimeHelper};
use attendance_domain::*;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct MobileProtectedCursorQuery {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub wifi_ssid: Option<String>,
    pub wifi_bssid: Option<String>,
    pub shard_bucket: Option<i32>,
    pub search: Option<String>,
    pub after_username: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize)]
pub struct MobileProtectedAccessQuery {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub wifi_ssid: Option<String>,
    pub wifi_bssid: Option<String>,
}

pub fn mobile_router() -> Router<AppState> {
    Router::new()
        .route("/policy", get(get_mobile_policy).post(update_mobile_policy))
        .route("/verify-access", post(verify_mobile_access_status))
        .route("/enforce-access", post(enforce_mobile_access_strict))
        .route("/users-cursor", get(get_mobile_sharded_users_cursor))
        .route("/bulk-provision", post(handle_mobile_bulk_provision))
        .route("/scale-dashboard", get(get_mobile_scale_dashboard))
}

pub fn downloads_router() -> Router<AppState> {
    Router::new()
        .route("/info", get(get_downloads_info))
        .route("/:artifact", get(download_platform_artifact))
}

async fn get_mobile_policy(
    State(state): State<AppState>,
) -> std::result::Result<Json<MobileAccessPolicy>, ApiError> {
    let policy = state.db.get_mobile_access_policy()?;
    Ok(Json(policy))
}

async fn update_mobile_policy(
    State(state): State<AppState>,
    Json(payload): Json<UpdateMobileAccessPolicyRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let updated = state.db.update_mobile_access_policy(&payload)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Mobile Mandatory Location & Wi-Fi Policy updated successfully (Location Mandatory is permanently enforced)",
        "policy": updated
    })))
}

async fn verify_mobile_access_status(
    State(state): State<AppState>,
    Json(payload): Json<MobileLocationWifiCheckRequest>,
) -> std::result::Result<Json<MobileLocationWifiCheckResponse>, ApiError> {
    let res = state.db.evaluate_mobile_location_and_wifi(&payload)?;
    Ok(Json(res))
}

async fn enforce_mobile_access_strict(
    State(state): State<AppState>,
    Json(payload): Json<MobileLocationWifiCheckRequest>,
) -> std::result::Result<Json<MobileLocationWifiCheckResponse>, ApiError> {
    let res = state.db.verify_mobile_access_strict(&payload)?;
    Ok(Json(res))
}

async fn get_mobile_sharded_users_cursor(
    State(state): State<AppState>,
    Query(q): Query<MobileProtectedCursorQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    // Strictly enforce mandatory GPS Location (and Wi-Fi if enabled) before allowing Mobile User Management
    let access = state.db.verify_mobile_access_strict(&MobileLocationWifiCheckRequest {
        latitude: q.latitude,
        longitude: q.longitude,
        gps_accuracy_meters: None,
        wifi_ssid: q.wifi_ssid.clone(),
        wifi_bssid: q.wifi_bssid.clone(),
        device_id: Some("MOBILE_ADMIN_CONSOLE".to_string()),
    })?;

    let mut data = state.db.get_cursor_paginated_users(
        q.shard_bucket,
        q.search.as_deref(),
        q.after_username.as_deref(),
        q.limit.unwrap_or(50),
    )?;

    if let Some(obj) = data.as_object_mut() {
        obj.insert(
            "mobile_security_verification".to_string(),
            serde_json::to_value(access).unwrap_or(serde_json::Value::Null),
        );
    }

    Ok(Json(data))
}

async fn handle_mobile_bulk_provision(
    State(state): State<AppState>,
    Json(payload): Json<MobileBulkProvisionRequest>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    // Strictly enforce mandatory GPS Location (and Wi-Fi if enabled)
    let access = state.db.verify_mobile_access_strict(&MobileLocationWifiCheckRequest {
        latitude: payload.latitude,
        longitude: payload.longitude,
        gps_accuracy_meters: None,
        wifi_ssid: payload.wifi_ssid.clone(),
        wifi_bssid: payload.wifi_bssid.clone(),
        device_id: Some("MOBILE_BULK_PROVISIONER".to_string()),
    })?;

    let company_id = payload.company_id.as_deref().unwrap_or("HQ_001");
    let prefix = payload.prefix.as_deref().unwrap_or("MOB_STAFF");
    let desig = payload
        .designation
        .as_deref()
        .unwrap_or("Mobile Managed Staff");

    let mut res = state
        .db
        .bulk_provision_sharded_users(company_id, prefix, payload.count, desig)?;

    if let Some(obj) = res.as_object_mut() {
        obj.insert(
            "mobile_security_verification".to_string(),
            serde_json::to_value(access).unwrap_or(serde_json::Value::Null),
        );
    }

    Ok(Json(res))
}

async fn get_mobile_scale_dashboard(
    State(state): State<AppState>,
    Query(q): Query<MobileProtectedAccessQuery>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let access = state.db.verify_mobile_access_strict(&MobileLocationWifiCheckRequest {
        latitude: q.latitude,
        longitude: q.longitude,
        gps_accuracy_meters: None,
        wifi_ssid: q.wifi_ssid.clone(),
        wifi_bssid: q.wifi_bssid.clone(),
        device_id: Some("MOBILE_SCALE_DASHBOARD".to_string()),
    })?;

    let today = TimeHelper::current_date_dhaka();
    let summary = state.reporting.get_daily_summary(Some(&today))?;
    let scale = state.db.get_scale_architecture_status()?;

    Ok(Json(serde_json::json!({
        "success": true,
        "mobile_security_verification": access,
        "daily_summary": summary,
        "scale_architecture": scale
    })))
}

fn resolve_downloads_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("dist/downloads"),
        PathBuf::from("deploy/staging/downloads"),
        PathBuf::from("/home/opc/attendance_system/dist/downloads"),
    ];
    for c in &candidates {
        if c.exists() {
            return c.clone();
        }
    }
    PathBuf::from("dist/downloads")
}

fn artifact_metadata(filename: &str, platform: &str, title: &str, arch: &str, download_slug: &str, install_guide: &str) -> serde_json::Value {
    let dir = resolve_downloads_dir();
    let path = dir.join(filename);
    let (exists, size_bytes, sha256_hex) = if let Ok(bytes) = std::fs::read(&path) {
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = format!("{:x}", hasher.finalize());
        (true, bytes.len(), hash)
    } else {
        (false, 0, String::new())
    };

    serde_json::json!({
        "platform": platform,
        "title": title,
        "filename": filename,
        "architecture": arch,
        "ready": exists,
        "size_bytes": size_bytes,
        "size_human": if size_bytes > 1024 * 1024 {
            format!("{:.2} MB", (size_bytes as f64) / (1024.0 * 1024.0))
        } else if size_bytes > 1024 {
            format!("{:.1} KB", (size_bytes as f64) / 1024.0)
        } else {
            format!("{} B", size_bytes)
        },
        "sha256": sha256_hex,
        "download_url": format!("/api/v1/downloads/{}", download_slug),
        "install_guide": install_guide
    })
}

async fn get_downloads_info() -> Json<serde_json::Value> {
    let artifacts = vec![
        artifact_metadata(
            "SmartAttendance-Desktop-Windows-x64-Setup.exe",
            "Windows Desktop Installer (.exe)",
            "Smart Attendance Rust Desktop — Windows 10/11 (64-bit) One-Click Setup Installer",
            "x86_64-pc-windows-gnu (PE32+ Native Rust GUI/Desktop + NSIS Installer)",
            "windows-installer",
            "ডাউনলোড করে ডাবল-ক্লিক করুন। এটি স্বয়ংক্রিয়ভাবে Desktop ও Start Menu শর্টকাট তৈরি করবে এবং অ্যাপ চালু করবে।"
        ),
        artifact_metadata(
            "SmartAttendance-Desktop-Windows-x64-Portable.zip",
            "Windows Portable Desktop (.zip)",
            "Smart Attendance Rust Desktop — Windows 10/11 (64-bit) Portable Edition",
            "x86_64-pc-windows-gnu (attendance-desktop.exe + Launcher)",
            "windows-portable",
            "ZIP ফাইলটি Extract করে attendance-desktop.exe অথবা Install-Windows-Desktop.bat এ ডাবল-ক্লিক করুন।"
        ),
        artifact_metadata(
            "SmartAttendance-Desktop-macOS-Universal.zip",
            "macOS Desktop App (.app Bundle)",
            "Smart Attendance Rust Desktop — macOS Universal (Apple Silicon M1/M2/M3/M4 & Intel)",
            "macOS Universal (.app Bundle + Install-macOS.command + Rust Desktop Source)",
            "macos-desktop",
            "ZIP আনজিপ করে Install-macOS.command অথবা SmartAttendanceDesktop.app ওপেন করুন।"
        ),
        artifact_metadata(
            "SmartAttendance-Desktop-Linux-x86_64.tar.gz",
            "Linux Desktop & Server (.tar.gz)",
            "Smart Attendance Rust Desktop — Linux x86_64 (Ubuntu/Debian/Fedora/Oracle/Arch)",
            "x86_64-unknown-linux-gnu (ELF 64-bit Native Rust Desktop + Server Suite)",
            "linux-desktop",
            "টার্মিনালে tar -xzf SmartAttendance-Desktop-Linux-x86_64.tar.gz চালিয়ে ./install-linux-desktop.sh অথবা ./attendance-desktop চালান।"
        ),
        artifact_metadata(
            "SmartAttendance-Mobile-Android.apk",
            "Android Mobile App (.apk)",
            "Smart Attendance Android App — Millions-of-Users Mobile Admin & Kiosk (Mandatory GPS Location + Wi-Fi Option)",
            "Android Universal APK (V1+V2 Signed DEX + Native LocationManager & WifiManager Bridge)",
            "android-apk",
            "অ্যান্ড্রয়েড ফোনে APK ডাউনলোড করে Install করুন। ফোনের GPS Location অবশ্যই চালু থাকতে হবে (লোকেশন ছাড়া কাজ করবে না)।"
        ),
    ];

    Json(serde_json::json!({
        "success": true,
        "version": "8.0.0-pure-rust",
        "mobile_web_app_url": "/mobile",
        "mandatory_mobile_location": true,
        "wifi_option_supported": true,
        "artifacts": artifacts
    }))
}

async fn download_platform_artifact(
    Path(artifact): Path<String>,
) -> std::result::Result<Response, ApiError> {
    let (filename, content_type) = match artifact.as_str() {
        "windows-installer" | "SmartAttendance-Desktop-Windows-x64-Setup.exe" => (
            "SmartAttendance-Desktop-Windows-x64-Setup.exe",
            "application/vnd.microsoft.portable-executable",
        ),
        "windows-portable" | "SmartAttendance-Desktop-Windows-x64-Portable.zip" => (
            "SmartAttendance-Desktop-Windows-x64-Portable.zip",
            "application/zip",
        ),
        "macos-desktop" | "SmartAttendance-Desktop-macOS-Universal.zip" => (
            "SmartAttendance-Desktop-macOS-Universal.zip",
            "application/zip",
        ),
        "linux-desktop" | "SmartAttendance-Desktop-Linux-x86_64.tar.gz" => (
            "SmartAttendance-Desktop-Linux-x86_64.tar.gz",
            "application/gzip",
        ),
        "android-apk" | "SmartAttendance-Mobile-Android.apk" => (
            "SmartAttendance-Mobile-Android.apk",
            "application/vnd.android.package-archive",
        ),
        _ => {
            return Err(AppError::NotFound(format!(
                "Requested download artifact '{}' not found",
                artifact
            ))
            .into());
        }
    };

    let path = resolve_downloads_dir().join(filename);
    let bytes = std::fs::read(&path).map_err(|_| {
        AppError::NotFound(format!(
            "Artifact file '{}' is not yet staged in dist/downloads",
            filename
        ))
    })?;

    let disposition = format!("attachment; filename=\"{}\"", filename);
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type.to_string()),
            (header::CONTENT_DISPOSITION, disposition),
            (header::CACHE_CONTROL, "no-cache".to_string()),
        ],
        bytes,
    )
        .into_response())
}
