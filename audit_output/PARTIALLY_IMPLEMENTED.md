# PARTIALLY IMPLEMENTED & BROKEN CAPABILITIES — Forensic Audit

> **Audit Scope:** Capabilities where a Rust struct, function, or REST route exists, but behavioral diverges, dead code, or UI/API wiring defects prevent true parity with `attendance_v8.py`.

---

## 1. Single-Instance Lock & Cooldown (`SingleInstanceManager` — Dead Code in Rust)
- **Python Source:** `SingleInstanceManager` (`attendance_v8.py:41–232`) — locks `~/attendance_system.lock` (`fcntl.flock`) on startup, scans running processes via `psutil`, and enforces a 1-minute restart cooldown via `~/.last_attendance`.
- **Rust Source:** `crates/attendance-common/src/single_instance.rs:12–89` & `crates/attendance-core/src/cooldown.rs:7–53`.
- **Gap Analysis:**
  1. `SingleInstanceManager` in `attendance-common` is **100% dead code** — it is never instantiated or called in `crates/attendance-api/src/main.rs`.
  2. `CooldownManager` in `attendance-core` only stores cooldowns in an in-memory `DashMap` (`user_cooldown_secs = 60`, `device_cooldown_secs = 5`), losing all state on service restart instead of persisting `~/.last_attendance`.
- **Verification Method:** `grep -rn "SingleInstanceManager" crates/` and inspection of `crates/attendance-api/src/main.rs`.
- **Confidence Level:** HIGH.

---

## 2. Unused Biometric Modules Inside `attendance-face` (Dead Code)
- **Rust Files:**
  - `crates/attendance-face/src/quality.rs:1–85` (`ImageQualityAssessor`: brightness, sharpness, face size ratio)
  - `crates/attendance-face/src/enrollment.rs:1–95` (`EnrollmentSession`: multi-angle session tracker)
  - `crates/attendance-face/src/hnsw.rs:1–140` (`HnswIndex`: graph-based nearest neighbor index)
  - `crates/attendance-face/src/clustering.rs:1–90` (`FaceClusterer`: k-means centroid builder)
- **Gap Analysis:**
  - None of these 4 modules are ever called by `attendance-api`! Instead, `face_routes.rs` and `user_routes.rs` call `attendance_face::extract_face_embedding_vec` directly (without `ImageQualityAssessor` or `EnrollmentSession`) and store raw vectors in `FaceIndex` (`crates/attendance-face/src/index.rs`), which performs a brute-force linear scan.
- **Verification Method:** Cross-crate symbol usage search across `crates/attendance-api/`.
- **Confidence Level:** HIGH.

---

## 3. 20-Pose Camera Registration (`capture_faces_continuous` vs. `runAuto20PoseCapture`)
- **Python Source:** `FaceRecognitionSystem.capture_faces_continuous` (`attendance_v8.py:1073–1287`) & `get_action_instructions` (`1330–1395`).
  - Captures 20 distinct frames with a **4-second countdown per pose** (`preview_time = 4`, line 1087), displaying trilingual instructions (English, Bengali, Banglish) and verifying via `verify_face_detection` (line 1397) that each frame contains a real detected face before advancing.
- **Rust Source:** `crates/attendance-api/src/ui/scripts.rs:16–37`, `1259–1355` & `crates/attendance-api/src/routes/user_routes.rs:76–202`.
- **Gap Analysis:**
  1. **Blocked in Browser UI:** `openHrAction('reg-camera')` (`scripts.rs:1038`) crashes in the browser because `scripts.rs` looks up `document.getElementById('hr-workspace-content')` (`null`), whereas `kiosk_view.rs:699` defines `id="hr-workspace-body"`.
  2. **No Pose Timing or Validation:** `runAuto20PoseCapture()` (`scripts.rs:1291–1304`) loops 20 times with a `120ms` delay (`2.4 seconds` total), never calls `/api/v1/face/extract-frame` during capture, and accepts 20 identical frames (or synthetic canvas drawings when no webcam is attached).
  3. **21 Embeddings Enrolled Instead of 20:** `user_routes.rs:92–100` pushes all 20 elements of `images_b64` AND then pushes `image_b64` (`regCapturedFrames[0]`) as a 21st frame (`embeddings_enrolled: 21`).
- **Verification Method:** Inspection of `kiosk_view.rs:699`, `scripts.rs:1038, 1291–1355`, and `user_routes.rs:92–100`.
- **Confidence Level:** HIGH.

---

## 4. Delete Face Data (`delete_user_profiles` Leaves Stale Data in RAM & `users` Table)
- **Python Source:** `App.delete_face_data` (`attendance_v8.py:10242–10384`) — removes the user's embeddings from FAISS (`face_encodings.index`), `user_mappings.txt`, and SQLite (`users.face_encoding`), and syncs the deletion.
- **Rust Source:** `crates/attendance-api/src/routes/face_routes.rs:183–193` (`delete_user_profiles`).
- **Gap Analysis:**
  - `DELETE /api/v1/face/profiles/:username` only runs `state.db.delete_face_profiles(&username)` on the SQLite `face_profiles` table.
  - It **fails to remove the user from `state.face_index` in RAM** and **fails to clear `users.face_encoding` in SQLite**, so the deleted user continues to match camera punches until the server is restarted (and even after restart, `bootstrap_face_index` reloads `users.face_encoding` back into RAM!).
- **Verification Method:** Inspection of `crates/attendance-api/src/routes/face_routes.rs:183–193` and `crates/attendance-api/src/main.rs:78–95`.
- **Confidence Level:** HIGH.

---

## 5. Upload AI Model (`handle_upload_ai_model` Does Not Reload ONNX Session)
- **Python Source:** `App.upload_model` (`attendance_v8.py:10098–10240`) — copies `.h5`/model weights to `.deepface/weights/` and re-initializes the recognizer.
- **Rust Source:** `crates/attendance-api/src/routes/face_routes.rs:468–493`.
- **Gap Analysis:**
  - Writes base64 bytes to `models/<safe_name>` on disk, but `state.deepface_state` in `AppState` is immutable (`Option<Arc<DeepFaceState>>`) and is never reloaded in memory.
- **Verification Method:** Inspection of `crates/attendance-api/src/routes/face_routes.rs:468–493` and `crates/attendance-api/src/main.rs:45–60`.
- **Confidence Level:** HIGH.
