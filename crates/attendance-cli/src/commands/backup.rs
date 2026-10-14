use attendance_common::{AppError, Result, TimeHelper};
use attendance_database::DbPool;
use std::fs;
use std::path::PathBuf;

pub struct BackupCommand;

impl BackupCommand {
    pub fn execute(db: &DbPool, target_dir: Option<&str>) -> Result<()> {
        let dir_path = target_dir.unwrap_or("./data/backups");
        fs::create_dir_all(dir_path)?;

        let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_file = format!("{}/attendance_backup_{}.db", dir_path, timestamp);
        let backup_path = PathBuf::from(&backup_file);

        println!("============================================================");
        println!("              ONLINE SQLITE DATABASE BACKUP                 ");
        println!("============================================================");
        println!("Initiating hot online backup via VACUUM INTO...");
        println!("Target path: {}", backup_file);

        let conn = db.get_connection();
        let query = format!("VACUUM INTO '{}';", backup_file);
        conn.execute(&query, [])
            .map_err(|e| AppError::Database(format!("Backup failed: {}", e)))?;

        // Calculate file size and verify
        let metadata = fs::metadata(&backup_path)?;
        let size_kb = metadata.len() as f64 / 1024.0;

        println!("✓ Backup completed successfully!");
        println!("  File size: {:.2} KB", size_kb);
        println!("  Timestamp: {}", TimeHelper::now_utc_sqlite_string());

        // Check retention: list existing backups and report
        if let Ok(entries) = fs::read_dir(dir_path) {
            let mut count = 0;
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "db" {
                        count += 1;
                    }
                }
            }
            println!("  Total retained backups in archive: {}", count);
        }

        println!("============================================================");
        Ok(())
    }
}
