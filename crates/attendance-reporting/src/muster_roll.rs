use attendance_common::{Result, TimeHelper};
use attendance_database::DbPool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmployeeMusterRow {
    pub employee_code: String,
    pub employee_name: String,
    pub department: String,
    // Day of month (1..31) -> Status symbol: "P", "L", "HD", "A", "W", "H", "LV"
    pub days: HashMap<u32, String>,
    pub total_present: u32,
    pub total_late: u32,
    pub total_absent: u32,
    pub total_leaves: u32,
}

pub struct MusterRollGenerator {
    db: DbPool,
}

impl MusterRollGenerator {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Generate monthly muster roll matrix
    pub fn generate_monthly_muster_roll(&self, year: i32, month: u32, company_id: &str) -> Result<Vec<EmployeeMusterRow>> {
        let mut employees = self.db.list_employees(company_id)?;
        if let Ok(users) = self.db.get_all_users() {
            let existing_codes: std::collections::HashSet<String> = employees
                .iter()
                .map(|e| e.employee_code.to_uppercase())
                .collect();
            for u in users {
                let code = u.username.trim().to_uppercase();
                if !existing_codes.contains(&code) {
                    employees.push(attendance_domain::Employee {
                        id: None,
                        user_uuid: u.company_user_uuid.clone(),
                        employee_code: code.clone(),
                        full_name: u.username.clone(),
                        email: None,
                        phone: None,
                        department_id: None,
                        shift_id: None,
                        designation: Some("Employee".to_string()),
                        joining_date: None,
                        is_active: true,
                        pin_code: None,
                        rfid_uid: None,
                        company_id: company_id.to_string(),
                        created_at: None,
                        updated_at: None,
                    });
                }
            }
        }
        let mut rows = Vec::new();

        let days_in_month = match month {
            4 | 6 | 9 | 11 => 30,
            2 => if (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0) { 29 } else { 28 },
            _ => 31,
        };

        for emp in employees {

            let mut days_map = HashMap::new();
            let mut total_present = 0;
            let mut total_late = 0;
            let mut total_absent = 0;
            let mut total_leaves = 0;

            for day in 1..=days_in_month {
                let date_str = format!("{:04}-{:02}-{:02}", year, month, day);

                // 1. Check weekend
                if TimeHelper::is_weekend_in_dhaka(&date_str).unwrap_or(false) {
                    days_map.insert(day, "W".to_string());
                    continue;
                }

                // 2. Check holiday
                if self.db.is_holiday(company_id, &date_str).unwrap_or(false) {
                    days_map.insert(day, "H".to_string());
                    continue;
                }

                // 3. Check leave
                if self.db.is_on_leave_today(&emp.employee_code, &date_str).unwrap_or(false) {
                    days_map.insert(day, "LV".to_string());
                    total_leaves += 1;
                    continue;
                }

                // 4. Check attendance record
                let rec = self.db.get_attendance_record(&emp.employee_code, &date_str)?;
                if let Some(r) = rec {
                    if r.late_minutes > 0 {
                        days_map.insert(day, "L".to_string());
                        total_late += 1;
                        total_present += 1;
                    } else if r.in_time.is_some() {
                        days_map.insert(day, "P".to_string());
                        total_present += 1;
                    } else {
                        days_map.insert(day, "A".to_string());
                        total_absent += 1;
                    }
                } else {
                    days_map.insert(day, "A".to_string());
                    total_absent += 1;
                }
            }

            let dept_name = if let Some(d_id) = emp.department_id {
                let depts = self.db.list_departments(company_id).unwrap_or_default();
                depts
                    .into_iter()
                    .find(|d| d.id == Some(d_id))
                    .map(|d| d.code)
                    .unwrap_or_else(|| "ENG".to_string())
            } else {
                "ENG".to_string()
            };

            rows.push(EmployeeMusterRow {
                employee_code: emp.employee_code,
                employee_name: emp.full_name,
                department: dept_name,
                days: days_map,
                total_present,
                total_late,
                total_absent,
                total_leaves,
            });
        }

        Ok(rows)
    }
}
