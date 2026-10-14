use attendance_common::{AppError, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepartmentAttendanceStat {
    pub department_name: String,
    pub total_employees: usize,
    pub present_count: usize,
    pub attendance_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyPunchDistribution {
    pub hour: u32,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyEmployeeLedger {
    pub employee_code: String,
    pub full_name: String,
    pub department: String,
    pub total_days: u32,
    pub present_days: u32,
    pub absent_days: u32,
    pub late_days: u32,
    pub total_overtime_minutes: u32,
}

pub struct StatsRepo;

impl StatsRepo {
    pub fn get_department_stats_by_date(
        conn: &Connection,
        date: &str,
    ) -> Result<Vec<DepartmentAttendanceStat>> {
        let mut stmt = conn
            .prepare(
                "SELECT d.name, COUNT(DISTINCT e.employee_code) as total,
                        COUNT(DISTINCT a.username) as present
                 FROM departments d
                 JOIN employees e ON e.department_id = d.id AND e.is_active = 1
                 LEFT JOIN users u ON u.username = e.user_uuid OR u.company_user_uuid = e.user_uuid
                 LEFT JOIN attendance_records a ON a.username = u.username AND a.date = ?1
                 GROUP BY d.id, d.name",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![date], |row| {
                let name: String = row.get(0)?;
                let total: i64 = row.get(1)?;
                let present: i64 = row.get(2)?;
                let pct = if total > 0 {
                    (present as f64 / total as f64) * 100.0
                } else {
                    0.0
                };

                Ok(DepartmentAttendanceStat {
                    department_name: name,
                    total_employees: total as usize,
                    present_count: present as usize,
                    attendance_percentage: pct,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_hourly_punch_distribution(
        conn: &Connection,
        date: &str,
    ) -> Result<Vec<HourlyPunchDistribution>> {
        let mut stmt = conn
            .prepare(
                "SELECT strftime('%H', punch_time) as hr, COUNT(*) as cnt
                 FROM punch_logs
                 WHERE date(punch_time) = date(?1)
                 GROUP BY hr
                 ORDER BY hr ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![date], |row| {
                let hr_str: String = row.get(0)?;
                let cnt: i64 = row.get(1)?;
                let hr = hr_str.parse::<u32>().unwrap_or(0);

                Ok(HourlyPunchDistribution {
                    hour: hr,
                    count: cnt as usize,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_monthly_attendance_ledger(
        conn: &Connection,
        month_prefix: &str, // e.g. "2026-10"
    ) -> Result<Vec<MonthlyEmployeeLedger>> {
        let mut stmt = conn
            .prepare(
                "SELECT e.employee_code, e.full_name, COALESCE(d.name, 'General') as dept,
                        COUNT(DISTINCT a.date) as present_days,
                        SUM(CASE WHEN a.shift_status = 'LATE' THEN 1 ELSE 0 END) as late_days
                 FROM employees e
                 LEFT JOIN departments d ON d.id = e.department_id
                 LEFT JOIN users u ON u.username = e.employee_code OR u.user_uuid = e.user_uuid
                 LEFT JOIN attendance_records a ON (a.username = u.username OR a.username = e.employee_code) 
                      AND a.date LIKE ?1 || '%'
                 WHERE e.is_active = 1
                 GROUP BY e.employee_code, e.full_name, d.name
                 ORDER BY e.employee_code ASC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![month_prefix], |row| {
                let code: String = row.get(0)?;
                let name: String = row.get(1)?;
                let dept: String = row.get(2)?;
                let present: i64 = row.get(3)?;
                let late: i64 = row.get(4)?;

                Ok(MonthlyEmployeeLedger {
                    employee_code: code,
                    full_name: name,
                    department: dept,
                    total_days: 30, // standard base
                    present_days: present as u32,
                    absent_days: (30 - present.min(30)) as u32,
                    late_days: late as u32,
                    total_overtime_minutes: 0,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
