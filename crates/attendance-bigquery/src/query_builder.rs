use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryTimePartition {
    pub partition_type: String, // "DAY", "HOUR", "MONTH", "YEAR"
    pub field: Option<String>,
    pub expiration_ms: Option<u64>,
}

pub struct BigQueryQueryBuilder;

impl BigQueryQueryBuilder {
    /// Build SQL for monthly department absenteeism analysis in BigQuery
    pub fn build_department_absenteeism_sql(
        project_id: &str,
        dataset: &str,
        from_date: &str,
        to_date: &str,
    ) -> String {
        format!(
            r#"SELECT 
                u.company_name,
                u.user_designation,
                COUNT(DISTINCT a.username) AS active_attendees,
                COUNT(a.record_id) AS total_punches,
                AVG(a.late_minutes) AS avg_late_minutes,
                SUM(a.overtime_hours) AS total_overtime_hours
             FROM `{project}.{dataset}.attendance_records_analytics` a
             JOIN `{project}.{dataset}.users_analytics` u ON u.username = a.username
             WHERE a.date BETWEEN DATE('{from}') AND DATE('{to}')
             GROUP BY u.company_name, u.user_designation
             ORDER BY avg_late_minutes DESC"#,
            project = project_id,
            dataset = dataset,
            from = from_date,
            to = to_date
        )
    }

    /// Build daily muster roll rollup table query
    pub fn build_daily_rollup_sql(project_id: &str, dataset: &str, target_date: &str) -> String {
        format!(
            r#"SELECT 
                date,
                COUNT(DISTINCT username) as total_present,
                COUNTIF(late_minutes > 0) as total_late,
                AVG(work_hours) as avg_work_hours,
                SUM(overtime_hours) as total_overtime_hours
             FROM `{project}.{dataset}.attendance_records_analytics`
             WHERE date = DATE('{target}')
             GROUP BY date"#,
            project = project_id,
            dataset = dataset,
            target = target_date
        )
    }

    /// Build query for audit trail reconciliation
    pub fn build_audit_reconciliation_sql(
        project_id: &str,
        dataset: &str,
        min_timestamp: &str,
    ) -> String {
        format!(
            r#"SELECT 
                record_id,
                username,
                date,
                _event_operation,
                _synced_at
             FROM `{project}.{dataset}.attendance_records_analytics`
             WHERE _synced_at >= TIMESTAMP('{min_ts}')
             ORDER BY _synced_at ASC
             LIMIT 1000"#,
            project = project_id,
            dataset = dataset,
            min_ts = min_timestamp
        )
    }
}
