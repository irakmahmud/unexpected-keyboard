use attendance_common::{AppError, Result};
use attendance_database::DbPool;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadLetterItem {
    pub event_id: String,
    pub table_name: String,
    pub operation: String,
    pub payload_json: String,
    pub failure_reason: String,
    pub failed_at: String,
    pub retry_count: usize,
}

pub struct DeadLetterQueueManager {
    db: DbPool,
}

impl DeadLetterQueueManager {
    pub fn new(db: DbPool) -> Self {
        Self { db }
    }

    /// Retrieve all permanently failed events that exceeded maximum retry attempts
    pub fn list_dead_letters(&self, limit: usize) -> Result<Vec<DeadLetterItem>> {
        let conn = self.db.get_connection();
        let mut stmt = conn
            .prepare(
                "SELECT event_id, table_name, operation, payload_json, COALESCE(last_error, 'Max retries exceeded'), updated_at, retry_count
                 FROM bigquery_outbox
                 WHERE status = 'FAILED' AND retry_count >= 5
                 ORDER BY updated_at DESC
                 LIMIT ?1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([limit as i64], |row| {
                Ok(DeadLetterItem {
                    event_id: row.get(0)?,
                    table_name: row.get(1)?,
                    operation: row.get(2)?,
                    payload_json: row.get(3)?,
                    failure_reason: row.get(4)?,
                    failed_at: row.get(5)?,
                    retry_count: row.get::<_, i64>(6)? as usize,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    /// Reset a dead-letter event back to 'PENDING' for re-processing after fixing schema or network issues
    pub fn retry_event(&self, event_id: &str) -> Result<bool> {
        let conn = self.db.get_connection();
        let rows = conn
            .execute(
                "UPDATE bigquery_outbox 
                 SET status = 'PENDING', retry_count = 0, last_error = NULL, updated_at = strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')
                 WHERE event_id = ?1",
                rusqlite::params![event_id],
            )
            .map_err(|e| AppError::Database(format!("Failed to reset outbox event: {}", e)))?;

        Ok(rows > 0)
    }

    /// Discard/purge a dead-letter event that cannot be recovered
    pub fn purge_event(&self, event_id: &str) -> Result<bool> {
        let conn = self.db.get_connection();
        let rows = conn
            .execute(
                "DELETE FROM bigquery_outbox WHERE event_id = ?1 AND status = 'FAILED'",
                rusqlite::params![event_id],
            )
            .map_err(|e| AppError::Database(format!("Failed to delete dead-letter event: {}", e)))?;

        Ok(rows > 0)
    }
}
