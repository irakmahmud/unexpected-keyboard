# Behavioral Contracts: Python to Rust Behavior Equivalence

## 1. Login Type Determination (`determine_login_type`)
- **Contract**: If employee has no punch today in SQLite local or analytical store -> `IN_TIME`. If employee already has `in_time` today -> `OUT_TIME`.
- **Rust Target**: `attendance-core::attendance::determine_login_type`

## 2. Attendance Cooldown (`check_attendance_cooldown`)
- **Contract**: Prevent duplicate punches within cooldown period (default: 60 seconds) for the same user.
- **Rust Target**: `attendance-core::attendance::validate_cooldown`

## 3. Vector Distance & Face Verification Thresholds
- **Contract**: Calculate L2 Euclidean distance between 512-D normalized float vectors. Distance < 0.60 indicates a verified match. Top-1 nearest neighbor selection.
- **Rust Target**: `attendance-face::vector_search::find_best_match`

## 4. Single Instance Locking
- **Contract**: Ensure only one instance of the service runs at a time using local socket/file lock.
- **Rust Target**: `attendance-common::single_instance::acquire_lock`

## 5. Offline Durability & Outbox Pattern
- **Contract**: All attendance punches must commit locally to SQLite immediately. Cloud sync runs asynchronously and never blocks local punch or causes failure if offline.
- **Rust Target**: `attendance-sync::outbox::OutboxWorker`

