pub mod repos;
pub mod schema;
pub mod sharded_engine;

use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::*;
use parking_lot::Mutex;
use repos::*;
use rusqlite::Connection;
use sharded_engine::PhysicalShardedEngine;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
    pub sharded_engine: Option<PhysicalShardedEngine>,
}

impl DbPool {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db_path = path.as_ref().to_path_buf();
        let parent_dir = db_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("data"));
        std::fs::create_dir_all(&parent_dir)?;

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Database(format!("Failed to open SQLite database: {}", e)))?;

        // Production SQLite PRAGMAs tuned for 50-Year / 10-Lakh Employee Partitioned Architecture
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -64000;
             PRAGMA temp_store = MEMORY;
             PRAGMA mmap_size = 268435456;",
        )
        .map_err(|e| AppError::Database(format!("Failed to set SQLite pragmas: {}", e)))?;

        let sharded_engine = PhysicalShardedEngine::new(&parent_dir).ok();

        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
            sharded_engine,
        };

        pool.initialize_schema()?;
        pool.sync_existing_records_to_physical_shards();
        Ok(pool)
    }

    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| AppError::Database(format!("In-memory DB open failed: {}", e)))?;

        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| AppError::Database(e.to_string()))?;

        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path: PathBuf::from(":memory:"),
            sharded_engine: None,
        };
        pool.initialize_schema()?;
        Ok(pool)
    }

    fn sync_existing_records_to_physical_shards(&self) {
        if let Some(ref engine) = self.sharded_engine {
            let rows: Vec<(String, String, String, Option<String>, Option<String>, String)> = {
                let conn = self.conn.lock();
                let mut list = Vec::new();
                if let Ok(mut stmt) = conn.prepare(
                    "SELECT record_id, username, date, in_time, out_time, status FROM attendance_records",
                ) {
                    if let Ok(mapped) = stmt.query_map([], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, String>(1)?,
                            r.get::<_, String>(2)?,
                            r.get::<_, Option<String>>(3)?,
                            r.get::<_, Option<String>>(4)?,
                            r.get::<_, String>(5)?,
                        ))
                    }) {
                        for item in mapped.flatten() {
                            list.push(item);
                        }
                    }
                }
                list
            };
            for (rec_id, uname, dt, in_t, out_t, st) in rows {
                let _ = engine.route_and_insert(
                    &rec_id,
                    &uname,
                    &dt,
                    in_t.as_deref(),
                    out_t.as_deref(),
                    &st,
                );
            }
        }
    }

    pub fn get_connection(&self) -> parking_lot::MutexGuard<'_, Connection> {
        self.conn.lock()
    }

    pub fn initialize_schema(&self) -> Result<()> {
        let conn = self.conn.lock();
        conn.execute_batch(schema::SQL_INIT_SCHEMA)
            .map_err(|e| AppError::Database(format!("Schema initialization failed: {}", e)))?;
        schema::apply_column_migrations(&conn);
        tracing::info!("SQLite schema initialized successfully at {:?}", self.db_path);
        Ok(())
    }

    // --- USER METHODS ---
    pub fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        let conn = self.conn.lock();
        UserRepo::get_by_username(&conn, username)
    }

    pub fn get_all_users(&self) -> Result<Vec<User>> {
        let conn = self.conn.lock();
        UserRepo::get_all(&conn)
    }

    pub fn get_user_by_android_id(&self, android_id: &str) -> Result<Option<User>> {
        let conn = self.conn.lock();
        UserRepo::get_by_android_id(&conn, android_id)
    }

    pub fn insert_or_update_user(&self, user: &User) -> Result<()> {
        let conn = self.conn.lock();
        UserRepo::upsert(&conn, user)
    }

    pub fn delete_user(&self, username: &str) -> Result<bool> {
        let conn = self.conn.lock();
        UserRepo::delete_by_username(&conn, username)
    }

    // --- EMPLOYEE METHODS ---
    pub fn get_employee_by_code(&self, code: &str) -> Result<Option<Employee>> {
        let conn = self.conn.lock();
        EmployeeRepo::get_by_code(&conn, code)
    }

    pub fn save_employee(&self, emp: &Employee) -> Result<()> {
        let conn = self.conn.lock();
        EmployeeRepo::insert_or_update(&conn, emp)
    }

    pub fn list_employees(&self, company_id: &str) -> Result<Vec<Employee>> {
        let conn = self.conn.lock();
        EmployeeRepo::list_by_company(&conn, company_id)
    }

    // --- ATTENDANCE METHODS ---
    pub fn get_attendance_record(&self, username: &str, date: &str) -> Result<Option<AttendanceRecord>> {
        let conn = self.conn.lock();
        AttendanceRepo::get_record(&conn, username, date)
    }

    pub fn save_in_time(&self, rec: &AttendanceRecord) -> Result<()> {
        {
            let conn = self.conn.lock();
            AttendanceRepo::insert_in_time(&conn, rec)?;
        }
        if let Some(ref engine) = self.sharded_engine {
            let status_str = rec.status.to_string();
            let _ = engine.route_and_insert(
                &rec.record_id,
                &rec.username,
                &rec.date,
                rec.in_time.as_deref(),
                rec.out_time.as_deref(),
                &status_str,
            );
        }
        Ok(())
    }

    pub fn save_out_time_detailed(
        &self,
        username: &str,
        date: &str,
        out_time: &str,
        video_path: Option<&str>,
        with_camera: bool,
        with_qr: bool,
        work_hours: f64,
        overtime_hours: f64,
        early_exit_minutes: u32,
        status: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock();
        AttendanceRepo::update_out_time_detailed(
            &conn,
            username,
            date,
            out_time,
            video_path,
            with_camera,
            with_qr,
            work_hours,
            overtime_hours,
            early_exit_minutes,
            status,
        )
    }

    pub fn save_out_time(
        &self,
        username: &str,
        date: &str,
        out_time: &str,
        video_path: Option<&str>,
        with_camera: bool,
        with_qr: bool,
    ) -> Result<()> {
        let conn = self.conn.lock();
        // Compute actual work hours if in_time exists
        let mut work_hours = 8.0f64;
        let overtime_hours = 0.0f64;
        if let Ok(Some(rec)) = AttendanceRepo::get_record(&conn, username, date) {
            if let Some(in_t) = rec.in_time {
                if let (Ok(t_in), Ok(t_out)) = (TimeHelper::parse_time(&in_t), TimeHelper::parse_time(out_time)) {
                    let diff = (t_out - t_in).num_minutes();
                    if diff > 0 {
                        work_hours = diff as f64 / 60.0;
                    }
                }
            }
        }

        AttendanceRepo::update_out_time_detailed(
            &conn,
            username,
            date,
            out_time,
            video_path,
            with_camera,
            with_qr,
            work_hours,
            overtime_hours,
            0,
            None,
        )
    }

    pub fn log_punch(&self, log: &PunchLog) -> Result<()> {
        let conn = self.conn.lock();
        AttendanceRepo::log_punch_event(&conn, log)
    }

    pub fn get_attendance_by_date(&self, date: &str) -> Result<Vec<AttendanceRecord>> {
        let conn = self.conn.lock();
        AttendanceRepo::get_by_date(&conn, date)
    }

    pub fn get_attendance_records_by_range(
        &self,
        from_date: &str,
        to_date: &str,
        username: Option<&str>,
    ) -> Result<Vec<AttendanceRecord>> {
        let conn = self.conn.lock();
        AttendanceRepo::get_records_by_range(&conn, from_date, to_date, username)
    }

    pub fn get_user_punch_logs(&self, username: &str, limit: usize) -> Result<Vec<PunchLog>> {
        let conn = self.conn.lock();
        AttendanceRepo::get_user_punch_logs(&conn, username, limit)
    }

    pub fn get_last_updated_attendance_username(&self) -> Result<Option<String>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT username FROM attendance_records
                 WHERE (in_time IS NOT NULL OR out_time IS NOT NULL)
                 ORDER BY CASE WHEN out_time IS NOT NULL THEN out_time ELSE in_time END DESC
                 LIMIT 1",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        let mut rows = stmt.query([]).map_err(|e| AppError::Database(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| AppError::Database(e.to_string()))? {
            let uname: String = row.get(0).map_err(|e| AppError::Database(e.to_string()))?;
            Ok(Some(uname))
        } else {
            Ok(None)
        }
    }

    // --- COMPANY & HR METHODS ---
    pub fn get_company(&self, id: &str) -> Result<Option<CompanyInfo>> {
        let conn = self.conn.lock();
        CompanyRepo::get_company_by_id(&conn, id)
    }

    pub fn list_companies(&self) -> Result<Vec<CompanyInfo>> {
        let conn = self.conn.lock();
        CompanyRepo::list_companies(&conn)
    }

    pub fn save_company(&self, info: &CompanyInfo) -> Result<()> {
        let conn = self.conn.lock();
        CompanyRepo::upsert_company(&conn, info)
    }

    pub fn get_hr_password(&self, company_uuid: &str) -> Result<Option<HrPassword>> {
        let conn = self.conn.lock();
        CompanyRepo::get_hr_password(&conn, company_uuid)
    }

    pub fn set_hr_password(&self, hr: &HrPassword) -> Result<()> {
        let conn = self.conn.lock();
        CompanyRepo::set_hr_password(&conn, hr)
    }

    pub fn list_departments(&self, company_id: &str) -> Result<Vec<Department>> {
        let conn = self.conn.lock();
        CompanyRepo::list_departments(&conn, company_id)
    }

    pub fn save_department(&self, dept: &Department) -> Result<i64> {
        let conn = self.conn.lock();
        CompanyRepo::save_department(&conn, dept)
    }

    // --- SHIFT METHODS ---
    pub fn get_shift(&self, id: i64) -> Result<Option<Shift>> {
        let conn = self.conn.lock();
        ShiftRepo::get_by_id(&conn, id)
    }

    pub fn create_shift(&self, shift: &Shift) -> Result<i64> {
        let conn = self.conn.lock();
        ShiftRepo::insert(&conn, shift)
    }

    pub fn list_shifts(&self, company_id: &str) -> Result<Vec<Shift>> {
        let conn = self.conn.lock();
        ShiftRepo::list_by_company(&conn, company_id)
    }

    // --- LEAVE METHODS ---
    pub fn apply_leave(&self, app: &LeaveApplication) -> Result<i64> {
        let conn = self.conn.lock();
        LeaveRepo::apply(&conn, app)
    }

    pub fn update_leave_status(&self, app_num: &str, status: LeaveStatus, approver: &str, remarks: Option<&str>) -> Result<()> {
        let conn = self.conn.lock();
        LeaveRepo::update_status(&conn, app_num, status, approver, remarks)
    }

    pub fn is_on_leave_today(&self, username: &str, today: &str) -> Result<bool> {
        let conn = self.conn.lock();
        LeaveRepo::is_employee_on_leave_today(&conn, username, today)
    }

    pub fn list_employee_leaves(&self, emp_code: &str) -> Result<Vec<LeaveApplication>> {
        let conn = self.conn.lock();
        LeaveRepo::list_by_employee(&conn, emp_code)
    }

    // --- HOLIDAY METHODS ---
    pub fn add_holiday(&self, holiday: &Holiday) -> Result<i64> {
        let conn = self.conn.lock();
        HolidayRepo::insert(&conn, holiday)
    }

    pub fn is_holiday(&self, company_id: &str, date: &str) -> Result<bool> {
        let conn = self.conn.lock();
        HolidayRepo::is_holiday(&conn, company_id, date)
    }

    pub fn list_holidays(&self, company_id: &str) -> Result<Vec<Holiday>> {
        let conn = self.conn.lock();
        HolidayRepo::list_all_holidays(&conn, company_id)
    }

    // --- DEVICE METHODS ---
    pub fn register_device(&self, dev: &RegisteredDevice) -> Result<()> {
        let conn = self.conn.lock();
        DeviceRepo::upsert_device(&conn, dev)
    }

    pub fn save_device_session(&self, sess: &DeviceSession) -> Result<()> {
        let conn = self.conn.lock();
        DeviceRepo::save_session(&conn, sess)
    }

    pub fn get_device_session(&self, session_id: &str) -> Result<Option<DeviceSession>> {
        let conn = self.conn.lock();
        DeviceRepo::get_session(&conn, session_id)
    }

    // --- OUTBOX METHODS ---
    pub fn fetch_pending_outbox_events(&self, limit: usize) -> Result<Vec<BigQueryOutboxEvent>> {
        let conn = self.conn.lock();
        OutboxRepo::fetch_pending(&conn, limit)
    }

    pub fn mark_outbox_event_completed(&self, event_id: &str) -> Result<()> {
        let conn = self.conn.lock();
        OutboxRepo::mark_completed(&conn, event_id)
    }

    pub fn mark_outbox_event_failed(&self, event_id: &str, error: &str) -> Result<()> {
        let conn = self.conn.lock();
        OutboxRepo::mark_failed(&conn, event_id, error)
    }

    // --- AUDIT METHODS ---
    pub fn log_security_event(&self, log: &SecurityAuditLog) -> Result<i64> {
        let conn = self.conn.lock();
        AuditRepo::log_event(&conn, log)
    }

    pub fn get_recent_audit_logs(&self, limit: usize) -> Result<Vec<SecurityAuditLog>> {
        let conn = self.conn.lock();
        AuditRepo::list_recent(&conn, limit)
    }

    // --- BIOMETRICS METHODS ---
    pub fn save_face_profile(
        &self,
        username: &str,
        employee_code: Option<&str>,
        embedding: &[f32],
        quality: f32,
        pose: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock();
        BiometricsRepo::save_profile(&conn, username, employee_code, embedding, quality, pose)
    }

    pub fn get_face_profiles(&self, username: &str) -> Result<Vec<FaceProfile>> {
        let conn = self.conn.lock();
        BiometricsRepo::get_profiles_by_username(&conn, username)
    }

    pub fn get_all_face_profiles(&self) -> Result<Vec<FaceProfile>> {
        let conn = self.conn.lock();
        BiometricsRepo::get_all_profiles(&conn)
    }

    pub fn delete_face_profiles(&self, username: &str) -> Result<usize> {
        let conn = self.conn.lock();
        BiometricsRepo::delete_profiles_by_username(&conn, username)
    }

    pub fn delete_all_face_profiles(&self) -> Result<usize> {
        let conn = self.conn.lock();
        BiometricsRepo::delete_all_profiles(&conn)
    }

    pub fn count_enrolled_faces(&self) -> Result<usize> {
        let conn = self.conn.lock();
        BiometricsRepo::count_total_enrolled_profiles(&conn)
    }

    // --- PAYROLL METHODS ---
    pub fn save_salary_structure(&self, salary: &SalaryStructure) -> Result<()> {
        let conn = self.conn.lock();
        PayrollRepo::save_salary_structure(&conn, salary)
    }

    pub fn get_salary_structure(&self, employee_code: &str) -> Result<Option<SalaryStructure>> {
        let conn = self.conn.lock();
        PayrollRepo::get_salary_structure(&conn, employee_code)
    }

    pub fn save_payslip(&self, slip: &PaySlip) -> Result<()> {
        let conn = self.conn.lock();
        PayrollRepo::save_payslip(&conn, slip)
    }

    pub fn get_payslip(&self, employee_code: &str, month_year: &str) -> Result<Option<PaySlip>> {
        let conn = self.conn.lock();
        PayrollRepo::get_payslip(&conn, employee_code, month_year)
    }

    // --- BACKUP & MAINTENANCE METHODS ---
    pub fn vacuum_database(&self) -> Result<()> {
        let conn = self.conn.lock();
        BackupManager::perform_vacuum(&conn)
    }

    pub fn verify_database_integrity(&self) -> Result<String> {
        let conn = self.conn.lock();
        BackupManager::check_integrity(&conn)
    }

    pub fn create_backup<P: AsRef<Path>>(&self, backup_dir: P) -> Result<DatabaseBackupInfo> {
        let conn = self.conn.lock();
        BackupManager::create_backup(&conn, backup_dir)
    }

    pub fn list_backups(&self) -> Result<Vec<DatabaseBackupInfo>> {
        let conn = self.conn.lock();
        BackupManager::list_backups(&conn)
    }

    // --- STATS METHODS ---
    pub fn get_department_stats(&self, date: &str) -> Result<Vec<DepartmentAttendanceStat>> {
        let conn = self.conn.lock();
        StatsRepo::get_department_stats_by_date(&conn, date)
    }

    pub fn get_hourly_punches(&self, date: &str) -> Result<Vec<HourlyPunchDistribution>> {
        let conn = self.conn.lock();
        StatsRepo::get_hourly_punch_distribution(&conn, date)
    }

    pub fn get_monthly_ledger(&self, month: &str) -> Result<Vec<MonthlyEmployeeLedger>> {
        let conn = self.conn.lock();
        StatsRepo::get_monthly_attendance_ledger(&conn, month)
    }

    // --- 50-YEAR / 10-LAKH EMPLOYEE PLUGGABLE STORAGE & PARTITION ENGINE ---
    pub fn list_storage_partitions(&self) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock();
        let mut stmt = conn
            .prepare(
                "SELECT partition_key, table_name, partition_year, partition_month, shard_count, storage_tier, active_driver, row_count, updated_at
                 FROM storage_partitions
                 ORDER BY partition_year DESC, partition_month DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "partition_key": row.get::<_, String>(0)?,
                    "table_name": row.get::<_, String>(1)?,
                    "partition_year": row.get::<_, i32>(2)?,
                    "partition_month": row.get::<_, i32>(3)?,
                    "shard_count": row.get::<_, i32>(4)?,
                    "storage_tier": row.get::<_, String>(5)?,
                    "active_driver": row.get::<_, String>(6)?,
                    "row_count": row.get::<_, i64>(7)?,
                    "updated_at": row.get::<_, Option<String>>(8)?
                }))
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            if let Ok(v) = r {
                list.push(v);
            }
        }
        Ok(list)
    }

    pub fn get_scale_architecture_status(&self) -> Result<serde_json::Value> {
        let partitions = self.list_storage_partitions().unwrap_or_default();
        let physical_shards_summary = self
            .sharded_engine
            .as_ref()
            .and_then(|e| e.inspect_physical_shards_summary().ok())
            .unwrap_or_else(|| serde_json::json!({ "physical_shard_files_count": 0 }));

        let conn = self.conn.lock();

        let mut config_map = serde_json::Map::new();
        if let Ok(mut stmt) = conn.prepare("SELECT config_key, config_value FROM storage_engine_config") {
            if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
                for pair in rows.flatten() {
                    config_map.insert(pair.0, serde_json::Value::String(pair.1));
                }
            }
        }

        let mut indexes = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND (name LIKE 'idx_att_%' OR name LIKE 'idx_users_%' OR name LIKE 'idx_punch_%') ORDER BY name",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for idx in rows.flatten() {
                    indexes.push(idx);
                }
            }
        }

        Ok(serde_json::json!({
            "architecture": "50-Year / 10-Lakh (1,000,000) Employee Physical 64-Shard & Time-Partitioned Storage Engine",
            "active_driver": config_map.get("active_storage_driver").cloned().unwrap_or_else(|| serde_json::json!("SQLITE_WAL_PARTITIONED")),
            "switchable_drivers": [
                "SQLITE_WAL_PARTITIONED (64 Physical Shard DBs + Monthly Physical Tables)",
                "POSTGRESQL_PARTITIONED (Declarative RANGE + HASH 64 Subpartitions)",
                "TIMESCALEDB_HYPERTABLE (Time-Series Chunk Compression)",
                "CLICKHOUSE_CLUSTER (Columnar Analytical Warehouse)"
            ],
            "physical_sharding_and_partitioning": physical_shards_summary,
            "capacity_design": {
                "retention_horizon_years": 50,
                "max_concurrent_employees": 1_000_000,
                "estimated_total_rows_50yr": "18,250,000,000 to 30,000,000,000 rows",
                "employee_hash_shards": 64,
                "pagination_engine": "O(1) Keyset Cursor Pagination on (date DESC, record_id DESC)",
                "storage_tiers": {
                    "HOT": "64 Physical SQLite WAL Shards (data/shards/shard_00..63.db) + 64MB RAM Cache",
                    "WARM": "Physical Monthly Partition Tables (attendance_records_yYYYY_mMM)",
                    "COLD": "Transactional Outbox -> PostgreSQL / BigQuery / Parquet Archive"
                }
            },
            "active_composite_indexes": indexes,
            "registered_partitions": partitions,
            "engine_config": config_map
        }))
    }

    pub fn get_cursor_paginated_attendance(
        &self,
        partition_year: Option<i32>,
        partition_month: Option<i32>,
        after_date: Option<&str>,
        after_id: Option<&str>,
        limit: usize,
    ) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();
        let safe_limit = limit.clamp(1, 500) as i64;
        let conn = self.conn.lock();

        let cursor_date = after_date.unwrap_or("9999-12-31");
        let cursor_id = after_id.unwrap_or("~~~~~~~~~~~~~~~~~~~~");

        // When both year and month are omitted, use the pure O(1) index-backed query without nullable filter scan
        let (query_sql, explain_sql) = if partition_year.is_none() && partition_month.is_none() {
            let q = "SELECT record_id, username, date, in_time, out_time, status, partition_year, partition_month, shard_bucket, storage_tier
                     FROM attendance_records INDEXED BY idx_att_cursor_pagination
                     WHERE (date < ?1 OR (date = ?1 AND record_id < ?2))
                     ORDER BY date DESC, record_id DESC
                     LIMIT ?3";
            (q, "EXPLAIN QUERY PLAN SELECT record_id, username, date FROM attendance_records INDEXED BY idx_att_cursor_pagination WHERE (date < ?1 OR (date = ?1 AND record_id < ?2)) ORDER BY date DESC, record_id DESC LIMIT ?3")
        } else {
            let q = "SELECT record_id, username, date, in_time, out_time, status, partition_year, partition_month, shard_bucket, storage_tier
                     FROM attendance_records
                     WHERE (?1 IS NULL OR partition_year = ?1)
                       AND (?2 IS NULL OR partition_month = ?2)
                       AND (date < ?3 OR (date = ?3 AND record_id < ?4))
                     ORDER BY date DESC, record_id DESC
                     LIMIT ?5";
            (q, "EXPLAIN QUERY PLAN SELECT record_id, username, date FROM attendance_records WHERE (?1 IS NULL OR partition_year = ?1) AND (?2 IS NULL OR partition_month = ?2) AND (date < ?3 OR (date = ?3 AND record_id < ?4)) ORDER BY date DESC, record_id DESC LIMIT ?5")
        };

        let mut explain_plan = Vec::new();
        if partition_year.is_none() && partition_month.is_none() {
            if let Ok(mut estmt) = conn.prepare(explain_sql) {
                if let Ok(rows) = estmt.query_map(rusqlite::params![cursor_date, cursor_id, safe_limit], |r| r.get::<_, String>(3)) {
                    for s in rows.flatten() {
                        explain_plan.push(s);
                    }
                }
            }
        } else if let Ok(mut estmt) = conn.prepare(explain_sql) {
            if let Ok(rows) = estmt.query_map(rusqlite::params![partition_year, partition_month, cursor_date, cursor_id, safe_limit], |r| r.get::<_, String>(3)) {
                for s in rows.flatten() {
                    explain_plan.push(s);
                }
            }
        }

        let mut stmt = conn
            .prepare(query_sql)
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut items = Vec::new();
        if partition_year.is_none() && partition_month.is_none() {
            let rows = stmt
                .query_map(
                    rusqlite::params![cursor_date, cursor_id, safe_limit],
                    |row| {
                        Ok(serde_json::json!({
                            "record_id": row.get::<_, String>(0)?,
                            "username": row.get::<_, String>(1)?,
                            "date": row.get::<_, String>(2)?,
                            "in_time": row.get::<_, Option<String>>(3)?,
                            "out_time": row.get::<_, Option<String>>(4)?,
                            "status": row.get::<_, String>(5)?,
                            "partition_year": row.get::<_, i32>(6)?,
                            "partition_month": row.get::<_, i32>(7)?,
                            "shard_bucket": row.get::<_, i32>(8)?,
                            "storage_tier": row.get::<_, String>(9)?
                        }))
                    },
                )
                .map_err(|e| AppError::Database(e.to_string()))?;
            for r in rows.flatten() {
                items.push(r);
            }
        } else {
            let rows = stmt
                .query_map(
                    rusqlite::params![partition_year, partition_month, cursor_date, cursor_id, safe_limit],
                    |row| {
                        Ok(serde_json::json!({
                            "record_id": row.get::<_, String>(0)?,
                            "username": row.get::<_, String>(1)?,
                            "date": row.get::<_, String>(2)?,
                            "in_time": row.get::<_, Option<String>>(3)?,
                            "out_time": row.get::<_, Option<String>>(4)?,
                            "status": row.get::<_, String>(5)?,
                            "partition_year": row.get::<_, i32>(6)?,
                            "partition_month": row.get::<_, i32>(7)?,
                            "shard_bucket": row.get::<_, i32>(8)?,
                            "storage_tier": row.get::<_, String>(9)?
                        }))
                    },
                )
                .map_err(|e| AppError::Database(e.to_string()))?;
            for r in rows.flatten() {
                items.push(r);
            }
        }

        let elapsed_us = start.elapsed().as_micros();
        let next_cursor = items.last().map(|last| {
            serde_json::json!({
                "after_date": last.get("date").and_then(|v| v.as_str()).unwrap_or(""),
                "after_id": last.get("record_id").and_then(|v| v.as_str()).unwrap_or("")
            })
        });

        Ok(serde_json::json!({
            "pagination_engine": "KEYSET_CURSOR_O1",
            "execution_time_us": elapsed_us,
            "execution_time_ms": (elapsed_us as f64) / 1000.0,
            "explain_query_plan": explain_plan,
            "count": items.len(),
            "limit": safe_limit,
            "next_cursor": next_cursor,
            "records": items
        }))
    }

    pub fn run_empirical_scale_benchmark(&self, rows: usize) -> Result<serde_json::Value> {
        let bench_dir = std::env::temp_dir().join("attendance_scale_benchmark_64shards");
        let _ = std::fs::remove_dir_all(&bench_dir);
        let bench_engine = PhysicalShardedEngine::new(&bench_dir)?;
        let report = bench_engine.run_empirical_benchmark(rows)?;
        let _ = std::fs::remove_dir_all(&bench_dir);
        Ok(report)
    }

    pub fn generate_postgres_migration_ddl(&self) -> String {
        if let Some(ref engine) = self.sharded_engine {
            engine.generate_postgres_declarative_partition_sql(&[1977, 2001, 2025, 2026], 8)
        } else {
            String::new()
        }
    }

    // --- MOBILE MANDATORY GEOFENCE & WI-FI POLICY METHODS ---
    pub fn get_mobile_access_policy(&self) -> Result<MobileAccessPolicy> {
        let conn = self.conn.lock();
        let res = conn.query_row(
            "SELECT policy_id, company_id, location_name, target_latitude, target_longitude,
                    allowed_radius_meters, location_mandatory, wifi_option_enabled,
                    allowed_wifi_ssids, allowed_wifi_bssids, max_gps_accuracy_meters, updated_at
             FROM mobile_geofence_policy
             WHERE policy_id = 'DEFAULT_POLICY'
             LIMIT 1",
            [],
            |row| {
                Ok(MobileAccessPolicy {
                    policy_id: row.get(0)?,
                    company_id: row.get(1)?,
                    location_name: row.get(2)?,
                    target_latitude: row.get(3)?,
                    target_longitude: row.get(4)?,
                    allowed_radius_meters: row.get(5)?,
                    location_mandatory: true, // Always strictly mandatory for mobile
                    wifi_option_enabled: row.get::<_, i32>(7)? != 0,
                    allowed_wifi_ssids: row.get(8)?,
                    allowed_wifi_bssids: row.get(9)?,
                    max_gps_accuracy_meters: row.get(10)?,
                    updated_at: row.get(11)?,
                })
            },
        );

        match res {
            Ok(p) => Ok(p),
            Err(_) => Ok(MobileAccessPolicy {
                policy_id: "DEFAULT_POLICY".to_string(),
                company_id: "HQ_001".to_string(),
                location_name: "Home / Office Authorized Geofence".to_string(),
                target_latitude: 23.8103,
                target_longitude: 90.4125,
                allowed_radius_meters: 150.0,
                location_mandatory: true,
                wifi_option_enabled: false,
                allowed_wifi_ssids: "Office_WiFi,Home_WiFi,SmartAttendance_5G".to_string(),
                allowed_wifi_bssids: String::new(),
                max_gps_accuracy_meters: 250.0,
                updated_at: Some(TimeHelper::now_utc_sqlite_string()),
            }),
        }
    }

    pub fn update_mobile_access_policy(
        &self,
        req: &UpdateMobileAccessPolicyRequest,
    ) -> Result<MobileAccessPolicy> {
        if !req.target_latitude.is_finite()
            || !req.target_longitude.is_finite()
            || req.target_latitude < -90.0
            || req.target_latitude > 90.0
            || req.target_longitude < -180.0
            || req.target_longitude > 180.0
        {
            return Err(AppError::Validation(
                "Invalid GPS coordinates: latitude must be [-90, 90] and longitude [-180, 180]"
                    .to_string(),
            ));
        }

        let current = self.get_mobile_access_policy()?;
        let company_id = req
            .company_id
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&current.company_id)
            .to_string();
        let location_name = req
            .location_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or(&current.location_name)
            .to_string();
        let radius = req
            .allowed_radius_meters
            .unwrap_or(current.allowed_radius_meters)
            .clamp(10.0, 50_000.0);
        let wifi_enabled = req
            .wifi_option_enabled
            .unwrap_or(current.wifi_option_enabled);
        let wifi_ssids = req
            .allowed_wifi_ssids
            .as_ref()
            .map(|s| s.trim().to_string())
            .unwrap_or(current.allowed_wifi_ssids);
        let wifi_bssids = req
            .allowed_wifi_bssids
            .as_ref()
            .map(|s| s.trim().to_string())
            .unwrap_or(current.allowed_wifi_bssids);
        let max_acc = req
            .max_gps_accuracy_meters
            .unwrap_or(current.max_gps_accuracy_meters)
            .clamp(10.0, 5_000.0);
        let now = TimeHelper::now_utc_sqlite_string();

        {
            let conn = self.conn.lock();
            conn.execute(
                "INSERT INTO mobile_geofence_policy (
                    policy_id, company_id, location_name, target_latitude, target_longitude,
                    allowed_radius_meters, location_mandatory, wifi_option_enabled,
                    allowed_wifi_ssids, allowed_wifi_bssids, max_gps_accuracy_meters, updated_at
                ) VALUES (
                    'DEFAULT_POLICY', ?1, ?2, ?3, ?4, ?5, 1, ?6, ?7, ?8, ?9, ?10
                )
                ON CONFLICT(policy_id) DO UPDATE SET
                    company_id = excluded.company_id,
                    location_name = excluded.location_name,
                    target_latitude = excluded.target_latitude,
                    target_longitude = excluded.target_longitude,
                    allowed_radius_meters = excluded.allowed_radius_meters,
                    location_mandatory = 1,
                    wifi_option_enabled = excluded.wifi_option_enabled,
                    allowed_wifi_ssids = excluded.allowed_wifi_ssids,
                    allowed_wifi_bssids = excluded.allowed_wifi_bssids,
                    max_gps_accuracy_meters = excluded.max_gps_accuracy_meters,
                    updated_at = excluded.updated_at",
                rusqlite::params![
                    company_id,
                    location_name,
                    req.target_latitude,
                    req.target_longitude,
                    radius,
                    if wifi_enabled { 1 } else { 0 },
                    wifi_ssids,
                    wifi_bssids,
                    max_acc,
                    now
                ],
            )
            .map_err(|e| AppError::Database(e.to_string()))?;
        }

        self.get_mobile_access_policy()
    }

    pub fn evaluate_mobile_location_and_wifi(
        &self,
        req: &MobileLocationWifiCheckRequest,
    ) -> Result<MobileLocationWifiCheckResponse> {
        let policy = self.get_mobile_access_policy()?;
        let connected_ssid = req
            .wifi_ssid
            .as_ref()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // 1. MANDATORY LOCATION CHECK (Non-negotiable: Mobile App NEVER works without valid GPS location)
        let (location_verified, distance_meters, loc_reason) = match (req.latitude, req.longitude) {
            (Some(lat), Some(lon))
                if lat.is_finite()
                    && lon.is_finite()
                    && (-90.0..=90.0).contains(&lat)
                    && (-180.0..=180.0).contains(&lon)
                    && !(lat.abs() < 0.0001 && lon.abs() < 0.0001) =>
            {
                let r = 6_371_000.0_f64;
                let d_lat = (policy.target_latitude - lat).to_radians();
                let d_lon = (policy.target_longitude - lon).to_radians();
                let a = (d_lat / 2.0).sin().powi(2)
                    + lat.to_radians().cos()
                        * policy.target_latitude.to_radians().cos()
                        * (d_lon / 2.0).sin().powi(2);
                let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
                let dist = r * c;

                if dist <= policy.allowed_radius_meters {
                    (
                        true,
                        Some(dist),
                        format!(
                            "Location verified inside '{}' ({:.1}m <= {:.0}m radius)",
                            policy.location_name, dist, policy.allowed_radius_meters
                        ),
                    )
                } else {
                    (
                        false,
                        Some(dist),
                        format!(
                            "লোকেশন সীমার বাইরে! আপনি নির্ধারিত স্থান ({}) থেকে {:.1} মিটার দূরে আছেন (অনুমোদিত সীমা: {:.0} মিটার)। নির্ধারিত লোকেশনে আসলে মোবাইল অ্যাপ কাজ করবে।",
                            policy.location_name, dist, policy.allowed_radius_meters
                        ),
                    )
                }
            }
            _ => (
                false,
                None,
                "লোকেশন ছাড়া মোবাইল অ্যাপ কাজ করবে না! অনুগ্রহ করে ফোনের GPS Location চালু করুন এবং লোকেশন পারমিশন দিন। (Mandatory GPS Location required — Mobile App cannot work without Location)".to_string(),
            ),
        };

        // 2. CONFIGURABLE WI-FI OPTION CHECK
        let (wifi_verified, wifi_reason) = if !policy.wifi_option_enabled {
            (
                true,
                "Wi-Fi option is available (currently set to optional; GPS Location is strictly enforced)".to_string(),
            )
        } else {
            let allowed_list: Vec<String> = policy
                .allowed_wifi_ssids
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            match &connected_ssid {
                Some(ssid) if allowed_list.is_empty() || allowed_list.contains(&ssid.to_lowercase()) => (
                    true,
                    format!("Connected to authorized Wi-Fi '{}'", ssid),
                ),
                Some(ssid) => (
                    false,
                    format!(
                        "নির্ধারিত Wi-Fi সংযোগ মেলেনি! বর্তমান Wi-Fi '{}' অনুমোদিত তালিকায় ({}) নেই।",
                        ssid, policy.allowed_wifi_ssids
                    ),
                ),
                None => (
                    false,
                    format!(
                        "এই অ্যাপ ব্যবহারের জন্য নির্ধারিত Wi-Fi ({}) এর সাথে সংযুক্ত থাকা আবশ্যক!",
                        policy.allowed_wifi_ssids
                    ),
                ),
            }
        };

        // CRITICAL INVARIANT: Location is ALWAYS mandatory. Even if Wi-Fi is verified, missing/outside location is REJECTED.
        let allowed = location_verified && wifi_verified;
        let final_reason = if !location_verified {
            loc_reason
        } else if !wifi_verified {
            wifi_reason
        } else {
            format!("{} | {}", loc_reason, wifi_reason)
        };

        Ok(MobileLocationWifiCheckResponse {
            allowed,
            location_verified,
            wifi_verified,
            wifi_option_enabled: policy.wifi_option_enabled,
            location_mandatory: true,
            distance_meters,
            allowed_radius_meters: policy.allowed_radius_meters,
            target_location_name: policy.location_name,
            target_latitude: policy.target_latitude,
            target_longitude: policy.target_longitude,
            allowed_wifi_ssids: policy.allowed_wifi_ssids,
            connected_wifi_ssid: connected_ssid,
            reason: final_reason,
        })
    }

    pub fn verify_mobile_access_strict(
        &self,
        req: &MobileLocationWifiCheckRequest,
    ) -> Result<MobileLocationWifiCheckResponse> {
        let check = self.evaluate_mobile_location_and_wifi(req)?;
        if !check.allowed {
            return Err(AppError::Forbidden(check.reason));
        }
        Ok(check)
    }

    // --- MILLIONS-OF-USERS MOBILE SHARDED DIRECTORY & BULK PROVISIONING ---
    pub fn get_cursor_paginated_users(
        &self,
        shard_bucket: Option<i32>,
        search: Option<&str>,
        after_username: Option<&str>,
        limit: usize,
    ) -> Result<serde_json::Value> {
        let start = std::time::Instant::now();
        let safe_limit = limit.clamp(1, 500) as i64;
        let cursor_user = after_username.unwrap_or("").trim().to_uppercase();
        let search_pattern = search
            .map(|s| s.trim().to_uppercase())
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s));

        let conn = self.conn.lock();
        let total_users: i64 = conn
            .query_row("SELECT COUNT(*) FROM users", [], |r| r.get(0))
            .unwrap_or(0);

        let mut stmt = conn
            .prepare(
                "SELECT company_user_uuid, username, company_name, company_uuid, user_designation,
                        user_android_id, register_with_camera, role, status, shard_bucket, created_at
                 FROM users
                 WHERE (?1 IS NULL OR shard_bucket = ?1)
                   AND (?2 IS NULL OR UPPER(username) LIKE ?2 OR UPPER(COALESCE(user_designation, '')) LIKE ?2)
                   AND UPPER(username) > ?3
                 ORDER BY UPPER(username) ASC
                 LIMIT ?4",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(
                rusqlite::params![shard_bucket, search_pattern, cursor_user, safe_limit],
                |row| {
                    Ok(serde_json::json!({
                        "company_user_uuid": row.get::<_, String>(0)?,
                        "username": row.get::<_, String>(1)?,
                        "company_name": row.get::<_, Option<String>>(2)?,
                        "company_uuid": row.get::<_, Option<String>>(3)?,
                        "user_designation": row.get::<_, Option<String>>(4)?,
                        "user_android_id": row.get::<_, Option<String>>(5)?,
                        "register_with_camera": row.get::<_, Option<String>>(6)?,
                        "role": row.get::<_, String>(7)?,
                        "status": row.get::<_, String>(8)?,
                        "shard_bucket": row.get::<_, i32>(9)?,
                        "created_at": row.get::<_, Option<String>>(10)?
                    }))
                },
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut users = Vec::new();
        for r in rows.flatten() {
            users.push(r);
        }

        let next_cursor = users
            .last()
            .and_then(|u| u.get("username").and_then(|v| v.as_str()))
            .map(|s| s.to_string());
        let elapsed_us = start.elapsed().as_micros();

        Ok(serde_json::json!({
            "pagination_engine": "SHARDED_KEYSET_CURSOR_O1",
            "total_registered_users": total_users,
            "sharding_capacity": "64 Hash Shards (Up to 10,000,000+ Mobile Managed Users)",
            "returned_count": users.len(),
            "limit": safe_limit,
            "shard_filter": shard_bucket,
            "next_cursor_username": next_cursor,
            "execution_time_us": elapsed_us,
            "users": users
        }))
    }

    pub fn bulk_provision_sharded_users(
        &self,
        company_id: &str,
        prefix: &str,
        count: usize,
        designation: &str,
    ) -> Result<serde_json::Value> {
        let safe_count = count.clamp(1, 5_000);
        let clean_prefix = prefix
            .trim()
            .to_uppercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>();
        let prefix_final = if clean_prefix.is_empty() {
            "MOB_EMP".to_string()
        } else {
            clean_prefix
        };
        let now = TimeHelper::now_utc_sqlite_string();
        let default_pw_hash = attendance_common::CryptoHelper::sha256_hex("1234");
        let start = std::time::Instant::now();

        let mut conn = self.conn.lock();
        let tx = conn
            .transaction()
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut shards_touched = std::collections::HashSet::new();
        let mut inserted = 0usize;

        for idx in 1..=safe_count {
            let uname = format!("{}_{:06}", prefix_final, idx);
            let shard = schema::compute_shard_bucket(&uname);
            shards_touched.insert(shard);
            let cu_uuid = format!("CU_{}_{:06}", prefix_final, idx);
            let u_uuid = format!("U_{}_{:06}", prefix_final, idx);

            let _ = tx.execute(
                "INSERT OR IGNORE INTO users (
                    company_user_uuid, company_name, company_uuid, username, user_uuid,
                    user_designation, register_with_camera, password, role, status, shard_bucket, created_at, last_updated
                ) VALUES (?1, 'Smart Attendance HQ', ?2, ?3, ?4, ?5, 'N', ?6, 'Employee', 'Active', ?7, ?8, ?8)",
                rusqlite::params![cu_uuid, company_id, uname, u_uuid, designation, default_pw_hash, shard, now],
            );
            inserted += 1;
        }

        tx.commit().map_err(|e| AppError::Database(e.to_string()))?;
        let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

        Ok(serde_json::json!({
            "success": true,
            "provisioned_count": inserted,
            "prefix": prefix_final,
            "shards_distributed_across": shards_touched.len(),
            "execution_time_ms": elapsed_ms,
            "message": format!("Successfully provisioned {} sharded users across {} hash shards from Mobile Admin", inserted, shards_touched.len())
        }))
    }
}


pub trait EnterpriseStorageBackend: Send + Sync {
    fn scale_architecture_status(&self) -> Result<serde_json::Value>;
    fn list_partitions(&self) -> Result<Vec<serde_json::Value>>;
    fn query_cursor_paginated(
        &self,
        partition_year: Option<i32>,
        partition_month: Option<i32>,
        after_date: Option<&str>,
        after_id: Option<&str>,
        limit: usize,
    ) -> Result<serde_json::Value>;
}

impl EnterpriseStorageBackend for DbPool {
    fn scale_architecture_status(&self) -> Result<serde_json::Value> {
        self.get_scale_architecture_status()
    }

    fn list_partitions(&self) -> Result<Vec<serde_json::Value>> {
        self.list_storage_partitions()
    }

    fn query_cursor_paginated(
        &self,
        partition_year: Option<i32>,
        partition_month: Option<i32>,
        after_date: Option<&str>,
        after_id: Option<&str>,
        limit: usize,
    ) -> Result<serde_json::Value> {
        self.get_cursor_paginated_attendance(partition_year, partition_month, after_date, after_id, limit)
    }
}

#[cfg(test)]
mod scale_tests {
    use super::*;

    #[test]
    fn test_50yr_physical_64_shards_and_explain_plan() {
        let tmp_dir = std::env::temp_dir().join("test_att_64shards_unit");
        let _ = std::fs::remove_dir_all(&tmp_dir);
        let engine = PhysicalShardedEngine::new(&tmp_dir).expect("Failed to init 64-shard engine");
        let report = engine
            .run_empirical_benchmark(10_000)
            .expect("Benchmark failed");

        assert_eq!(report["benchmark_rows_inserted"], 10_000);
        assert_eq!(report["physical_shard_files_used"], 64);
        assert_eq!(report["concurrent_rw_lock_errors"], 0);
        assert_eq!(
            report["backup_and_recovery_verification"]["restored_integrity_check"],
            "ok"
        );

        let plan_1977 = report["historical_queries"]["year_1977_50yr_ago"]["explain_query_plan"]
            .as_array()
            .expect("Missing explain plan");
        assert!(!plan_1977.is_empty());
        let plan_str = plan_1977[0].as_str().unwrap_or("");
        assert!(
            plan_str.contains("USING COVERING INDEX") || plan_str.contains("USING INDEX"),
            "Expected index seek in EXPLAIN QUERY PLAN, got: {}",
            plan_str
        );

        let pg_ddl = engine.generate_postgres_declarative_partition_sql(&[1977, 2026], 4);
        assert!(pg_ddl.contains("PARTITION BY RANGE (partition_year, partition_month)"));
        assert!(pg_ddl.contains("PARTITION BY HASH (shard_bucket)"));

        let _ = std::fs::remove_dir_all(&tmp_dir);
    }
}

