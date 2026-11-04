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
}

impl Default for DesktopConfig {
    fn default() -> Self {
        Self {
            app_name: "Smart Attendance System — Native Rust Desktop".to_string(),
            version: "8.0.0-pure-rust".to_string(),
            cloud_server_url: "https://confident-officials-relatively-julie.trycloudflare.com".to_string(),
            local_server_url: "http://127.0.0.1:8080".to_string(),
            desktop_bridge_port: 18080,
            company_name: "Smart Attendance HQ".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDesktopUser {
    pub username: String,
    pub designation: String,
    pub pin_hash: String,
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
    pub synced_to_cloud: bool,
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
                    created_at: now.clone(),
                },
                LocalDesktopUser {
                    username: "RAHIM".to_string(),
                    designation: "Software Architect".to_string(),
                    pin_hash: sha256_hex("1234"),
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
        if let Ok(store) = serde_json::from_str::<LocalDesktopStore>(&txt) {
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
    let uname = username.trim().to_uppercase();
    if uname.is_empty() {
        return Err("Username is required".to_string());
    }
    let user_opt = store.users.iter().find(|u| u.username == uname);
    if let Some(u) = user_opt {
        let provided_pin = pin.map(|s| s.trim()).unwrap_or("");
        if provided_pin.is_empty() || sha256_hex(provided_pin) != u.pin_hash {
            return Err("Invalid PIN or password".to_string());
        }
    } else {
        let new_pin = pin.map(|s| s.trim()).filter(|s| !s.is_empty()).unwrap_or("1234");
        store.users.push(LocalDesktopUser {
            username: uname.clone(),
            designation: "Desktop User".to_string(),
            pin_hash: sha256_hex(new_pin),
            created_at: dhaka_datetime_str(),
        });
    }

    let today = dhaka_date_str();
    let now_time = dhaka_time_str();
    if let Some(rec) = store
        .records
        .iter_mut()
        .find(|r| r.username == uname && r.date == today)
    {
        rec.out_time = Some(now_time);
        rec.status = "CHECKED_OUT".to_string();
        return Ok(rec.clone());
    }

    let rec = LocalDesktopAttendance {
        record_id: format!("DESK_{}_{}", uname, today),
        username: uname,
        date: today,
        in_time: Some(now_time),
        out_time: None,
        status: "PRESENT".to_string(),
        synced_to_cloud: false,
    };
    store.records.insert(0, rec.clone());
    Ok(rec)
}

fn launch_native_desktop_window(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let app_arg = format!("--app={}", url);
        let edge_launched = Command::new("cmd")
            .args(["/C", "start", "", "msedge", &app_arg, "--window-size=1320,860"])
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
            let mut guard = store.lock().unwrap();
            match process_local_desktop_punch(&mut guard, uname, pin) {
                Ok(rec) => {
                    let _ = save_store(&guard);
                    let json = serde_json::json!({
                        "success": true,
                        "record": rec,
                        "message": format!("Desktop Attendance recorded for {}", rec.username)
                    });
                    ("200 OK", "application/json", json.to_string())
                }
                Err(e) => {
                    let json = serde_json::json!({ "success": false, "error": e });
                    ("400 Bad Request", "application/json", json.to_string())
                }
            }
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
            padding: 10px 18px;
            display: flex; justify-content: space-between; align-items: center; gap: 12px;
        }}
        .brand {{ display: flex; align-items: center; gap: 10px; font-weight: 800; color: #00f2fe; font-size: 1rem; }}
        .controls {{ display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }}
        .btn {{
            padding: 7px 13px; border-radius: 8px; border: 1px solid rgba(0,242,254,0.4);
            background: rgba(0, 242, 254, 0.14); color: #fff; font-weight: 700; font-size: 0.8rem; cursor: pointer;
        }}
        .btn.active {{ background: #00f2fe; color: #041421; }}
        .url-input {{
            background: #090d16; border: 1px solid rgba(255,255,255,0.2); color: #e2e8f0;
            padding: 6px 10px; border-radius: 8px; width: 320px; font-size: 0.8rem;
        }}
        .workspace {{ flex: 1; position: relative; display: flex; }}
        iframe {{ width: 100%; height: 100%; border: none; background: #090d16; }}
        #offline-panel {{
            display: none; width: 100%; height: 100%; padding: 28px; overflow-y: auto;
            background: radial-gradient(circle at top right, #1e1b4b, #070b14 70%);
        }}
        .grid {{ display: grid; grid-template-columns: 380px 1fr; gap: 22px; max-width: 1180px; margin: 0 auto; }}
        .card {{ background: rgba(15, 23, 42, 0.92); border: 1px solid rgba(0,242,254,0.25); border-radius: 14px; padding: 20px; }}
        .input {{ width: 100%; padding: 10px 12px; border-radius: 8px; background: #090d16; border: 1px solid #334155; color: #fff; margin-bottom: 12px; }}
        table {{ width: 100%; border-collapse: collapse; font-size: 0.86rem; margin-top: 10px; }}
        th, td {{ padding: 10px; border-bottom: 1px solid rgba(255,255,255,0.08); text-align: left; }}
        th {{ color: #00f2fe; }}
    </style>
</head>
<body>
    <div class="titlebar">
        <div class="brand">
            <span>⚡</span>
            <span>SMART ATTENDANCE DESKTOP (Rust Native • {os}/{arch})</span>
        </div>
        <div class="controls">
            <input type="text" id="server-url-input" class="url-input" value="{cloud_url}">
            <button class="btn active" id="btn-cloud" onclick="connectLiveServer()">🌐 Live Server View</button>
            <button class="btn" id="btn-local8080" onclick="connectLocal8080()">🖥️ Local :8080</button>
            <button class="btn" id="btn-offline" onclick="showOfflineDesktopKiosk()">🛡️ Built-in Offline PC Kiosk</button>
        </div>
    </div>
    <div class="workspace">
        <iframe id="portal-frame" src="{cloud_url}"></iframe>
        <div id="offline-panel">
            <div class="grid">
                <div class="card">
                    <h3 style="color:#00f2fe; margin-bottom:10px;">🛡️ Native Offline Desktop Kiosk</h3>
                    <p style="font-size:0.82rem; color:#94a3b8; margin-bottom:14px;">
                        ইন্টারনেট না থাকলেও আপনার পিসির এই Rust Desktop অ্যাপে সরাসরি উপস্থিতি রেকর্ড ও সংরক্ষণ করুন:
                    </p>
                    <label style="font-size:0.78rem; color:#cbd5e1;">USERNAME</label>
                    <input type="text" id="off-user" class="input" placeholder="e.g. RAHIM or ADMIN" value="RAHIM">
                    <label style="font-size:0.78rem; color:#cbd5e1;">PIN / PASSWORD</label>
                    <input type="password" id="off-pin" class="input" value="1234">
                    <button class="btn active" style="width:100%; padding:12px; font-size:0.92rem;" onclick="submitOfflinePunch()">
                        ✅ Record Desktop IN / OUT Punch
                    </button>
                    <div id="off-msg" style="margin-top:12px; font-weight:700; font-size:0.86rem;"></div>
                </div>
                <div class="card">
                    <h3 style="color:#00e676; margin-bottom:10px;">📋 Local PC Attendance Records (Stored on this Computer)</h3>
                    <table>
                        <thead>
                            <tr><th>Username</th><th>Date</th><th>In Time</th><th>Out Time</th><th>Status</th></tr>
                        </thead>
                        <tbody id="off-tbody"></tbody>
                    </table>
                </div>
            </div>
        </div>
    </div>
    <script>
        function connectLiveServer() {{
            const url = document.getElementById('server-url-input').value.trim();
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
            document.getElementById('server-url-input').value = '{local_url}';
            connectLiveServer();
        }}
        function showOfflineDesktopKiosk() {{
            document.getElementById('portal-frame').style.display = 'none';
            document.getElementById('offline-panel').style.display = 'block';
            refreshOfflineState();
        }}
        async function refreshOfflineState() {{
            const r = await fetch('/api/desktop/state');
            const d = await r.json();
            const tbody = document.getElementById('off-tbody');
            tbody.innerHTML = (d.records || []).map(rec => `
                <tr>
                    <td><b>${{rec.username}}</b></td>
                    <td>${{rec.date}}</td>
                    <td>${{rec.in_time || '---'}}</td>
                    <td>${{rec.out_time || '---'}}</td>
                    <td style="color:#00e676;">${{rec.status}}</td>
                </tr>
            `).join('') || '<tr><td colspan="5" style="color:#94a3b8;">No local offline punches yet.</td></tr>';
        }}
        async function submitOfflinePunch() {{
            const u = document.getElementById('off-user').value.trim();
            const p = document.getElementById('off-pin').value.trim();
            const msg = document.getElementById('off-msg');
            const r = await fetch('/api/desktop/punch', {{
                method: 'POST',
                body: JSON.stringify({{ username: u, pin: p }})
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
    </script>
</body>
</html>"##,
        os = env::consts::OS,
        arch = env::consts::ARCH,
        cloud_url = store.config.cloud_server_url,
        local_url = store.config.local_server_url
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.iter().any(|a| a == "--version" || a == "-v") {
        println!(
            "attendance-desktop v8.0.0 ({}/{}) — Pure Rust Cross-Platform Desktop Client",
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

    let desktop_url = format!("http://127.0.0.1:{}", active_port);
    println!("==========================================================================");
    println!("⚡ Smart Attendance System — Native Rust Desktop App ({}/{})", env::consts::OS, env::consts::ARCH);
    println!("Local Desktop Bridge: {}", desktop_url);
    println!("Cloud Server Target:  {}", store.lock().unwrap().config.cloud_server_url);
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
