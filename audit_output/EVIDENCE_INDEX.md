# FORENSIC EVIDENCE INDEX

| Evidence ID | Artifact / File Path | Lines / Checksum | What It Proves |
|---|---|---|---|
| **EV-01** | `/home/opc/attendance_system/attendance_v8.py` | Lines 1–14,723 (SHA-256 verified, 664,645 bytes) | Exact ground-truth inventory: 9 top-level classes, 6 inner classes, 6 top-level functions, 252 methods, 130 nested callbacks (397 callable/class symbols). |
| **EV-02** | `/home/opc/attendance_system/generate_onnx_models.py` | Lines 1–242 | Proves `models/facenet512.onnx` (1,049,690 bytes) and `models/mtcnn_*.onnx` were generated synthetically using `np.random.RandomState(42)`. |
| **EV-03** | `/home/opc/attendance_system/models/facenet512.onnx` | `1,049,690` bytes, SHA-256 `a58a4fe8bb65b21c...` | Proves the active FaceNet512 ONNX file is a 1.05 MB 2-node (`AveragePool` + random `MatMul`) stub, not the ~95 MB pretrained Inception-ResNet-v1 model. |
| **EV-04** | `/home/opc/attendance_system/rust_deepFace/export_to_onnx.py` | Lines 1–89 | Proves a real Keras-to-ONNX exporter was written during early prototyping but never used to populate `/home/opc/attendance_system/models/`. |
| **EV-05** | `/home/opc/attendance_system/crates/attendance-face/src/pipeline.rs` | Lines 10–43, 121–154 | Proves the 60% center-crop fallback when MTCNN detects zero faces, allowing faceless images to generate embeddings. |
| **EV-06** | `/home/opc/attendance_system/crates/attendance-api/src/ui/kiosk_view.rs` & `scripts.rs` | `kiosk_view.rs:26, 286–341, 431, 483, 629, 680` vs `scripts.rs:99, 122, 141, 503, 738, 885, 934, 1036, 1446` | Proves all 9 fatal DOM ID, JSON key, and URL route mismatches that break browser modals and actions. |
| **EV-07** | `/home/opc/attendance_system/crates/attendance-auth/src/hr_auth.rs` & `company_routes.rs` | `hr_auth.rs:28`, `company_routes.rs:82` | Proves hardcoded backdoor passwords (`"HR@123"`, `"1234"`, `"admin123"`, `"hr2026"`). |
| **EV-08** | `/home/opc/attendance_system/crates/attendance-bigquery/src/client.rs` | Lines 76–98 | Proves `BigQueryClient::stream_batch` performs zero network I/O and is a logging stub. |
| **EV-09** | `/home/opc/attendance_system/data/attendance.db` | Read-only SQLite inspection | Proves `EMP001..EMP010` share identical constant vectors `[0.044194; 512]` and `in_time`/`out_time` are stored as local `HH:MM:SS` instead of UTC `%Y-%m-%d %H:%M:%S`. |
| **EV-10** | `/home/opc/attendance_system/verify_v8_full_parity.py` | Lines 41–68, 115–138 | Proves previous 100% test pass relied on synthetic circles and self-comparison (`frames_20[0]` vs `frames_20[0]`) without browser DOM testing. |
