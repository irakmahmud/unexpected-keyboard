pub mod repos;
pub mod schema;

use attendance_common::{AppError, Result};
use attendance_domain::*;
use parking_lot::Mutex;
use repos::*;
use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone)]
pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
    db_path: PathBuf,
}

impl DbPool {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let db_path = path.as_ref().to_path_buf();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::Database(format!("Failed to open SQLite database: {}", e)))?;

        // Production SQLite PRAGMAs: WAL mode, synchronous normal, foreign keys enabled
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;",
        )
        .map_err(|e| AppError::Database(format!("Failed to set SQLite pragmas: {}", e)))?;

        let pool = Self {
            conn: Arc::new(Mutex::new(conn)),
            db_path,
        };

        pool.initialize_schema()?;
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
        };
        pool.initialize_schema()?;
        Ok(pool)
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
        let conn = self.conn.lock();
        AttendanceRepo::insert_in_time(&conn, rec)
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
        let mut overtime_hours = 0.0f64;
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
}

