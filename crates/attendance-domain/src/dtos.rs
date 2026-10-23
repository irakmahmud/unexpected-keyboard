use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunchRequest {
    pub username: String,
    pub method: Option<String>, // 'CAMERA', 'QR', 'ANDROID_ID', 'MANUAL', 'WEB', 'RFID', 'PIN', 'FACE'
    pub pin: Option<String>,
    pub android_id: Option<String>,
    pub video_path: Option<String>,
    pub face_embedding: Option<Vec<f32>>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub device_id: Option<String>,
    pub remarks: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PunchResponse {
    pub success: bool,
    pub login_type: String, // "IN_TIME" or "OUT_TIME"
    pub username: String,
    pub date: String,
    pub timestamp: String,
    pub record_id: String,
    pub shift_status: Option<String>, // "ON_TIME", "LATE", "EARLY_OUT", "OVERTIME"
    pub message: String,
    pub late_minutes: Option<u32>,
    pub overtime_minutes: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPunchRequest {
    pub punches: Vec<PunchRequest>,
    pub device_id: String,
    pub batch_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPunchResponse {
    pub total: usize,
    pub successful: usize,
    pub failed: usize,
    pub results: Vec<PunchResponse>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub company_name: Option<String>,
    pub company_uuid: Option<String>,
    #[serde(alias = "designation")]
    pub user_designation: Option<String>,
    #[serde(alias = "android_id", alias = "device_id")]
    pub user_android_id: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub face_embedding: Option<Vec<f32>>,
    pub image_b64: Option<String>,
    pub images_b64: Option<Vec<String>>,
    pub overwrite_existing: Option<bool>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    #[serde(alias = "designation")]
    pub user_designation: Option<String>,
    #[serde(alias = "android_id", alias = "device_id")]
    pub user_android_id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub is_active: Option<bool>,
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangePasswordRequest {
    pub old_password: Option<String>,
    pub new_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: Option<String>,
    pub pin: Option<String>,
    pub client_info: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub username: String,
    pub role: String,
    pub expires_in_seconds: u64,
    pub company_id: Option<String>,
    pub full_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidIdRegistrationRequest {
    pub session_id: Option<String>,
    pub android_id: String,
    pub device_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttendanceSummaryResponse {
    pub date: String,
    pub total_employees: usize,
    pub present_today: usize,
    pub checked_in: usize,
    pub checked_out: usize,
    pub pending_out: usize,
    pub on_leave_today: usize,
    pub late_arrivals_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyLeaveRequest {
    pub employee_code: String,
    pub username: String,
    pub leave_type: String,
    pub start_date: String,
    pub end_date: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveDecisionRequest {
    pub status: String, // "APPROVED", "REJECTED"
    pub remarks: Option<String>,
    pub approved_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateShiftRequest {
    pub company_id: String,
    pub name: String,
    pub code: String,
    pub start_time: String,
    pub end_time: String,
    pub grace_period_minutes: u32,
    pub half_day_hours: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateShiftRequest {
    pub name: Option<String>,
    pub start_time: Option<String>,
    pub end_time: Option<String>,
    pub grace_period_minutes: Option<u32>,
    pub half_day_hours: Option<f64>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignShiftRequest {
    pub employee_code: String,
    pub shift_id: String,
    pub effective_from: String,
    pub effective_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanySetupRequest {
    pub company_id: Option<String>,
    pub company_name: String,
    pub password: String,
    pub address_road: Option<String>,
    pub address_city: Option<String>,
    pub address_state: Option<String>,
    pub address_postal_code: Option<String>,
    pub address_country: Option<String>,
    pub contact_person_name: Option<String>,
    pub contact_person_designation: Option<String>,
    pub contact_person_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceEnrollRequest {
    pub username: String,
    pub employee_code: Option<String>,
    pub embedding: Vec<f32>,
    pub pose: Option<String>, // "FRONTAL", "LEFT", "RIGHT", "UP", "DOWN"
    pub quality_score: Option<f32>,
    pub overwrite: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceEnrollResponse {
    pub success: bool,
    pub username: String,
    pub embeddings_count: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatchRequest {
    pub embedding: Vec<f32>,
    #[serde(default)]
    pub candidate_username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatchResponse {
    pub matched: bool,
    pub username: String,
    pub distance: f32,
    pub threshold: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceIdentifyRequest {
    pub embedding: Vec<f32>,
    pub top_k: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceIdentifyResponse {
    pub identified: bool,
    pub top_match: Option<FaceCandidateMatch>,
    pub all_candidates: Vec<FaceCandidateMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceCandidateMatch {
    pub username: String,
    pub distance: f32,
    pub confidence: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HolidayCreateRequest {
    pub company_id: String,
    pub name: String,
    pub holiday_date: String,
    pub holiday_type: Option<String>,
    pub description: Option<String>,
    pub is_recurring: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceRegisterRequest {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub ip_address: Option<String>,
    pub mac_address: Option<String>,
    pub location_tag: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceHeartbeatRequest {
    pub device_id: String,
    pub status: String,
    pub battery_level: Option<u8>,
    pub firmware_version: Option<String>,
    pub queue_size: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualAttendanceAdjustmentRequest {
    pub record_id: Option<String>,
    pub username: String,
    pub date: String,
    pub in_time: Option<String>,
    pub out_time: Option<String>,
    pub reason: String,
    pub approved_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyReportQuery {
    pub month: u32,
    pub year: i32,
    pub department: Option<String>,
    pub employee_code: Option<String>,
}
