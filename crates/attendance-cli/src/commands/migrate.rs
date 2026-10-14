use attendance_common::{AppError, Result};
use attendance_database::DbPool;

pub struct MigrateCommand;

impl MigrateCommand {
    pub fn execute(db: &DbPool) -> Result<()> {
        println!("============================================================");
        println!("          ATTENDANCE SYSTEM DATABASE MIGRATION & AUDIT      ");
        println!("============================================================");

        // 1. Initialize & verify schema
        println!("[1/4] Checking and applying database schema migrations...");
        db.initialize_schema()?;
        println!("      ✓ Schema initialized / updated successfully.");

        // 2. Perform SQLite WAL Checkpoint
        println!("[2/4] Executing WAL truncation checkpoint...");
        let conn = db.get_connection();
        let checkpoint_res: String = conn
            .query_row("PRAGMA wal_checkpoint(TRUNCATE);", [], |row| {
                let busy: i64 = row.get(0)?;
                let log: i64 = row.get(1)?;
                let checkpointed: i64 = row.get(2)?;
                Ok(format!(
                    "Busy: {}, Log pages: {}, Checkpointed pages: {}",
                    busy, log, checkpointed
                ))
            })
            .map_err(|e| AppError::Database(format!("WAL checkpoint error: {}", e)))?;
        println!("      ✓ WAL Checkpoint: {}", checkpoint_res);

        // 3. Run SQLite Integrity Check
        println!("[3/4] Running PRAGMA integrity_check...");
        let integrity: String = conn
            .query_row("PRAGMA integrity_check;", [], |row| row.get(0))
            .map_err(|e| AppError::Database(format!("Integrity check error: {}", e)))?;

        if integrity == "ok" {
            println!("      ✓ Database integrity check passed: OK");
        } else {
            eprintln!("      ✗ Integrity check returned warnings: {}", integrity);
        }

        // 4. Print table row counts
        println!("[4/4] Auditing table statistics:");
        let tables = [
            "company_info",
            "departments",
            "shifts",
            "users",
            "employees",
            "attendance_records",
            "face_profiles",
            "leaves",
            "holidays",
            "salary_structures",
            "payslips",
            "registered_devices",
            "bigquery_outbox",
            "security_audit_logs",
        ];

        for table in tables {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {}", table), [], |r| r.get(0))
                .unwrap_or(-1);

            println!("      - {:<24} : {:>6} rows", table, count);
        }

        println!("============================================================");
        println!("Migration and audit completed successfully.");
        Ok(())
    }
}
