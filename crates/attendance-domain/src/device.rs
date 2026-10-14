use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeviceStatus {
    Online,
    Offline,
    Standby,
    Maintenance,
    Unregistered,
}

impl Default for DeviceStatus {
    fn default() -> Self {
        DeviceStatus::Online
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSession {
    pub session_id: String,
    pub created_at: String,
    pub registered_android_id: Option<String>,
    pub status: String, // "PENDING", "CONNECTED", "EXPIRED"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisteredDevice {
    pub device_id: String,
    pub device_name: String,
    pub android_id: Option<String>,
    pub mac_address: Option<String>,
    pub ip_address: Option<String>,
    pub device_type: String, // "KIOSK", "MOBILE", "TURNSTILE", "TABLET"
    pub location_name: Option<String>,
    pub status: DeviceStatus,
    pub last_heartbeat: Option<String>,
    pub app_version: Option<String>,
    pub created_at: Option<String>,
}
