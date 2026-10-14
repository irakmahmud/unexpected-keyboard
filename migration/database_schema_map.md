# Database Schema Map: Python SQLite to Rust SQLx Migration

## Original SQLite Tables Discovered in `attendance_v8.py`

## Additional Operational Tables for Production Rust Architecture

### Table: `bigquery_outbox`
```sql
CREATE TABLE IF NOT EXISTS bigquery_outbox (
    event_id TEXT PRIMARY KEY,
    table_name TEXT NOT NULL,
    operation TEXT NOT NULL, -- INSERT, UPDATE, DELETE
    payload_json TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'PENDING', -- PENDING, IN_FLIGHT, COMPLETED, FAILED
    retry_count INTEGER NOT NULL DEFAULT 0,
    last_error TEXT,
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);
```

### Table: `audit_logs`
```sql
CREATE TABLE IF NOT EXISTS audit_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    event_type TEXT NOT NULL,
    actor_id TEXT,
    resource_id TEXT,
    ip_address TEXT,
    details_json TEXT,
    timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%d %H:%M:%S', 'now', 'utc'))
);
```
