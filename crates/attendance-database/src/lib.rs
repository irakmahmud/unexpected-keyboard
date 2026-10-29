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

