# DATABASE COMPATIBILITY & HISTORICAL DATA AUDIT

> **Audit Scope:** Comparison of SQLite schema, file paths, timestamp encodings, and stored records between `attendance_v8.py` (`DatabasePool`, lines 3864–5561) and `crates/attendance-database` + live `data/attendance.db`.

---

## 1. Database File Path Divergence
- **Python (`attendance_v8.py:488, 3915`):** Stores SQLite database at `~/db/company_info.db` (`os.path.expanduser('~/db/company_info.db')`).
- **Rust (`crates/attendance-api/src/main.rs:41`):** Stores SQLite database at `data/attendance.db`.
- **Impact:** Rust does not read or migrate an existing `~/db/company_info.db` installation unless manually copied.

---

## 2. Critical Timestamp Incompatibility in `attendance_records` (`UTC` vs. Local `HH:MM:SS`)
- **Python Behavior (`attendance_v8.py:5095–5100`):**
  - Converts `Asia/Dhaka` login/logout timestamps to **full UTC datetime strings (`%Y-%m-%d %H:%M:%S`)** before inserting into `attendance_records.in_time` and `out_time`.
  - When displaying records or generating reports (`attendance_v8.py:7071, 13050`), Python parses `%Y-%m-%d %H:%M:%S` as UTC and converts it to `Asia/Dhaka` (`+06:00`).
- **Rust Behavior (`crates/attendance-core/src/attendance_engine.rs:119–120`):**
  - Formats `in_time` and `out_time` as **local `Asia/Dhaka` time-only strings (`HH:MM:SS`, e.g., `"15:11:32"`)** while SQLite sets `created_at` to UTC (`"2026-10-22 08:55:52"`).
- **Live `data/attendance.db` Evidence:**
  ```text
  record_id: e9b82f33-cbb2-44e2-9c79-ccaae376ab50_22_Oct_2026
  username:  ONNX_CAM_USER
  date:      2026-10-22
  in_time:   14:55:52   (Asia/Dhaka local HH:MM:SS)
  out_time:  15:11:32   (Asia/Dhaka local HH:MM:SS)
  created_at: 2026-10-22 08:55:52 (UTC)
  ```
- **Impact:**
  1. If Python reads Rust-created rows, `datetime.strptime(in_time, '%Y-%m-%d %H:%M:%S')` throws a `ValueError`.
  2. If Rust reads historical Python-created rows, Rust displays the raw UTC timestamp without adding `+06:00` (off by 6 hours) and fails `NaiveTime::parse_from_str(..., "%H:%M:%S")` in duration/late calculations!

---

## 3. Date Column Format Divergence (`DD_Mon_YYYY` vs. `YYYY-MM-DD`)
- **Python (`attendance_v8.py:5087`):** Stores `date` in `attendance_records` as `%d_%b_%Y` (e.g., `"22_Oct_2026"`).
- **Rust (`crates/attendance-core/src/attendance_engine.rs:112–118`):** Uses `%d_%b_%Y` when constructing `record_id` (`<uuid>_22_Oct_2026`), but stores `%Y-%m-%d` (`"2026-10-22"`) in the `date` column.

---

## 4. Corrupted / Synthetic Seed Data in `data/attendance.db`
Read-only forensic inspection of `data/attendance.db` (`crates/attendance-database/src/seed.rs`) revealed:
1. **`face_profiles` table (31 rows):**
   - Seeded employees `EMP001` through `EMP010` (`rows 1–10`) **all have the exact same constant 512-float vector** `[0.04419417382415922; 512]` ($1/\sqrt{512} pprox 0.044194$)! Because all 10 users have identical vectors, any query near `[0.044194; 512]` collides across all 10 seeded employees.
2. **`users` table (9 rows):**
   - Seeded users `C_DEV001`..`C_OPS004` (`RAHIM`, `KARIM`, `FATEMA`, `JAMAL`, `NASRIN`, `TARIQ`) have synthetic arithmetic ramp vectors (`[0.0, 0.0001953, 0.0003906, ...]`) that cannot match any real camera image.
   - `ONNX_CAM_USER` has 21 vectors generated from the 1.05 MB random-weight stub model.
