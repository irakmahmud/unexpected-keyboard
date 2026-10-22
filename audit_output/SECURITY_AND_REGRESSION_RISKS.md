# SECURITY VULNERABILITIES & REGRESSION RISKS

---

## 1. Hardcoded Universal Backdoor Passwords in Rust Auth (`CRITICAL`)
- **Evidence 1 (`crates/attendance-auth/src/hr_auth.rs:28`):**
  ```rust
  let is_valid = AuthService::verify_password(entered_password, &stored)
      || entered_password == "HR@123"
      || entered_password == "1234"
      || entered_password == "admin123"
      || entered_password == "hr2026";
  ```
  Even after an HR administrator changes the company's HR password in SQLite, **`"HR@123"`, `"1234"`, `"admin123"`, and `"hr2026"` remain permanently valid backdoor passwords**!
- **Evidence 2 (`crates/attendance-api/src/routes/company_routes.rs:82`):**
  ```rust
  let valid = company.password.as_deref() == Some(entered) || entered == "admin123";
  ```
  Any company can be unlocked or edited using the hardcoded password `"admin123"`.

---

## 2. Unauthenticated Admin & Destructive Endpoints (`CRITICAL`)
- **Evidence (`crates/attendance-api/src/routes/face_routes.rs`, `user_routes.rs`, `company_routes.rs`):**
  - `POST /api/v1/face/reset` (wipes all enrolled face profiles in SQLite & RAM)
  - `DELETE /api/v1/face/profiles/:username`
  - `POST /api/v1/face/upload-model` (writes arbitrary files into `models/`)
  - `POST /api/v1/users` with `overwrite_existing: true`
  - `PUT /api/v1/companies/:id`
  - None of these endpoints require an HR session token or Authorization header on the backend; HR password check is purely client-side UI gating.

---

## 3. HR Password Overwrite Bug on `/api/v1/auth/sync-hr-password` (`HIGH`)
- **Evidence (`crates/attendance-api/src/routes/auth_routes.rs:33–38, 53, 99–112`):**
  - `/sync-hr-password` is routed to `handle_set_hr_password` with `#[serde(default = "default_hr_pw")]` (`"HR@123"`). Calling `POST /api/v1/auth/sync-hr-password` without a password body **resets the company's local HR password back to `"HR@123"`**!

---

## 4. Biometric False Acceptance & Faceless Image Matching (`CRITICAL`)
- **Evidence (`crates/attendance-face/src/pipeline.rs:14–31`, `models/facenet512.onnx`):**
  - Because `pipeline.rs` falls back to a 60% center crop when MTCNN detects no face, and `models/facenet512.onnx` is a random linear projection of $10 	imes 10$ average-pooled pixels, **two photos of the same empty room background** produce a cosine distance near `0.0000` (`99.9%` confidence match) even with no human in front of the camera!
