# Complete Regression Test & End-to-End Verification Results (`REGRESSION_TEST_RESULTS.md`)

**Date:** 2026-10-22  
**Workspace:** `/home/opc/attendance_system`  
**Remote CircleCI Build:** `#48` (Commit `6ff68ab`, Branch `attendance-staging`, **0 local Rust builds**)  
**Staging Binary:** `/home/opc/attendance_system/deploy/staging_verified/attendance-api` (`35,833,456` bytes, SHA-256: `7ecf2b339f5b9919a9a444a4b0bb65057033dd9619487d1622a37ff1679b294c`)

---

## 1. Remote CircleCI Rust Workspace Test Suite (`cargo test --release --workspace`)

Executed on CircleCI (`cimg/rust:1.80.0`, `large` executor) in Build `#48` (`circleci_build_47_tests.log`):

| Test Target / Suite | Crate | Tests Executed | Passed | Failed | Status |
| :--- | :--- | ---: | ---: | ---: | :--- |
| `attendance_auth` (unit tests) | `crates/attendance-auth` | `1` | `1` | `0` | **PASS** |
| `attendance_common` (unit tests) | `crates/attendance-common` | `8` | `8` | `0` | **PASS** |
| `attendance_core` (unit tests) | `crates/attendance-core` | `1` | `1` | `0` | **PASS** |
| `attendance_face` (unit tests) | `crates/attendance-face` | `10` | `10` | `0` | **PASS** |
| `attendance_reporting` (unit tests) | `crates/attendance-reporting` | `1` | `1` | `0` | **PASS** |
| `api_routes_integration_test` | `crates/attendance-tests` | `5` | `5` | `0` | **PASS** |
| `api_security_rbac_test` | `crates/attendance-tests` | `5` | `5` | `0` | **PASS** |
| `concurrency_race_condition_test` | `crates/attendance-tests` | `3` | `3` | `0` | **PASS** |
| `database_integrity_test` | `crates/attendance-tests` | `4` | `4` | `0` | **PASS** |
| `end_to_end_attendance_test` | `crates/attendance-tests` | `3` | `3` | `0` | **PASS** |
| `face_biometric_matching_test` | `crates/attendance-tests` | `6` | `6` | `0` | **PASS** |
| `holiday_sandwich_and_leave_test` | `crates/attendance-tests` | `3` | `3` | `0` | **PASS** |
| `muster_roll_report_generation_test` | `crates/attendance-tests` | `4` | `4` | `0` | **PASS** |
| `outbox_bigquery_sync_test` | `crates/attendance-tests` | `3` | `3` | `0` | **PASS** |
| `payroll_and_overtime_test` | `crates/attendance-tests` | `3` | `3` | `0` | **PASS** |
| `rust_deepface_parity_test` | `crates/attendance-tests` | `6` | `6` | `0` | **PASS** |
| `shift_roster_and_anti_tamper_test` | `crates/attendance-tests` | `3` | `3` | `0` | **PASS** |
| `vector_performance_test` | `crates/attendance-tests` | `5` | `5` | `0` | **PASS** |
| **TOTAL RUST WORKSPACE TESTS** | **All 12 Workspace Crates** | **`70`** | **`70`** | **`0`** | **100% PASS** |

---

## 2. Live Staging End-to-End Verification Suite (`63 / 63 PASS` on Port 8090)

Executed via `scripts/run_staging_e2e_verification.py` against `deploy/staging_verified/attendance-api` (`http://127.0.0.1:8090`, isolated database `data/staging_attendance.db`), leaving the live production binary on port `8080` completely untouched. Full JSON results saved in `staging_verification_results.json`.

### 2.1 Startup & System Integrity (2 / 2 PASS)
- `[PASS]` **Staging server listening on port 8090**: `HTTP 200` on `http://127.0.0.1:8090/`
- `[PASS]` **Time integrity file (`~/.attendance_time_check`)**: Created and verified at `/home/opc/.attendance_time_check`

### 2.2 Biometric ONNX FaceNet512 + MTCNN Verification & Detection Enforcement (18 / 18 PASS)
- `[PASS]` **Same-person verify (`Jolie_1.jpg` vs `Jolie_3.jpg`)**: `HTTP 200`, `verified=True`, `distance=0.1813` (`<= 0.30`)
- `[PASS]` **Same-person verify (`Jolie_1.jpg` vs `Jolie_6.webp`)**: `HTTP 200`, `verified=True`, `distance=0.1763` (`<= 0.30`)
- `[PASS]` **Same-person verify (`Jolie_1.jpg` vs `face8.jpg`)**: `HTTP 200`, `verified=True`, `distance=0.2942` (`<= 0.30`)
- `[PASS]` **Same-person verify (`Einstein_1.jpg` vs `Einstein_2.jpg`)**: `HTTP 200`, `verified=True`, `distance=0.0547` (`<= 0.30`)
- `[PASS]` **Same-person verify (`Hadi_1.jpeg` vs `Hadi_2.jpg`)**: `HTTP 200`, `verified=True`, `distance=0.2665` (`<= 0.30`)
- `[PASS]` **Same-person verify (`Hadi_1.jpeg` vs `Hadi_3.jpg`)**: `HTTP 200`, `verified=True`, `distance=0.1634` (`<= 0.30`)
- `[PASS]` **Different-person reject (`Jolie_1.jpg` vs `Einstein_1.jpg`)**: `HTTP 200`, `verified=False`, `distance=1.0786` (`> 0.30`)
- `[PASS]` **Different-person reject (`Jolie_1.jpg` vs `Hadi_1.jpeg`)**: `HTTP 200`, `verified=False`, `distance=0.8334` (`> 0.30`)
- `[PASS]` **Different-person reject (`Einstein_1.jpg` vs `Hadi_1.jpeg`)**: `HTTP 200`, `verified=False`, `distance=0.9355` (`> 0.30`)
- `[PASS]` **Different-person reject (`Jolie_1.jpg` vs `pit.jpg`)**: `HTTP 200`, `verified=False`, `distance=1.2160` (`> 0.30`)
- `[PASS]` **0-face rejection (`Blank_Img.jpg`)**: `HTTP 422`, `{"error": "No face detected in image", "success": false}`
- `[PASS]` **0-face rejection (`Noise_Img.png`)**: `HTTP 422`, `{"error": "No face detected in image", "success": false}`
- `[PASS]` **Multi-face (2 faces) rejection (`100.jpeg`)**: `HTTP 422`, `{"error": "Multiple faces detected in image (2 faces); please ensure only 1 face is in frame", "success": false}`
- `[PASS]` **Multi-angle ONNX registration (`JOLIE_TEST`)**: `HTTP 200`, `embeddings_enrolled=3`, `with_camera=True`
- `[PASS]` **Persistent FAISS index & `user_mappings.txt` sync**: Verified `/home/opc/db/face_encodings.index` & `/home/opc/db/user_mappings.txt`
- `[PASS]` **Identity verification on unseen photo (`Jolie_6.webp -> JOLIE_TEST`)**: `HTTP 200`, `matched=True`, `min_distance=0.1763`
- `[PASS]` **Biometric camera punch-in (`face8.jpg -> JOLIE_TEST`)**: `HTTP 200`, `login_type=IN_TIME`, `record_id=25aafa91-9fb8-4e65-9866-1a2ab680d174_23_Oct_2026`
- `[PASS]` **Cooldown file (`~/.last_attendance`) updated**: Verified `/home/opc/.last_attendance`

### 2.3 Security, Revocation & Authentication Hardening (14 / 14 PASS)
- `[PASS]` **Duplicate face enrollment blocked (`JOLIE_CLONE`)**: `HTTP 400`, `Duplicate face detected: this face is already registered to user 'JOLIE_TEST' (cosine distance 0.1763)`
- `[PASS]` **Duplicate Android Device ID blocked across users (`JOLIE_DUP_DEV`)**: `HTTP 400`, `Device Android ID 'ANDR_JOLIE_01' is already registered to user 'JOLIE_TEST'`
- `[PASS]` **Delete face profile (`DELETE /api/v1/face/profiles/JOLIE_TEST`)**: `HTTP 200`, `deleted_count=4`, `ram_vectors_removed=3`
- `[PASS]` **Deleted face profile cleared `users.face_encoding=NULL` & `register_with_camera='N'`**: `HTTP 200`, `face_encoding=None`, `register_with_camera='N'`
- `[PASS]` **Deleted face profile stays purged after `sync/users-to-local` & `face/index/rebuild`**: `HTTP 200`, `matched=False`
- `[PASS]` **Change HR password from default to `CustomHR#2026`**: `HTTP 200`
- `[PASS]` **Reject hardcoded HR backdoor `'HR@123'`**: `HTTP 401 Unauthorized`
- `[PASS]` **Reject hardcoded HR backdoor `'1234'`**: `HTTP 401 Unauthorized`
- `[PASS]` **Reject hardcoded HR backdoor `'admin123'`**: `HTTP 401 Unauthorized`
- `[PASS]` **Reject hardcoded HR backdoor `'hr2026'`**: `HTTP 401 Unauthorized`
- `[PASS]` **Accept legitimate custom HR password `'CustomHR#2026'`**: `HTTP 200`, `verified=True`
- `[PASS]` **Block unauthenticated HR password overwrite & preserve `CustomHR#2026`**: `set_code=401`, `sync_explicit_code=401`, `verify=True`
- `[PASS]` **Reject public designation in forgot-password verification**: `HTTP 401 Unauthorized`
- `[PASS]` **Path traversal & Corrupted ONNX upload blocked on `/api/v1/face/upload-model`**: `../../etc/passwd.onnx` -> `HTTP 400`; invalid ONNX bytes -> `HTTP 400` before disk write (`size_before=94176340`, `size_after=94176340`)

### 2.4 Database Dual-Format & Reporting Endpoints (5 / 5 PASS)
- `[PASS]` **Dual-format `DD_Mon_YYYY` date (`20_Oct_2026 -> 2026-10-20`) & 19-char UTC timestamp (`2026-10-20 03:05:00 -> 09:05:00` Dhaka) normalization**: `HTTP 200`
- `[PASS]` **Customized Attendance Report CSV (`/api/v1/reports/customized.csv`)**: `HTTP 200` (`5,355` bytes)
- `[PASS]` **Valid OpenXML `.xlsx` Report (`/api/v1/reports/customized.xlsx`)**: `HTTP 200` (`67,888` bytes, valid ZIP archive containing `xl/worksheets/sheet1.xml`)
- `[PASS]` **HTML Monthly Muster Roll Report (`/api/v1/reports/muster-roll/html`)**: `HTTP 200` (`22,720` bytes)
- `[PASS]` **Face Embeddings Analysis CSV Report (`/api/v1/face/embeddings-report.csv`)**: `HTTP 200` (`249` bytes)

### 2.5 Headless Chromium / Playwright UI E2E Suite (25 / 25 PASS)
- **25 / 25 UI & modal workflow checks passed** with **`0` uncaught JS `pageerror` exceptions** (`FRONTEND_FUNCTIONAL_TESTS.md`), including verification that `UI_TEST_USER` registered via UI persisted `user_designation='QA Engineer'` and `register_with_camera='N'`.
