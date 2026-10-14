use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeaveType {
    Casual,
    Sick,
    AnnualEarned,
    Maternity,
    Paternity,
    Bereavement,
    CompensatoryOff,
    UnpaidLeave,
}

impl Default for LeaveType {
    fn default() -> Self {
        LeaveType::Casual
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LeaveStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl Default for LeaveStatus {
    fn default() -> Self {
        LeaveStatus::Pending
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveApplication {
    pub id: Option<i64>,
    pub application_number: String,
    pub employee_code: String,
    pub username: String,
    pub leave_type: LeaveType,
    pub start_date: String, // YYYY-MM-DD
    pub end_date: String,   // YYYY-MM-DD
    pub total_days: f64,
    pub reason: String,
    pub status: LeaveStatus,
    pub approved_by_username: Option<String>,
    pub decision_remarks: Option<String>,
    pub applied_at: Option<String>,
    pub decided_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaveBalance {
    pub employee_code: String,
    pub year: i32,
    pub casual_leave_total: f64,
    pub casual_leave_used: f64,
    pub sick_leave_total: f64,
    pub sick_leave_used: f64,
    pub annual_leave_total: f64,
    pub annual_leave_used: f64,
}
