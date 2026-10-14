use attendance_common::Result;
use attendance_database::DbPool;

pub struct ReconciliationService {
    db: DbPool,
}

impl ReconciliationService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Auto-close open in-time punches at day cutoff (e.g. 23:59:59) for unattended departures
    pub fn auto_checkout_unattended(&self, date: &str, default_exit_time: &str) -> Result<usize> {
        let records = self.db.get_attendance_by_date(date)?;
        let mut closed_count = 0;

        for r in records {
            if r.in_time.is_some() && r.out_time.is_none() {
                self.db.save_out_time(
                    &r.username,
                    date,
                    default_exit_time,
                    None,
                    false,
                    false,
                )?;
                closed_count += 1;
                tracing::info!("Auto-checkout performed for user {} on date {}", r.username, date);
            }
        }

        Ok(closed_count)
    }

    /// Identify employees who have not clocked in today (excluding approved leaves and holidays)
    pub fn identify_absentees(&self, date: &str, company_id: &str) -> Result<Vec<String>> {
        let all_employees = self.db.list_employees(company_id)?;
        let present_records = self.db.get_attendance_by_date(date)?;
        let present_usernames: std::collections::HashSet<String> = present_records
            .into_iter()
            .map(|r| r.username.to_uppercase())
            .collect();

        let mut absentees = Vec::new();
        for emp in all_employees {
            let code = emp.employee_code.to_uppercase();
            if !present_usernames.contains(&code) {
                // Check if on leave
                let on_leave = self.db.is_on_leave_today(&code, date).unwrap_or(false);
                if !on_leave {
                    absentees.push(code);
                }
            }
        }

        Ok(absentees)
    }
}
