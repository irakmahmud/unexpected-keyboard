use attendance_common::Result;
use attendance_database::DbPool;
use attendance_sync::DeadLetterQueueManager;

pub struct SyncCommand;

impl SyncCommand {
    pub fn execute(db: &DbPool) -> Result<()> {
        println!("============================================================");
        println!("            GOOGLE BIGQUERY OUTBOX SYNC STATUS              ");
        println!("============================================================");

        let pending = db.fetch_pending_outbox_events(100)?;
        println!("Current Outbox Queue Status:");
        println!("  - Total pending events queued for BigQuery: {}", pending.len());

        let dlq = DeadLetterQueueManager::new(db.clone());
        let failed = dlq.list_dead_letters(20)?;
        println!("  - Dead Letter Queue (failed) events:        {}", failed.len());

        if !pending.is_empty() {
            println!("\nNext 5 pending events in buffer:");
            for evt in pending.iter().take(5) {
                println!(
                    "    [{}] Table: {:<20} Retry: {} Created: {}",
                    evt.event_id, evt.table_name, evt.retry_count, evt.created_at
                );
            }
        }

        if !failed.is_empty() {
            println!("\nTop Dead Letter Queue events requiring attention:");
            for item in failed.iter().take(5) {
                println!(
                    "    [!] ID: {} Table: {} Error: {}",
                    item.event_id, item.table_name, item.failure_reason
                );
            }
        }

        println!("============================================================");
        Ok(())
    }
}
