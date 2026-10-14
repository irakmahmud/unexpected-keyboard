use attendance_core::{GeoPoint, GeofenceEngine, HolidayEngine};
use attendance_domain::*;
use attendance_tests::TestFixtures;

#[test]
fn test_leave_application_and_approval_workflow() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_LEAVE_001", "Mahmud", "Hasan")
        .expect("Employee creation failed");

    let app = LeaveApplication {
        id: None,
        application_number: "LA-2026-0001".to_string(),
        employee_code: emp.employee_code.clone(),
        username: "mahmud".to_string(),
        leave_type: LeaveType::AnnualEarned,
        start_date: "2026-10-20".to_string(),
        end_date: "2026-10-22".to_string(),
        total_days: 3.0,
        reason: "Family vacation".to_string(),
        status: LeaveStatus::Pending,
        approved_by_username: None,
        decision_remarks: None,
        applied_at: Some("2026-10-10T10:00:00Z".to_string()),
        decided_at: None,
    };

    let leave_id = db.apply_leave(&app).expect("Apply leave failed");
    assert!(leave_id > 0);

    // Verify employee has pending leave
    let leaves = db.list_employee_leaves(&emp.employee_code).expect("Query failed");
    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].status, LeaveStatus::Pending);

    // HR Approves leave
    db.update_leave_status("LA-2026-0001", LeaveStatus::Approved, "HR_ADMIN", Some("Approved by Manager"))
        .expect("Approve leave failed");

    // Check if on leave on 2026-10-21
    let is_on_leave = db.is_on_leave_today("mahmud", "2026-10-21").expect("Query failed");
    assert!(is_on_leave, "Employee must be verified as on leave on 2026-10-21");

    // Check if on leave on 2026-10-25 (outside leave range)
    let is_on_leave_after = db.is_on_leave_today("mahmud", "2026-10-25").expect("Query failed");
    assert!(!is_on_leave_after, "Employee must not be on leave outside approved window");
}

#[test]
fn test_holiday_sandwich_rule_evaluation() {
    let leave_dates = vec![
        "2026-10-15".to_string(), // Thursday (working day)
        "2026-10-17".to_string(), // Saturday (following weekend day)
    ];

    // Evaluating 2026-10-16 (Friday weekend)
    let is_sandwiched = HolidayEngine::evaluate_sandwich_rule("2026-10-16", &leave_dates);
    assert!(is_sandwiched, "Friday sandwiched between Thursday and Saturday leave must be detected");

    // Date not sandwiched
    let no_sandwich_dates = vec!["2026-10-15".to_string()];
    let not_sandwiched = HolidayEngine::evaluate_sandwich_rule("2026-10-16", &no_sandwich_dates);
    assert!(!not_sandwiched, "Without subsequent leave, sandwich rule must not apply");
}

#[test]
fn test_geofencing_ray_casting_and_radius() {
    // Office center: Gulshan 2, Dhaka (Lat: 23.7925, Lon: 90.4078)
    let office = GeoPoint {
        latitude: 23.7925,
        longitude: 90.4078,
    };
    let radius_meters = 200.0;

    // Point 1: 50 meters away (inside)
    let inside_point = GeoPoint {
        latitude: 23.7926,
        longitude: 90.4080,
    };
    let dist_inside = GeofenceEngine::haversine_distance_meters(&office, &inside_point);
    assert!(dist_inside <= radius_meters, "Point must be inside geofence perimeter");

    // Point 2: Dhanmondi, Dhaka (Lat: 23.7461, Lon: 90.3742) -> ~6 km away (outside)
    let outside_point = GeoPoint {
        latitude: 23.7461,
        longitude: 90.3742,
    };
    let dist_outside = GeofenceEngine::haversine_distance_meters(&office, &outside_point);
    assert!(dist_outside > radius_meters, "Point far away must be outside geofence");
    assert!(dist_outside > 5000.0, "Dhanmondi is > 5km from Gulshan");
}
