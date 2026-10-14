use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ShiftType {
    RegularDay,
    Morning,
    Evening,
    NightCrossMidnight,
    Rotational,
    Flexible,
}

impl Default for ShiftType {
    fn default() -> Self {
        ShiftType::RegularDay
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shift {
    pub id: Option<i64>,
    pub company_id: String,
    pub name: String,
    pub code: String,
    pub shift_type: ShiftType,
    pub start_time: String, // HH:MM:SS
    pub end_time: String,   // HH:MM:SS
    pub grace_period_minutes: u32,
    pub half_day_threshold_hours: f64,
    pub min_work_hours_full_day: f64,
    pub break_duration_minutes: u32,
    pub is_overtime_enabled: bool,
    pub overtime_starts_after_minutes: u32,
    pub is_active: bool,
    pub created_at: Option<String>,
}

impl Default for Shift {
    fn default() -> Self {
        Self {
            id: None,
            company_id: "COMP_DEFAULT".to_string(),
            name: "Standard Office Shift".to_string(),
            code: "STD_09_17".to_string(),
            shift_type: ShiftType::RegularDay,
            start_time: "09:00:00".to_string(),
            end_time: "17:00:00".to_string(),
            grace_period_minutes: 15,
            half_day_threshold_hours: 4.0,
            min_work_hours_full_day: 8.0,
            break_duration_minutes: 60,
            is_overtime_enabled: true,
            overtime_starts_after_minutes: 30,
            is_active: true,
            created_at: Some(chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string()),
        }
    }
}
