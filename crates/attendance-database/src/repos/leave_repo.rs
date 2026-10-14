use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{LeaveApplication, LeaveStatus, LeaveType};
use rusqlite::{params, Connection};

pub struct LeaveRepo;

impl LeaveRepo {
    pub fn apply(conn: &Connection, app: &LeaveApplication) -> Result<i64> {
        let now = TimeHelper::now_utc_sqlite_string();
        let leave_type_str = format!("{:?}", app.leave_type);
        let status_str = format!("{:?}", app.status);

        conn.execute(
            "INSERT INTO leaves (
                application_number, employee_code, username, leave_type, start_date, end_date,
                total_days, reason, status, approved_by_username, decision_remarks, applied_at, decided_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
            params![
                app.application_number,
                app.employee_code.to_uppercase(),
                app.username.to_uppercase(),
                leave_type_str,
                app.start_date,
                app.end_date,
                app.total_days,
                app.reason,
                status_str,
                app.approved_by_username,
                app.decision_remarks,
                app.applied_at.as_deref().unwrap_or(&now),
                app.decided_at
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn update_status(conn: &Connection, app_num: &str, status: LeaveStatus, approver: &str, remarks: Option<&str>) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        let status_str = format!("{:?}", status);

        conn.execute(
            "UPDATE leaves
             SET status = ?1, approved_by_username = ?2, decision_remarks = ?3, decided_at = ?4
             WHERE application_number = ?5",
            params![status_str, approver.to_uppercase(), remarks, now, app_num],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_by_employee(conn: &Connection, emp_code: &str) -> Result<Vec<LeaveApplication>> {
        let mut stmt = conn.prepare(
            "SELECT id, application_number, employee_code, username, leave_type, start_date, end_date,
                    total_days, reason, status, approved_by_username, decision_remarks, applied_at, decided_at
             FROM leaves WHERE UPPER(employee_code) = ? ORDER BY id DESC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![emp_code.to_uppercase()], |row| {
            let status_str: String = row.get(9).unwrap_or_else(|_| "Pending".to_string());
            let status = match status_str.as_str() {
                "Approved" => LeaveStatus::Approved,
                "Rejected" => LeaveStatus::Rejected,
                "Cancelled" => LeaveStatus::Cancelled,
                _ => LeaveStatus::Pending,
            };

            Ok(LeaveApplication {
                id: row.get(0)?,
                application_number: row.get(1)?,
                employee_code: row.get(2)?,
                username: row.get(3)?,
                leave_type: LeaveType::Casual,
                start_date: row.get(5)?,
                end_date: row.get(6)?,
                total_days: row.get(7)?,
                reason: row.get(8)?,
                status,
                approved_by_username: row.get(10)?,
                decision_remarks: row.get(11)?,
                applied_at: row.get(12)?,
                decided_at: row.get(13)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn is_employee_on_leave_today(conn: &Connection, username: &str, today: &str) -> Result<bool> {
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM leaves
             WHERE UPPER(username) = ? AND status = 'Approved' AND ? >= start_date AND ? <= end_date"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let count: i64 = stmt.query_row(params![username.to_uppercase(), today, today], |row| row.get(0))
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count > 0)
    }
}
