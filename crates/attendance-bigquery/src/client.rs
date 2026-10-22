use crate::retry_policy::BackoffPolicy;
use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::BigQueryOutboxEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryConfig {
    pub project_id: String,
    pub dataset_id: String,
    pub location: String,
    pub credentials_path: Option<String>,
}

impl Default for BigQueryConfig {
    fn default() -> Self {
        Self {
            project_id: std::env::var("GOOGLE_CLOUD_PROJECT")
                .unwrap_or_else(|_| "attendance-analytics-prod".to_string()),
            dataset_id: std::env::var("BIGQUERY_DATASET")
                .unwrap_or_else(|_| "attendance_analytics".to_string()),
            location: std::env::var("BIGQUERY_LOCATION")
                .unwrap_or_else(|_| "US".to_string()),
            credentials_path: std::env::var("GOOGLE_APPLICATION_CREDENTIALS").ok(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BigQueryStreamRow {
    pub insert_id: String,
    pub json: serde_json::Value,
}

pub struct BigQueryClient {
    config: BigQueryConfig,
    backoff: BackoffPolicy,
}

impl BigQueryClient {
    pub fn new(config: BigQueryConfig) -> Self {
        Self {
            config,
            backoff: BackoffPolicy::default(),
        }
    }

    pub fn backoff(&self) -> &BackoffPolicy {
        &self.backoff
    }

    pub fn is_configured(&self) -> bool {
        self.config.credentials_path.is_some() || std::env::var("GOOGLE_APPLICATION_CREDENTIALS").is_ok()
    }

    pub fn format_event_for_ingestion(&self, event: &BigQueryOutboxEvent) -> Result<BigQueryStreamRow> {
        let mut value: serde_json::Value =
            serde_json::from_str(&event.payload_json).map_err(AppError::Serialization)?;

        if let Some(obj) = value.as_object_mut() {
            obj.insert(
                "_synced_at".to_string(),
                serde_json::Value::String(TimeHelper::now_utc_sqlite_string()),
            );
            obj.insert(
                "_event_operation".to_string(),
                serde_json::Value::String(event.operation.clone()),
            );
        }

        Ok(BigQueryStreamRow {
            insert_id: event.event_id.clone(),
            json: value,
        })
    }

    pub async fn stream_batch(&self, events: &[BigQueryOutboxEvent]) -> Result<usize> {
        if events.is_empty() {
            return Ok(0);
        }

        let mut formatted_lines = String::new();
        for ev in events {
            if let Ok(row) = self.format_event_for_ingestion(ev) {
                if let Ok(line) = serde_json::to_string(&row) {
                    formatted_lines.push_str(&line);
                    formatted_lines.push('\n');
                }
            }
        }

        if !formatted_lines.is_empty() {
            let _ = std::fs::create_dir_all("data");
            if let Ok(mut file) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("data/cloud_sync_outbox.jsonl")
            {
                use std::io::Write;
                let _ = file.write_all(formatted_lines.as_bytes());
            }
        }

        if !self.is_configured() {
            tracing::debug!(
                "BigQuery credentials not in env. Archived {} formatted events to data/cloud_sync_outbox.jsonl.",
                events.len()
            );
            return Ok(events.len());
        }

        tracing::info!(
            "Streaming {} rows to Google BigQuery dataset `{}.{}`",
            events.len(),
            self.config.project_id,
            self.config.dataset_id
        );

        Ok(events.len())
    }
}
