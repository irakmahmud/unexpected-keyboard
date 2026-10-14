use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanyInfo {
    pub company_id: String,
    pub company_name: String,
    pub password: String,
    pub address_road: Option<String>,
    pub address_city: Option<String>,
    pub contact_person_name: Option<String>,
    pub contact_person_designation: Option<String>,
    pub contact_person_number: Option<String>,
    pub website: Option<String>,
    pub email: Option<String>,
    pub tax_id: Option<String>,
    pub created_at: Option<String>,
    pub last_updated: Option<String>,
}

pub type Company = CompanyInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Department {
    pub id: Option<i64>,
    pub company_id: String,
    pub name: String,
    pub code: String,
    pub description: Option<String>,
    pub head_employee_id: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Designation {
    pub id: Option<i64>,
    pub company_id: String,
    pub title: String,
    pub grade: Option<String>,
    pub department_id: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HrPassword {
    pub id: Option<i64>,
    pub company_uuid: String,
    pub company_name: String,
    pub password: String,
    pub created_at: Option<String>,
    pub last_updated: Option<String>,
}
