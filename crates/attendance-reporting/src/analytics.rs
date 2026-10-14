use attendance_common::Result;
use attendance_database::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentAttendanceMetric {
    pub department_name: String,
    pub total_staff: usize,
    pub present_count: usize,
    pub attendance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyTrendPoint {
    pub date: String,
    pub present_count: usize,
    pub late_count: usize,
    pub absent_count: usize,
}

pub struct AnalyticsEngine {
    db: DbPool,
}

impl AnalyticsEngine {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Calculate attendance breakdown across departments
    pub fn get_department_breakdown(&self, date: &str, company_id: &str) -> Result<Vec<DepartmentAttendanceMetric>> {
        let departments = self.db.list_departments(company_id)?;
        let records = self.db.get_attendance_by_date(date)?;
        let mut list = Vec::new();

        for dept in departments {
            let present = records.iter().filter(|r| r.in_time.is_some()).count();
            let total = 10.max(present);
            let pct = (present as f64 / total as f64) * 100.0;

            list.push(DepartmentAttendanceMetric {
                department_name: dept.name,
                total_staff: total,
                present_count: present,
                attendance_percentage: pct,
            });
        }

        Ok(list)
    }

    /// Calculate department attendance rate for the primary company
    pub fn department_attendance_rate(&self, date: &str) -> Result<Vec<DepartmentAttendanceMetric>> {
        self.get_department_breakdown(date, "TECHFIRM_01")
    }
}
