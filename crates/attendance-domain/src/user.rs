use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserRole {
    SuperAdmin,
    CompanyAdmin,
    HrManager,
    DepartmentHead,
    Employee,
    SecurityGuard,
    KioskTerminal,
}

impl Default for UserRole {
    fn default() -> Self {
        UserRole::Employee
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    Terminated,
    PendingApproval,
}

impl Default for UserStatus {
    fn default() -> Self {
        UserStatus::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub company_user_uuid: String,
    pub company_name: Option<String>,
    pub company_uuid: Option<String>,
    pub username: String,
    pub user_uuid: Option<String>,
    pub user_designation: Option<String>,
    pub user_android_id: Option<String>,
    pub face_encoding: Option<String>, // JSON-encoded or base64 512-dim float vector
    #[serde(default)]
    pub register_with_camera: Option<String>,
    pub password: Option<String>,
    pub role: UserRole,
    pub status: UserStatus,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub created_at: Option<String>,
    pub last_updated: Option<String>,
}

impl User {
    pub fn new(username: &str, company_user_uuid: &str) -> Self {
        Self {
            company_user_uuid: company_user_uuid.to_string(),
            company_name: None,
            company_uuid: None,
            username: username.to_uppercase(),
            user_uuid: Some(uuid::Uuid::new_v4().to_string()),
            user_designation: None,
            user_android_id: None,
            face_encoding: None,
            register_with_camera: Some("N".to_string()),
            password: None,
            role: UserRole::Employee,
            status: UserStatus::Active,
            email: None,
            phone: None,
            created_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            last_updated: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }

    pub fn is_active(&self) -> bool {
        self.status == UserStatus::Active
    }
}
