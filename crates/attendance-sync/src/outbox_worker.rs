use crate::network_monitor::NetworkMonitor;
use attendance_bigquery::BigQueryClient;
use attendance_common::Result;
use attendance_database::DbPool;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub struct OutboxWorker {
    db: DbPool,
    bq: Arc<BigQueryClient>,
    poll_interval_secs: u64,
}

impl OutboxWorker {
    pub fn new(db: DbPool, bq: Arc<BigQueryClient>, poll_interval_secs: u64) -> Self {
        Self {
            db,
            bq,
            poll_interval_secs,
        }
    }

    pub async fn process_cycle(&self) -> Result<usize> {
        let events = self.db.fetch_pending_outbox_events(50)?;
        if events.is_empty() {
            return Ok(0);
        }

        // Fast internet health check
        let is_online = NetworkMonitor::check_internet_connection();
        if !is_online && self.bq.is_configured() {
            tracing::debug!("Internet unreachable. Retaining {} events in local SQLite outbox.", events.len());
            return Ok(0);
        }

        let total = events.len();
        match self.bq.stream_batch(&events).await {
            Ok(synced_count) => {
                if synced_count > 0 {
                    for ev in &events[..synced_count] {
                        let _ = self.db.mark_outbox_event_completed(&ev.event_id);
                    }
                    tracing::info!("Synced {} events to cloud outbox", synced_count);
                }
            }
            Err(e) => {
                let err_msg = e.to_string();
                tracing::warn!("Cloud sync failed: {}", err_msg);
                for ev in &events {
                    let _ = self.db.mark_outbox_event_failed(&ev.event_id, &err_msg);
                }
            }
        }

        Ok(total)
    }

    pub fn start(self: Arc<Self>) {
        tokio::spawn(async move {
            tracing::info!("Durable Outbox Worker running (interval: {}s)", self.poll_interval_secs);
            loop {
                let _ = self.process_cycle().await;
                sleep(Duration::from_secs(self.poll_interval_secs)).await;
            }
        });
    }
}
