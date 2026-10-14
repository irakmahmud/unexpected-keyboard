use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LoginType {
    InTime,
    OutTime,
}

impl std::fmt::Display for LoginType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoginType::InTime => write!(f, "IN_TIME"),
            LoginType::OutTime => write!(f, "OUT_TIME"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttendanceStatus {
    Present,
    Late,
    LateAbsent,
    HalfDay,
    EarlyDeparture,
    Absent,
    OnLeave,
    Holiday,
    Weekend,
}

impl Default for AttendanceStatus {
    fn default() -> Self {
        AttendanceStatus::Present
    }
}

impl std::fmt::Display for AttendanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttendanceStatus::Present => write!(f, "PRESENT"),
            AttendanceStatus::Late => write!(f, "LATE"),
            AttendanceStatus::LateAbsent => write!(f, "LATE_ABSENT"),
            AttendanceStatus::HalfDay => write!(f, "HALF_DAY"),
            AttendanceStatus::EarlyDeparture => write!(f, "EARLY_DEPARTURE"),
            AttendanceStatus::Absent => write!(f, "ABSENT"),
            AttendanceStatus::OnLeave => write!(f, "ON_LEAVE"),
            AttendanceStatus::Holiday => write!(f, "HOLIDAY"),
            AttendanceStatus::Weekend => write!(f, "WEEKEND"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PunchMethod {
    Camera,
    Qr,
    AndroidId,
    Rfid,
    Pin,
    Manual,
    Web,
    BiometricFingerprint,
}

impl Default for PunchMethod {
    fn default() -> Self {
        PunchMethod::Camera
    }
}

impl std::fmt::Display for PunchMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PunchMethod::Camera => write!(f, "CAMERA"),
            PunchMethod::Qr => write!(f, "QR"),
            PunchMethod::AndroidId => write!(f, "ANDROID_ID"),
            PunchMethod::Rfid => write!(f, "RFID"),
            PunchMethod::Pin => write!(f, "PIN"),
            PunchMethod::Manual => write!(f, "MANUAL"),
            PunchMethod::Web => write!(f, "WEB"),
            PunchMethod::BiometricFingerprint => write!(f, "BIOMETRIC"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceRecord {
    pub record_id: String,
    pub company_user_uuid: Option<String>,
    pub username: String,
    pub date: String, // YYYY-MM-DD
    pub in_time: Option<String>, // HH:MM:SS
    pub out_time: Option<String>, // HH:MM:SS
    pub video_path_in: Option<String>,
    pub video_path_out: Option<String>,
    pub in_time_with_camera: Option<String>, // 'Y' or 'N'
    pub out_time_with_camera: Option<String>,
    pub in_time_with_qr: Option<String>,
    pub out_time_with_qr: Option<String>,
    pub status: AttendanceStatus,
    pub work_hours: f64,
    pub overtime_hours: f64,
    pub late_minutes: u32,
    pub early_exit_minutes: u32,
    pub ip_address: Option<String>,
    pub device_id: Option<String>,
    pub remarks: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunchLog {
    pub id: Option<i64>,
    pub username: String,
    pub punch_time: String,
    pub punch_type: LoginType,
    pub punch_method: PunchMethod,
    pub device_id: Option<String>,
    pub ip_address: Option<String>,
    pub liveness_score: Option<f32>,
    pub verified_by_face: bool,
    pub created_at: Option<String>,
}
