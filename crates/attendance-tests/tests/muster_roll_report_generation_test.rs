use attendance_common::TimeHelper;
use attendance_domain::*;
use attendance_reporting::{
    AnalyticsEngine, HtmlReportRenderer, MusterRollGenerator, PunctualityAnalyzer, ReportMetadata,
};
use attendance_tests::TestFixtures;

#[test]
fn test_monthly_muster_roll_matrix_generation() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_MUST_001", "Shahidul", "Alam")
        .expect("Employee creation failed");

    // Add some attendance records for October 2026
    let record1 = AttendanceRecord {
        record_id: format!("{}_2026-10-01", emp.employee_code),
        company_user_uuid: Some("UUID_EMP_MUST_001".to_string()),
        username: emp.employee_code.clone(),
        date: "2026-10-01".to_string(),
        in_time: Some("08:58:00".to_string()),
        out_time: Some("18:02:00".to_string()),
        video_path_in: None,
        video_path_out: None,
        in_time_with_camera: Some("Y".to_string()),
        out_time_with_camera: Some("Y".to_string()),
        in_time_with_qr: Some("N".to_string()),
        out_time_with_qr: Some("N".to_string()),
        status: AttendanceStatus::Present,
        work_hours: 9.0,
        overtime_hours: 0.0,
        late_minutes: 0,
        early_exit_minutes: 0,
        ip_address: None,
        device_id: Some("GATE_01".to_string()),
        remarks: None,
        created_at: Some(TimeHelper::now_utc_sqlite_string()),
    };
    db.save_in_time(&record1).expect("Save record failed");

    let generator = MusterRollGenerator::new(db.clone());
    let rows = generator
        .generate_monthly_muster_roll(2026, 10, "TECHFIRM_01")
        .expect("Muster roll generation failed");

    assert!(!rows.is_empty(), "Muster roll must contain employee rows");
    let emp_row = rows
        .iter()
        .find(|r| r.employee_code == emp.employee_code)
        .expect("Employee row must exist");

    assert!(emp_row.total_present >= 1);
    assert_eq!(emp_row.department, "ENG");
}

#[test]
fn test_html_muster_roll_rendering() {
    let metadata = ReportMetadata {
        company_name: "TechFirm Global Limited".to_string(),
        title: "Monthly Attendance Muster Roll - October 2026".to_string(),
        month_year: "2026-10".to_string(),
        generated_at: "2026-11-01 00:00:00".to_string(),
        generated_by: "Test Harness".to_string(),
    };

    let generator_rows = vec![];
    let html = HtmlReportRenderer::render_muster_roll_html(&metadata, &generator_rows, 31);

    assert!(html.contains("<!DOCTYPE html>"));
    assert!(html.contains("TechFirm Global Limited"));
    assert!(html.contains("Monthly Attendance Muster Roll"));
    assert!(html.contains("<table"));
}

#[test]
fn test_punctuality_scorecard_grades() {
    // 100% on time -> A+ with bonus
    let card_perfect = PunctualityAnalyzer::compute_scorecard(
        "EMP001",
        "Tanvir Hasan",
        "2026-10",
        22, // 22 working days
        22, // 22 on time
        0,
        0,
        0,
    ).expect("Calculation failed");

    assert_eq!(card_perfect.grade, "A+");
    assert!(card_perfect.score >= 100.0);
    assert_eq!(card_perfect.bonus_points, 5.0);

    // 5 late arrivals, 2 unapproved absences -> penalty deduction
    let card_poor = PunctualityAnalyzer::compute_scorecard(
        "EMP002",
        "Late Worker",
        "2026-10",
        22,
        15,
        5,  // 5 * 3 = 15 penalty
        0,
        2,  // 2 * 10 = 20 penalty -> total 35 penalty
    ).expect("Calculation failed");

    assert_eq!(card_poor.penalty_points, 35.0);
    assert_eq!(card_poor.score, 65.0);
    assert_eq!(card_poor.grade, "C");
}

#[test]
fn test_department_analytics_breakdown() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let analytics = AnalyticsEngine::new(db.clone());

    let metrics = analytics.get_department_breakdown("2026-10-15", "TECHFIRM_01").expect("Analytics failed");
    assert_eq!(metrics.len(), 3, "All 3 seeded departments must be in breakdown");
}
