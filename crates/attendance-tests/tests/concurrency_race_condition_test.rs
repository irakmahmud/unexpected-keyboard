use attendance_common::{RateLimiter, TimeHelper};
use attendance_core::AttendanceEngine;
use attendance_domain::*;
use attendance_tests::TestFixtures;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

#[test]
fn test_concurrent_double_punch_race_condition() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let emp = TestFixtures::create_test_employee(&db, "EMP_RACE_001", "Tariq", "Zaman")
        .expect("Employee creation failed");

    let engine = Arc::new(AttendanceEngine::with_cooldown(db.clone(), 30, 30));

    let success_count = Arc::new(AtomicUsize::new(0));
    let rejected_count = Arc::new(AtomicUsize::new(0));

    let mut handles = Vec::new();

    // Launch 20 concurrent punch attempts at identical timestamp
    for _ in 0..20 {
        let engine_clone = Arc::clone(&engine);
        let emp_code = emp.employee_code.clone();
        let s_count = Arc::clone(&success_count);
        let r_count = Arc::clone(&rejected_count);

        handles.push(thread::spawn(move || {
            let req = PunchRequest {
                username: emp_code,
                method: Some("CAMERA".to_string()),
                pin: None,
                android_id: None,
                video_path: None,
                face_embedding: None,
                latitude: None,
                longitude: None,
                device_id: Some("GATE_01".to_string()),
                remarks: None,
            };

            let res = engine_clone.process_punch(req);
            match res {
                Ok(r) if r.success => {
                    s_count.fetch_add(1, Ordering::SeqCst);
                }
                _ => {
                    r_count.fetch_add(1, Ordering::SeqCst);
                }
            }
        }));
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }

    // Exactly 1 punch must succeed, remaining 19 must be safely rejected by cooldown lock
    assert_eq!(
        success_count.load(Ordering::SeqCst),
        1,
        "Exactly one concurrent punch must succeed"
    );
    assert_eq!(
        rejected_count.load(Ordering::SeqCst),
        19,
        "Remaining 19 concurrent punches must be rejected by cooldown lock"
    );
}

#[test]
fn test_50_concurrent_distinct_employees_punch_in() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");

    // Pre-create 50 distinct employees
    let mut emp_codes = Vec::new();
    for i in 1..=50 {
        let code = format!("EMP_BULK_{:03}", i);
        let emp = TestFixtures::create_test_employee(&db, &code, &format!("First{}", i), &format!("Last{}", i))
            .expect("Employee creation failed");
        emp_codes.push(emp.employee_code);
    }

    let engine = Arc::new(AttendanceEngine::with_cooldown(db.clone(), 0, 0));

    let mut handles = Vec::new();

    for code in emp_codes.clone() {
        let engine_clone = Arc::clone(&engine);
        handles.push(thread::spawn(move || {
            let req = PunchRequest {
                username: code,
                method: Some("RFID".to_string()),
                pin: None,
                android_id: None,
                video_path: None,
                face_embedding: None,
                latitude: None,
                longitude: None,
                device_id: Some("GATE_BULK".to_string()),
                remarks: None,
            };
            engine_clone
                .process_punch(req)
                .expect("Concurrent distinct punch must succeed");
        }));
    }

    for h in handles {
        h.join().expect("Thread panicked");
    }

    // Verify all 50 records exist in SQLite
    let today = TimeHelper::current_date_dhaka();
    let all_records = db
        .get_attendance_by_date(&today)
        .expect("Query failed");

    assert_eq!(
        all_records.len(),
        50,
        "All 50 distinct employees must have their attendance persisted concurrently"
    );
}

#[test]
fn test_rate_limiter_concurrency() {
    let limiter = Arc::new(RateLimiter::new(10.0, 10.0)); // 10 tokens per second
    let client_ip = "192.168.1.100";

    // First 10 immediate requests should succeed
    for _ in 0..10 {
        assert!(limiter.check_allowed(client_ip), "Token should be available");
    }

    // 11th request must be rejected
    assert!(
        !limiter.check_allowed(client_ip),
        "11th request must exceed token bucket limit"
    );
}
