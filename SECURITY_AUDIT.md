# Security Audit & Vulnerability Remediation Report (`SECURITY_AUDIT.md`)

**Workspace**: `/home/opc/attendance_system`  
**Audit Scope**: Authentication (`crates/attendance-auth`), API Routes (`crates/attendance-api/src/routes`), Path Validation (`crates/attendance-common`), Biometric Pipeline (`crates/attendance-face`), Database Persistence (`crates/attendance-database`)

---

## 1. Executive Summary of Vulnerabilities Remediated

| ID | Vulnerability Description | Severity | Location | Status |
|---|---|---|---|---|
| **SEC-01** | Hardcoded HR Master Password Backdoors (`"HR@123"`, `"1234"`, `"admin123"`, `"hr2026"`) bypassing custom HR passwords | **CRITICAL** | `crates/attendance-auth/src/hr_auth.rs`, `crates/attendance-api/src/routes/auth_routes.rs` | **FIXED & VERIFIED** |
| **SEC-02** | Hardcoded Company Verification Backdoor (`payload.password == "admin123"`) & plaintext default company password seed | **CRITICAL** | `crates/attendance-api/src/routes/company_routes.rs`, `crates/attendance-api/src/main.rs` | **FIXED & VERIFIED** |
| **SEC-03** | Unauthenticated HR Password Overwrite via `POST /api/v1/auth/set-hr-password` & `POST /api/v1/auth/sync-hr-password`, plus public `user_designation` bypass in `POST /api/v1/auth/forgot-password` | **HIGH** | `crates/attendance-api/src/routes/auth_routes.rs` | **FIXED & VERIFIED** |
| **SEC-04** | Silent 60% Center-Crop Fallback on Face Detection Failure (`0` faces or noise image producing valid 512-D embedding) | **CRITICAL** | `crates/attendance-face/src/pipeline.rs`, `crates/attendance-face/src/mtcnn.rs` | **FIXED & VERIFIED** |
| **SEC-05** | Corrupted Constant Seed Embeddings (`[0.04419417; 512]`) in `face_profiles` matching any constant vector as `ADMIN` | **CRITICAL** | `crates/attendance-face/src/index.rs`, `crates/attendance-api/src/main.rs` | **FIXED & VERIFIED** |
| **SEC-06** | Incomplete Biometric Revocation (`DELETE /api/v1/face/profiles/:username` failed to clear `users.face_encoding` due to missing `register_with_camera` column, allowing deleted faces to resurrect on sync/rebuild) | **CRITICAL** | `crates/attendance-database/src/schema.rs`, `crates/attendance-database/src/repos/biometrics_repo.rs`, `crates/attendance-api/src/routes/face_routes.rs`, `crates/attendance-api/src/routes/user_routes.rs` | **FIXED & VERIFIED** |
| **SEC-07** | Path Traversal & Destructive Unvalidated Model Upload on `/api/v1/face/upload-model` (overwriting `models/facenet512.onnx` before validating ONNX protobuf structure) | **HIGH** | `crates/attendance-common/src/validation.rs`, `crates/attendance-face/src/inference.rs`, `crates/attendance-api/src/routes/face_routes.rs`, `crates/attendance-api/src/routes/attendance_routes.rs` | **FIXED & VERIFIED** |
| **SEC-08** | Missing Duplicate Android Device ID Enforcement on `POST /api/v1/users` & `PUT /api/v1/users/:username` | **MEDIUM** | `crates/attendance-api/src/routes/user_routes.rs` | **FIXED & VERIFIED** |

---

## 2. Detailed Remediation & Code Evidence

### 2.1 Removal of Hardcoded HR & Company Password Backdoors & Overwrite Vectors (`SEC-01`, `SEC-02`, `SEC-03`)
- **Root Cause**:
  - `HrAuthService::verify_hr_access()` (`crates/attendance-auth/src/hr_auth.rs`) and `auth_routes.rs` previously accepted hardcoded backdoor strings (`"1234"`, `"admin123"`, `"hr2026"`, `"HR@123"`).
  - `company_routes.rs::verify_company_password()` accepted `payload.password == "admin123"` unconditionally, and `main.rs` seeded `HQ_001` with plaintext `"admin123"`.
  - `auth_routes.rs::handle_set_hr_password()` and `handle_sync_hr_password()` allowed unauthenticated callers to overwrite an existing custom HR password in `hr_passwords`.
  - `auth_routes.rs::handle_forgot_password_post()` accepted a user's public `user_designation` as verification to reset their password.
- **Remediation**:
  - `HrAuthService::verify_hr_access()` queries `hr_passwords` (`db.get_hr_password(company_uuid)`) and only auto-initializes `"HR@123"` when no record exists for that company. Invalid passwords return `Err(AppError::Unauthorized)` (`HTTP 401`).
  - `handle_set_hr_password()` and `handle_sync_hr_password()` now verify whether a non-default HR password is already configured; if so, they require valid `current_password` verification via `HrAuthService::verify_hr_access()` (`HTTP 401` on unauthorized overwrite attempts).
  - Removed the `verified_by_desig` bypass from `handle_forgot_password_post()`; password reset now requires either a valid HR password or the user's registered `user_android_id`.
  - Removed the `"admin123"` backdoor in `company_routes.rs` and hashed the default company password in `main.rs` via `AuthService::hash_password("admin123")`.

### 2.2 Biometric Enforcement, Anti-Spoofing & Complete Revocation (`SEC-04`, `SEC-05`, `SEC-06`, `SEC-08`)
- **Root Cause**:
  - `pipeline.rs` previously fell back to a 60% center-crop when `mtcnn::detect_face` failed.
  - `schema.rs` omitted `register_with_camera TEXT DEFAULT 'N'` on the `users` table, causing `UPDATE users SET face_encoding = NULL, register_with_camera = 'N' WHERE UPPER(username) = ?1` in `BiometricsRepo::delete_profiles_by_username` to fail silently. As a result, `users.face_encoding` remained populated after `DELETE /api/v1/face/profiles/:username`, and `/api/v1/sync/users-to-local` or `/api/v1/face/index/rebuild` resurrected deleted faces back into RAM.
- **Remediation**:
  - Removed the 60% center-crop fallback in `pipeline.rs`; `detect_face` strictly enforces `NoFaceDetected` (`HTTP 422`) and `MultipleFacesDetected(n)` (`HTTP 422`).
  - Added `register_with_camera TEXT DEFAULT 'N'` to `users` in `schema.rs` (`SQL_INIT_SCHEMA` and `apply_column_migrations`) and removed `.unwrap_or(0)` error swallowing in `BiometricsRepo::delete_profiles_by_username` and `delete_all_profiles`.
  - Verified in staging that after `DELETE /api/v1/face/profiles/JOLIE_TEST`, `users.face_encoding` is `NULL`, `users.register_with_camera` is `'N'`, and subsequent calls to `/api/v1/sync/users-to-local` and `/api/v1/face/index/rebuild` do not resurrect the deleted face.
  - Enforced duplicate face check (`find_duplicate_user` `< 0.30`) and duplicate `user_android_id` check across users in `user_routes.rs`.

### 2.3 Path Traversal & Pre-Write ONNX Graph Validation (`SEC-07`, `CAP-04`, `CAP-31`)
- **Remediation**:
  - Implemented `attendance_common::validate_filepath(file_path, allowed_dir, allowed_extensions)` and `sanitize_filename(name)` blocking `..`, `/`, `\`, null bytes, and disallowed extensions.
  - Added `attendance_face::validate_onnx_model_bytes(&bytes)` (using `tract_onnx::onnx().model_for_read(...)`) in `crates/attendance-face/src/inference.rs` and called it in `handle_upload_ai_model` (`face_routes.rs`) **before** `std::fs::write`, ensuring corrupted or non-ONNX uploads are rejected with `HTTP 400` without touching existing model files on disk.
