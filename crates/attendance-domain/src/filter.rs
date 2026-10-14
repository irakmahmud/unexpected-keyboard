use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u64>,
    pub page_size: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>, // "asc" or "desc"
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            page_size: Some(50),
            sort_by: Some("id".to_string()),
            sort_order: Some("desc".to_string()),
        }
    }
}

impl PaginationParams {
    pub fn offset(&self) -> u64 {
        let p = self.page.unwrap_or(1).max(1);
        let ps = self.page_size.unwrap_or(50).min(1000);
        (p - 1) * ps
    }

    pub fn limit(&self) -> u64 {
        self.page_size.unwrap_or(50).min(1000)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttendanceFilter {
    pub employee_code: Option<String>,
    pub username: Option<String>,
    pub department: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
    pub shift_status: Option<String>,
    pub punch_method: Option<String>,
    pub is_late: Option<bool>,
    pub has_overtime: Option<bool>,
    pub device_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmployeeFilter {
    pub department: Option<String>,
    pub designation: Option<String>,
    pub is_active: Option<bool>,
    pub search_query: Option<String>,
    pub company_id: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuditLogFilter {
    pub actor_username: Option<String>,
    pub action: Option<String>,
    pub target_entity: Option<String>,
    pub from_date: Option<String>,
    pub to_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u64,
    pub page_size: u64,
    pub total_pages: u64,
}

impl<T> PaginatedResponse<T> {
    pub fn new(items: Vec<T>, total_count: u64, page: u64, page_size: u64) -> Self {
        let total_pages = if page_size == 0 {
            0
        } else {
            (total_count + page_size - 1) / page_size
        };
        Self {
            items,
            total_count,
            page,
            page_size,
            total_pages,
        }
    }
}
