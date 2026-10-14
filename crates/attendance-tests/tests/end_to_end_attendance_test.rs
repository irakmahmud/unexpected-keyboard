use attendance_common::TimeHelper;
use attendance_core::AttendanceEngine;
use attendance_domain::*;
use attendance_tests::TestFixtures;

#[test]
fn test_standard_punch_in_and_punch_out_cycle() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_TEST_001", "Rahim", "Uddin")
        .expect("Employee creation failed");

    let engine = AttendanceEngine::with_cooldown(db.clone(), 0, 0); // 0s cooldown for immediate out punch
    let today = TimeHelper::current_date_dhaka();

    // 1. Punch In
    let req_in = PunchRequest {
        username: emp.employee_code.clone(),
        method: Some("CAMERA".to_string()),
        pin: None,
        android_id: None,
        video_path: Some("/tmp/in.mp4".to_string()),
        face_embedding: None,
        latitude: Some(23.8103),
        longitude: Some(90.4125),
        device_id: Some("DEV_GATE_01".to_string()),
        remarks: None,
    };

    let res_in = engine.process_punch(req_in).expect("Punch in should succeed");
    assert!(res_in.success, "Punch-in must be successful");
    assert_eq!(res_in.login_type, "IN_TIME");

    // Verify record in SQLite database
    let record = db
        .get_attendance_record(&emp.employee_code, &today)
        .expect("Query failed")
        .expect("Attendance record must exist");

    assert!(record.in_time.is_some());
    assert!(record.out_time.is_none());

    // 2. Punch Out
    let req_out = PunchRequest {
        username: emp.employee_code.clone(),
        method: Some("CAMERA".to_string()),
        pin: None,
        android_id: None,
        video_path: Some("/tmp/out.mp4".to_string()),
        face_embedding: None,
        latitude: Some(23.8103),
        longitude: Some(90.4125),
        device_id: Some("DEV_GATE_01".to_string()),
        remarks: None,
    };

    let res_out = engine.process_punch(req_out).expect("Punch out should succeed");
    assert!(res_out.success, "Punch-out must be successful");
    assert_eq!(res_out.login_type, "OUT_TIME");

    // Verify updated record in SQLite database
    let record_updated = db
        .get_attendance_record(&emp.employee_code, &today)
        .expect("Query failed")
        .expect("Record must exist");

    assert!(record_updated.out_time.is_some());
}

#[test]
fn test_double_punch_cooldown_rejection() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_TEST_002", "Karim", "Chowdhury")
        .expect("Employee creation failed");

    let engine = AttendanceEngine::with_cooldown(db.clone(), 60, 60); // 60s cooldown

    let req1 = PunchRequest {
        username: emp.employee_code.clone(),
        method: Some("CAMERA".to_string()),
        pin: None,
        android_id: None,
        video_path: None,
        face_embedding: None,
        latitude: None,
        longitude: None,
        device_id: Some("DEV_GATE_01".to_string()),
        remarks: None,
    };

    // First punch succeeds
    let res1 = engine.process_punch(req1.clone()).expect("First punch should succeed");
    assert!(res1.success);

    // Second immediate punch within 60s window must be rejected
    let res2 = engine.process_punch(req1);
    assert!(
        res2.is_err(),
        "Immediate second punch within cooldown window must be rejected"
    );
}

#[test]
fn test_multiple_employee_punches() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp1 = TestFixtures::create_test_employee(&db, "EMP_TEST_003", "Nasir", "Ahmed")
        .expect("Employee creation failed");
    let emp2 = TestFixtures::create_test_employee(&db, "EMP_TEST_004", "Fatima", "Begum")
        .expect("Employee creation failed");

    let engine = AttendanceEngine::with_cooldown(db.clone(), 0, 0);

    let req1 = PunchRequest {
        username: emp1.employee_code.clone(),
        method: Some("RFID".to_string()),
        pin: None,
        android_id: None,
        video_path: None,
        face_embedding: None,
        latitude: None,
        longitude: None,
        device_id: Some("DEV_GATE_01".to_string()),
        remarks: None,
    };

    let req2 = PunchRequest {
        username: emp2.employee_code.clone(),
        method: Some("RFID".to_string()),
        pin: None,
        android_id: None,
        video_path: None,
        face_embedding: None,
        latitude: None,
        longitude: None,
        device_id: Some("DEV_GATE_02".to_string()),
        remarks: None,
    };

    let res1 = engine.process_punch(req1).expect("Emp 1 punch should succeed");
    let res2 = engine.process_punch(req2).expect("Emp 2 punch should succeed");

    assert!(res1.success);
    assert!(res2.success);

    let today = TimeHelper::current_date_dhaka();
    let records = db.get_attendance_by_date(&today).expect("Query failed");
    assert_eq!(records.len(), 2, "Both attendance punches must be recorded");
}
