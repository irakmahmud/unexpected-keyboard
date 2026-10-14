use attendance_bigquery::BackoffPolicy;
use attendance_domain::*;
use attendance_sync::DeadLetterQueueManager;
use attendance_tests::TestFixtures;
use std::time::Duration;

#[test]
fn test_outbox_event_creation_and_completion() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");

    let event_id = uuid::Uuid::new_v4().to_string();
    let payload = serde_json::json!({
        "employee_code": "EMP_OUT_001",
        "punch_time": "2026-10-15T09:00:00+06:00",
        "direction": "IN",
        "terminal": "GATE_01"
    });

    let now = chrono::Utc::now().to_rfc3339();
    let event = BigQueryOutboxEvent {
        event_id: event_id.clone(),
        table_name: "attendance_logs".to_string(),
        operation: "INSERT".to_string(),
        payload_json: payload.to_string(),
        status: OutboxStatus::Pending.to_string(),
        retry_count: 0,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = db.get_connection();
    attendance_database::repos::OutboxRepo::push(&conn, &event).expect("Insert outbox event failed");
    drop(conn);

    // Fetch pending
    let pending = db.fetch_pending_outbox_events(10).expect("Fetch pending failed");
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].event_id, event_id);
    assert_eq!(pending[0].status, "PENDING");

    // Complete the event
    db.mark_outbox_event_completed(&event_id).expect("Mark completed failed");

    // Pending should now be empty
    let pending_after = db.fetch_pending_outbox_events(10).expect("Fetch pending failed");
    assert!(pending_after.is_empty(), "Completed event must not be returned in pending list");
}

#[test]
fn test_retry_policy_exponential_backoff() {
    let policy = BackoffPolicy::default();

    let delay_0 = policy.calculate_delay(0);
    let delay_1 = policy.calculate_delay(1);
    let delay_2 = policy.calculate_delay(2);

    assert!(delay_1 >= delay_0, "Backoff delay must increase with retry count");
    assert!(delay_2 >= delay_1, "Backoff delay must continue increasing");
    assert!(delay_2 <= Duration::from_secs(60), "Backoff should not exceed max backoff ceiling");
}

#[test]
fn test_dead_letter_queue_handling() {
    let db = TestFixtures::setup_in_memory_db().expect("DB setup failed");
    let dlq = DeadLetterQueueManager::new(db.clone());

    let event_id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let event = BigQueryOutboxEvent {
        event_id: event_id.clone(),
        table_name: "biometric_templates".to_string(),
        operation: "INSERT".to_string(),
        payload_json: "{}".to_string(),
        status: "PENDING".to_string(),
        retry_count: 0,
        last_error: None,
        created_at: now.clone(),
        updated_at: now,
    };

    let conn = db.get_connection();
    attendance_database::repos::OutboxRepo::push(&conn, &event).expect("Insert failed");
    drop(conn);

    // Mark failed multiple times to simulate repeated upstream network outage
    for i in 1..=5 {
        db.mark_outbox_event_failed(&event_id, &format!("Network timeout attempt {}", i))
            .expect("Mark failed failed");
    }

    // Inspect failed items
    let dead_letters = dlq.list_dead_letters(10).expect("List dead letters failed");
    assert!(!dead_letters.is_empty(), "Failed event must be tracked in DLQ");
    assert_eq!(dead_letters[0].event_id, event_id);
    assert!(dead_letters[0].retry_count >= 5);
}
