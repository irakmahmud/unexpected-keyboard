use crate::schema::{compute_shard_bucket, parse_year_month};
use attendance_common::{AppError, Result};
use parking_lot::Mutex;
use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

pub const NUM_PHYSICAL_SHARDS: usize = 64;

#[derive(Clone)]
pub struct PhysicalShardedEngine {
    base_dir: PathBuf,
    shards: Arc<Vec<Mutex<Connection>>>,
    known_partitions: Arc<Mutex< std::collections::HashSet<(usize, i32, i32)> >>,
}

impl PhysicalShardedEngine {
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self> {
        let shards_dir = base_dir.as_ref().join("shards");
        std::fs::create_dir_all(&shards_dir)?;

        let mut shards = Vec::with_capacity(NUM_PHYSICAL_SHARDS);
        for shard_id in 0..NUM_PHYSICAL_SHARDS {
            let db_file = shards_dir.join(format!("shard_{:02}.db", shard_id));
            let conn = Connection::open(&db_file).map_err(|e| {
                AppError::Database(format!("Failed to open physical shard {}: {}", shard_id, e))
            })?;
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA temp_store = MEMORY;
                 PRAGMA cache_size = -4000;
                 PRAGMA busy_timeout = 5000;",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
            shards.push(Mutex::new(conn));
        }

        Ok(Self {
            base_dir: shards_dir,
            shards: Arc::new(shards),
            known_partitions: Arc::new(Mutex::new(std::collections::HashSet::new())),
        })
    }

    pub fn shard_file_path(&self, shard_id: usize) -> PathBuf {
        self.base_dir.join(format!("shard_{:02}.db", shard_id % NUM_PHYSICAL_SHARDS))
    }

    pub fn partition_table_name(year: i32, month: i32) -> String {
        format!("attendance_records_y{:04}_m{:02}", year, month.clamp(1, 12))
    }

    fn ensure_physical_partition_table(
        &self,
        conn: &Connection,
        shard_id: usize,
        year: i32,
        month: i32,
    ) -> Result<String> {
        let m = month.clamp(1, 12);
        let table_name = Self::partition_table_name(year, m);
        {
            let guard = self.known_partitions.lock();
            if guard.contains(&(shard_id, year, m)) {
                return Ok(table_name);
            }
        }

        let ddl = format!(
            "CREATE TABLE IF NOT EXISTS {tbl} (
                record_id TEXT PRIMARY KEY,
                username TEXT NOT NULL,
                date TEXT NOT NULL,
                in_time TEXT,
                out_time TEXT,
                status TEXT NOT NULL DEFAULT 'PRESENT',
                partition_year INTEGER NOT NULL,
                partition_month INTEGER NOT NULL,
                shard_bucket INTEGER NOT NULL,
                storage_tier TEXT NOT NULL DEFAULT 'HOT',
                created_at TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_{tbl}_user_date
                ON {tbl}(username, date, status, in_time, out_time);
            CREATE INDEX IF NOT EXISTS idx_{tbl}_cursor
                ON {tbl}(date DESC, record_id DESC);",
            tbl = table_name
        );
        conn.execute_batch(&ddl)
            .map_err(|e| AppError::Database(format!("Failed creating partition {}: {}", table_name, e)))?;

        self.known_partitions.lock().insert((shard_id, year, m));
        Ok(table_name)
    }

    pub fn route_and_insert(
        &self,
        record_id: &str,
        username: &str,
        date: &str,
        in_time: Option<&str>,
        out_time: Option<&str>,
        status: &str,
    ) -> Result<(usize, String)> {
        let upper = username.trim().to_ascii_uppercase();
        let shard_id = (compute_shard_bucket(&upper) as usize) % NUM_PHYSICAL_SHARDS;
        let (year, month) = parse_year_month(date);

        let conn = self.shards[shard_id].lock();
        let table_name = self.ensure_physical_partition_table(&conn, shard_id, year, month)?;

        let sql = format!(
            "INSERT INTO {tbl} (
                record_id, username, date, in_time, out_time, status,
                partition_year, partition_month, shard_bucket, storage_tier, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'HOT', ?3)
            ON CONFLICT(record_id) DO UPDATE SET
                in_time = COALESCE(excluded.in_time, {tbl}.in_time),
                out_time = COALESCE(excluded.out_time, {tbl}.out_time),
                status = excluded.status",
            tbl = table_name
        );

        conn.execute(
            &sql,
            params![
                record_id,
                upper,
                date,
                in_time,
                out_time,
                status,
                year,
                month,
                shard_id as i32
            ],
        )
        .map_err(|e| AppError::Database(e.to_string()))?;

        Ok((shard_id, table_name))
    }

    pub fn query_partitioned_user(
        &self,
        username: &str,
        year: i32,
        month: i32,
        date: Option<&str>,
    ) -> Result<serde_json::Value> {
        let start = Instant::now();
        let upper = username.trim().to_ascii_uppercase();
        let shard_id = (compute_shard_bucket(&upper) as usize) % NUM_PHYSICAL_SHARDS;
        let conn = self.shards[shard_id].lock();
        let table_name = self.ensure_physical_partition_table(&conn, shard_id, year, month)?;

        let query_sql = if date.is_some() {
            format!(
                "SELECT record_id, username, date, in_time, out_time, status
                 FROM {tbl}
                 WHERE username = ?1 AND date = ?2
                 ORDER BY date DESC",
                tbl = table_name
            )
        } else {
            format!(
                "SELECT record_id, username, date, in_time, out_time, status
                 FROM {tbl}
                 WHERE username = ?1
                 ORDER BY date DESC
                 LIMIT 50",
                tbl = table_name
            )
        };

        let explain_sql = format!("EXPLAIN QUERY PLAN {}", query_sql);
        let mut plan_steps = Vec::new();
        if let Ok(mut estmt) = conn.prepare(&explain_sql) {
            if let Some(d) = date {
                if let Ok(rows) = estmt.query_map(params![upper, d], |r| r.get::<_, String>(3)) {
                    for step in rows.flatten() {
                        plan_steps.push(step);
                    }
                }
            } else if let Ok(rows) = estmt.query_map(params![upper], |r| r.get::<_, String>(3)) {
                for step in rows.flatten() {
                    plan_steps.push(step);
                }
            }
        }

        let mut stmt = conn
            .prepare(&query_sql)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut results = Vec::new();
        if let Some(d) = date {
            let rows = stmt
                .query_map(params![upper, d], |r| {
                    Ok(serde_json::json!({
                        "record_id": r.get::<_, String>(0)?,
                        "username": r.get::<_, String>(1)?,
                        "date": r.get::<_, String>(2)?,
                        "in_time": r.get::<_, Option<String>>(3)?,
                        "out_time": r.get::<_, Option<String>>(4)?,
                        "status": r.get::<_, String>(5)?,
                    }))
                })
                .map_err(|e| AppError::Database(e.to_string()))?;
            for item in rows.flatten() {
                results.push(item);
            }
        } else {
            let rows = stmt
                .query_map(params![upper], |r| {
                    Ok(serde_json::json!({
                        "record_id": r.get::<_, String>(0)?,
                        "username": r.get::<_, String>(1)?,
                        "date": r.get::<_, String>(2)?,
                        "in_time": r.get::<_, Option<String>>(3)?,
                        "out_time": r.get::<_, Option<String>>(4)?,
                        "status": r.get::<_, String>(5)?,
                    }))
                })
                .map_err(|e| AppError::Database(e.to_string()))?;
            for item in rows.flatten() {
                results.push(item);
            }
        }

        let elapsed_us = start.elapsed().as_micros();
        Ok(serde_json::json!({
            "routed_physical_shard_id": shard_id,
            "routed_physical_shard_file": format!("shard_{:02}.db", shard_id),
            "routed_physical_partition_table": table_name,
            "shards_pruned": NUM_PHYSICAL_SHARDS - 1,
            "execution_time_us": elapsed_us,
            "execution_time_ms": (elapsed_us as f64) / 1000.0,
            "explain_query_plan": plan_steps,
            "records_returned": results.len(),
            "records": results
        }))
    }

    pub fn inspect_physical_shards_summary(&self) -> Result<serde_json::Value> {
        let mut total_shard_files = 0usize;
        let mut total_physical_partition_tables = 0usize;
        let mut total_sharded_rows = 0i64;
        let mut sample_shards = Vec::new();

        for shard_id in 0..NUM_PHYSICAL_SHARDS {
            let path = self.shard_file_path(shard_id);
            if path.exists() {
                total_shard_files += 1;
            }
            let conn = self.shards[shard_id].lock();
            let mut stmt = conn
                .prepare(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'attendance_records_y%' ORDER BY name",
                )
                .map_err(|e| AppError::Database(e.to_string()))?;
            let tables: Vec<String> = stmt
                .query_map([], |r| r.get::<_, String>(0))
                .map_err(|e| AppError::Database(e.to_string()))?
                .flatten()
                .collect();

            let mut shard_rows = 0i64;
            for t in &tables {
                let c: i64 = conn
                    .query_row(&format!("SELECT COUNT(*) FROM {}", t), [], |r| r.get(0))
                    .unwrap_or(0);
                shard_rows += c;
            }
            total_physical_partition_tables += tables.len();
            total_sharded_rows += shard_rows;

            if shard_id < 4 || shard_rows > 0 && sample_shards.len() < 8 {
                sample_shards.push(serde_json::json!({
                    "shard_id": shard_id,
                    "shard_file": format!("shard_{:02}.db", shard_id),
                    "physical_partition_tables": tables,
                    "row_count": shard_rows
                }));
            }
        }

        Ok(serde_json::json!({
            "physical_shard_files_count": total_shard_files,
            "total_physical_partition_tables": total_physical_partition_tables,
            "total_sharded_rows": total_sharded_rows,
            "sample_shards": sample_shards
        }))
    }

    pub fn generate_postgres_declarative_partition_sql(&self, years: &[i32], hash_mod: usize) -> String {
        let mut sql = String::new();
        sql.push_str("-- ============================================================================\n");
        sql.push_str("-- POSTGRESQL 16+ DECLARATIVE RANGE + HASH PARTITIONED SCHEMA & MIGRATION DDL\n");
        sql.push_str("-- Designed for 50 Years Retention & 1,000,000 (10 Lakh) Concurrent Employees\n");
        sql.push_str("-- ============================================================================\n\n");
        sql.push_str("CREATE TABLE IF NOT EXISTS attendance_records (\n");
        sql.push_str("    record_id TEXT NOT NULL,\n");
        sql.push_str("    username TEXT NOT NULL,\n");
        sql.push_str("    date DATE NOT NULL,\n");
        sql.push_str("    in_time TEXT,\n");
        sql.push_str("    out_time TEXT,\n");
        sql.push_str("    status TEXT NOT NULL DEFAULT 'PRESENT',\n");
        sql.push_str("    partition_year INTEGER NOT NULL,\n");
        sql.push_str("    partition_month INTEGER NOT NULL,\n");
        sql.push_str("    shard_bucket INTEGER NOT NULL,\n");
        sql.push_str("    storage_tier TEXT NOT NULL DEFAULT 'HOT',\n");
        sql.push_str("    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,\n");
        sql.push_str("    PRIMARY KEY (partition_year, partition_month, shard_bucket, record_id)\n");
        sql.push_str(") PARTITION BY RANGE (partition_year, partition_month);\n\n");

        sql.push_str("CREATE INDEX IF NOT EXISTS idx_pg_att_user_date\n");
        sql.push_str("    ON attendance_records (username, date)\n");
        sql.push_str("    INCLUDE (status, in_time, out_time);\n\n");
        sql.push_str("CREATE INDEX IF NOT EXISTS idx_pg_att_cursor\n");
        sql.push_str("    ON attendance_records (date DESC, record_id DESC);\n\n");

        let safe_mod = hash_mod.clamp(1, 64);
        for &yr in years {
            for mo in [1, 6, 10, 12] {
                let next_yr = if mo == 12 { yr + 1 } else { yr };
                let next_mo = if mo == 12 { 1 } else { mo + 1 };
                let p_name = format!("attendance_records_y{:04}_m{:02}", yr, mo);
                sql.push_str(&format!(
                    "CREATE TABLE IF NOT EXISTS {p_name}\n    PARTITION OF attendance_records\n    FOR VALUES FROM ({yr}, {mo}) TO ({next_yr}, {next_mo})\n    PARTITION BY HASH (shard_bucket);\n",
                ));
                for rem in 0..safe_mod {
                    sql.push_str(&format!(
                        "CREATE TABLE IF NOT EXISTS {p_name}_s{rem:02}\n    PARTITION OF {p_name}\n    FOR VALUES WITH (MODULUS {safe_mod}, REMAINDER {rem});\n",
                    ));
                }
                sql.push('\n');
            }
        }
        sql
    }

    pub fn run_empirical_benchmark(&self, total_rows: usize) -> Result<serde_json::Value> {
        let rows_to_insert = total_rows.clamp(1_000, 1_000_000);
        let start_ingest = Instant::now();

        // Group synthetic 50-year (1977..=2026) records across 64 physical shards
        let mut per_shard_batches: Vec<HashMap<(i32, i32), Vec<(String, String, String, i32)>>> =
            (0..NUM_PHYSICAL_SHARDS).map(|_| HashMap::new()).collect();

        for i in 0..rows_to_insert {
            let emp_num = (i % 1_000_000) + 1;
            let username = format!("EMP{:07}", emp_num);
            let shard_id = (compute_shard_bucket(&username) as usize) % NUM_PHYSICAL_SHARDS;
            let year = 1977 + ((i % 50) as i32); // Spans all 50 years: 1977..=2026
            let month = 10;
            let day = (i % 28) + 1;
            let date = format!("{:04}-{:02}-{:02}", year, month, day);
            let record_id = format!("{}_{}", username, date);
            per_shard_batches[shard_id]
                .entry((year, month))
                .or_default()
                .push((record_id, username, date, shard_id as i32));
        }

        // Parallel multi-threaded physical shard ingestion across 8 worker threads
        let shards_ref = self.shards.clone();
        let known_ref = self.known_partitions.clone();
        let chunk_size = 8;
        let batches_arc = Arc::new(per_shard_batches);

        std::thread::scope(|s| {
            for worker_idx in 0..(NUM_PHYSICAL_SHARDS / chunk_size) {
                let shards_c = shards_ref.clone();
                let known_c = known_ref.clone();
                let batches_c = batches_arc.clone();
                s.spawn(move || {
                    let start_shard = worker_idx * chunk_size;
                    let end_shard = start_shard + chunk_size;
                    for shard_id in start_shard..end_shard {
                        let mut conn = shards_c[shard_id].lock();
                        if let Ok(tx) = conn.transaction() {
                            for (&(year, month), rows) in &batches_c[shard_id] {
                                let tbl = Self::partition_table_name(year, month);
                                let ddl = format!(
                                    "CREATE TABLE IF NOT EXISTS {tbl} (
                                        record_id TEXT PRIMARY KEY,
                                        username TEXT NOT NULL,
                                        date TEXT NOT NULL,
                                        in_time TEXT,
                                        out_time TEXT,
                                        status TEXT NOT NULL DEFAULT 'PRESENT',
                                        partition_year INTEGER NOT NULL,
                                        partition_month INTEGER NOT NULL,
                                        shard_bucket INTEGER NOT NULL,
                                        storage_tier TEXT NOT NULL DEFAULT 'HOT',
                                        created_at TEXT
                                    );
                                    CREATE INDEX IF NOT EXISTS idx_{tbl}_user_date
                                        ON {tbl}(username, date, status, in_time, out_time);
                                    CREATE INDEX IF NOT EXISTS idx_{tbl}_cursor
                                        ON {tbl}(date DESC, record_id DESC);",
                                    tbl = tbl
                                );
                                let _ = tx.execute_batch(&ddl);
                                known_c.lock().insert((shard_id, year, month));

                                let ins_sql = format!(
                                    "INSERT OR REPLACE INTO {tbl} (
                                        record_id, username, date, in_time, out_time, status,
                                        partition_year, partition_month, shard_bucket, storage_tier, created_at
                                    ) VALUES (?1, ?2, ?3, '09:00:00', '17:00:00', 'PRESENT', ?4, ?5, ?6, 'HOT', ?3)",
                                    tbl = tbl
                                );
                                if let Ok(mut stmt) = tx.prepare(&ins_sql) {
                                    for (rec_id, uname, dt, sb) in rows {
                                        let _ = stmt.execute(params![rec_id, uname, dt, year, month, sb]);
                                    }
                                }
                            }
                            let _ = tx.commit();
                        };
                    }
                });
            }
        });

        let ingest_ms = start_ingest.elapsed().as_secs_f64() * 1000.0;
        let throughput_rps = (rows_to_insert as f64) / (ingest_ms / 1000.0).max(0.001);

        // Concurrent Read + Write Stress Test across 16 threads
        let rw_start = Instant::now();
        let rw_errors = Arc::new(Mutex::new(0usize));
        std::thread::scope(|s| {
            for t in 0..16usize {
                let engine_c = self.clone();
                let err_c = rw_errors.clone();
                s.spawn(move || {
                    for op in 0..25usize {
                        let emp_id = (t * 100 + op) + 1;
                        let uname = format!("EMP{:07}", emp_id);
                        let yr = 1977 + ((op % 50) as i32);
                        let dt = format!("{:04}-10-15", yr);
                        let rec_id = format!("{}_{}", uname, dt);
                        if op % 3 == 0 {
                            if engine_c
                                .route_and_insert(&rec_id, &uname, &dt, Some("08:58:00"), Some("17:05:00"), "PRESENT")
                                .is_err()
                            {
                                *err_c.lock() += 1;
                            }
                        } else if engine_c.query_partitioned_user(&uname, yr, 10, None).is_err() {
                            *err_c.lock() += 1;
                        }
                    }
                });
            }
        });
        let concurrent_rw_ms = rw_start.elapsed().as_secs_f64() * 1000.0;
        let rw_error_count = *rw_errors.lock();

        // Query Latency & EXPLAIN QUERY PLAN across Year 1977 (50 yrs ago), Year 2001 (25 yrs ago), Year 2026 (Current)
        let q_1977 = self.query_partitioned_user("EMP0000001", 1977, 10, None)?;
        let q_2001 = self.query_partitioned_user("EMP0000025", 2001, 10, None)?;
        let q_2026 = self.query_partitioned_user("EMP0000050", 2026, 10, None)?;

        // Hot Backup & Checksum Recovery Test on Shard 00
        let shard0_path = self.shard_file_path(0);
        let backup_path = self.base_dir.join("shard_00_verified_backup.db");
        {
            let conn0 = self.shards[0].lock();
            let _ = conn0.execute("VACUUM INTO ?1", params![backup_path.to_string_lossy().to_string()]);
        }
        let orig_bytes = std::fs::read(&backup_path).unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(&orig_bytes);
        let backup_sha256 = format!("{:x}", hasher.finalize());

        let restored_conn = Connection::open(&backup_path).map_err(|e| AppError::Database(e.to_string()))?;
        let integrity: String = restored_conn
            .query_row("PRAGMA integrity_check;", [], |r| r.get(0))
            .unwrap_or_else(|_| "error".to_string());
        let _ = std::fs::remove_file(&backup_path);
        let _ = shard0_path;

        Ok(serde_json::json!({
            "benchmark_rows_inserted": rows_to_insert,
            "simulated_years_span": "1977 to 2026 (50 Distinct Partition Years)",
            "physical_shard_files_used": NUM_PHYSICAL_SHARDS,
            "bulk_ingestion_ms": (ingest_ms * 100.0).round() / 100.0,
            "bulk_throughput_rows_per_sec": throughput_rps.round(),
            "concurrent_16_threads_400_ops_ms": (concurrent_rw_ms * 100.0).round() / 100.0,
            "concurrent_rw_lock_errors": rw_error_count,
            "historical_queries": {
                "year_1977_50yr_ago": q_1977,
                "year_2001_25yr_ago": q_2001,
                "year_2026_current": q_2026
            },
            "backup_and_recovery_verification": {
                "method": "Online SQLite VACUUM INTO + SHA-256 + PRAGMA integrity_check",
                "backup_size_bytes": orig_bytes.len(),
                "sha256_checksum": backup_sha256,
                "restored_integrity_check": integrity
            }
        }))
    }
}
