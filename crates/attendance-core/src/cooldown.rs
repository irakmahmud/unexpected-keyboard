use attendance_common::{AppError, Result};
use chrono::{DateTime, Utc};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;

pub const DEFAULT_USER_COOLDOWN_SECONDS: i64 = 60;
pub const DEFAULT_DEVICE_COOLDOWN_SECONDS: i64 = 5;

#[derive(Clone)]
pub struct CooldownManager {
    user_punches: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    device_punches: Arc<Mutex<HashMap<String, DateTime<Utc>>>>,
    user_cooldown_secs: i64,
    device_cooldown_secs: i64,
}

impl Default for CooldownManager {
    fn default() -> Self {
        Self::new(DEFAULT_USER_COOLDOWN_SECONDS, DEFAULT_DEVICE_COOLDOWN_SECONDS)
    }
}

impl CooldownManager {
    pub fn new(user_cooldown_secs: i64, device_cooldown_secs: i64) -> Self {
        let mut initial_map = HashMap::new();
        if user_cooldown_secs == DEFAULT_USER_COOLDOWN_SECONDS
            && device_cooldown_secs == DEFAULT_DEVICE_COOLDOWN_SECONDS
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/opc".to_string());
            let paths = [
                std::path::PathBuf::from(&home).join(".last_attendance"),
                std::path::PathBuf::from("data").join(".last_attendance"),
            ];
            for p in &paths {
                if let Ok(content) = std::fs::read_to_string(p) {
                    let trimmed = content.trim();
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if let Some(users_obj) = json_val.get("users").and_then(|v| v.as_object()) {
                            for (u, v) in users_obj {
                                if let Some(ts) = v.as_i64() {
                                    if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
                                        initial_map.insert(u.trim().to_uppercase(), dt);
                                    }
                                }
                            }
                        }
                        if let Some(last_str) = json_val.get("last_attendance").and_then(|v| v.as_str()) {
                            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(last_str, "%Y-%m-%d %H:%M:%S") {
                                let dt = DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc);
                                initial_map.insert("__GLOBAL__".to_string(), dt);
                            }
                        }
                    } else {
                        for line in trimmed.lines() {
                            let line = line.trim();
                            if let Some((user, ts_str)) = line.split_once(':') {
                                if let Ok(ts) = ts_str.trim().parse::<i64>() {
                                    if let Some(dt) = DateTime::<Utc>::from_timestamp(ts, 0) {
                                        initial_map.insert(user.trim().to_uppercase(), dt);
                                    }
                                }
                            } else if let Ok(ts) = line.parse::<f64>() {
                                if let Some(dt) = DateTime::<Utc>::from_timestamp(ts as i64, 0) {
                                    initial_map.insert("__GLOBAL__".to_string(), dt);
                                }
                            }
                        }
                    }
                }
            }
        }
        Self {
            user_punches: Arc::new(Mutex::new(initial_map)),
            device_punches: Arc::new(Mutex::new(HashMap::new())),
            user_cooldown_secs,
            device_cooldown_secs,
        }
    }

    fn persist_user_punches(map: &HashMap<String, DateTime<Utc>>) {
        let mut users_json = serde_json::Map::new();
        let mut latest_dt: Option<DateTime<Utc>> = None;
        for (u, dt) in map.iter() {
            users_json.insert(u.clone(), serde_json::Value::Number(dt.timestamp().into()));
            if latest_dt.map_or(true, |prev| *dt > prev) {
                latest_dt = Some(*dt);
            }
        }
        let last_att_str = latest_dt
            .unwrap_or_else(Utc::now)
            .with_timezone(&attendance_common::TimeHelper::dhaka_offset())
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let payload = serde_json::json!({
            "last_attendance": last_att_str,
            "pid": std::process::id(),
            "users": users_json,
        });
        let out = serde_json::to_string(&payload).unwrap_or_default();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/opc".to_string());
        let paths = [
            std::path::PathBuf::from(&home).join(".last_attendance"),
            std::path::PathBuf::from("data").join(".last_attendance"),
        ];
        for p in &paths {
            if let Some(parent) = p.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(p, out.as_bytes());
        }
    }

    /// Check if user cooldown is active. If not, record punch time and persist to ~/.last_attendance.
    pub fn check_user_cooldown(&self, username: &str) -> Result<()> {
        let upper = username.to_uppercase();
        let now = Utc::now();
        let mut map = self.user_punches.lock();

        if let Some(last_time) = map.get(&upper) {
            let elapsed = (now - *last_time).num_seconds();
            if elapsed < self.user_cooldown_secs {
                let remaining = self.user_cooldown_secs - elapsed;
                return Err(AppError::Cooldown(format!(
                    "Cooldown active for user {}. Please wait {} more seconds before next punch.",
                    upper, remaining
                )));
            }
        }

        map.insert(upper, now);
        Self::persist_user_punches(&map);
        Ok(())
    }

    /// Check if device cooldown is active (prevents hardware button bounce / double scan)
    pub fn check_device_cooldown(&self, device_id: &str) -> Result<()> {
        let now = Utc::now();
        let mut map = self.device_punches.lock();

        if let Some(last_time) = map.get(device_id) {
            let elapsed = (now - *last_time).num_seconds();
            if elapsed < self.device_cooldown_secs {
                return Err(AppError::Cooldown(format!(
                    "Device cooldown: please wait {} seconds.",
                    self.device_cooldown_secs - elapsed
                )));
            }
        }

        map.insert(device_id.to_string(), now);
        Ok(())
    }
}
