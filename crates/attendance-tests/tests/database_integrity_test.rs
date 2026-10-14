use attendance_common::TimeHelper;
use attendance_database::DbPool;
use attendance_domain::*;
use attendance_tests::TestFixtures;

#[test]
fn test_database_foreign_key_enforcement() {
    let db: DbPool = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let conn = db.get_connection();

    // Verify foreign keys pragma is active
    let fk: i64 = conn
        .query_row("PRAGMA foreign_keys;", [], |r| r.get(0))
        .expect("PRAGMA query failed");
    assert_eq!(fk, 1, "Foreign keys MUST be strictly enabled");

    // Attempting to insert an employee with invalid department ID must fail
    let invalid_emp_res = conn.execute(
        "INSERT INTO employees (employee_code, user_uuid, company_id, first_name, last_name, full_name, department_id, join_date)
         VALUES ('EMP_BAD', 'UUID_BAD', 'TECHFIRM_01', 'Bad', 'Emp', 'Bad Emp', 99999, '2026-01-01')",
        [],
    );

    assert!(
        invalid_emp_res.is_err(),
        "Inserting employee with non-existent department_id 99999 must violate foreign key"
    );
}

#[test]
fn test_transaction_rollback_on_error() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let mut conn = db.get_connection();

    let tx = conn.transaction().expect("Begin transaction failed");

    // Valid insert inside transaction
    tx.execute(
        "INSERT INTO shifts (company_id, name, code, shift_type, start_time, end_time)
         VALUES ('TECHFIRM_01', 'Temporary Shift', 'TEMP_01', 'RegularDay', '08:00:00', '16:00:00')",
        [],
    ).expect("Insert shift failed");

    // Deliberate error: duplicate unique code
    let dup_res = tx.execute(
        "INSERT INTO shifts (company_id, name, code, shift_type, start_time, end_time)
         VALUES ('TECHFIRM_01', 'Duplicate Shift', 'TEMP_01', 'RegularDay', '08:00:00', '16:00:00')",
        [],
    );
    assert!(dup_res.is_err(), "Duplicate unique code must error");

    // Rollback
    tx.rollback().expect("Rollback failed");

    // Verify 'TEMP_01' does not exist in shifts
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM shifts WHERE code = 'TEMP_01'", [], |r| r.get(0))
        .expect("Query failed");

    assert_eq!(count, 0, "Rolled back transaction must leave zero persistent rows");
}

#[test]
fn test_security_audit_log_recording_and_query() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");

    let now = TimeHelper::now_utc_sqlite_string();
    let audit = SecurityAuditLog {
        id: None,
        event_type: "ADMIN_LOGIN".to_string(),
        actor_username: Some("admin".to_string()),
        target_resource: Some("AUTH".to_string()),
        ip_address: Some("192.168.1.50".to_string()),
        severity: AuditSeverity::Info,
        details_json: Some("Successful multi-factor authentication".to_string()),
        created_at: now.clone(),
    };

    let log_id = db.log_security_event(&audit).expect("Audit log failed");
    assert!(log_id > 0);

    let logs = db.get_recent_audit_logs(10).expect("Query failed");
    assert!(!logs.is_empty());
    assert_eq!(logs[0].actor_username.as_deref(), Some("admin"));
    assert_eq!(logs[0].event_type, "ADMIN_LOGIN");
}
