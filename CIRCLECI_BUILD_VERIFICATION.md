# CircleCI Remote Build & Staging Deployment Verification (`CIRCLECI_BUILD_VERIFICATION.md`)

**Date:** 2026-10-22  
**Workspace:** `/home/opc/attendance_system`  
**Remote CI Pipeline:** CircleCI (`irakmahmud/unexpected-keyboard`, branch `attendance-staging`, config `.circleci/config.yml`)  
**Policy Enforcement:** **ZERO LOCAL RUST BUILD ON ORACLE VM** — All Rust compilation (`cargo build --release --workspace`) and test execution (`cargo test --release --workspace`) were executed remotely on CircleCI `cimg/rust:1.80.0` (`large` executor).

---

## 1. Remote CircleCI Build Progression & Verification

| CircleCI Build # | Git Commit SHA | Branch | Trigger / Scope | Remote Compilation | Remote Test Suite (`cargo test --release --workspace`) | Status |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| **Build #44** | `128b3fb128473fb60743f9da3daa7ffa829e0c0b` | `attendance-staging` | Skip `REJECT` tier candidates in `FaceVectorIndex::search_tiered` | PASS (`0` errors) | `67 passed; 0 failed` | **SUCCESS** |
| **Build #45** | `744c719496c2c5c73df9c71e3176eb1ebe47bcab` | `attendance-staging` | Use `cosine_distance` in `FaceVectorIndex::search_tiered` & alias `old_password` | PASS (`0` errors) | `67 passed; 0 failed` | **SUCCESS** |
| **Build #47** | `f455dd6f0f6a28481e81540b1fc6c4321dde07dc` | `attendance-staging` | Fix `users.register_with_camera` schema/migration, biometric revocation (`users.face_encoding = NULL`), pre-write `tract-onnx` model upload validation, duplicate Android ID check, HR password overwrite protection, forgot-password bypass removal, dynamic muster roll company resolution | PASS (`0` errors) | `70 passed; 0 failed` (`21` unit + `49` integration) | **SUCCESS** |
| **Build #48** | `6ff68abfaf5278838baa9860b9297adc71048231` | `attendance-staging` | Fix UI registration JSON payload (`user_designation` field) | PASS (`0` errors) | `70 passed; 0 failed` (`21` unit + `49` integration) | **SUCCESS** |

---

## 2. Downloaded Release Binary & Artifact Checksums (Build #48)

Downloaded from CircleCI Build `#48` (`https://circleci.com/api/v1.1/project/github/irakmahmud/unexpected-keyboard/48/artifacts`) into `/home/opc/attendance_system/deploy/staging_verified/`:

| Artifact File | Local Staging Path | Size (Bytes) | SHA-256 Checksum | Architecture / Format |
| :--- | :--- | ---: | :--- | :--- |
| `attendance-api` | `deploy/staging_verified/attendance-api` | `35,833,456` (`35 MB`) | `7ecf2b339f5b9919a9a444a4b0bb65057033dd9619487d1622a37ff1679b294c` | `ELF 64-bit LSB pie executable, x86-64, dynamically linked, for GNU/Linux 3.2.0` |
| `attendance-cli` | (in `attendance-system-linux-x86_64.tar.gz`) | `5,767,168` (`5.5 MB`) | `217e6e6ede201e793d20714054495db27140c37aff4dabe0992a37f711b0d120` | `ELF 64-bit LSB pie executable, x86-64` |
| `attendance-device-agent` | (in `attendance-system-linux-x86_64.tar.gz`) | `5,767,168` (`5.5 MB`) | `1eab95858366f19018a64a83756fa0676ade72c0ba0c9581d26c24cef8b899ef` | `ELF 64-bit LSB pie executable, x86-64` |
| `attendance-system-linux-x86_64.tar.gz` | `deploy/staging_verified/attendance-system-linux-x86_64.tar.gz` | `14,246,743` (`14 MB`) | Verified against `SHA256SUMS.txt` | Gzipped tar release bundle |

---

## 3. Staging Deployment & Production Isolation (Phase 8 Compliance)

1. **Production Service Untouched**:
   - Live production `attendance-api` on port `8080` (`target/release/attendance-api`) and production database (`data/attendance.db`) were **never stopped, overwritten, or modified**.
2. **Isolated Staging Verification (`Port 8090`)**:
   - Executed `deploy/staging_verified/attendance-api` on `PORT=8090` with `ATTENDANCE_DB_PATH=/home/opc/attendance_system/data/staging_attendance.db` and `MODELS_DIR=/home/opc/attendance_system/models`.
   - Loaded all 4 authentic ONNX models (`facenet512.onnx`, `mtcnn_pnet.onnx`, `mtcnn_rnet.onnx`, `mtcnn_onet.onnx`) cleanly via `tract-onnx`.
   - Ran the full 63-check end-to-end verification suite (`scripts/run_staging_e2e_verification.py`), covering biometric verification, multi-face/zero-face rejection, duplicate face & duplicate Android ID blocking, biometric revocation & sync non-resurrection, HR password security, ONNX upload validation, dual-format database compatibility, CSV/XLSX/HTML reporting, and 25 headless Chromium Playwright UI tests.
   - **Staging Result**: **`63 / 63 PASS`** (`staging_verification_results.json`).
