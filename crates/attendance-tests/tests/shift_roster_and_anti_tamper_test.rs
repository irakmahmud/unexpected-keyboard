use attendance_core::{AntiTamperService, RosterScheduler, ShiftCalculator};
use attendance_domain::*;

#[test]
fn test_cross_midnight_night_shift_calculation() {
    let night_shift = Shift {
        id: Some(2),
        company_id: "TECHFIRM_01".to_string(),
        name: "Night Cross-Midnight Shift".to_string(),
        code: "NIGHT_21_06".to_string(),
        shift_type: ShiftType::NightCrossMidnight,
        start_time: "21:00:00".to_string(),
        end_time: "06:00:00".to_string(), // Crosses into next morning
        grace_period_minutes: 15,
        half_day_threshold_hours: 4.0,
        min_work_hours_full_day: 9.0,
        break_duration_minutes: 60,
        is_overtime_enabled: true,
        overtime_starts_after_minutes: 30,
        is_active: true,
        created_at: Some("2026-01-01 00:00:00".to_string()),
    };

    let in_time = "21:20:00"; // 20 mins after 21:00 (exceeds 15m grace)
    let out_time = "06:00:00";

    let eval = ShiftCalculator::evaluate_attendance(Some(in_time), Some(out_time), &night_shift);

    assert_eq!(eval.status, AttendanceStatus::Late);
    assert_eq!(eval.late_minutes, 20);
}

#[test]
fn test_anti_tamper_geofence_and_clock_integrity() {
    let office_lat = 23.8103;
    let office_lon = 90.4125;
    let allowed_radius = 500.0; // 500 meters

    // Location inside perimeter (30m away)
    let inside_lat = Some(23.8105);
    let inside_lon = Some(90.4126);
    let inside_check = AntiTamperService::verify_geofence(
        inside_lat,
        inside_lon,
        office_lat,
        office_lon,
        allowed_radius,
    );
    assert!(inside_check.is_ok(), "Inside geofence location must be accepted");

    // Location outside perimeter (2.5 km away)
    let outside_lat = Some(23.8300);
    let outside_lon = Some(90.4300);
    let outside_check = AntiTamperService::verify_geofence(
        outside_lat,
        outside_lon,
        office_lat,
        office_lon,
        allowed_radius,
    );
    assert!(
        outside_check.is_err(),
        "Location 2.5km away must trigger geofence violation"
    );

    // Clock integrity check: current local time must have minimal drift
    let now_str = chrono::Utc::now().to_rfc3339();
    let clock_ok = AntiTamperService::verify_clock_integrity(Some(&now_str), 300);
    assert!(clock_ok.is_ok(), "Normal synchronized clock should have zero drift");
}

#[test]
fn test_roster_scheduler_auto_match_shift_by_punch_time() {
    let morning = Shift {
        id: Some(1),
        company_id: "TECHFIRM_01".to_string(),
        name: "Morning Shift".to_string(),
        code: "MORN".to_string(),
        shift_type: ShiftType::Morning,
        start_time: "07:00:00".to_string(),
        end_time: "15:30:00".to_string(),
        grace_period_minutes: 15,
        half_day_threshold_hours: 4.0,
        min_work_hours_full_day: 8.5,
        break_duration_minutes: 30,
        is_overtime_enabled: true,
        overtime_starts_after_minutes: 15,
        is_active: true,
        created_at: None,
    };

    let regular = Shift {
        id: Some(2),
        company_id: "TECHFIRM_01".to_string(),
        name: "Regular Day Shift".to_string(),
        code: "REG".to_string(),
        shift_type: ShiftType::RegularDay,
        start_time: "09:00:00".to_string(),
        end_time: "18:00:00".to_string(),
        grace_period_minutes: 15,
        half_day_threshold_hours: 4.0,
        min_work_hours_full_day: 9.0,
        break_duration_minutes: 60,
        is_overtime_enabled: true,
        overtime_starts_after_minutes: 15,
        is_active: true,
        created_at: None,
    };

    let evening = Shift {
        id: Some(3),
        company_id: "TECHFIRM_01".to_string(),
        name: "Evening Shift".to_string(),
        code: "EVE".to_string(),
        shift_type: ShiftType::Evening,
        start_time: "16:00:00".to_string(),
        end_time: "00:00:00".to_string(),
        grace_period_minutes: 15,
        half_day_threshold_hours: 4.0,
        min_work_hours_full_day: 8.0,
        break_duration_minutes: 30,
        is_overtime_enabled: true,
        overtime_starts_after_minutes: 15,
        is_active: true,
        created_at: None,
    };

    let shifts = vec![morning, regular, evening];

    // Punch at 07:05:00 should auto-match Morning Shift
    let matched_morn = RosterScheduler::match_shift_by_punch_time("07:05:00", &shifts);
    assert!(matched_morn.is_some());
    assert_eq!(matched_morn.unwrap().code, "MORN");

    // Punch at 08:55:00 should auto-match Regular Day Shift
    let matched_reg = RosterScheduler::match_shift_by_punch_time("08:55:00", &shifts);
    assert!(matched_reg.is_some());
    assert_eq!(matched_reg.unwrap().code, "REG");

    // Punch at 15:50:00 should auto-match Evening Shift
    let matched_eve = RosterScheduler::match_shift_by_punch_time("15:50:00", &shifts);
    assert!(matched_eve.is_some());
    assert_eq!(matched_eve.unwrap().code, "EVE");
}
