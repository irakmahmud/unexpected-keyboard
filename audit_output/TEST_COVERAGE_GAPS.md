# TEST INDEPENDENCE & COVERAGE GAPS — Phase 5 Audit of `verify_v8_full_parity.py`

> **Audit Target:** `/home/opc/attendance_system/verify_v8_full_parity.py` (197 lines) and `crates/attendance-tests/tests/*.rs`.

---

## 1. Forensic Audit of `verify_v8_full_parity.py`

Why did `verify_v8_full_parity.py` report `✅ ALL 8 PARITY TEST SUITES PASSED 100%` despite 9 broken browser modals, a 1.05 MB random-weight ONNX model, and stubbed cloud sync?

| Flaw Category | Evidence in `verify_v8_full_parity.py` | Why the Test Result Is Invalid as Proof of Parity |
|---|---|---|
| **1. Self-Comparison Biometric Fallacy** | `lines 115–138`: Generates `frames_20 = [make_synthetic_face_b64(i) for i in range(20)]`, registers `ONNX_CAM_USER` with `frames_20`, and immediately calls `/api/v1/face/verify-identity` with **`frames_20[0]`** (the exact same byte array). | Any deterministic mathematical function $f(x)$ — even a random matrix multiply or SHA-256 hash — produces distance `0.0000` when given the exact same byte array $x$ twice! It never tested two different photos of the same person or photos of two different real people. |
| **2. Synthetic Cartoon PNGs Instead of Real Faces** | `lines 41–68` (`make_synthetic_face_b64`): Draws a flat RGB circle (`dx*dx + dy*dy < 3200`) with no eyes, nose, or facial texture. | Real MTCNN rejects flat circles (`face_detected = False`). It only "succeeded" because `pipeline.rs:14–31` silently falls back to a 60% center crop when MTCNN fails! |
| **3. Zero Browser DOM / JS Execution** | `lines 22–39` (`req()`): Uses `urllib.request.urlopen` directly against REST endpoints. | Never loads `/` in a headless browser, never executes `scripts.rs`, and therefore completely missed all **9 fatal DOM ID & route mismatches** between `kiosk_view.rs` and `scripts.rs`. |
| **4. Missing Negative & Security Tests** | `lines 70–193`: Only tests happy-path status codes (`assert st == 200`). | Never tests wrong HR passwords, wrong company passwords, impostor faces (False Acceptance Rate), faceless images (empty background), duplicate face rejection across users, or SQL/path injection. |
| **5. Untested Report Endpoints** | `lines 182–190`: Only checks `/api/v1/face/embeddings-report`, `/api/v1/reports/verify-images`, and `/api/v1/attendance/user-stats/ONNX_CAM_USER`. | Did not test `/api/v1/reports/customized.csv` (the URL called by `scripts.rs:1446`, which returns 404) or `/api/v1/reports/muster-roll?month=2026-10` (which returns 400). |
| **6. Zero Historical Data Compatibility Tests** | Never loads a database or row created by `attendance_v8.py`. | Missed the UTC (`%Y-%m-%d %H:%M:%S`) vs. local `HH:MM:SS` timestamp incompatibility and `~/db/company_info.db` path difference. |
