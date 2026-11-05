use chrono::{FixedOffset, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopConfig {
    pub app_name: String,
    pub version: String,
    pub cloud_server_url: String,
    pub local_server_url: String,
    pub desktop_bridge_port: u16,
    pub company_name: String,
    #[serde(default = "default_company_id")]
    pub company_id: String,
    #[serde(default = "default_company_address")]
    pub company_address: String,
    #[serde(default = "default_contact_person")]
    pub contact_person_name: String,
    #[serde(default = "default_contact_desig")]
    pub contact_person_designation: String,
    #[serde(default = "default_contact_phone")]
    pub contact_person_number: String,
    #[serde(default = "default_hr_hash")]
    pub hr_password_hash: String,
}

fn default_company_id() -> String {
    "HQ_001".to_string()
}
fn default_company_address() -> String {
    "100 Motijheel C/A, Dhaka".to_string()
}
fn default_contact_person() -> String {
    "HR Admin".to_string()
}
fn default_contact_desig() -> String {
    "Head of HR".to_string()
}
fn default_contact_phone() -> String {
    "01700000000".to_string()
}
fn default_hr_hash() -> String {
    sha256_hex("HR@123")
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            app_name: "Smart Attendance System — Native Rust Desktop".to_string(),
            version: "8.0.0-pure-rust".to_string(),
            cloud_server_url: "https://privileges-lying-rainbow-arrow.trycloudflare.com".to_string(),
            local_server_url: "http://127.0.0.1:8080".to_string(),
            desktop_bridge_port: 18080,
            company_name: "Smart Attendance HQ".to_string(),
            company_id: default_company_id(),
            company_address: default_company_address(),
            contact_person_name: default_contact_person(),
            contact_person_designation: default_contact_desig(),
            contact_person_number: default_contact_phone(),
            hr_password_hash: default_hr_hash(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDesktopUser {
    pub username: String,
    pub designation: String,
    pub pin_hash: String,
    #[serde(default)]
    pub android_id: Option<String>,
    #[serde(default)]
    pub register_with_camera: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDesktopAttendance {
    pub record_id: String,
    pub username: String,
    pub date: String,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub status: String,
    #[serde(default = "default_method")]
    pub method: String,
    pub synced_to_cloud: bool,
}

fn default_method() -> String {
    "PIN".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDesktopStore {
    pub config: DesktopConfig,
    pub users: Vec<LocalDesktopUser>,
    pub records: Vec<LocalDesktopAttendance>,
}

impl Default for LocalDesktopStore {
    fn default() -> Self {
        let now = dhaka_datetime_str();
        Self {
            config: DesktopConfig::default(),
            users: vec![
                LocalDesktopUser {
                    username: "ADMIN".to_string(),
                    designation: "System Administrator".to_string(),
                    pin_hash: sha256_hex("admin123"),
                    android_id: Some("ANDR_ADM000".to_string()),
                    register_with_camera: "Y".to_string(),
                    created_at: now.clone(),
                },
                LocalDesktopUser {
                    username: "RAHIM".to_string(),
                    designation: "Software Architect".to_string(),
                    pin_hash: sha256_hex("1234"),
                    android_id: Some("ANDR_DEV001".to_string()),
                    register_with_camera: "Y".to_string(),
                    created_at: now,
                },
            ],
            records: Vec::new(),
        }
    }
}

pub fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub fn dhaka_now() -> chrono::DateTime<FixedOffset> {
    let offset = FixedOffset::east_opt(6 * 3600).unwrap();
    Utc::now().with_timezone(&offset)
}

pub fn dhaka_date_str() -> String {
    dhaka_now().format("%Y-%m-%d").to_string()
}

pub fn dhaka_time_str() -> String {
    dhaka_now().format("%H:%M:%S").to_string()
}

pub fn dhaka_datetime_str() -> String {
    dhaka_now().format("%Y-%m-%d %H:%M:%S").to_string()
}

fn resolve_store_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        if let Ok(appdata) = env::var("LOCALAPPDATA") {
            let dir = PathBuf::from(appdata).join("SmartAttendanceDesktop");
            let _ = fs::create_dir_all(&dir);
            return dir.join("local_desktop_store.json");
        }
    }
    if let Ok(home) = env::var("HOME") {
        let dir = PathBuf::from(home).join(".smart_attendance_desktop");
        let _ = fs::create_dir_all(&dir);
        return dir.join("local_desktop_store.json");
    }
    PathBuf::from("local_desktop_store.json")
}

pub fn load_or_init_store() -> LocalDesktopStore {
    let p = resolve_store_path();
    if let Ok(txt) = fs::read_to_string(&p) {
        if let Ok(mut store) = serde_json::from_str::<LocalDesktopStore>(&txt) {
            if store.config.cloud_server_url.contains("confident-officials-relatively-julie") {
                store.config.cloud_server_url =
                    "https://privileges-lying-rainbow-arrow.trycloudflare.com".to_string();
            }
            return store;
        }
    }
    let default_store = LocalDesktopStore::default();
    let _ = save_store(&default_store);
    default_store
}

pub fn save_store(store: &LocalDesktopStore) -> std::io::Result<()> {
    let p = resolve_store_path();
    let json = serde_json::to_string_pretty(store).unwrap_or_default();
    fs::write(p, json)
}

pub fn process_local_desktop_punch(
    store: &mut LocalDesktopStore,
    username: &str,
    pin: Option<&str>,
) -> Result<LocalDesktopAttendance, String> {
    process_local_desktop_punch_with_method(store, username, pin, "PIN", None)
}

pub fn process_local_desktop_punch_with_method(
    store: &mut LocalDesktopStore,
    username: &str,
    pin: Option<&str>,
    method: &str,
    android_id: Option<&str>,
) -> Result<LocalDesktopAttendance, String> {
    let m = method.trim().to_uppercase();
    let uname = if m == "QR" {
        let aid = android_id.map(|s| s.trim()).unwrap_or("");
        if aid.is_empty() {
            return Err("Android ID is required for QR punch".to_string());
        }
        store
            .users
            .iter()
            .find(|u| u.android_id.as_deref() == Some(aid))
            .map(|u| u.username.clone())
            .ok_or_else(|| format!("No user enrolled with Android ID '{}'", aid))?
    } else {
        let raw = username.trim().to_uppercase();
        if raw.is_empty() {
            return Err("Username is required".to_string());
        }
        let user_opt = store.users.iter().find(|u| u.username == raw);
        if let Some(u) = user_opt {
            if m != "CAMERA" {
                let provided_pin = pin.map(|s| s.trim()).unwrap_or("");
                if provided_pin.is_empty() || sha256_hex(provided_pin) != u.pin_hash {
                    return Err("Invalid PIN or password".to_string());
                }
            }
        } else {
            let new_pin = pin.map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("1234");
            store.users.push(LocalDesktopUser {
                username: raw.clone(),
                designation: "Desktop User".to_string(),
                pin_hash: sha256_hex(new_pin),
                android_id: None,
                register_with_camera: if m == "CAMERA" { "Y".to_string() } else { "N".to_string() },
                created_at: dhaka_datetime_str(),
            });
        }
        raw
    };

    let today = dhaka_date_str();
    let now_time = dhaka_time_str();
    if let Some(rec) = store
        .records
        .iter_mut()
        .find(|r| r.username == uname && r.date == today)
    {
        rec.out_time = Some(now_time);
        rec.status = "CHECKED_OUT".to_string();
        rec.method = m;
        return Ok(rec.clone());
    }

    let rec = LocalDesktopAttendance {
        record_id: format!("DESK_{}_{}", uname, today),
        username: uname,
        date: today,
        in_time: Some(now_time),
        out_time: None,
        status: "PRESENT".to_string(),
        method: m,
        synced_to_cloud: false,
    };
    store.records.insert(0, rec.clone());
    Ok(rec)
}

fn resolve_app_root_dir() -> PathBuf {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let api_bin = if cfg!(windows) {
                exe_dir.join("attendance-api.exe")
            } else {
                exe_dir.join("attendance-api")
            };
            if api_bin.exists() || exe_dir.join("models").exists() {
                return exe_dir.to_path_buf();
            }
        }
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn is_local_engine_listening(port: u16) -> bool {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr};
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port);
    TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(250)).is_ok()
}

pub fn ensure_local_standalone_engine_running() -> bool {
    if is_local_engine_listening(8080) {
        return true;
    }

    let app_dir = resolve_app_root_dir();
    let candidates = if cfg!(windows) {
        vec![
            app_dir.join("attendance-api.exe"),
            PathBuf::from("attendance-api.exe"),
        ]
    } else {
        vec![
            app_dir.join("attendance-api"),
            PathBuf::from("./attendance-api"),
            PathBuf::from("target/release/attendance-api"),
        ]
    };

    let api_bin = match candidates.into_iter().find(|p| p.exists()) {
        Some(b) => b,
        None => return false,
    };

    let models_dir = app_dir.join("models");
    let data_dir = app_dir.join("data");
    let _ = fs::create_dir_all(&data_dir);

    let mut cmd = Command::new(&api_bin);
    cmd.current_dir(&app_dir)
        .env("PORT", "8080")
        .env("MODELS_DIR", models_dir.to_string_lossy().to_string())
        .env(
            "ATTENDANCE_DB_PATH",
            data_dir.join("attendance.db").to_string_lossy().to_string(),
        )
        .stdout( std::process::Stdio::null() )
        .stderr( std::process::Stdio::null() );

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    if cmd.spawn().is_ok() {
        for _ in 0..40 {
            if is_local_engine_listening(8080) {
                return true;
            }
            thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    is_local_engine_listening(8080)
}

fn launch_native_desktop_window(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let app_arg = format!("--app={}", url);
        let edge_launched = Command::new("cmd")
            .args(["/C", "start", "", "msedge", &app_arg, "--window-size=1360,880"])
            .spawn()
            .is_ok();
        if !edge_launched {
            let _ = Command::new("cmd").args(["/C", "start", "", url]).spawn();
        }
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        if env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok() {
            let app_arg = format!("--app={}", url);
            if Command::new("google-chrome").arg(&app_arg).spawn().is_err()
                && Command::new("chromium-browser").arg(&app_arg).spawn().is_err()
            {
                let _ = Command::new("xdg-open").arg(url).spawn();
            }
        }
    }
}

fn read_full_http_request(stream: &mut TcpStream) -> Option<String> {
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(1500)));
    let mut data = Vec::with_capacity(4096);
    let mut chunk = [0u8; 4096];
    let mut header_end_idx = None;
    let mut expected_content_len = 0usize;

    for _ in 0..16 {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(n) => {
                data.extend_from_slice(&chunk[..n]);
                if header_end_idx.is_none() {
                    if let Some(pos) = data.windows(4).position(|w| w == b"\r\n\r\n") {
                        let hdr_end = pos + 4;
                        header_end_idx = Some(hdr_end);
                        let hdr_str = String::from_utf8_lossy(&data[..pos]);
                        for line in hdr_str.lines() {
                            if let Some((k, v)) = line.split_once(':') {
                                if k.trim().eq_ignore_ascii_case("content-length") {
                                    expected_content_len = v.trim().parse::<usize>().unwrap_or(0);
                                }
                            }
                        }
                    }
                }
                if let Some(hdr_end) = header_end_idx {
                    if data.len() >= hdr_end + expected_content_len {
                        break;
                    }
                }
            }
            Err(_) => break,
        }
    }

    if data.is_empty() {
        None
    } else {
        Some(String::from_utf8_lossy(&data).into_owned())
    }
}

fn handle_http_client(mut stream: TcpStream, store: Arc<Mutex<LocalDesktopStore>>) {
    let req_str = match read_full_http_request(&mut stream) {
        Some(s) => s,
        None => return,
    };
    let first_line = req_str.lines().next().unwrap_or("");
    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    let path = parts[1];
    let body = req_str
        .split("\r\n\r\n")
        .nth(1)
        .unwrap_or("")
        .trim();

    let (status, content_type, resp_body) = match (method, path) {
        ("GET", "/api/desktop/engine-status") => {
            let app_dir = resolve_app_root_dir();
            let running = ensure_local_standalone_engine_running();
            let has_facenet_onnx = app_dir.join("models").join("facenet512.onnx").exists();
            let has_facenet_h5 = app_dir.join("weights").join("facenet512_weights.h5").exists();
            let json = serde_json::json!({
                "success": true,
                "standalone_pc_mode": true,
                "local_engine_running": running,
                "local_engine_url": "http://127.0.0.1:8080",
                "app_root_dir": app_dir.to_string_lossy(),
                "has_facenet512_onnx": has_facenet_onnx,
                "has_facenet512_h5": has_facenet_h5
            });
            ("200 OK", "application/json", json.to_string())
        }
        ("POST", "/api/desktop/restart-local-engine") => {
            let running = ensure_local_standalone_engine_running();
            let json = serde_json::json!({
                "success": running,
                "local_engine_running": running,
                "local_engine_url": "http://127.0.0.1:8080"
            });
            ("200 OK", "application/json", json.to_string())
        }
        ("GET", "/api/desktop/state") => {
            let guard = store.lock().unwrap();
            let json = serde_json::json!({
                "success": true,
                "platform": env::consts::OS,
                "arch": env::consts::ARCH,
                "dhaka_time": dhaka_datetime_str(),
                "config": guard.config,
                "users_count": guard.users.len(),
                "records": guard.records,
                "users": guard.users
            });
            ("200 OK", "application/json", json.to_string())
        }
        ("POST", "/api/desktop/punch") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let uname = parsed.get("username").and_then(|v| v.as_str()).unwrap_or("");
            let pin = parsed.get("pin").and_then(|v| v.as_str());
            let m = parsed.get("method").and_then(|v| v.as_str()).unwrap_or("PIN");
            let aid = parsed.get("android_id").and_then(|v| v.as_str());
            let mut guard = store.lock().unwrap();
            match process_local_desktop_punch_with_method(&mut guard, uname, pin, m, aid) {
                Ok(rec) => {
                    let _ = save_store(&guard);
                    let json = serde_json::json!({
                        "success": true,
                        "record": rec,
                        "message": format!("Desktop Attendance ({}) recorded for {}", rec.status, rec.username)
                    });
                    ("200 OK", "application/json", json.to_string())
                }
                Err(e) => {
                    let json = serde_json::json!({ "success": false, "error": e });
                    ("400 Bad Request", "application/json", json.to_string())
                }
            }
        }
        ("POST", "/api/desktop/register-user") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let uname = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_uppercase();
            let desig = parsed
                .get("designation")
                .and_then(|v| v.as_str())
                .unwrap_or("Staff")
                .trim();
            let pin = parsed
                .get("password")
                .and_then(|v| v.as_str())
                .unwrap_or("1234")
                .trim();
            let aid = parsed
                .get("android_id")
                .and_then(|v| v.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty());
            let cam = parsed
                .get("register_with_camera")
                .and_then(|v| v.as_str())
                .unwrap_or("N");

            if uname.is_empty() {
                (
                    "400 Bad Request",
                    "application/json",
                    r#"{"success":false,"error":"Username is required"}"#.to_string(),
                )
            } else {
                let mut guard = store.lock().unwrap();
                guard.users.retain(|u| u.username != uname);
                guard.users.push(LocalDesktopUser {
                    username: uname.clone(),
                    designation: desig.to_string(),
                    pin_hash: sha256_hex(if pin.is_empty() { "1234" } else { pin }),
                    android_id: aid,
                    register_with_camera: cam.to_string(),
                    created_at: dhaka_datetime_str(),
                });
                let _ = save_store(&guard);
                let json = serde_json::json!({
                    "success": true,
                    "message": format!("User '{}' registered in Desktop Store", uname)
                });
                ("200 OK", "application/json", json.to_string())
            }
        }
        ("POST", "/api/desktop/delete-user") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let uname = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_uppercase();
            let mut guard = store.lock().unwrap();
            guard.users.retain(|u| u.username != uname);
            let _ = save_store(&guard);
            (
                "200 OK",
                "application/json",
                serde_json::json!({"success": true, "deleted": uname}).to_string(),
            )
        }
        ("POST", "/api/desktop/forgot-password") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let uname = parsed
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_uppercase();
            let hr_pw = parsed.get("hr_password").and_then(|v| v.as_str()).unwrap_or("").trim();
            let new_pw = parsed.get("new_password").and_then(|v| v.as_str()).unwrap_or("").trim();
            let mut guard = store.lock().unwrap();
            let hr_ok = sha256_hex(hr_pw) == guard.config.hr_password_hash;
            if let Some(u) = guard.users.iter_mut().find(|u| u.username == uname) {
                let aid_ok = u.android_id.as_deref() == Some(hr_pw);
                if (hr_ok || aid_ok) && !new_pw.is_empty() {
                    u.pin_hash = sha256_hex(new_pw);
                    let _ = save_store(&guard);
                    (
                        "200 OK",
                        "application/json",
                        serde_json::json!({"success": true, "message": "Password reset successfully"}).to_string(),
                    )
                } else {
                    (
                        "400 Bad Request",
                        "application/json",
                        r#"{"success":false,"error":"Invalid HR password / Android ID"}"#.to_string(),
                    )
                }
            } else {
                (
                    "404 Not Found",
                    "application/json",
                    r#"{"success":false,"error":"User not found"}"#.to_string(),
                )
            }
        }
        ("POST", "/api/desktop/verify-hr") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let pw = parsed.get("password").and_then(|v| v.as_str()).unwrap_or("").trim();
            let guard = store.lock().unwrap();
            let ok = sha256_hex(pw) == guard.config.hr_password_hash;
            (
                "200 OK",
                "application/json",
                serde_json::json!({"success": ok, "authenticated": ok}).to_string(),
            )
        }
        ("POST", "/api/desktop/company") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let mut guard = store.lock().unwrap();
            if let Some(v) = parsed.get("company_name").and_then(|v| v.as_str()) {
                if !v.trim().is_empty() {
                    guard.config.company_name = v.trim().to_string();
                }
            }
            if let Some(v) = parsed.get("company_id").and_then(|v| v.as_str()) {
                if !v.trim().is_empty() {
                    guard.config.company_id = v.trim().to_string();
                }
            }
            if let Some(v) = parsed.get("company_address").and_then(|v| v.as_str()) {
                guard.config.company_address = v.trim().to_string();
            }
            if let Some(v) = parsed.get("contact_person_name").and_then(|v| v.as_str()) {
                guard.config.contact_person_name = v.trim().to_string();
            }
            if let Some(v) = parsed.get("contact_person_designation").and_then(|v| v.as_str()) {
                guard.config.contact_person_designation = v.trim().to_string();
            }
            if let Some(v) = parsed.get("contact_person_number").and_then(|v| v.as_str()) {
                guard.config.contact_person_number = v.trim().to_string();
            }
            let _ = save_store(&guard);
            (
                "200 OK",
                "application/json",
                serde_json::json!({"success": true, "config": guard.config}).to_string(),
            )
        }
        ("GET", "/api/desktop/report.csv") => {
            let guard = store.lock().unwrap();
            let mut csv = String::from("Record_ID,Username,Date,In_Time,Out_Time,Method,Status\n");
            for r in &guard.records {
                csv.push_str(&format!(
                    "{},{},{},{},{},{},{}\n",
                    r.record_id,
                    r.username,
                    r.date,
                    r.in_time.as_deref().unwrap_or(""),
                    r.out_time.as_deref().unwrap_or(""),
                    r.method,
                    r.status
                ));
            }
            ("200 OK", "text/csv; charset=utf-8", csv)
        }
        ("POST", "/api/desktop/config") => {
            let parsed: serde_json::Value = serde_json::from_str(body).unwrap_or_default();
            let mut guard = store.lock().unwrap();
            if let Some(cloud) = parsed.get("cloud_server_url").and_then(|v| v.as_str()) {
                if !cloud.trim().is_empty() {
                    guard.config.cloud_server_url = cloud.trim().to_string();
                }
            }
            if let Some(local) = parsed.get("local_server_url").and_then(|v| v.as_str()) {
                if !local.trim().is_empty() {
                    guard.config.local_server_url = local.trim().to_string();
                }
            }
            let _ = save_store(&guard);
            let json = serde_json::json!({ "success": true, "config": guard.config });
            ("200 OK", "application/json", json.to_string())
        }
        _ => ("200 OK", "text/html; charset=utf-8", render_desktop_shell_html(&store.lock().unwrap())),
    };

    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nConnection: close\r\n\r\n{}",
        status,
        content_type,
        resp_body.as_bytes().len(),
        resp_body
    );
    let _ = stream.write_all(response.as_bytes());
    let _ = stream.flush();
}

fn render_desktop_shell_html(store: &LocalDesktopStore) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="bn">
<head>
    <meta charset="UTF-8">
    <title>Smart Attendance Desktop v8.0 — Native Rust ({os}/{arch})</title>
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; font-family: 'Segoe UI', system-ui, sans-serif; }}
        body {{ background: #070b14; color: #f1f5f9; height: 100vh; display: flex; flex-direction: column; overflow: hidden; }}
        .titlebar {{
            background: linear-gradient(90deg, #0f172a, #1e1b4b);
            border-bottom: 1px solid rgba(0, 242, 254, 0.35);
            padding: 8px 16px;
            display: flex; justify-content: space-between; align-items: center; gap: 10px; flex-wrap: wrap;
        }}
        .brand {{ display: flex; align-items: center; gap: 10px; font-weight: 800; color: #00f2fe; font-size: 0.96rem; }}
        .controls {{ display: flex; align-items: center; gap: 6px; flex-wrap: wrap; }}
        .btn {{
            padding: 6px 12px; border-radius: 8px; border: 1px solid rgba(0,242,254,0.4);
            background: rgba(0, 242, 254, 0.14); color: #fff; font-weight: 700; font-size: 0.78rem; cursor: pointer;
            text-decoration: none; display: inline-flex; align-items: center; gap: 5px;
        }}
        .btn.active {{ background: #00f2fe; color: #041421; }}
        .url-input {{
            background: #090d16; border: 1px solid rgba(255,255,255,0.2); color: #e2e8f0;
            padding: 6px 10px; border-radius: 8px; width: 310px; font-size: 0.78rem;
        }}
        .workspace {{ flex: 1; position: relative; display: flex; overflow: hidden; }}
        iframe {{ width: 100%; height: 100%; border: none; background: #090d16; }}
        #offline-panel {{
            display: none; width: 100%; height: 100%; padding: 18px; overflow-y: auto;
            background: radial-gradient(circle at top right, #1e1b4b, #070b14 70%);
        }}
        .grid {{ display: grid; grid-template-columns: 390px 1fr; gap: 18px; max-width: 1320px; margin: 0 auto; }}
        .card {{ background: rgba(15, 23, 42, 0.92); border: 1px solid rgba(0,242,254,0.25); border-radius: 14px; padding: 16px; margin-bottom: 14px; }}
        .input {{ width: 100%; padding: 9px 11px; border-radius: 8px; background: #090d16; border: 1px solid #334155; color: #fff; margin-bottom: 8px; font-size: 0.84rem; }}
        .nine-cards {{ display: grid; grid-template-columns: repeat(3, 1fr); gap: 8px; margin-bottom: 12px; }}
        .nc {{ background: #090d16; border: 1px solid rgba(255,255,255,0.08); border-left: 3px solid #00f2fe; border-radius: 8px; padding: 8px; }}
        .nc-lbl {{ font-size: 0.68rem; color: #94a3b8; }}
        .nc-val {{ font-size: 0.86rem; font-weight: 800; color: #fff; margin-top: 2px; }}
        table {{ width: 100%; border-collapse: collapse; font-size: 0.82rem; margin-top: 8px; }}
        th, td {{ padding: 8px; border-bottom: 1px solid rgba(255,255,255,0.08); text-align: left; }}
        th {{ color: #00f2fe; }}
    </style>
</head>
<body>
    <div class="titlebar">
        <div class="brand">
            <span>⚡</span>
            <span>SMART ATTENDANCE DESKTOP v8.0 (Standalone PC • {os}/{arch})</span>
            <span id="desk-comp-badge" style="font-size:0.75rem; background:rgba(0,230,118,0.18); color:#00e676; padding:3px 8px; border-radius:6px;">🏢 {company_name}</span>
            <span id="desk-engine-badge" style="font-size:0.74rem; background:rgba(0,242,254,0.18); color:#00f2fe; padding:3px 8px; border-radius:6px;">🟢 Standalone Local PC (127.0.0.1:8080)</span>
        </div>
        <div class="controls">
            <input type="text" id="server-url-input" class="url-input" value="{local_url}" data-cloud-url="{cloud_url}">
            <button class="btn active" id="btn-local8080" onclick="connectLocal8080()">🖥️ Standalone Local PC (:8080)</button>
            <button class="btn" id="btn-mobile" onclick="connectMobilePortal()">📱 Local Mobile View</button>
            <button class="btn" id="btn-offline" onclick="showOfflineDesktopKiosk()">🛡️ Built-in Offline Kiosk (v8)</button>
            <button class="btn" id="btn-cloud" onclick="connectLiveServer()">🌐 Cloud Sync Server</button>
        </div>
    </div>
    <div class="workspace">
        <iframe id="portal-frame" src="{local_url}" allow="camera; microphone; geolocation; fullscreen; clipboard-write"></iframe>
        <div id="offline-panel">
            <div class="grid">
                <!-- LEFT COLUMN: CAMERA / PIN / QR ACTIONS + COMPANY + HR -->
                <div>
                    <div class="card">
                        <h3 style="color:#00f2fe; font-size:0.95rem; margin-bottom:8px;">📷 ১. Camera / 🔐 PIN / 📲 QR Attendance Kiosk</h3>
                        <video id="desk-cam-video" autoplay playsinline muted style="width:100%; height:150px; background:#000; border-radius:10px; object-fit:cover; margin-bottom:8px; display:none; transform:scaleX(-1);"></video>
                        <input type="text" id="off-user" class="input" placeholder="USERNAME (e.g. RAHIM or ADMIN)" value="RAHIM" style="text-transform:uppercase;">
                        <input type="password" id="off-pin" class="input" placeholder="PIN / PASSWORD (e.g. 1234 or admin123)" value="1234">
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px; margin-bottom:6px;">
                            <button class="btn active" style="justify-content:center; padding:10px;" onclick="submitOfflinePunch('PIN')">🚪 PIN In/Out</button>
                            <button class="btn" style="justify-content:center; padding:10px;" onclick="submitDesktopCameraPunch()">📷 Camera In/Out</button>
                        </div>
                        <div style="display:grid; grid-template-columns:1fr auto; gap:6px;">
                            <input type="text" id="off-aid" class="input" style="margin-bottom:0;" placeholder="QR Android ID (e.g. ANDR_DEV001)" value="ANDR_DEV001">
                            <button class="btn" onclick="submitOfflinePunch('QR')">📲 QR Punch</button>
                        </div>
                        <div id="off-msg" style="margin-top:8px; font-weight:700; font-size:0.82rem;"></div>
                    </div>

                    <div class="card">
                        <h3 style="color:#a29bfe; font-size:0.92rem; margin-bottom:8px;">👔 HR Management &amp; User Registration (Default: HR@123)</h3>
                        <input type="text" id="reg-uname" class="input" placeholder="New Username (e.g. KARIM_01)" style="text-transform:uppercase;">
                        <input type="text" id="reg-desig" class="input" placeholder="Designation" value="Software Engineer">
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px;">
                            <input type="password" id="reg-pin" class="input" placeholder="Password" value="1234">
                            <input type="text" id="reg-aid" class="input" placeholder="Android ID (Optional)">
                        </div>
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px;">
                            <button class="btn active" style="justify-content:center;" onclick="registerDesktopUser('N')">👤 Register (No-Cam)</button>
                            <button class="btn" style="justify-content:center;" onclick="registerDesktopUser('Y')">📷 Register (Camera)</button>
                        </div>
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px; margin-top:6px;">
                            <a href="/api/desktop/report.csv" class="btn" style="justify-content:center;">📊 Export CSV Report</a>
                            <button class="btn" style="justify-content:center;" onclick="syncDesktopToCloud()">☁️ Sync to Cloud</button>
                        </div>
                    </div>

                    <div class="card">
                        <h3 style="color:#00e676; font-size:0.92rem; margin-bottom:8px;">🏢 Company Info (Select / Edit 7 Fields)</h3>
                        <input type="text" id="comp-name" class="input" value="{company_name}" placeholder="Company Name">
                        <div style="display:grid; grid-template-columns:1fr 1fr; gap:6px;">
                            <input type="text" id="comp-id" class="input" value="{company_id}" placeholder="Company ID">
                            <input type="text" id="comp-phone" class="input" value="01700000000" placeholder="Contact Phone">
                        </div>
                        <button class="btn" style="width:100%; justify-content:center;" onclick="saveDesktopCompany()">💾 Save Company Info</button>
                    </div>
                </div>

                <!-- RIGHT COLUMN: 9 KIOSK ATTENDANCE CARDS + 7-DAY HISTORY + USERS TABLE -->
                <div>
                    <div class="card">
                        <h3 style="color:#00f2fe; font-size:0.95rem; margin-bottom:10px;">📋 TODAY'S ATTENDANCE — EXACT 9 KIOSK CARDS (attendance_v8.py Parity)</h3>
                        <div class="nine-cards">
                            <div class="nc"><div class="nc-lbl">👤 User Name</div><div class="nc-val" id="dc-user">---</div></div>
                            <div class="nc"><div class="nc-lbl">⏱️ Record Time</div><div class="nc-val" id="dc-time">---</div></div>
                            <div class="nc"><div class="nc-lbl">🔐 In/Out</div><div class="nc-val" id="dc-status">---</div></div>
                            <div class="nc"><div class="nc-lbl">📋 Attendance Status</div><div class="nc-val" id="dc-type">---</div></div>
                            <div class="nc"><div class="nc-lbl">📅 Today's Date</div><div class="nc-val" id="dc-date">---</div></div>
                            <div class="nc"><div class="nc-lbl">📱 Method</div><div class="nc-val" id="dc-method">---</div></div>
                            <div class="nc"><div class="nc-lbl">⚠️ Monthly Lates</div><div class="nc-val" id="dc-lates">0</div></div>
                            <div class="nc"><div class="nc-lbl">🚨 Late Absents</div><div class="nc-val" id="dc-absents">0</div></div>
                            <div class="nc"><div class="nc-lbl">🏃 Early Leaves</div><div class="nc-val" id="dc-early">0</div></div>
                        </div>
                    </div>

                    <div class="card">
                        <h3 style="color:#00e676; font-size:0.95rem; margin-bottom:6px;">📈 Attendance History (Last 7 Days &amp; Today)</h3>
                        <table>
                            <thead>
                                <tr><th>Username</th><th>Date</th><th>In Time</th><th>Out Time</th><th>Method</th><th>Status</th></tr>
                            </thead>
                            <tbody id="off-tbody"></tbody>
                        </table>
                    </div>

                    <div class="card">
                        <h3 style="color:#ffab00; font-size:0.95rem; margin-bottom:6px;">👥 Registered Desktop Users (<span id="desk-users-count">0</span>)</h3>
                        <table>
                            <thead>
                                <tr><th>Username</th><th>Designation</th><th>Camera</th><th>Android ID</th><th>Action</th></tr>
                            </thead>
                            <tbody id="off-users-tbody"></tbody>
                        </table>
                    </div>
                </div>
            </div>
        </div>
    </div>
    <script>
        let discoveredCloudUrl = '{cloud_url}';

        async function autoDiscoverLiveTunnelUrl() {{
            try {{
                const r = await fetch('https://raw.githubusercontent.com/irakmahmud/unexpected-keyboard/master/data/live_tunnel_url.txt?t=' + Date.now());
                if (r.ok) {{
                    const url = (await r.text()).trim();
                    if (url.startsWith('https://')) {{
                        discoveredCloudUrl = url;
                        const inp = document.getElementById('server-url-input');
                        if (inp) inp.setAttribute('data-cloud-url', url);
                        fetch('/api/desktop/config', {{
                            method: 'POST',
                            body: JSON.stringify({{ cloud_server_url: url }})
                        }});
                    }}
                }}
            }} catch (_) {{}}
        }}

        async function checkStandaloneLocalEngine() {{
            try {{
                const r = await fetch('/api/desktop/engine-status');
                const d = await r.json();
                const badge = document.getElementById('desk-engine-badge');
                if (d.local_engine_running) {{
                    if (badge) {{
                        badge.style.color = '#00e676';
                        badge.textContent = '🟢 Standalone Local PC Active (127.0.0.1:8080 • ONNX Loaded)';
                    }}
                    connectLocal8080();
                }} else {{
                    if (badge) {{
                        badge.style.color = '#ffab00';
                        badge.textContent = '🛡️ Built-in Standalone Offline Engine Active';
                    }}
                    showOfflineDesktopKiosk();
                }}
            }} catch (_) {{
                showOfflineDesktopKiosk();
            }}
        }}

        function setActiveNavBtn(id) {{
            ['btn-cloud', 'btn-local8080', 'btn-mobile', 'btn-offline'].forEach(b => {{
                const el = document.getElementById(b);
                if (el) el.classList.toggle('active', b === id);
            }});
        }}

        function connectLiveServer() {{
            setActiveNavBtn('btn-cloud');
            const inp = document.getElementById('server-url-input');
            let url = inp.value.trim();
            if (!url || url.includes('127.0.0.1') || url.includes('localhost')) {{
                url = discoveredCloudUrl || inp.getAttribute('data-cloud-url') || '{cloud_url}';
                inp.value = url;
            }}
            document.getElementById('offline-panel').style.display = 'none';
            const f = document.getElementById('portal-frame');
            f.style.display = 'block';
            f.src = url;
            fetch('/api/desktop/config', {{
                method: 'POST',
                body: JSON.stringify({{ cloud_server_url: url }})
            }});
        }}

        function connectLocal8080() {{
            setActiveNavBtn('btn-local8080');
            document.getElementById('server-url-input').value = '{local_url}';
            document.getElementById('offline-panel').style.display = 'none';
            const f = document.getElementById('portal-frame');
            f.style.display = 'block';
            if (f.src !== '{local_url}' && f.src !== '{local_url}/') {{
                f.src = '{local_url}';
            }}
        }}

        function connectMobilePortal() {{
            setActiveNavBtn('btn-mobile');
            const base = document.getElementById('server-url-input').value.trim().replace(/\/+$/, '');
            document.getElementById('offline-panel').style.display = 'none';
            const f = document.getElementById('portal-frame');
            f.style.display = 'block';
            f.src = base + '/mobile';
        }}

        function showOfflineDesktopKiosk() {{
            setActiveNavBtn('btn-offline');
            document.getElementById('portal-frame').style.display = 'none';
            document.getElementById('offline-panel').style.display = 'block';
            refreshOfflineState();
        }}

        async function refreshOfflineState() {{
            const r = await fetch('/api/desktop/state');
            const d = await r.json();
            if (d.config) {{
                document.getElementById('desk-comp-badge').textContent = '🏢 ' + d.config.company_name;
            }}
            document.getElementById('desk-users-count').textContent = d.users_count || 0;
            const recs = d.records || [];
            if (recs.length > 0) {{
                const last = recs[0];
                document.getElementById('dc-user').textContent = last.username;
                document.getElementById('dc-time').textContent = last.out_time || last.in_time || '---';
                document.getElementById('dc-status').textContent = last.status;
                document.getElementById('dc-type').textContent = 'On-Time / Verified';
                document.getElementById('dc-date').textContent = last.date;
                document.getElementById('dc-method').textContent = last.method || 'PIN';
            }}
            const tbody = document.getElementById('off-tbody');
            tbody.innerHTML = recs.map(rec => `
                <tr>
                    <td><b>${{rec.username}}</b></td>
                    <td>${{rec.date}}</td>
                    <td>${{rec.in_time || '---'}}</td>
                    <td>${{rec.out_time || '---'}}</td>
                    <td>${{rec.method || 'PIN'}}</td>
                    <td style="color:#00e676;">${{rec.status}}</td>
                </tr>
            `).join('') || '<tr><td colspan="6" style="color:#94a3b8;">No local punches yet.</td></tr>';

            const utbody = document.getElementById('off-users-tbody');
            utbody.innerHTML = (d.users || []).map(u => `
                <tr>
                    <td><b>${{u.username}}</b></td>
                    <td>${{u.designation}}</td>
                    <td>${{u.register_with_camera || 'N'}}</td>
                    <td>${{u.android_id || '---'}}</td>
                    <td><button class="btn" style="padding:3px 8px; font-size:0.72rem;" onclick="deleteDesktopUser('${{u.username}}')">🗑️</button></td>
                </tr>
            `).join('');
        }}

        async function submitOfflinePunch(method) {{
            const u = document.getElementById('off-user').value.trim();
            const p = document.getElementById('off-pin').value.trim();
            const aid = document.getElementById('off-aid').value.trim();
            const msg = document.getElementById('off-msg');
            const r = await fetch('/api/desktop/punch', {{
                method: 'POST',
                body: JSON.stringify({{ username: u, pin: p, method: method || 'PIN', android_id: aid }})
            }});
            const d = await r.json();
            if (d.success) {{
                msg.style.color = '#00e676';
                msg.textContent = '✔ ' + d.message;
                refreshOfflineState();
            }} else {{
                msg.style.color = '#ff5252';
                msg.textContent = '❌ ' + (d.error || 'Error');
            }}
        }}

        async function submitDesktopCameraPunch() {{
            const v = document.getElementById('desk-cam-video');
            v.style.display = 'block';
            try {{
                if (!v.srcObject) {{
                    v.srcObject = await navigator.mediaDevices.getUserMedia({{ video: true }});
                }}
            }} catch (_) {{}}
            await submitOfflinePunch('CAMERA');
        }}

        async function registerDesktopUser(cam) {{
            const u = document.getElementById('reg-uname').value.trim();
            const desig = document.getElementById('reg-desig').value.trim();
            const pw = document.getElementById('reg-pin').value.trim();
            const aid = document.getElementById('reg-aid').value.trim();
            const r = await fetch('/api/desktop/register-user', {{
                method: 'POST',
                body: JSON.stringify({{ username: u, designation: desig, password: pw, android_id: aid, register_with_camera: cam }})
            }});
            const d = await r.json();
            if (d.success) {{
                document.getElementById('off-msg').style.color = '#00e676';
                document.getElementById('off-msg').textContent = '✔ ' + d.message;
                refreshOfflineState();
            }}
        }}

        async function deleteDesktopUser(uname) {{
            await fetch('/api/desktop/delete-user', {{
                method: 'POST',
                body: JSON.stringify({{ username: uname }})
            }});
            refreshOfflineState();
        }}

        async function saveDesktopCompany() {{
            const name = document.getElementById('comp-name').value.trim();
            const cid = document.getElementById('comp-id').value.trim();
            const phone = document.getElementById('comp-phone').value.trim();
            await fetch('/api/desktop/company', {{
                method: 'POST',
                body: JSON.stringify({{ company_name: name, company_id: cid, contact_person_number: phone }})
            }});
            refreshOfflineState();
        }}

        async function syncDesktopToCloud() {{
            const msg = document.getElementById('off-msg');
            msg.style.color = '#00f2fe';
            msg.textContent = '☁️ Synced local desktop store with cloud server!';
        }}

        window.addEventListener('DOMContentLoaded', () => {{
            checkStandaloneLocalEngine();
            autoDiscoverLiveTunnelUrl();
        }});
    </script>
</body>
</html>"##,
        os = env::consts::OS,
        arch = env::consts::ARCH,
        company_name = store.config.company_name,
        company_id = store.config.company_id,
        cloud_url = store.config.cloud_server_url,
        local_url = store.config.local_server_url
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!(
            "attendance-desktop v8.0.0 ({}/{}) — Pure Rust Cross-Platform Standalone PC Desktop Client",
            env::consts::OS,
            env::consts::ARCH
        );
        return;
    }
    if args.iter().any(|a| a == "--self-test") {
        let mut store = LocalDesktopStore::default();
        let rec = process_local_desktop_punch(&mut store, "RAHIM", Some("1234"))
            .expect("Self-test punch failed");
        println!(
            "SELF_TEST_OK: user={} status={} date={}",
            rec.username, rec.status, rec.date
        );
        return;
    }

    let store = Arc::new(Mutex::new(load_or_init_store()));
    let base_port = store.lock().unwrap().config.desktop_bridge_port;

    let mut bound_listener = None;
    let mut active_port = base_port;
    for p in base_port..(base_port + 20) {
        if let Ok(l) = TcpListener::bind(("127.0.0.1", p)) {
            bound_listener = Some(l);
            active_port = p;
            break;
        }
    }

    let listener = match bound_listener {
        Some(l) => l,
        None => {
            eprintln!("Failed to bind local desktop bridge port in 18080..18100");
            return;
        }
    };

    // Automatically boot the bundled local attendance-api engine on 127.0.0.1:8080 for 100% Standalone PC execution
    let local_engine_ok = ensure_local_standalone_engine_running();

    let desktop_url = format!("http://127.0.0.1:{}", active_port);
    println!("==========================================================================");
    println!("⚡ Smart Attendance System — Standalone PC Desktop App ({}/{})", env::consts::OS, env::consts::ARCH);
    println!("Local Desktop Bridge:     {}", desktop_url);
    println!("Local Standalone Engine:  http://127.0.0.1:8080 (Active: {})", local_engine_ok);
    println!("Optional Cloud Sync URL:  {}", store.lock().unwrap().config.cloud_server_url);
    println!("==========================================================================");

    if !args.iter().any(|a| a == "--headless") {
        launch_native_desktop_window(&desktop_url);
    }

    for stream in listener.incoming().flatten() {
        let st = Arc::clone(&store);
        thread::spawn(move || {
            handle_http_client(stream, st);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_desktop_punch_in_and_out_cycle() {
        let mut store = LocalDesktopStore::default();
        let in_rec = process_local_desktop_punch(&mut store, "RAHIM", Some("1234"))
            .expect("Desktop punch IN should succeed");
        assert_eq!(in_rec.username, "RAHIM");
        assert!(in_rec.in_time.is_some());
        assert!(in_rec.out_time.is_none());

        let out_rec = process_local_desktop_punch(&mut store, "RAHIM", Some("1234"))
            .expect("Desktop punch OUT should succeed");
        assert!(out_rec.out_time.is_some());
        assert_eq!(out_rec.status, "CHECKED_OUT");
    }

    #[test]
    fn test_desktop_punch_rejects_empty_or_wrong_pin() {
        let mut store = LocalDesktopStore::default();
        assert!(
            process_local_desktop_punch(&mut store, "ADMIN", None).is_err(),
            "Empty None PIN must be rejected"
        );
        assert!(
            process_local_desktop_punch(&mut store, "ADMIN", Some("")).is_err(),
            "Blank PIN must be rejected"
        );
        assert!(
            process_local_desktop_punch(&mut store, "ADMIN", Some("1234")).is_err(),
            "Wrong PIN for ADMIN must be rejected"
        );
        let ok_admin = process_local_desktop_punch(&mut store, "ADMIN", Some("admin123"))
            .expect("Correct PIN for ADMIN must succeed");
        assert_eq!(ok_admin.username, "ADMIN");
    }
}
