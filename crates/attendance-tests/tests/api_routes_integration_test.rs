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

#[test]
fn test_mobile_mandatory_location_and_wifi_policy_enforcement() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let policy = db.get_mobile_access_policy().expect("Must get default policy");
    assert!(policy.location_mandatory, "Location must always be mandatory for mobile");

    // 1. Missing location must ALWAYS be rejected even if Wi-Fi is provided
    let no_loc = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: None,
            longitude: None,
            gps_accuracy_meters: None,
            wifi_ssid: Some("Office_WiFi".to_string()),
            wifi_bssid: None,
            device_id: Some("MOBILE_TEST".to_string()),
        })
        .expect("Eval failed");
    assert!(!no_loc.allowed, "Mobile access without location must NEVER be allowed");
    assert!(!no_loc.location_verified);
    assert!(
        db.verify_mobile_access_strict(&MobileLocationWifiCheckRequest {
            latitude: None,
            longitude: None,
            gps_accuracy_meters: None,
            wifi_ssid: Some("Office_WiFi".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .is_err(),
        "Strict check without location must return Err(Forbidden)"
    );

    // 2. Outside geofence location (e.g. 5km away) must be rejected
    let outside = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8600),
            longitude: Some(90.4600),
            gps_accuracy_meters: Some(10.0),
            wifi_ssid: Some("Office_WiFi".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(!outside.allowed, "Outside geofence must be rejected");

    // 3. Inside geofence location with Wi-Fi option disabled (default) must be accepted
    let inside_ok = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8103),
            longitude: Some(90.4125),
            gps_accuracy_meters: Some(8.0),
            wifi_ssid: None,
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(inside_ok.allowed, "Inside geofence must be allowed when Wi-Fi option is off");

    // 4. Enable Wi-Fi Option ("এই ওয়াইফাই কানেক্ট হলে কাজ করবে... তবে লোকেশন ছাড়া কাজ করবে না")
    let updated_policy = db
        .update_mobile_access_policy(&UpdateMobileAccessPolicyRequest {
            company_id: Some("HQ_001".to_string()),
            location_name: Some("My Home Geofence".to_string()),
            target_latitude: 23.8103,
            target_longitude: 90.4125,
            allowed_radius_meters: Some(150.0),
            wifi_option_enabled: Some(true),
            allowed_wifi_ssids: Some("MyHomeWiFi_5G,Office_WiFi".to_string()),
            allowed_wifi_bssids: None,
            max_gps_accuracy_meters: Some(200.0),
        })
        .expect("Policy update failed");
    assert!(updated_policy.wifi_option_enabled);
    assert!(updated_policy.location_mandatory);

    // 4a. Inside location + Wrong Wi-Fi -> Rejected
    let wrong_wifi = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8103),
            longitude: Some(90.4125),
            gps_accuracy_meters: Some(10.0),
            wifi_ssid: Some("Unknown_Cafe_WiFi".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(!wrong_wifi.allowed, "Wrong Wi-Fi must be rejected when Wi-Fi option is enabled");
    assert!(wrong_wifi.location_verified);
    assert!(!wrong_wifi.wifi_verified);

    // 4b. Inside location + Correct Wi-Fi -> Accepted
    let both_ok = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8104),
            longitude: Some(90.4125),
            gps_accuracy_meters: Some(10.0),
            wifi_ssid: Some("MyHomeWiFi_5G".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(both_ok.allowed, "Inside location + matching Wi-Fi must be allowed");

    // 4c. Correct Wi-Fi + Missing Location -> Still Rejected!
    let wifi_only_no_loc = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: None,
            longitude: None,
            gps_accuracy_meters: None,
            wifi_ssid: Some("MyHomeWiFi_5G".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(
        !wifi_only_no_loc.allowed,
        "Even with matching Wi-Fi, missing location MUST be rejected"
    );

    // 4d. Coarse/spoofed GPS accuracy (> max_gps_accuracy_meters 200m) -> Rejected!
    let coarse_gps = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8103),
            longitude: Some(90.4125),
            gps_accuracy_meters: Some(1500.0),
            wifi_ssid: Some("MyHomeWiFi_5G".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(
        !coarse_gps.allowed && !coarse_gps.location_verified,
        "Coarse GPS accuracy exceeding max_gps_accuracy_meters must be rejected"
    );

    // 4e. Android '<unknown ssid>' when Wi-Fi option is enabled -> Rejected!
    let unknown_ssid = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8103),
            longitude: Some(90.4125),
            gps_accuracy_meters: Some(12.0),
            wifi_ssid: Some("<unknown ssid>".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(
        !unknown_ssid.allowed && !unknown_ssid.wifi_verified,
        "Android '<unknown ssid>' placeholder must not satisfy Wi-Fi verification"
    );

    // 4f. Wi-Fi option enabled with empty allowed_wifi_ssids and empty allowed_wifi_bssids -> Rejected!
    db.update_mobile_access_policy(&UpdateMobileAccessPolicyRequest {
        company_id: Some("HQ_001".to_string()),
        location_name: Some("My Home Geofence".to_string()),
        target_latitude: 23.8103,
        target_longitude: 90.4125,
        allowed_radius_meters: Some(150.0),
        wifi_option_enabled: Some(true),
        allowed_wifi_ssids: Some("".to_string()),
        allowed_wifi_bssids: Some("".to_string()),
        max_gps_accuracy_meters: Some(200.0),
    })
    .expect("Policy update failed");

    let empty_allowed_list_check = db
        .evaluate_mobile_location_and_wifi(&MobileLocationWifiCheckRequest {
            latitude: Some(23.8103),
            longitude: Some(90.4125),
            gps_accuracy_meters: Some(10.0),
            wifi_ssid: Some("AnyRandomWiFi".to_string()),
            wifi_bssid: None,
            device_id: None,
        })
        .expect("Eval failed");
    assert!(
        !empty_allowed_list_check.allowed && !empty_allowed_list_check.wifi_verified,
        "Empty allowed_wifi_ssids when wifi_option_enabled=true must NOT allow arbitrary Wi-Fi SSIDs"
    );
}

#[test]
fn test_mobile_engine_punch_rejects_without_location_and_accepts_with_location() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_MOB_001", "Sajid", "Hasan")
        .expect("Employee creation failed");
    let engine = attendance_core::AttendanceEngine::with_cooldown(db.clone(), 60, 60);

    // Mobile punch without location must fail and NOT lock user cooldown
    let bad_req = PunchRequest {
        username: emp.employee_code.clone(),
        method: Some("CAMERA".to_string()),
        pin: None,
        android_id: None,
        video_path: None,
        face_embedding: None,
        latitude: None,
        longitude: None,
        device_id: Some("MOBILE_ANDROID_01".to_string()),
        remarks: None,
    };
    assert!(
        engine.process_punch(bad_req).is_err(),
        "Mobile punch without GPS coordinates must be rejected"
    );

    // Immediate retry with valid Home/Office GPS coordinates must succeed
    let good_req = PunchRequest {
        username: emp.employee_code.clone(),
        method: Some("CAMERA".to_string()),
        pin: None,
        android_id: None,
        video_path: None,
        face_embedding: None,
        latitude: Some(23.8103),
        longitude: Some(90.4125),
        device_id: Some("MOBILE_ANDROID_01".to_string()),
        remarks: None,
    };
    let res = engine
        .process_punch(good_req)
        .expect("Mobile punch inside geofence must succeed");
    assert!(res.success);
}

#[test]
fn test_mobile_sharded_bulk_provision_and_cursor_pagination() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let single = db
        .register_mobile_sharded_user("HQ_001", "MOB_SINGLE_01", "Lead Engineer", "1234")
        .expect("Single mobile user registration failed");
    assert_eq!(single["success"], true);
    assert_eq!(single["username"], "MOB_SINGLE_01");

    let prov = db
        .bulk_provision_sharded_users("HQ_001", "MOB_MILLIONS", 120, "Mobile Managed Staff")
        .expect("Bulk provision failed");
    assert_eq!(prov["provisioned_count"], 120);
    assert!(prov["shards_distributed_across"].as_u64().unwrap_or(0) > 10);

    let page1 = db
        .get_cursor_paginated_users(None, Some("MOB_MILLIONS"), None, 50)
        .expect("Cursor page 1 failed");
    assert_eq!(page1["returned_count"], 50);
    let next_cursor = page1["next_cursor_username"]
        .as_str()
        .expect("Must have next cursor");

    let page2 = db
        .get_cursor_paginated_users(None, Some("MOB_MILLIONS"), Some(next_cursor), 50)
        .expect("Cursor page 2 failed");
    assert_eq!(page2["returned_count"], 50);
}

