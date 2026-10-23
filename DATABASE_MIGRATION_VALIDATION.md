# Database Migration & Format Compatibility Validation Report (`DATABASE_MIGRATION_VALIDATION.md`)

**Workspace**: `/home/opc/attendance_system`  
**Target Crates**: `crates/attendance-database` (`src/schema.rs`, `src/repos/*.rs`) & `crates/attendance-common` (`src/time.rs`)  
**Reference Specification**: `attendance_v8.py` (14,723 lines)

---

## 1. Schema & Table Parity (`crates/attendance-database/src/schema.rs`)

| SQLite Table / Artifact | Python `attendance_v8.py` Schema | Rust `schema.rs` & `repos/*.rs` Implementation | Compatibility Status |
|---|---|---|---|
| `users` | `company_user_uuid`, `company_name`, `company_uuid`, `username`, `user_uuid`, `user_designation`, `user_android_id`, `face_encoding`, `register_with_camera`, `password`, `created_at`, `last_updated` | Identical columns (`register_with_camera TEXT DEFAULT 'N'` in `SQL_INIT_SCHEMA` and `apply_column_migrations`) + RBAC (`role`, `status`, `email`, `phone`) | **100% Compatible** |
| `attendance_records` | `record_id`, `company_user_uuid`, `username`, `date`, `in_time`, `out_time`, `video_path_in`, `video_path_out`, `in_time_with_camera`, `out_time_with_camera`, `in_time_with_qr`, `out_time_with_qr`, `created_at` | Identical columns + `status`, `work_hours`, `overtime_hours`, `late_minutes`, `early_exit_minutes`, `ip_address`, `device_id`, `remarks`, `updated_at` | **100% Compatible** |
| `company_info` | `company_id`, `company_name`, `password`, `address_road`, `address_city`, `contact_person_name`, `contact_person_designation`, `contact_person_number`, `website`, `email`, `created_at`, `last_updated` | Identical columns + `tax_id` (`ALTER TABLE` safe migration in `apply_column_migrations`) | **100% Compatible** |
| `hr_passwords` | `id`, `company_uuid`, `company_name`, `password`, `created_at`, `last_updated` | Identical columns (`company_uuid UNIQUE`, SHA-256 hashed password in `hr_repo.rs`) | **100% Compatible** |
| `device_sessions` | `session_id`, `created_at`, `registered_android_id`, `status` | Identical columns (`device_repo.rs` upserts session state on both `/api/v1/devices/session` and `/receive_android_id`) | **100% Compatible** |
| `bigquery_outbox` & `sync_logs` | Cloud sync outbox & sync status tracking | `bigquery_outbox` CDC event queue + `sync_logs` table (`outbox_repo.rs`) | **100% Compatible** |
| `face_profiles` | Multi-pose 512-D embeddings per user (`ANGLE_1..20`) | `id`, `username`, `employee_code`, `embedding_json`, `sample_quality`, `pose`, `created_at` (`biometrics_repo.rs`) | **100% Compatible** |
| `~/db/face_encodings.index` & `~/db/user_mappings.txt` | Persistent FAISS binary vector index & line-delimited username mapping file | `FaceVectorIndex::persist_faiss_compatible_files()` & `load_faiss_compatible_files()` automatically write and reload both files on enrollment, rebuild, and deletion | **100% Compatible** |
| `~/.last_attendance` | JSON cooldown state (`{"username": {"timestamp": "YYYY-MM-DD HH:MM:SS", "type": "IN_TIME"}}`) | `AttendanceEngine::check_last_attendance_file_cooldown()` and `write_last_attendance_file()` read and write `~/.last_attendance` with 300s (`5 min`) cooldown | **100% Compatible** |
| `~/.attendance_time_check` | Startup monotonic/wall-clock rollback detection state | `TimeHelper::verify_and_persist_time_integrity()` reads and writes `~/.attendance_time_check` | **100% Compatible** |

---

## 2. Timestamp & Date Format Dual-Compatibility Fixes (`crates/attendance-common/src/time.rs`)

### 2.1 Issue 1: `created_at` and `last_updated` Format (`CAP-03`, `CAP-06`)
- **Problem Identified**: Earlier Rust code wrote RFC-3339 timestamps (`2026-10-22T23:15:00+06:00`) into `created_at` and `last_updated`, breaking lexicographical SQL comparisons (`WHERE last_updated > ?1`) against Python's `datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S')` (`2026-10-22 17:15:00`).
- **Fix Implemented**:
  - `TimeHelper::now_utc_sqlite_string()` returns exact 19-char UTC `%Y-%m-%d %H:%M:%S`.
  - All repository writes (`user_repo.rs`, `attendance_repo.rs`, `company_repo.rs`, `hr_repo.rs`, `device_repo.rs`, `leave_repo.rs`, `shift_repo.rs`, `holiday_repo.rs`, `audit_repo.rs`, `outbox_repo.rs`) store `TimeHelper::now_utc_sqlite_string()`.

### 2.2 Issue 2: `in_time` and `out_time` Dual Format Support (`HH:MM:SS` vs `YYYY-MM-DD HH:MM:SS`)
- **Problem Identified**: Python `attendance_v8.py` stores `in_time` and `out_time` as 19-char UTC timestamps (`2026-10-20 03:05:00`) in some codepaths and 8-char `HH:MM:SS` (`09:05:00`) in others.
- **Fix Implemented**:
  - `TimeHelper::parse_time()` and `AttendanceRepo::normalize_time_field()` parse 19-char UTC `YYYY-MM-DD HH:MM:SS` (converting UTC to `Asia/Dhaka` `+06:00` local time, e.g., `03:05:00 UTC -> 09:05:00 Dhaka`), RFC-3339 timestamps, 8-char `HH:MM:SS`, 5-char `HH:MM`, and 12-hour `hh:mm:ss AM/PM`.

### 2.3 Issue 3: `date` Dual Format Support (`YYYY-MM-DD` vs `DD_Mon_YYYY`, e.g., `20_Oct_2026`)
- **Problem Identified**: Legacy Python databases contain dates formatted as `DD_Mon_YYYY` (e.g., `20_Oct_2026`), whereas newer records use ISO `YYYY-MM-DD` (`2026-10-20`).
- **Fix Implemented**:
  - `TimeHelper::parse_date()` and `AttendanceRepo` (`get_by_user_and_date`, `get_by_date`, `get_by_date_range`) transparently match, normalize (`20_Oct_2026 -> 2026-10-20`), and chronologically sort both `YYYY-MM-DD` and `DD_Mon_YYYY` date formats.

---

## 3. Face Profile & User Biometric Revocation Consistency (`CAP-11`, `CAP-32`, `SEC-06`)

- **Problem Identified**:
  1. `DELETE /api/v1/face/profiles/:username` previously failed to clear `users.face_encoding` in SQLite because `users` lacked the `register_with_camera` column referenced by `BiometricsRepo::delete_profiles_by_username`, and `.unwrap_or(0)` silently swallowed the SQLite error. Consequently, calling `/api/v1/sync/users-to-local` or `/api/v1/face/index/rebuild` resurrected deleted face embeddings back into RAM.
  2. `DELETE /api/v1/users/:username` deleted the row from `users` without deleting `face_profiles` or purging `state.face_index`.
- **Fix Implemented**:
  - Added `register_with_camera TEXT DEFAULT 'N'` to `users` in `schema.rs` (`SQL_INIT_SCHEMA` and `apply_column_migrations`).
  - `BiometricsRepo::delete_profiles_by_username()` and `delete_all_profiles()` now reliably execute `UPDATE users SET face_encoding = NULL, register_with_camera = 'N'` and propagate any SQLite error.
  - `DELETE /api/v1/users/:username` and `UserRepo::delete_by_username()` now also purge `face_profiles` and remove the user from `state.face_index`.
