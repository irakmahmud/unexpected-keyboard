pub mod analytics;
pub mod audit_exporter;
pub mod csv_exporter;
pub mod html_report;
pub mod muster_roll;
pub mod punctuality_scorecard;

use attendance_common::{Result, TimeHelper};
use attendance_database::DbPool;
use attendance_domain::AttendanceSummaryResponse;

pub use analytics::AnalyticsEngine;
pub use audit_exporter::AuditLogExporter;
pub use csv_exporter::CsvExporter;
pub use html_report::{HtmlReportRenderer, ReportMetadata};
pub use muster_roll::{EmployeeMusterRow, MusterRollGenerator};
pub use punctuality_scorecard::{PunctualityAnalyzer, PunctualityScorecard};

#[derive(Clone)]
pub struct ReportingService {
    db: DbPool,
}

impl ReportingService {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    pub fn get_daily_summary(&self, date: Option<&str>) -> Result<AttendanceSummaryResponse> {
        let query_date = match date {
            Some(d) => d.to_string(),
            None => TimeHelper::current_date_dhaka(),
        };

        let users = self.db.get_all_users()?;
        let records = self.db.get_attendance_by_date(&query_date)?;

        let total_employees = users.len();
        let present_today = records.len();
        let mut checked_in: usize = 0;
        let mut checked_out: usize = 0;
        let mut late_count: usize = 0;

        for r in &records {
            if r.in_time.is_some() {
                checked_in += 1;
            }
            if r.out_time.is_some() {
                checked_out += 1;
            }
            if r.late_minutes > 0 {
                late_count += 1;
            }
        }

        let pending_out = checked_in.saturating_sub(checked_out);

        Ok(AttendanceSummaryResponse {
            date: query_date,
            total_employees,
            present_today,
            checked_in,
            checked_out,
            pending_out,
            on_leave_today: 0,
            late_arrivals_count: late_count,
        })
    }

    pub fn generate_csv_report(&self, date: &str) -> Result<String> {
        let exporter = CsvExporter::new(self.db.clone());
        exporter.export_date_range_csv(date, date)
    }

    pub fn generate_customized_csv_report(
        &self,
        username: Option<&str>,
        year: i32,
        month: u32,
        from_day: u32,
        to_day: u32,
        in_time: &str,
        out_time: &str,
    ) -> Result<String> {
        let exporter = CsvExporter::new(self.db.clone());
        exporter.export_customized_attendance_matrix_csv(
            username,
            year,
            month,
            from_day,
            to_day,
            in_time,
            out_time,
        )
    }

    pub fn generate_muster_roll(
        &self,
        year: i32,
        month: u32,
        company_id: &str,
    ) -> Result<Vec<EmployeeMusterRow>> {
        let gen = MusterRollGenerator::new(self.db.clone());
        gen.generate_monthly_muster_roll(year, month, company_id)
    }
}
