pub const SQL_INIT_SCHEMA: &str = r#"
-- 1. Users table (preserving original schema from attendance_v8.py)
CREATE TABLE IF NOT EXISTS users (
    company_user_uuid TEXT PRIMARY KEY,
    company_name TEXT,
    company_uuid TEXT,
    username TEXT UNIQUE NOT NULL,
    user_uuid TEXT UNIQUE,
    user_designation TEXT,
    user_android_id TEXT,
    face_encoding TEXT,
    register_with_camera TEXT DEFAULT 'N',
    password TEXT,
    role TEXT NOT NULL DEFAULT 'Employee',
    status TEXT NOT NULL DEFAULT 'Active',
    email TEXT,
    phone TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    last_updated TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 2. Companies table
CREATE TABLE IF NOT EXISTS company_info (
    company_id TEXT PRIMARY KEY,
    company_name TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL,
    address_road TEXT,
    address_city TEXT,
    contact_person_name TEXT,
    contact_person_designation TEXT,
    contact_person_number TEXT,
    website TEXT,
    email TEXT,
    tax_id TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    last_updated TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 3. Departments table
CREATE TABLE IF NOT EXISTS departments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL REFERENCES company_info(company_id),
    name TEXT NOT NULL,
    code TEXT NOT NULL,
    description TEXT,
    head_employee_id TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    UNIQUE(company_id, name)
);

-- 4. Shifts table
CREATE TABLE IF NOT EXISTS shifts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    code TEXT NOT NULL UNIQUE,
    shift_type TEXT NOT NULL DEFAULT 'RegularDay',
    start_time TEXT NOT NULL DEFAULT '09:00:00',
    end_time TEXT NOT NULL DEFAULT '17:00:00',
    grace_period_minutes INTEGER NOT NULL DEFAULT 15,
    half_day_threshold_hours REAL NOT NULL DEFAULT 4.0,
    min_work_hours_full_day REAL NOT NULL DEFAULT 8.0,
    break_duration_minutes INTEGER NOT NULL DEFAULT 60,
    is_overtime_enabled INTEGER NOT NULL DEFAULT 1,
    overtime_starts_after_minutes INTEGER NOT NULL DEFAULT 30,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 4b. Designations table
CREATE TABLE IF NOT EXISTS designations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL,
    title TEXT NOT NULL,
    grade TEXT,
    department_id INTEGER REFERENCES departments(id),
    description TEXT
);

-- 5. Employees table
CREATE TABLE IF NOT EXISTS employees (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    employee_code TEXT NOT NULL UNIQUE,
    user_uuid TEXT NOT NULL UNIQUE,
    company_id TEXT NOT NULL,
    first_name TEXT NOT NULL,
    last_name TEXT NOT NULL,
    full_name TEXT NOT NULL,
    email TEXT,
    phone TEXT,
    department_id INTEGER REFERENCES departments(id),
    designation_id INTEGER REFERENCES designations(id),
    shift_id INTEGER REFERENCES shifts(id),
    rfid_card_uid TEXT UNIQUE,
    pin_code TEXT,
    employment_type TEXT NOT NULL DEFAULT 'Permanent',
    join_date TEXT NOT NULL,
    end_date TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    avatar_url TEXT,
    emergency_contact_name TEXT,
    emergency_contact_phone TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 6. Attendance records table (preserving exact schema from attendance_v8.py lines 4220-4240)
CREATE TABLE IF NOT EXISTS attendance_records (
    record_id TEXT PRIMARY KEY,
    company_user_uuid TEXT REFERENCES users(company_user_uuid),
    username TEXT NOT NULL,
    date TEXT NOT NULL,
    in_time TEXT,
    out_time TEXT,
    video_path_in TEXT,
    video_path_out TEXT,
    in_time_with_camera TEXT CHECK(in_time_with_camera IN ('Y', 'N')),
    out_time_with_camera TEXT CHECK(out_time_with_camera IN ('Y', 'N')),
    in_time_with_qr TEXT CHECK(in_time_with_qr IN ('Y', 'N')),
    out_time_with_qr TEXT CHECK(out_time_with_qr IN ('Y', 'N')),
    status TEXT NOT NULL DEFAULT 'PRESENT',
    work_hours REAL DEFAULT 0.0,
    overtime_hours REAL DEFAULT 0.0,
    late_minutes INTEGER DEFAULT 0,
    early_exit_minutes INTEGER DEFAULT 0,
    ip_address TEXT,
    device_id TEXT,
    remarks TEXT,
    updated_at TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    UNIQUE(username, date)
);

-- 7. High-resolution Punch Logs (every single tap/scan)
CREATE TABLE IF NOT EXISTS punch_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    punch_time TEXT NOT NULL,
    punch_type TEXT NOT NULL, -- IN_TIME, OUT_TIME
    punch_method TEXT NOT NULL, -- CAMERA, QR, ANDROID_ID, RFID, PIN, MANUAL
    device_id TEXT,
    ip_address TEXT,
    liveness_score REAL,
    verified_by_face INTEGER NOT NULL DEFAULT 0,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 8. Leaves table
CREATE TABLE IF NOT EXISTS leaves (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    application_number TEXT NOT NULL UNIQUE,
    employee_code TEXT NOT NULL,
    username TEXT NOT NULL,
    leave_type TEXT NOT NULL,
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    total_days REAL NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    approved_by_username TEXT,
    decision_remarks TEXT,
    applied_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    decided_at TEXT
);

-- 9. Leave Balances table
CREATE TABLE IF NOT EXISTS leave_balances (
    employee_code TEXT NOT NULL,
    year INTEGER NOT NULL,
    casual_leave_total REAL NOT NULL DEFAULT 10.0,
    casual_leave_used REAL NOT NULL DEFAULT 0.0,
    sick_leave_total REAL NOT NULL DEFAULT 14.0,
    sick_leave_used REAL NOT NULL DEFAULT 0.0,
    annual_leave_total REAL NOT NULL DEFAULT 15.0,
    annual_leave_used REAL NOT NULL DEFAULT 0.0,
    PRIMARY KEY(employee_code, year)
);

-- 10. Holidays table
CREATE TABLE IF NOT EXISTS holidays (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_id TEXT NOT NULL,
    name TEXT NOT NULL,
    holiday_date TEXT NOT NULL,
    holiday_type TEXT NOT NULL DEFAULT 'NationalPublic',
    is_recurring_yearly INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    UNIQUE(company_id, holiday_date)
);

-- 11. Registered Devices table
CREATE TABLE IF NOT EXISTS registered_devices (
    device_id TEXT PRIMARY KEY,
    device_name TEXT NOT NULL,
    android_id TEXT UNIQUE,
    mac_address TEXT,
    ip_address TEXT,
    device_type TEXT NOT NULL DEFAULT 'KIOSK',
    location_name TEXT,
    status TEXT NOT NULL DEFAULT 'Online',
    last_heartbeat TEXT,
    app_version TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 12. Device Pairing Sessions table (Python AndroidIDReceiver parity)
CREATE TABLE IF NOT EXISTS device_sessions (
    session_id TEXT PRIMARY KEY,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    registered_android_id TEXT,
    status TEXT NOT NULL DEFAULT 'PENDING'
);

-- 13. Sync Status table (preserving original Python schema)
CREATE TABLE IF NOT EXISTS sync_status (
    record_id TEXT PRIMARY KEY,
    synced INTEGER DEFAULT 0,
    last_sync_attempt TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    retry_count INTEGER DEFAULT 0
);

-- 14. HR Passwords table (preserving original Python schema)
CREATE TABLE IF NOT EXISTS hr_passwords (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    company_uuid TEXT NOT NULL UNIQUE,
    company_name TEXT NOT NULL,
    password TEXT NOT NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    last_updated TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 15. Google BigQuery Transactional Outbox table
CREATE TABLE IF NOT EXISTS bigquery_outbox (
    event_id TEXT PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING',
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    updated_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 16. Security Audit Logs table
CREATE TABLE IF NOT EXISTS security_audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    actor_username TEXT,
    target_resource TEXT,
    ip_address TEXT,
    severity TEXT NOT NULL DEFAULT 'Info',
    details_json TEXT,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 17. Face Biometric Profiles table
CREATE TABLE IF NOT EXISTS face_profiles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL,
    employee_code TEXT,
    embedding_json TEXT NOT NULL,
    sample_quality REAL NOT NULL DEFAULT 1.0,
    pose TEXT NOT NULL DEFAULT 'FRONTAL',
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 18. Salary Structures table
CREATE TABLE IF NOT EXISTS salary_structures (
    employee_code TEXT PRIMARY KEY,
    basic_salary REAL NOT NULL,
    house_rent_allowance REAL NOT NULL,
    medical_allowance REAL NOT NULL,
    conveyance_allowance REAL NOT NULL,
    gross_salary REAL NOT NULL,
    provident_fund_rate REAL NOT NULL DEFAULT 0.05,
    tax_deduction_rate REAL NOT NULL DEFAULT 0.0,
    currency TEXT NOT NULL DEFAULT 'BDT',
    updated_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- 19. Payslips table
CREATE TABLE IF NOT EXISTS payslips (
    payslip_id TEXT PRIMARY KEY,
    employee_code TEXT NOT NULL,
    employee_name TEXT NOT NULL,
    month_year TEXT NOT NULL,
    working_days INTEGER NOT NULL,
    present_days INTEGER NOT NULL,
    absent_days INTEGER NOT NULL,
    leave_days INTEGER NOT NULL,
    holidays INTEGER NOT NULL,
    total_overtime_hours REAL NOT NULL DEFAULT 0.0,
    base_pay REAL NOT NULL,
    overtime_pay REAL NOT NULL DEFAULT 0.0,
    allowances_total REAL NOT NULL DEFAULT 0.0,
    late_deduction REAL NOT NULL DEFAULT 0.0,
    absence_deduction REAL NOT NULL DEFAULT 0.0,
    provident_fund_deduction REAL NOT NULL DEFAULT 0.0,
    tax_deduction REAL NOT NULL DEFAULT 0.0,
    gross_payable REAL NOT NULL,
    net_payable REAL NOT NULL,
    generated_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    status TEXT NOT NULL DEFAULT 'DRAFT'
);

-- 20. System Backups table
CREATE TABLE IF NOT EXISTS system_backups (
    backup_id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    size_bytes INTEGER NOT NULL,
    checksum_sha256 TEXT NOT NULL,
    total_records INTEGER NOT NULL,
    created_at TEXT DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);

-- Indexes for lightning fast queries
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_attendance_records_user_date ON attendance_records(username, date);
CREATE INDEX IF NOT EXISTS idx_attendance_records_date ON attendance_records(date);
CREATE INDEX IF NOT EXISTS idx_punch_logs_username_time ON punch_logs(username, punch_time);
CREATE INDEX IF NOT EXISTS idx_outbox_status_retry ON bigquery_outbox(status, retry_count);
CREATE INDEX IF NOT EXISTS idx_leaves_employee_status ON leaves(employee_code, status);
CREATE INDEX IF NOT EXISTS idx_audit_time ON security_audit_logs(created_at);
CREATE INDEX IF NOT EXISTS idx_face_profiles_user ON face_profiles(username);
CREATE INDEX IF NOT EXISTS idx_payslips_emp_month ON payslips(employee_code, month_year);
"#;

use rusqlite::Connection;

pub fn apply_column_migrations(conn: &Connection) {
    let migrations = [
        ("company_info", "website", "TEXT"),
        ("company_info", "email", "TEXT"),
        ("company_info", "tax_id", "TEXT"),
        ("users", "role", "TEXT NOT NULL DEFAULT 'Employee'"),
        ("users", "status", "TEXT NOT NULL DEFAULT 'Active'"),
        ("users", "email", "TEXT"),
        ("users", "phone", "TEXT"),
        ("users", "register_with_camera", "TEXT DEFAULT 'N'"),
        ("attendance_records", "status", "TEXT NOT NULL DEFAULT 'PRESENT'"),
        ("attendance_records", "work_hours", "REAL DEFAULT 0.0"),
        ("attendance_records", "overtime_hours", "REAL DEFAULT 0.0"),
        ("attendance_records", "late_minutes", "INTEGER DEFAULT 0"),
        ("attendance_records", "early_exit_minutes", "INTEGER DEFAULT 0"),
        ("attendance_records", "ip_address", "TEXT"),
        ("attendance_records", "device_id", "TEXT"),
        ("attendance_records", "remarks", "TEXT"),
        ("attendance_records", "updated_at", "TEXT"),
    ];

    for (table, col, col_def) in migrations {
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, col, col_def);
        if conn.execute(&sql, []).is_ok() && table == "users" && col == "register_with_camera" {
            let _ = conn.execute(
                "UPDATE users SET register_with_camera = 'Y' WHERE face_encoding IS NOT NULL AND face_encoding != '' AND face_encoding NOT LIKE '[0.0,0.0,%'",
                [],
            );
        }
    }
}
