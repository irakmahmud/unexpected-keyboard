use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::BigQueryOutboxEvent;
use rusqlite::{params, Connection};

pub struct OutboxRepo;

impl OutboxRepo {
    pub fn push(conn: &Connection, event: &BigQueryOutboxEvent) -> Result<()> {
        conn.execute(
            "INSERT INTO bigquery_outbox (event_id, table_name, operation, payload_json, status, retry_count, last_error, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                event.event_id,
                event.table_name,
                event.operation,
                event.payload_json,
                event.status,
                event.retry_count,
                event.last_error,
                event.created_at,
                event.updated_at
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn fetch_pending(conn: &Connection, limit: usize) -> Result<Vec<BigQueryOutboxEvent>> {
        let mut stmt = conn.prepare(
            "SELECT event_id, table_name, operation, payload_json, status, retry_count, last_error, created_at, updated_at
             FROM bigquery_outbox
             WHERE status = 'PENDING' OR (status = 'FAILED' AND retry_count < 10)
             ORDER BY created_at ASC LIMIT ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(BigQueryOutboxEvent {
                event_id: row.get(0)?,
                table_name: row.get(1)?,
                operation: row.get(2)?,
                payload_json: row.get(3)?,
                status: row.get(4)?,
                retry_count: row.get(5)?,
                last_error: row.get(6)?,
                created_at: row.get(7)?,
                updated_at: row.get(8)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut events = Vec::new();
        for r in rows {
            events.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(events)
    }

    pub fn mark_completed(conn: &Connection, event_id: &str) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        conn.execute(
            "UPDATE bigquery_outbox SET status = 'COMPLETED', updated_at = ? WHERE event_id = ?",
            params![now, event_id],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn mark_failed(conn: &Connection, event_id: &str, err: &str) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        conn.execute(
            "UPDATE bigquery_outbox
             SET status = 'FAILED', retry_count = retry_count + 1, last_error = ?, updated_at = ?
             WHERE event_id = ?",
            params![err, now, event_id],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn count_pending(conn: &Connection) -> Result<usize> {
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM bigquery_outbox WHERE status = 'PENDING' OR status = 'FAILED'"
        ).map_err(|e| AppError::Database(e.to_string()))?;
        let count: i64 = stmt.query_row([], |row| row.get(0))
            .map_err(|e| AppError::Database(e.to_string()))?;
        Ok(count as usize)
    }
}
