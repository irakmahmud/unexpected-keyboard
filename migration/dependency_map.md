# Dependency Map: Python to Pure Rust Replacement

| Python Library / Module | Purpose in `attendance_v8.py` | Rust Replacement Crate | Status / Strategy |
|---|---|---|---|
| `faiss` | 512D Vector Index & Similarity Search | Pure Rust Vector Matcher (L2 / Cosine SIMD) & `hnsw_rs` / `ndarray` | MIGRATED: Zero C++ ABI issues, instant startup |
| `opencv-python` (`cv2`) | Camera capture, frame processing, face bounding box | Pure Rust `rscam` / `nokhwa` / WebRTC client / Axum Multipart ingestion | MIGRATED: Client device capture + Axum ingest |
| `deepface` / `keras` | Face embeddings generation | Rust ONNX Runtime / `tract-onnx` / Device Face Client | REPLACED_WITH_EQUIVALENT |
| `psycopg2` / AWS RDS | Remote PostgreSQL synchronization | Google BigQuery Streaming Ingestion + Durable Outbox | REPLACED_WITH_EQUIVALENT |
| `boto3` / AWS S3 | Remote video & image storage | Local encrypted storage + BigQuery metadata outbox + Cloud Storage API | REPLACED_WITH_EQUIVALENT |
| `sqlite3` | Local relational database | `sqlx` with SQLite runtime, WAL mode & pool | MIGRATED: Full async safety |
| `tkinter` / `ttk` | Graphical User Interface & Kiosk | Native Client / Embedded Axum Web Kiosk (HTML5/Tailwind/JS) | CLIENT_SIDE_RESPONSIBILITY |
| `socket` (port 12345) | Single instance lock | `fs2` File locks & socket bind / OS mutex | MIGRATED |
| `winreg` / `win32com` | Windows registry & autostart | Platform-agnostic systemd service / cross-platform autostart | REPLACED_WITH_EQUIVALENT |
| `pytz` / `datetime` | Asia/Dhaka timezone conversion | `chrono` with Asia/Dhaka FixedOffset (+06:00) | MIGRATED |
| `pickle` | Storing embeddings & cache | `serde_json` / `bincode` (Safe, no code execution vulnerability) | REPLACED_WITH_EQUIVALENT (Secure) |
