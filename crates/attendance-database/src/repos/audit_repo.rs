use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{AuditSeverity, SecurityAuditLog};
use rusqlite::{params, Connection};

pub struct AuditRepo;

impl AuditRepo {
    pub fn log_event(conn: &Connection, log: &SecurityAuditLog) -> Result<i64> {
        let now = TimeHelper::now_utc_sqlite_string();
        let sev_str = format!("{:?}", log.severity);

        conn.execute(
            "INSERT INTO security_audit_logs (event_type, actor_username, target_resource, ip_address, severity, details_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                log.event_type,
                log.actor_username,
                log.target_resource,
                log.ip_address,
                sev_str,
                log.details_json,
                now
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_recent(conn: &Connection, limit: usize) -> Result<Vec<SecurityAuditLog>> {
        let mut stmt = conn.prepare(
            "SELECT id, event_type, actor_username, target_resource, ip_address, severity, details_json, created_at
             FROM security_audit_logs ORDER BY id DESC LIMIT ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            let sev_str: String = row.get(5).unwrap_or_else(|_| "Info".to_string());
            let severity = match sev_str.as_str() {
                "Warning" => AuditSeverity::Warning,
                "Critical" => AuditSeverity::Critical,
                "SecurityAlert" => AuditSeverity::SecurityAlert,
                _ => AuditSeverity::Info,
            };

            Ok(SecurityAuditLog {
                id: row.get(0)?,
                event_type: row.get(1)?,
                actor_username: row.get(2)?,
                target_resource: row.get(3)?,
                ip_address: row.get(4)?,
                severity,
                details_json: row.get(6)?,
                created_at: row.get(7)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
