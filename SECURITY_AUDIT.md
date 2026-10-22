# Security Audit & Vulnerability Remediation Report

**Workspace**: `/home/opc/attendance_system`
**Audit Scope**: Authentication (`attendance-auth`), API Routes (`attendance-api`), Path Validation (`attendance-common`), Biometric Pipeline (`attendance-face`)

---

## 1. Executive Summary of Vulnerabilities Remediated

| ID | Vulnerability Description | Severity | Location | Status |
|---|---|---|---|---|
| **SEC-01** | Hardcoded HR Master Password Backdoors (`"HR@123"`, `"1234"`, `"admin123"`, `"hr2026"`) bypassing custom HR passwords | **CRITICAL** | `crates/attendance-auth/src/hr_auth.rs`, `crates/attendance-api/src/routes/auth_routes.rs` | **FIXED & VERIFIED** |
| **SEC-02** | Hardcoded Company Verification Backdoor (`payload.password == "admin123"`) bypassing company password hash | **CRITICAL** | `crates/attendance-api/src/routes/company_routes.rs` | **FIXED & VERIFIED** |
| **SEC-03** | Unauthenticated HR Password Overwrite via `POST /api/v1/auth/sync-hr-password` resetting custom HR password to `"HR@123"` | **HIGH** | `crates/attendance-api/src/routes/auth_routes.rs` | **FIXED & VERIFIED** |
| **SEC-04** | Silent 60% Center-Crop Fallback on Face Detection Failure (`0` faces or noise image producing valid 512-D embedding) | **CRITICAL** | `crates/attendance-face/src/pipeline.rs`, `crates/attendance-face/src/mtcnn.rs` | **FIXED & VERIFIED** |
| **SEC-05** | Corrupted Constant Seed Embeddings (`[0.04419417; 512]`) in `face_profiles` matching any constant vector as `ADMIN` | **CRITICAL** | `crates/attendance-face/src/index.rs`, `crates/attendance-api/src/main.rs` | **FIXED & VERIFIED** |
| **SEC-06** | Incomplete Biometric Revocation (`DELETE /api/v1/face/profiles/:username` left vectors in RAM `FaceVectorIndex` and `users.face_encoding`) | **HIGH** | `crates/attendance-api/src/routes/face_routes.rs`, `crates/attendance-database/src/lib.rs` | **FIXED & VERIFIED** |
| **SEC-07** | Path Traversal Risk in Model Upload (`/api/v1/face/upload-model`) and Capture Image Serving (`/api/v1/captures/:filename`) | **HIGH** | `crates/attendance-common/src/validation.rs`, `face_routes.rs`, `attendance_routes.rs` | **FIXED & VERIFIED** |

---

## 2. Detailed Remediation & Code Evidence

### 2.1 Removal of Hardcoded HR & Company Password Backdoors (`SEC-01`, `SEC-02`, `SEC-03`)
- **Root Cause**:
  - `HrAuthService::verify_hr_password()` contained `if password == "HR@123" || password == "admin123" { return Ok(true); }` before checking the database record.
  - `auth_routes.rs::verify_hr()` contained fallback checks accepting `"1234"`, `"admin123"`, `"hr2026"`, and `"HR@123"` even when `hr_auth` had a custom password set.
  - `company_routes.rs::verify_company()` accepted `payload.password == "admin123"` unconditionally.
  - `auth_routes.rs::sync_hr_password()` unconditionally called `set_hr_password(db, company_id, company_name, "HR@123")`, overwriting custom HR passwords whenever the UI triggered a sync.
- **Remediation**:
  - `HrAuthService::verify_hr_password()` now queries `hr_auth` first. If a record exists for the company, it **strictly** verifies `PasswordHasher::verify_salted_sha256(password, &record.salt, &record.hr_password_hash)`. Only when `hr_auth` has **no** record yet for that company does it initialize the default `"HR@123"` hash.
  - Removed all hardcoded fallback strings (`"1234"`, `"admin123"`, `"hr2026"`, `"HR@123"`) from `auth_routes.rs::verify_hr()` and `company_routes.rs::verify_company()`.
  - Updated `auth_routes.rs::sync_hr_password()` so it checks `state.db.get_hr_auth(&cid)` first and only initializes `"HR@123"` if no HR password record exists (`already_initialized: true` otherwise).

### 2.2 Biometric Enforcement & Anti-Spoofing (`SEC-04`, `SEC-05`, `SEC-06`)
- **Root Cause**:
  - `pipeline.rs` caught `detect_face` errors and silently cropped the middle 60% of the image, allowing blank canvases (`Blank_Img.jpg`) or random noise (`Noise_Img.png`) to produce embeddings.
  - `mtcnn::detect_face()` did not reject multi-face frames (`100.jpeg`).
- **Remediation**:
  - Removed the 60% center-crop fallback completely from `pipeline.rs`.
  - `mtcnn::detect_face()` now enforces `enforce_detection=True`: `0` faces -> `Err(AppError::NoFaceDetected)` (HTTP 422), `>1` faces -> `Err(AppError::MultipleFacesDetected(n))` (HTTP 422).
  - Duplicate face prevention (`FaceVectorIndex::find_duplicate_user` with threshold `< 0.30`) blocks registering the same person under multiple usernames.

### 2.3 Path Traversal & File Upload Validation (`SEC-07`, `CAP-04`)
- **Remediation**:
  - Implemented `attendance_common::validate_filepath(file_path, allowed_dir, allowed_extensions)` and `sanitize_filename(name)` rejecting `..`, `/`, `\`, null bytes, and unauthorized extensions, and verifying canonical path containment (`joined.starts_with(&canonical_base)`).
  - Applied to `POST /api/v1/face/upload-model`, `GET /api/v1/captures/:filename`, and `POST /api/v1/attendance/punch` capture saving.
