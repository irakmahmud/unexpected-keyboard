use attendance_common::TimeHelper;
use attendance_database::DbPool;
use attendance_domain::*;
use attendance_tests::TestFixtures;

#[test]
fn test_api_system_health_and_info_payload() {
    let db: DbPool = TestFixtures::setup_in_memory_db().expect("DB setup failed");

    // Verify system diagnostic payload structure
    let conn = db.get_connection();
    let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0)).expect("Pragma error");
    let foreign_keys: i64 = conn.query_row("PRAGMA foreign_keys;", [], |r| r.get(0)).expect("Pragma error");

    assert_eq!(foreign_keys, 1, "Foreign keys must be strictly enforced");

    let payload = serde_json::json!({
        "status": "healthy",
        "service": "attendance-api",
        "version": "1.0.0",
        "runtime": "pure-rust-axum",
        "database": {
            "engine": "sqlite-wal",
            "journal_mode": journal_mode,
            "foreign_keys": foreign_keys == 1
        },
        "server_time": TimeHelper::now_utc_sqlite_string(),
        "timezone": "Asia/Dhaka"
    });

    assert_eq!(payload["status"], "healthy");
    assert_eq!(payload["runtime"], "pure-rust-axum");
    assert_eq!(payload["database"]["foreign_keys"], true);
}

#[test]
fn test_api_employee_registration_and_list_serialization() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");

    let emp = TestFixtures::create_test_employee(&db, "EMP_API_001", "Tanvir", "Ahmed")
        .expect("Employee creation failed");

    let employees = db.list_employees("TECHFIRM_01").expect("Query failed");
    assert!(!employees.is_empty());

    let json_bytes = serde_json::to_vec(&employees).expect("Serialization failed");
    let deserialized: Vec<Employee> = serde_json::from_slice(&json_bytes).expect("Deserialization failed");

    assert_eq!(deserialized.len(), employees.len());
    assert_eq!(deserialized[0].employee_code, emp.employee_code);
    assert_eq!(deserialized[0].full_name, "Tanvir Ahmed");
}

#[test]
fn test_api_punch_request_and_response_serialization() {
    let req = PunchRequest {
        username: "EMP_API_002".to_string(),
        method: Some("CAMERA".to_string()),
        pin: None,
        android_id: None,
        video_path: Some("/records/cam_01.mp4".to_string()),
        face_embedding: Some(vec![0.044f32; 512]),
        latitude: Some(23.8103),
        longitude: Some(90.4125),
        device_id: Some("GATE_API_01".to_string()),
        remarks: Some("Verified biometric scan".to_string()),
    };

    let req_json = serde_json::to_string(&req).expect("Serialization error");
    let parsed_req: PunchRequest = serde_json::from_str(&req_json).expect("Deserialization error");

    assert_eq!(parsed_req.username, "EMP_API_002");
    assert_eq!(parsed_req.method.as_deref(), Some("CAMERA"));
    assert_eq!(parsed_req.face_embedding.as_ref().map(|v| v.len()), Some(512));

    let res = PunchResponse {
        success: true,
        login_type: "IN_TIME".to_string(),
        username: "EMP_API_002".to_string(),
        date: "2026-10-15".to_string(),
        timestamp: "2026-10-15T09:00:00+06:00".to_string(),
        record_id: "EMP_API_002_2026-10-15".to_string(),
        shift_status: Some("ON_TIME".to_string()),
        message: "Attendance recorded successfully".to_string(),
        late_minutes: Some(0),
        overtime_minutes: Some(0),
    };

    let res_json = serde_json::to_string(&res).expect("Serialization error");
    let parsed_res: PunchResponse = serde_json::from_str(&res_json).expect("Deserialization error");

    assert!(parsed_res.success);
    assert_eq!(parsed_res.login_type, "IN_TIME");
    assert_eq!(parsed_res.record_id, "EMP_API_002_2026-10-15");
}

#[test]
fn test_api_batch_punch_serialization() {
    let mut punches = Vec::new();
    for i in 1..=10 {
        punches.push(PunchRequest {
            username: format!("EMP_BATCH_{:03}", i),
            method: Some("RFID".to_string()),
            pin: None,
            android_id: None,
            video_path: None,
            face_embedding: None,
            latitude: None,
            longitude: None,
            device_id: Some("TURNSTILE_01".to_string()),
            remarks: None,
        });
    }

    let batch = BatchPunchRequest {
        punches,
        device_id: "TURNSTILE_01".to_string(),
        batch_id: Some(uuid::Uuid::new_v4().to_string()),
    };

    let batch_json = serde_json::to_string(&batch).expect("Batch serialization error");
    let parsed_batch: BatchPunchRequest = serde_json::from_str(&batch_json).expect("Batch parse error");

    assert_eq!(parsed_batch.punches.len(), 10);
    assert_eq!(parsed_batch.device_id, "TURNSTILE_01");
}

#[test]
fn test_create_user_request_designation_and_android_id_aliases() {
    let raw_ui_json = r#"{
        "username": "UI_TEST_USER",
        "designation": "QA Engineer",
        "android_id": "ANDR_UI_999",
        "password": "secret"
    }"#;
    let parsed: CreateUserRequest = serde_json::from_str(raw_ui_json).expect("Must parse UI payload");
    assert_eq!(parsed.username, "UI_TEST_USER");
    assert_eq!(parsed.user_designation.as_deref(), Some("QA Engineer"));
    assert_eq!(parsed.user_android_id.as_deref(), Some("ANDR_UI_999"));
}
