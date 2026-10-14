use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub host: String,
    pub database_url: String,
    pub lock_port: u16,
    pub timezone: String,
    pub default_shift_start: String,
    pub default_shift_end: String,
    pub grace_period_minutes: u32,
    pub cooldown_seconds: i64,
    pub google_cloud_project: Option<String>,
    pub bigquery_dataset: Option<String>,
    pub bigquery_location: Option<String>,
    pub google_credentials_path: Option<String>,
    pub enable_cloud_sync: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "sqlite://./data/attendance.db".to_string()),
            lock_port: 12345, // Python SingleInstanceManager port 12345
            timezone: "Asia/Dhaka".to_string(),
            default_shift_start: "09:00".to_string(),
            default_shift_end: "17:00".to_string(),
            grace_period_minutes: 15,
            cooldown_seconds: 60,
            google_cloud_project: std::env::var("GOOGLE_CLOUD_PROJECT").ok(),
            bigquery_dataset: std::env::var("BIGQUERY_DATASET").ok(),
            bigquery_location: std::env::var("BIGQUERY_LOCATION").ok(),
            google_credentials_path: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
            enable_cloud_sync: std::env::var("ENABLE_CLOUD_SYNC")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true),
        }
    }
}
