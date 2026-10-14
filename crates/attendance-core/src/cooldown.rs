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
        Self {
            user_punches: Arc::new(Mutex::new(HashMap::new())),
            device_punches: Arc::new(Mutex::new(HashMap::new())),
            user_cooldown_secs,
            device_cooldown_secs,
        }
    }

    /// Check if user cooldown is active. If not, record punch time.
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
