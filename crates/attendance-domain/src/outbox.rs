use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OutboxStatus {
    Pending,
    InFlight,
    Completed,
    Failed,
    DeadLetter,
}

impl Default for OutboxStatus {
    fn default() -> Self {
        OutboxStatus::Pending
    }
}

impl std::fmt::Display for OutboxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboxStatus::Pending => write!(f, "PENDING"),
            OutboxStatus::InFlight => write!(f, "IN_FLIGHT"),
            OutboxStatus::Completed => write!(f, "COMPLETED"),
            OutboxStatus::Failed => write!(f, "FAILED"),
            OutboxStatus::DeadLetter => write!(f, "DEAD_LETTER"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryOutboxEvent {
    pub event_id: String,
    pub table_name: String,
    pub operation: String, // INSERT, UPDATE, DELETE
    pub payload_json: String,
    pub status: String,
    pub retry_count: i32,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub record_id: String,
    pub synced: i32, // 0 = false, 1 = true
    pub last_sync_attempt: Option<String>,
    pub retry_count: i32,
}
