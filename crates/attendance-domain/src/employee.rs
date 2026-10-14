use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EmploymentType {
    Permanent,
    Contractual,
    Probationary,
    PartTime,
    Intern,
}

impl Default for EmploymentType {
    fn default() -> Self {
        EmploymentType::Permanent
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Employee {
    pub id: Option<i64>,
    pub employee_code: String, // e.g. EMP001
    pub user_uuid: String,
    pub company_id: String,
    pub first_name: String,
    pub last_name: String,
    pub full_name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: Option<i64>,
    pub designation_id: Option<i64>,
    pub shift_id: Option<i64>,
    pub rfid_card_uid: Option<String>,
    pub pin_code: Option<String>,
    pub employment_type: EmploymentType,
    pub join_date: String,
    pub end_date: Option<String>,
    pub is_active: bool,
    pub avatar_url: Option<String>,
    pub emergency_contact_name: Option<String>,
    pub emergency_contact_phone: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

impl Employee {
    pub fn new(code: &str, first_name: &str, last_name: &str, company_id: &str, join_date: &str) -> Self {
        let full_name = format!("{} {}", first_name, last_name).trim().to_string();
        Self {
            id: None,
            employee_code: code.to_uppercase(),
            user_uuid: uuid::Uuid::new_v4().to_string(),
            company_id: company_id.to_string(),
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            full_name,
            email: None,
            phone: None,
            department_id: None,
            designation_id: None,
            shift_id: None,
            rfid_card_uid: None,
            pin_code: None,
            employment_type: EmploymentType::Permanent,
            join_date: join_date.to_string(),
            end_date: None,
            is_active: true,
            avatar_url: None,
            emergency_contact_name: None,
            emergency_contact_phone: None,
            created_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
            updated_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
