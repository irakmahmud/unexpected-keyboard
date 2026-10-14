use attendance_common::{AppError, Result};
use attendance_domain::DatabaseBackupInfo;
use rusqlite::Connection;
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub struct BackupManager;

impl BackupManager {
    pub fn perform_vacuum(conn: &Connection) -> Result<()> {
        conn.execute("VACUUM;", [])
            .map_err(|e| AppError::Database(format!("SQLite vacuum failed: {}", e)))?;
        Ok(())
    }

    pub fn check_integrity(conn: &Connection) -> Result<String> {
        let result: String = conn
            .query_row("PRAGMA integrity_check;", [], |row| row.get(0))
            .map_err(|e| AppError::Database(format!("Integrity check query failed: {}", e)))?;

        if result != "ok" {
            return Err(AppError::Database(format!("Database integrity compromised: {}", result)));
        }
        Ok(result)
    }

    pub fn create_backup<P: AsRef<Path>>(
        conn: &Connection,
        backup_dir: P,
    ) -> Result<DatabaseBackupInfo> {
        let dir = backup_dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let backup_id = uuid::Uuid::new_v4().to_string();
        let filename = format!("backup_attendance_{}.db", timestamp);
        let backup_path = dir.join(&filename);

        // SQLite VACUUM INTO creates a safe, atomic, read-only snapshot
        let vacuum_cmd = format!("VACUUM INTO '{}';", backup_path.to_string_lossy());
        conn.execute(&vacuum_cmd, [])
            .map_err(|e| AppError::Database(format!("VACUUM INTO backup failed: {}", e)))?;

        // Calculate file size and SHA256 checksum
        let metadata = std::fs::metadata(&backup_path)?;
        let size_bytes = metadata.len();

        let mut file = File::open(&backup_path)?;
        let mut hasher = sha2::Sha256::default();
        let mut buffer = [0u8; 8192];
        loop {
            let n = file.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            use sha2::Digest;
            hasher.update(&buffer[..n]);
        }
        use sha2::Digest;
        let checksum = format!("{:x}", hasher.finalize());

        let total_records: i64 = conn
            .query_row("SELECT COUNT(*) FROM attendance_records", [], |r| r.get(0))
            .unwrap_or(0);

        // Record in system_backups table
        let info = DatabaseBackupInfo {
            backup_id: backup_id.clone(),
            filename: filename.clone(),
            size_bytes,
            created_at: chrono::Utc::now().to_rfc3339(),
            checksum_sha256: checksum.clone(),
            total_records: total_records as u64,
        };

        conn.execute(
            "INSERT INTO system_backups (backup_id, filename, size_bytes, checksum_sha256, total_records, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                info.backup_id,
                info.filename,
                info.size_bytes as i64,
                info.checksum_sha256,
                info.total_records as i64,
                info.created_at
            ],
        )
        .map_err(|e| AppError::Database(format!("Failed to record backup metadata: {}", e)))?;

        Ok(info)
    }

    pub fn list_backups(conn: &Connection) -> Result<Vec<DatabaseBackupInfo>> {
        let mut stmt = conn
            .prepare(
                "SELECT backup_id, filename, size_bytes, checksum_sha256, total_records, created_at
                 FROM system_backups ORDER BY created_at DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id: String = row.get(0)?;
                let fname: String = row.get(1)?;
                let sz: i64 = row.get(2)?;
                let sha: String = row.get(3)?;
                let rec: i64 = row.get(4)?;
                let ca: String = row.get(5)?;

                Ok(DatabaseBackupInfo {
                    backup_id: id,
                    filename: fname,
                    size_bytes: sz as u64,
                    checksum_sha256: sha,
                    total_records: rec as u64,
                    created_at: ca,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
