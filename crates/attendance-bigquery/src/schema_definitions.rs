use serde_json::json;

pub struct BigQuerySchemaBuilder;

impl BigQuerySchemaBuilder {
    /// Schema for `attendance_records_analytics`
    pub fn attendance_records_schema() -> serde_json::Value {
        json!({
            "fields": [
                { "name": "record_id", "type": "STRING", "mode": "REQUIRED" },
                { "name": "company_user_uuid", "type": "STRING", "mode": "NULLABLE" },
                { "name": "username", "type": "STRING", "mode": "REQUIRED" },
                { "name": "date", "type": "DATE", "mode": "REQUIRED" },
                { "name": "in_time", "type": "TIME", "mode": "NULLABLE" },
                { "name": "out_time", "type": "TIME", "mode": "NULLABLE" },
                { "name": "with_camera", "type": "STRING", "mode": "NULLABLE" },
                { "name": "with_qr", "type": "STRING", "mode": "NULLABLE" },
                { "name": "work_hours", "type": "FLOAT64", "mode": "NULLABLE" },
                { "name": "overtime_hours", "type": "FLOAT64", "mode": "NULLABLE" },
                { "name": "late_minutes", "type": "INT64", "mode": "NULLABLE" },
                { "name": "_synced_at", "type": "TIMESTAMP", "mode": "REQUIRED" },
                { "name": "_event_operation", "type": "STRING", "mode": "REQUIRED" }
            ]
        })
    }

    /// Schema for `users_analytics`
    pub fn users_schema() -> serde_json::Value {
        json!({
            "fields": [
                { "name": "company_user_uuid", "type": "STRING", "mode": "REQUIRED" },
                { "name": "company_name", "type": "STRING", "mode": "NULLABLE" },
                { "name": "company_uuid", "type": "STRING", "mode": "NULLABLE" },
                { "name": "username", "type": "STRING", "mode": "REQUIRED" },
                { "name": "user_designation", "type": "STRING", "mode": "NULLABLE" },
                { "name": "user_android_id", "type": "STRING", "mode": "NULLABLE" },
                { "name": "role", "type": "STRING", "mode": "REQUIRED" },
                { "name": "status", "type": "STRING", "mode": "REQUIRED" },
                { "name": "_synced_at", "type": "TIMESTAMP", "mode": "REQUIRED" }
            ]
        })
    }
}
