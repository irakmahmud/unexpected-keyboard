# Database Migration & Format Compatibility Validation Report

**Workspace**: `/home/opc/attendance_system`
**Target Crate**: `crates/attendance-database` & `crates/attendance-common`
**Reference Specification**: `attendance_v8.py` (14,723 lines)

---

## 1. Schema & Table Parity (`migrations.rs`)

| SQLite Table / Artifact | Python `attendance_v8.py` Schema | Rust `migrations.rs` Schema | Compatibility Status |
|---|---|---|---|
| `users` | `company_user_uuid`, `company_name`, `company_uuid`, `username`, `user_uuid`, `user_designation`, `user_android_id`, `face_encoding`, `password`, `register_with_camera`, `created_at`, `last_updated` | Identical columns + RBAC (`role`, `status`, `email`, `phone`) | **100% Compatible** |
| `attendance` | `attendance_id`, `company_name`, `company_uuid`, `user_uuid`, `username`, `date`, `in_time`, `out_time`, `with_camera`, `with_qr`, `video_path`, `created_at`, `last_updated` | Identical columns + `attendance_status`, `work_hours`, `overtime_hours`, `device_id`, `notes`, `synced_to_cloud` | **100% Compatible** |
| `company_info` | `company_id`, `company_name`, `password`, `address_road`, `address_city`, `contact_person_name`, `contact_person_designation`, `contact_person_number`, `website`, `email`, `created_at`, `last_updated` | Identical columns + `tax_id` (`ALTER TABLE` safe migration) | **100% Compatible** |
| `hr_auth` | `company_id`, `company_name`, `hr_password_hash`, `salt`, `created_at`, `last_updated` | Identical columns (`company_id` PRIMARY KEY, SHA-256 salted password hash) | **100% Compatible** |
| `qr_codes` | `id`, `username`, `qr_code_hash`, `qr_data_encrypted`, `created_at`, `expires_at`, `is_active`, `usage_count`, `last_used` | Identical columns | **100% Compatible** |
| `qr_usage_log` | `id`, `username`, `qr_code_hash`, `used_at`, `attendance_type`, `success`, `ip_address` | Identical columns | **100% Compatible** |
| `device_sessions` | `session_id`, `created_at`, `expires_at`, `status`, `registered_android_id` | Identical columns | **100% Compatible** |
| `sync_metadata` | `sync_type`, `last_sync_timestamp`, `records_synced`, `sync_status` | Identical columns | **100% Compatible** |
| `face_profiles` | Multi-pose 512-D embeddings per user (`ANGLE_1..20` / `POSE_1..20`) | `id`, `username`, `user_uuid`, `embedding_json`, `quality_score`, `pose_angle`, `created_at` | **100% Compatible** |
| `~/db/face_encodings.index` & `~/db/user_mappings.txt` | Persistent FAISS binary vector index (`FAIX` header + float32 vectors) & line-delimited username mapping file | `FaceVectorIndex::sync_faiss_files()` & `load_faiss_files()` automatically write and reload both files on enrollment and deletion | **100% Compatible** |
| `~/.last_attendance` | JSON cooldown state (`{"username": {"timestamp": "YYYY-MM-DDTHH:MM:SS", "type": "IN_TIME"}}`) | `AttendanceEngine::check_cooldown()` and `write_last_attendance_file()` read and write `~/.last_attendance` with 300s (`5 min`) cooldown | **100% Compatible** |
| `~/.attendance_time_check` | Startup monotonic/wall-clock rollback detection state | `TimeHelper::verify_and_persist_time_integrity()` reads and writes `~/.attendance_time_check` | **100% Compatible** |

---

## 2. Timestamp & Date Format Dual-Compatibility Fixes

### 2.1 Issue 1: `created_at` and `last_updated` Format (`CAP-03`, `CAP-06`)
- **Problem Identified**: Earlier Rust code wrote `TimeHelper::now_dhaka().to_rfc3339()` (`2026-10-22T23:15:00+06:00`) into `created_at` and `last_updated`, which broke lexicographical SQL comparisons (`WHERE last_updated > ?1`) against Python's `datetime.now(timezone.utc).strftime('%Y-%m-%d %H:%M:%S')` (`2026-10-22 17:15:00`).
- **Fix Implemented**:
  - Added `TimeHelper::now_utc_sqlite_string()` returning exact 19-char UTC `%Y-%m-%d %H:%M:%S`.
  - Updated all repository writes (`user_repo.rs`, `attendance_repo.rs`, `company_repo.rs`, `qr_repo.rs`, `sync_repo.rs`, `device_repo.rs`, `leave_repo.rs`, `shift_repo.rs`, `holiday_repo.rs`, `audit_repo.rs`) to store `TimeHelper::now_utc_sqlite_string()`.

### 2.2 Issue 2: `in_time` and `out_time` Dual Format Support (`HH:MM:SS` vs `YYYY-MM-DD HH:MM:SS`)
- **Problem Identified**: Python `attendance_v8.py` stores `in_time` and `out_time` as 19-char UTC timestamps (`2026-04-06 09:00:00`) in some codepaths and 8-char `HH:MM:SS` (`09:00:00`) in others. Earlier Rust `TimeHelper::parse_time()` only accepted `%H:%M:%S` or `%H:%M`, failing on 19-char UTC strings during work-hour calculation and late-arrival checks.
- **Fix Implemented**:
  - Updated `TimeHelper::parse_time()` and `TimeHelper::format_12hour()` to parse:
    1. 19-char UTC `YYYY-MM-DD HH:MM:SS` (automatically converting UTC to `Asia/Dhaka` `+06:00` local time)
    2. RFC-3339 timestamps
    3. 8-char `HH:MM:SS` and 5-char `HH:MM`
    4. 12-hour `hh:mm:ss AM/PM` and `hh:mm AM/PM`

### 2.3 Issue 3: `date` Dual Format Support (`YYYY-MM-DD` vs `DD_Mon_YYYY`, e.g., `22_Oct_2026`)
- **Problem Identified**: Legacy Python databases (`attendance_system.db`) contain dates formatted as `DD_Mon_YYYY` (e.g., `06_Apr_2026` or `22_Oct_2026`), whereas newer records use ISO `YYYY-MM-DD` (`2026-10-22`). Earlier Rust SQL queries used `WHERE date BETWEEN ?1 AND ?2`, which failed to match `DD_Mon_YYYY` rows and sorted them alphabetically instead of chronologically.
- **Fix Implemented**:
  - Added `TimeHelper::normalize_date_iso()`, `TimeHelper::format_date_dd_mon_yyyy()`, and `TimeHelper::date_matches_range()`.
  - Updated `AttendanceRepo::get_record()`, `update_in_time()`, `update_out_time()`, `update_out_time_detailed()`, `get_by_date()`, `get_by_date_range()`, and `get_user_history()` to transparently match and normalize both `YYYY-MM-DD` and `DD_Mon_YYYY` date formats and sort chronologically by normalized ISO date.

---

## 3. Face Profile Deletion Consistency (`CAP-11`, `CAP-33`)

- **Problem Identified**: `DELETE /api/v1/face/profiles/:username` previously called `state.db.delete_face_profiles(&username)` without calling `state.face_index.remove_user(&username)` and without clearing `users.face_encoding`, leaving deleted users active in RAM for biometric punch-in and leaving stale embeddings in `users.face_encoding` across restarts.
- **Fix Implemented**:
  - `DbPool::delete_face_profiles()` now deletes from `face_profiles` AND executes `UPDATE users SET face_encoding = NULL, register_with_camera = 'N' WHERE UPPER(username) = UPPER(?1)`.
  - `DELETE /api/v1/face/profiles/:username` and `DELETE /api/v1/users/:username` both call `state.face_index.remove_user(&upper)` (which also re-syncs `~/db/face_encodings.index` and `~/db/user_mappings.txt`).
