use attendance_common::Result;
use attendance_domain::SecurityAuditLog;

pub struct AuditLogExporter;

impl AuditLogExporter {
    /// Format security audit logs into CSV representation
    pub fn to_csv(logs: &[SecurityAuditLog]) -> String {
        let mut csv = String::from("id,event_type,actor_username,target_resource,ip_address,severity,created_at,details\n");

        for l in logs {
            let details_escaped = l.details_json.as_deref().unwrap_or("").replace('"', "\"\"");
            csv.push_str(&format!(
                "{},\"{}\",\"{}\",\"{}\",\"{}\",\"{:?}\",\"{}\",\"{}\"\n",
                l.id.unwrap_or(0),
                l.event_type,
                l.actor_username.as_deref().unwrap_or("System"),
                l.target_resource.as_deref().unwrap_or(""),
                l.ip_address.as_deref().unwrap_or(""),
                l.severity,
                l.created_at,
                details_escaped
            ));
        }

        csv
    }

    /// Format security audit logs into pretty JSON
    pub fn to_json(logs: &[SecurityAuditLog]) -> Result<String> {
        let json_str = serde_json::to_string_pretty(logs)
            .map_err(attendance_common::AppError::Serialization)?;
        Ok(json_str)
    }
}
