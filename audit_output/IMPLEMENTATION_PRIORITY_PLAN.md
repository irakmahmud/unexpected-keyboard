# IMPLEMENTATION PRIORITY PLAN — Exact Work Remaining for 100% Parity

> **Note:** Per mission rules, **no fixes have been applied during this read-only audit**. Below is the prioritized engineering roadmap to achieve genuine, verified parity upon your explicit permission.

---

## Priority 0 (P0): Fix All 9 Browser UI / Backend DOM & Route Mismatches (`scripts.rs` & `kiosk_view.rs`)
1. **Header Company Badge (`kiosk_view.rs:26` vs `scripts.rs:122`):** Align ID to `header-company-name` (or `header-company-badge`).
2. **Company Management Modal (`kiosk_view.rs:483–615` vs `scripts.rs:141–315`):** Align view container IDs (`company-view-menu`, `company-view-select`, `company-view-create`, `company-view-edit`) and input IDs (`comp-select-search`, `comp-select-dropdown`, `comp-select-password`, `comp-create-*`, `comp-edit-*`).
3. **HR Authentication Modal (`kiosk_view.rs:629–670` vs `scripts.rs:922–1008`):** Align `#hr-auth-login-view`, `#hr-auth-change-view`, `#hr-auth-password`, `#hr-old-password`, `#hr-new-password`, `#hr-confirm-password`.
4. **HR Portal Workspace (`kiosk_view.rs:680–705` vs `scripts.rs:1015–1040`):** Align `#hr-menu-grid`, `#hr-workspace`, and `#hr-workspace-body`.
5. **Identity Verification Split-Window (`kiosk_view.rs:286–348` vs `scripts.rs:496–570`):** Align `#verify-confidence`, `#verify-min-dist`, `#verify-match-quality`, `#verify-user-display`, `#verify-type-display`, `#verify-time-display`, `#verify-date-display`, `#verify-countdown-label`, `#verify-question-label`, and `#btn-verify-yes`.
6. **QR Code Login Modal (`kiosk_view.rs:431–435` vs `scripts.rs:736–768`):** Align `#qr-login-box`, `#qr-session-status`, and check `pollData.received && pollData.android_id`.
7. **7-Day History Modal (`scripts.rs:882–895`):** Iterate over `const rowsData = data.history || [];` instead of `data.length`.
8. **Console Force Sync Button (`scripts.rs:99`):** Change `/api/v1/sync/process` to `/api/v1/sync/trigger`.
9. **Customized CSV & Muster Roll Reports (`scripts.rs:1430–1453`):** Parse `YYYY-MM-DD` into `year`, `month`, `from_day`, `to_day` and call `/api/v1/reports/customized-csv` and `/api/v1/reports/muster-roll?year=...&month=...`.

---

## Priority 1 (P1): Replace Synthetic 1.05 MB ONNX Models with Authentic Pretrained Weights
1. Run `rust_deepFace/export_to_onnx.py` (or download verified pretrained ONNX weights) to export the real ~92–95 MB Inception-ResNet-v1 `facenet512.onnx` and real pretrained `mtcnn_pnet.onnx`, `mtcnn_rnet.onnx`, and `mtcnn_onet.onnx` into `/home/opc/attendance_system/models/`.
2. Remove the silent 60% center-crop fallback in `crates/attendance-face/src/pipeline.rs:14–31` so faceless images are properly rejected (`enforce_detection = true`), and reject frames containing multiple faces (`faces.len() != 1`).
3. Remove the corrupted constant seed vectors (`[0.044194; 512]`) for `EMP001..EMP010` and synthetic ramp vectors in `seed.rs`.
4. Enforce real 20-pose capture intervals, face detection per pose, and inter-frame variation checks during camera registration.

---

## Priority 2 (P2): Remove Security Backdoors & Fix Delete/Sync Bugs
1. Remove hardcoded backdoor passwords (`"HR@123"`, `"1234"`, `"admin123"`, `"hr2026"`) from `crates/attendance-auth/src/hr_auth.rs:28` and `crates/attendance-api/src/routes/company_routes.rs:82` (only allow default `"HR@123"` when no row exists yet in `hr_password`).
2. Fix `DELETE /api/v1/face/profiles/:username` (`face_routes.rs:183–193`) so it also removes the user from `state.face_index` in RAM and sets `users.face_encoding = NULL` in SQLite.
3. Fix `/api/v1/auth/sync-hr-password` so it does not overwrite existing local HR passwords with `"HR@123"`.
4. Wire `SingleInstanceManager` (`crates/attendance-common/src/single_instance.rs`) into `main.rs` and persist `~/.last_attendance`.

---

## Priority 3 (P3): Database Timestamp Compatibility & Cloud Sync
1. Align `attendance_records.in_time` / `out_time` handling so Rust can parse both historical Python UTC timestamps (`%Y-%m-%d %H:%M:%S`) and local `Asia/Dhaka` times without 6-hour skew.
2. Enforce `company_uuid` filtering on users and attendance queries for true multi-company isolation.
3. Implement real PostgreSQL (`tokio-postgres` / `sqlx`) and S3 (`aws-sdk-s3` or `reqwest`) synchronization in `attendance-sync` if AWS cloud sync is required in production.
