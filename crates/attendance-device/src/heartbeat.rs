use attendance_domain::DeviceStatus;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct HeartbeatMonitor {
    // device_id -> last_seen_instant
    heartbeats: Arc<RwLock<HashMap<String, Instant>>>,
    timeout_duration: Duration,
}

pub type DeviceRegistry = HeartbeatMonitor;
pub type DeviceHeartbeatTracker = HeartbeatMonitor;

impl Default for HeartbeatMonitor {
    fn default() -> Self {
        Self::new(Duration::from_secs(120))
    }
}

impl HeartbeatMonitor {
    pub fn new(timeout_duration: Duration) -> Self {
        Self {
            heartbeats: Arc::new(RwLock::new(HashMap::new())),
            timeout_duration,
        }
    }

    pub fn record_heartbeat(&self, device_id: &str) {
        self.heartbeats.write().insert(device_id.to_string(), Instant::now());
    }

    pub fn get_device_status(&self, device_id: &str) -> DeviceStatus {
        let map = self.heartbeats.read();
        if let Some(last) = map.get(device_id) {
            if last.elapsed() < self.timeout_duration {
                DeviceStatus::Online
            } else {
                DeviceStatus::Offline
            }
        } else {
            DeviceStatus::Unregistered
        }
    }
}
