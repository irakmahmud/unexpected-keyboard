# MISSING FUNCTIONALITY — Zero-Trust Forensic Parity Audit

> **Audit Scope:** `/home/opc/attendance_system/attendance_v8.py` (14,723 lines) vs. `/home/opc/attendance_system/crates/*` (14 Rust crates).
> **Classification Criteria:** Capabilities present in the original Python application that have **no functional implementation** in the Rust codebase.

---

## 1. AWS RDS PostgreSQL Cloud Synchronization (`NetworkSyncManager` & `DatabasePool`)
- **Python Source Evidence:**
  - `AWSConfig` (`attendance_v8.py:3821–3845`)
  - `DatabasePool.initialize_aws_pool` & `AWSConnectionContext` (`attendance_v8.py:3902–3914`, `4156–4203`)
  - `DatabasePool.initialize_aws_tables` (`attendance_v8.py:4010–4133`)
  - `NetworkSyncManager.sync_from_cloud` (`attendance_v8.py:2941–2961`), `sync_users_to_local` (`2963–3028`), `sync_company_info_to_local` (`3029–3061`), `sync_attendance_to_local` (`3063–3131`)
  - `NetworkSyncManager.sync_to_cloud` (`attendance_v8.py:3133–3156`), `sync_company_info_to_cloud` (`3158–3220`), `sync_users_to_cloud` (`3222–3304`), `sync_attendance_to_cloud` (`3305–3436`)
- **Rust Counterpart Examined:**
  - `crates/attendance-sync/src/bi_directional_sync.rs:1–55` (unused stub)
  - `crates/attendance-sync/src/outbox_worker.rs:1–95`
  - `crates/attendance-bigquery/src/client.rs:76–98` (`BigQueryClient::stream_batch`)
  - `crates/attendance-api/src/routes/sync_routes.rs:34–108`
- **Forensic Finding:**
  - Python maintains a live `psycopg2.pool.ThreadedConnectionPool` against AWS RDS PostgreSQL and performs bi-directional synchronization of `company_info`, `users`, `attendance_records`, and `hr_passwords`.
  - Rust contains **zero PostgreSQL client dependencies** (`tokio-postgres` / `sqlx` are absent from `Cargo.toml`) and replaces AWS RDS with a `BigQueryClient` stub (`crates/attendance-bigquery/src/client.rs:76–98`) that contains **no HTTP client (`reqwest` is absent)** and simply logs `"Simulated streaming X rows to BigQuery"` before returning `Ok(events.len())`.
- **Verification Method:** Static inspection of `Cargo.toml`, `crates/attendance-bigquery/src/client.rs:76–98`, and `crates/attendance-api/src/routes/sync_routes.rs`.
- **Confidence Level:** HIGH (100%).

---

## 2. AWS S3 3-Second Attendance Video Recording & Cloud Upload (`capture_video` & `upload_video_to_s3`)
- **Python Source Evidence:**
  - `App.capture_video` (`attendance_v8.py:6413–6553`) — records a 3-second (`duration=3`, `fps=20`, XVID `.avi`) video clip during every login/logout event (both camera and no-camera modes).
  - `NetworkSyncManager.upload_video_to_s3` (`attendance_v8.py:3438–3473`) — uploads recorded `.avi`/`.jpg` media to AWS S3 bucket (`attendance-videos-bucket`) via `boto3` and stores the S3 URI in `video_path_in` / `video_path_out`.
  - `App.verify_attendance_images` (`attendance_v8.py:13158–13890`) — downloads and plays/displays S3 media to audit fake attendance.
- **Rust Counterpart Examined:**
  - `crates/attendance-api/src/routes/attendance_routes.rs:62–78`
  - `crates/attendance-api/src/routes/report_routes.rs:160–220`
- **Forensic Finding:**
  - Rust only decodes a single base64 still image (`image_b64`) when punching via camera, writes it locally to `data/captures/<USER>_<DATE>_<TIME>.jpg`, and stores the local endpoint `/api/v1/captures/<filename>`.
  - No 3-second video recording exists, no background camera capture occurs during non-camera login, and no AWS S3 upload/download integration exists.
- **Verification Method:** Source comparison of `attendance_v8.py:6413–6553` and `crates/attendance-api/src/routes/attendance_routes.rs:40–125`.
- **Confidence Level:** HIGH (100%).

---

## 3. FAISS `IndexFlatL2` Binary Index & `user_mappings.txt` Persistence
- **Python Source Evidence:**
  - `FaceRecognitionSystem.setup_database` (`attendance_v8.py:497–591`)
  - `FaceRecognitionSystem.save_user_mappings` & `load_user_mappings` (`attendance_v8.py:2772–2791`)
  - Persists `~/db/face_encodings.index` (`faiss.write_index` / `faiss.read_index`) and `~/db/user_mappings.txt` (`username:start_idx,end_idx`).
- **Rust Counterpart Examined:**
  - `crates/attendance-face/src/index.rs:1–240`
  - `crates/attendance-api/src/main.rs:62–100` (`bootstrap_face_index`)
- **Forensic Finding:**
  - Rust stores embeddings only in SQLite (`face_profiles` table and `users.face_encoding` JSON column) and rebuilds an in-memory `HashMap<String, Vec<Vec<f32>>>` on startup.
  - Existing Python FAISS index files (`~/db/face_encodings.index`) and index range mappings (`~/db/user_mappings.txt`) are neither read nor written by Rust.
- **Verification Method:** Source code inspection of `crates/attendance-face/src/index.rs` and `crates/attendance-api/src/main.rs`.
- **Confidence Level:** HIGH (100%).

---

## 4. External NTP & HTTP Time Synchronization (`TimeManager.get_network_time`)
- **Python Source Evidence:**
  - `TimeManager.get_network_time` (`attendance_v8.py:3705–3750`) — queries `pool.ntp.org` via `ntplib.NTPClient` and falls back to `http://worldtimeapi.org/api/timezone/Asia/Dhaka` and `https://timeapi.io/api/Time/current/zone?timeZone=Asia/Dhaka`.
  - `TimeManager.check_time_tampering` (`attendance_v8.py:3752–3794`) — persists and compares timestamps in `~/.attendance_time_check` to detect backward clock manipulation.
- **Rust Counterpart Examined:**
  - `crates/attendance-common/src/time.rs:14–53` (`TimeProvider`)
  - `crates/attendance-core/src/anti_tamper.rs:10–82` (`AntiTamperService`)
- **Forensic Finding:**
  - `TimeProvider` simply calls `Utc::now().with_timezone(&chrono_tz::Asia::Dhaka)` on the local OS clock with zero NTP or HTTP time queries and zero `~/.attendance_time_check` file persistence.
- **Verification Method:** Source inspection of `crates/attendance-common/src/time.rs` and `crates/attendance-core/src/anti_tamper.rs`.
- **Confidence Level:** HIGH (100%).

---

## 5. Interactive Forgot Password Verification & Password Reset Dialog (`forgot_password_window`)
- **Python Source Evidence:**
  - `App.forgot_password_window` (`attendance_v8.py:8039–8290`) — opens an interactive password recovery dialog allowing identity/company verification and password reset.
- **Rust Counterpart Examined:**
  - `crates/attendance-api/src/routes/auth_routes.rs:141–146` (`handle_forgot_password`)
  - `crates/attendance-api/src/ui/scripts.rs:661–669` (`showForgotPasswordPopup`)
- **Forensic Finding:**
  - Rust replaces the entire recovery workflow with a static JavaScript `alert()` telling the user to contact HR.
- **Verification Method:** Source inspection of `crates/attendance-api/src/ui/scripts.rs:661–669`.
- **Confidence Level:** HIGH (100%).

---

## 6. Dedicated User Management Window (`App.manage_users`)
- **Python Source Evidence:**
  - `App.manage_users` (`attendance_v8.py:8474–8998`) — interactive GUI window for searching, viewing, editing designations/passwords, and deleting users.
- **Rust Counterpart Examined:**
  - `crates/attendance-api/src/ui/kiosk_view.rs:365–415`
- **Forensic Finding:**
  - While backend REST routes `GET /api/v1/users` and `DELETE /api/v1/users/:username` exist (`crates/attendance-api/src/routes/user_routes.rs`), no User Management modal or table exists in `kiosk_view.rs`.
- **Verification Method:** Inspection of `crates/attendance-api/src/ui/kiosk_view.rs`.
- **Confidence Level:** HIGH (100%).

---

## 7. Multi-Sheet Styled Excel (`.xlsx`) Attendance Report Export
- **Python Source Evidence:**
  - `App.download_attendance_csv_new` & `App.generate_customized_report` (`attendance_v8.py:12548–13155`, `14309–14670`) — generates formatted `.xlsx` workbooks (`openpyxl`) alongside `.csv` exports, with per-user summary tabs, late/early-leave highlighting, and attendance percentages.
- **Rust Counterpart Examined:**
  - `crates/attendance-reporting/src/csv_exporter.rs:1–240`
  - `crates/attendance-reporting/src/muster_roll.rs:1–190`
- **Forensic Finding:**
  - Rust only exports plain `.csv` and an HTML muster roll table; no `.xlsx` export exists in `attendance-reporting`.
- **Verification Method:** Inspection of `crates/attendance-reporting/src/`.
- **Confidence Level:** HIGH (100%).

---

## 8. Filepath Validation Utility (`validate_filepath`)
- **Python Source Evidence:**
  - `validate_filepath` (`attendance_v8.py:424–451`) — enforces path traversal checks and extension whitelisting (`.avi`, `.mp4`, `.jpg`, `.png`, `.csv`, `.xlsx`, `.db`, `.txt`, `.index`, `.json`).
- **Rust Counterpart Examined:**
  - `crates/attendance-api/src/routes/face_routes.rs:474` (only strips `/` via `rsplit('/').next()`).
- **Forensic Finding:**
  - No general path validation helper exists in Rust; `handle_upload_ai_model` in `face_routes.rs:474` does not reject `..` or non-`.onnx` extensions when `model_name` is `".."` or `"foo.sh"`.
- **Verification Method:** Source inspection of `crates/attendance-api/src/routes/face_routes.rs:468–493`.
- **Confidence Level:** HIGH (100%).
