use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationChannel {
    Email,
    Sms,
    Webhook,
    InApp,
    KioskDisplay,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NotificationPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttendanceEvent {
    PunchSuccess {
        employee_code: String,
        punch_type: String,
        timestamp: String,
        method: String,
    },
    LateArrival {
        employee_code: String,
        minutes_late: u32,
        shift_name: String,
        date: String,
    },
    MissingPunch {
        employee_code: String,
        date: String,
        missing_type: String,
    },
    LeaveApplied {
        employee_code: String,
        leave_type: String,
        days: u32,
    },
    LeaveStatusChanged {
        employee_code: String,
        status: String,
        remarks: Option<String>,
    },
    DeviceOffline {
        device_id: String,
        last_seen: String,
    },
    BigQuerySyncFailed {
        error_message: String,
        pending_records: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMessage {
    pub id: String,
    pub title: String,
    pub body: String,
    pub channel: NotificationChannel,
    pub priority: NotificationPriority,
    pub recipient: String,
    pub created_at: String,
    pub delivered: bool,
    pub delivered_at: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl NotificationMessage {
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        channel: NotificationChannel,
        priority: NotificationPriority,
        recipient: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title: title.into(),
            body: body.into(),
            channel,
            priority,
            recipient: recipient.into(),
            created_at: chrono::Utc::now().to_rfc3339(),
            delivered: false,
            delivered_at: None,
            metadata: None,
        }
    }
}
