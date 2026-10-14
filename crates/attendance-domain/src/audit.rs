use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditSeverity {
    Info,
    Warning,
    Critical,
    SecurityAlert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAuditLog {
    pub id: Option<i64>,
    pub event_type: String, // LOGIN, LOGOUT, FAILED_AUTH, PUNCH, ADMIN_OVERRIDE, TIME_DRIFT
    pub actor_username: Option<String>,
    pub target_resource: Option<String>,
    pub ip_address: Option<String>,
    pub severity: AuditSeverity,
    pub details_json: Option<String>,
    pub created_at: String,
}
