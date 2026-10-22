use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{AttendanceRecord, AttendanceStatus, PunchLog};
use rusqlite::{params, Connection, OptionalExtension};

pub struct AttendanceRepo;

impl AttendanceRepo {
    fn normalize_db_time(raw: Option<String>) -> Option<String> {
        raw.map(|s| {
            let trimmed = s.trim();
            if trimmed.len() == 19 && trimmed.chars().nth(4) == Some('-') && trimmed.contains(' ') {
                if let Ok(dhaka_full) = TimeHelper::utc_to_dhaka_string(trimmed) {
                    if let Some(time_part) = dhaka_full.split_whitespace().nth(1) {
                        return time_part.to_string();
                    }
                }
            }
            trimmed.to_string()
        })
    }

    pub fn get_record(conn: &Connection, username: &str, date: &str) -> Result<Option<AttendanceRecord>> {
        let upper = username.to_uppercase();
        let (iso_date, py_date) = TimeHelper::date_variants(date);
        let mut stmt = conn.prepare(
            "SELECT record_id, company_user_uuid, username, date, in_time, out_time,
                    video_path_in, video_path_out, in_time_with_camera, out_time_with_camera,
                    in_time_with_qr, out_time_with_qr, status, work_hours, overtime_hours,
                    late_minutes, early_exit_minutes, ip_address, device_id, remarks, created_at
             FROM attendance_records WHERE UPPER(username) = ?1 AND (date = ?2 OR date = ?3)"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rec = stmt.query_row(params![upper, iso_date, py_date], |row| {
            let status_str: String = row.get(12).unwrap_or_else(|_| "PRESENT".to_string());
            let status = match status_str.as_str() {
                "LATE" => AttendanceStatus::Late,
                "LATE_ABSENT" => AttendanceStatus::LateAbsent,
                "HALF_DAY" => AttendanceStatus::HalfDay,
                "EARLY_DEPARTURE" => AttendanceStatus::EarlyDeparture,
                "ABSENT" => AttendanceStatus::Absent,
                "ON_LEAVE" => AttendanceStatus::OnLeave,
                "HOLIDAY" => AttendanceStatus::Holiday,
                "WEEKEND" => AttendanceStatus::Weekend,
                _ => AttendanceStatus::Present,
            };

            Ok(AttendanceRecord {
                record_id: row.get(0)?,
                company_user_uuid: row.get(1)?,
                username: row.get(2)?,
                date: row.get(3)?,
                in_time: Self::normalize_db_time(row.get(4)?),
                out_time: Self::normalize_db_time(row.get(5)?),
                video_path_in: row.get(6)?,
                video_path_out: row.get(7)?,
                in_time_with_camera: row.get(8)?,
                out_time_with_camera: row.get(9)?,
                in_time_with_qr: row.get(10)?,
                out_time_with_qr: row.get(11)?,
                status,
                work_hours: row.get(13)?,
                overtime_hours: row.get(14)?,
                late_minutes: row.get(15)?,
                early_exit_minutes: row.get(16)?,
                ip_address: row.get(17)?,
                device_id: row.get(18)?,
                remarks: row.get(19)?,
                created_at: row.get(20)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(rec)
    }

    pub fn insert_in_time(conn: &Connection, rec: &AttendanceRecord) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        let status_str = rec.status.to_string();

        conn.execute(
            "INSERT INTO attendance_records (
                record_id, company_user_uuid, username, date, in_time, out_time,
                video_path_in, in_time_with_camera, in_time_with_qr, status, work_hours,
                overtime_hours, late_minutes, early_exit_minutes, ip_address, device_id, remarks, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
            ON CONFLICT(username, date) DO UPDATE SET
                in_time = COALESCE(attendance_records.in_time, excluded.in_time),
                video_path_in = COALESCE(attendance_records.video_path_in, excluded.video_path_in),
                status = excluded.status,
                late_minutes = excluded.late_minutes",
            params![
                rec.record_id,
                rec.company_user_uuid,
                rec.username.to_uppercase(),
                rec.date,
                rec.in_time,
                rec.video_path_in,
                rec.in_time_with_camera,
                rec.in_time_with_qr,
                status_str,
                rec.work_hours,
                rec.overtime_hours,
                rec.late_minutes,
                rec.early_exit_minutes,
                rec.ip_address,
                rec.device_id,
                rec.remarks,
                rec.created_at.as_deref().unwrap_or(&now)
            ],
        ).map_err(|e| AppError::Database(format!("Failed to insert in_time: {}", e)))?;

        // Queue BigQuery Outbox event
        let event_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::to_string(rec)?;
        let _ = conn.execute(
            "INSERT INTO bigquery_outbox (event_id, table_name, operation, payload_json, created_at, updated_at)
             VALUES (?1, 'attendance_records', 'INSERT_IN_TIME', ?2, ?3, ?3)",
            params![event_id, payload, now],
        );

        Ok(())
    }

    pub fn update_out_time_detailed(
        conn: &Connection,
        username: &str,
        date: &str,
        out_time: &str,
        video_path: Option<&str>,
        with_camera: bool,
        with_qr: bool,
        work_hours: f64,
        overtime_hours: f64,
        early_exit_minutes: u32,
        status: Option<&str>,
    ) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        let cam_flag = if with_camera { "Y" } else { "N" };
        let qr_flag = if with_qr { "Y" } else { "N" };
        let (iso_date, py_date) = TimeHelper::date_variants(date);

        let affected = conn.execute(
            "UPDATE attendance_records
             SET out_time = ?1,
                 video_path_out = ?2,
                 out_time_with_camera = ?3,
                 out_time_with_qr = ?4,
                 work_hours = ?5,
                 overtime_hours = ?6,
                 early_exit_minutes = ?7,
                 status = COALESCE(?8, status)
             WHERE UPPER(username) = ?9 AND (date = ?10 OR date = ?11)",
            params![
                out_time,
                video_path,
                cam_flag,
                qr_flag,
                work_hours,
                overtime_hours,
                early_exit_minutes,
                status,
                username.to_uppercase(),
                iso_date,
                py_date
            ],
        ).map_err(|e| AppError::Database(format!("Failed to update out_time: {}", e)))?;

        if affected == 0 {
            let comp_uuid_opt: Option<String> = conn
                .query_row(
                    "SELECT company_user_uuid FROM users WHERE UPPER(username) = ?1",
                    params![username.to_uppercase()],
                    |r| r.get(0),
                )
                .optional()
                .unwrap_or(None);

            let upper_uname = username.to_uppercase();
            let comp_id = comp_uuid_opt.as_deref().unwrap_or(&upper_uname);
            let record_id = TimeHelper::generate_record_id(comp_id, date);
            let stat = status.unwrap_or("PRESENT");
            conn.execute(
                "INSERT INTO attendance_records (
                    record_id, company_user_uuid, username, date, out_time, video_path_out, out_time_with_camera,
                    out_time_with_qr, work_hours, overtime_hours, early_exit_minutes, status, created_at
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
                params![
                    record_id,
                    comp_uuid_opt,
                    username.to_uppercase(),
                    date,
                    out_time,
                    video_path,
                    cam_flag,
                    qr_flag,
                    work_hours,
                    overtime_hours,
                    early_exit_minutes,
                    stat,
                    now
                ],
            ).map_err(|e| AppError::Database(e.to_string()))?;
        }

        // Outbox event
        let event_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::json!({
            "username": username.to_uppercase(),
            "date": date,
            "out_time": out_time,
            "work_hours": work_hours,
            "overtime_hours": overtime_hours,
            "early_exit_minutes": early_exit_minutes,
            "status": status.unwrap_or("PRESENT")
        }).to_string();
        let _ = conn.execute(
            "INSERT INTO bigquery_outbox (event_id, table_name, operation, payload_json, created_at, updated_at)
             VALUES (?1, 'attendance_records', 'UPDATE_OUT_TIME', ?2, ?3, ?3)",
            params![event_id, payload, now],
        );

        Ok(())
    }

    pub fn update_out_time(
        conn: &Connection,
        username: &str,
        date: &str,
        out_time: &str,
        video_path: Option<&str>,
        with_camera: bool,
        with_qr: bool,
        work_hours: f64,
        overtime_hours: f64,
    ) -> Result<()> {
        Self::update_out_time_detailed(
            conn,
            username,
            date,
            out_time,
            video_path,
            with_camera,
            with_qr,
            work_hours,
            overtime_hours,
            0,
            None,
        )
    }

    pub fn log_punch_event(conn: &Connection, log: &PunchLog) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        conn.execute(
            "INSERT INTO punch_logs (
                username, punch_time, punch_type, punch_method, device_id, ip_address, liveness_score, verified_by_face, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                log.username.to_uppercase(),
                log.punch_time,
                log.punch_type.to_string(),
                log.punch_method.to_string(),
                log.device_id,
                log.ip_address,
                log.liveness_score,
                if log.verified_by_face { 1 } else { 0 },
                now
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_by_date(conn: &Connection, date: &str) -> Result<Vec<AttendanceRecord>> {
        let (iso_date, py_date) = TimeHelper::date_variants(date);
        let mut stmt = conn.prepare(
            "SELECT record_id, company_user_uuid, username, date, in_time, out_time,
                    video_path_in, video_path_out, in_time_with_camera, out_time_with_camera,
                    in_time_with_qr, out_time_with_qr, status, work_hours, overtime_hours,
                    late_minutes, early_exit_minutes, ip_address, device_id, remarks, created_at
             FROM attendance_records WHERE date = ?1 OR date = ?2 ORDER BY in_time DESC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![iso_date, py_date], |row| {
            let status_str: String = row.get(12).unwrap_or_else(|_| "PRESENT".to_string());
            let status = match status_str.as_str() {
                "LATE" => AttendanceStatus::Late,
                "LATE_ABSENT" => AttendanceStatus::LateAbsent,
                "HALF_DAY" => AttendanceStatus::HalfDay,
                "EARLY_DEPARTURE" => AttendanceStatus::EarlyDeparture,
                "ABSENT" => AttendanceStatus::Absent,
                "ON_LEAVE" => AttendanceStatus::OnLeave,
                "HOLIDAY" => AttendanceStatus::Holiday,
                "WEEKEND" => AttendanceStatus::Weekend,
                _ => AttendanceStatus::Present,
            };

            Ok(AttendanceRecord {
                record_id: row.get(0)?,
                company_user_uuid: row.get(1)?,
                username: row.get(2)?,
                date: row.get(3)?,
                in_time: Self::normalize_db_time(row.get(4)?),
                out_time: Self::normalize_db_time(row.get(5)?),
                video_path_in: row.get(6)?,
                video_path_out: row.get(7)?,
                in_time_with_camera: row.get(8)?,
                out_time_with_camera: row.get(9)?,
                in_time_with_qr: row.get(10)?,
                out_time_with_qr: row.get(11)?,
                status,
                work_hours: row.get(13)?,
                overtime_hours: row.get(14)?,
                late_minutes: row.get(15)?,
                early_exit_minutes: row.get(16)?,
                ip_address: row.get(17)?,
                device_id: row.get(18)?,
                remarks: row.get(19)?,
                created_at: row.get(20)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_records_by_range(
        conn: &Connection,
        from_date: &str,
        to_date: &str,
        username: Option<&str>,
    ) -> Result<Vec<AttendanceRecord>> {
        let start_naive = TimeHelper::parse_date(from_date).ok();
        let end_naive = TimeHelper::parse_date(to_date).ok();

        let (sql, params_vec): (String, Vec<rusqlite::types::Value>) = if let Some(u) = username {
            (
                "SELECT record_id, company_user_uuid, username, date, in_time, out_time,
                        video_path_in, video_path_out, in_time_with_camera, out_time_with_camera,
                        in_time_with_qr, out_time_with_qr, status, work_hours, overtime_hours,
                        late_minutes, early_exit_minutes, ip_address, device_id, remarks, created_at
                 FROM attendance_records 
                 WHERE UPPER(username) = ?1
                 ORDER BY date ASC, in_time ASC".to_string(),
                vec![rusqlite::types::Value::Text(u.to_uppercase())],
            )
        } else {
            (
                "SELECT record_id, company_user_uuid, username, date, in_time, out_time,
                        video_path_in, video_path_out, in_time_with_camera, out_time_with_camera,
                        in_time_with_qr, out_time_with_qr, status, work_hours, overtime_hours,
                        late_minutes, early_exit_minutes, ip_address, device_id, remarks, created_at
                 FROM attendance_records 
                 ORDER BY date ASC, in_time ASC".to_string(),
                vec![],
            )
        };

        let mut stmt = conn.prepare(&sql).map_err(|e| AppError::Database(e.to_string()))?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| {
            let status_str: String = row.get(12).unwrap_or_else(|_| "PRESENT".to_string());
            let status = match status_str.as_str() {
                "LATE" => AttendanceStatus::Late,
                "LATE_ABSENT" => AttendanceStatus::LateAbsent,
                "HALF_DAY" => AttendanceStatus::HalfDay,
                "EARLY_DEPARTURE" => AttendanceStatus::EarlyDeparture,
                "ABSENT" => AttendanceStatus::Absent,
                "ON_LEAVE" => AttendanceStatus::OnLeave,
                "HOLIDAY" => AttendanceStatus::Holiday,
                "WEEKEND" => AttendanceStatus::Weekend,
                _ => AttendanceStatus::Present,
            };

            let raw_date: String = row.get(3)?;
            let norm_date = TimeHelper::parse_date(&raw_date)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or(raw_date);

            Ok(AttendanceRecord {
                record_id: row.get(0)?,
                company_user_uuid: row.get(1)?,
                username: row.get(2)?,
                date: norm_date,
                in_time: Self::normalize_db_time(row.get(4)?),
                out_time: Self::normalize_db_time(row.get(5)?),
                video_path_in: row.get(6)?,
                video_path_out: row.get(7)?,
                in_time_with_camera: row.get(8)?,
                out_time_with_camera: row.get(9)?,
                in_time_with_qr: row.get(10)?,
                out_time_with_qr: row.get(11)?,
                status,
                work_hours: row.get(13)?,
                overtime_hours: row.get(14)?,
                late_minutes: row.get(15)?,
                early_exit_minutes: row.get(16)?,
                ip_address: row.get(17)?,
                device_id: row.get(18)?,
                remarks: row.get(19)?,
                created_at: row.get(20)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            let rec = r.map_err(|e| AppError::Database(e.to_string()))?;
            if let (Some(s), Some(e)) = (start_naive, end_naive) {
                if let Ok(rec_d) = TimeHelper::parse_date(&rec.date) {
                    if rec_d < s || rec_d > e {
                        continue;
                    }
                }
            } else if rec.date.as_str() < from_date || rec.date.as_str() > to_date {
                continue;
            }
            list.push(rec);
        }
        list.sort_by(|a, b| a.date.cmp(&b.date).then_with(|| a.in_time.cmp(&b.in_time)));
        Ok(list)
    }

    pub fn get_user_punch_logs(
        conn: &Connection,
        username: &str,
        limit: usize,
    ) -> Result<Vec<PunchLog>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, punch_time, punch_type, punch_method, device_id, ip_address, liveness_score, verified_by_face, created_at
             FROM punch_logs WHERE UPPER(username) = ?1 ORDER BY id DESC LIMIT ?2"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![username.to_uppercase(), limit as i64], |row| {
            let p_type_str: String = row.get(3)?;
            let p_method_str: String = row.get(4)?;
            let v_face: i32 = row.get(8)?;

            let p_type = if p_type_str == "OUT_TIME" || p_type_str == "OUT" {
                attendance_domain::LoginType::OutTime
            } else {
                attendance_domain::LoginType::InTime
            };

            let p_method = match p_method_str.as_str() {
                "QR" => attendance_domain::PunchMethod::Qr,
                "RFID" => attendance_domain::PunchMethod::Rfid,
                "PIN" => attendance_domain::PunchMethod::Pin,
                "MANUAL" => attendance_domain::PunchMethod::Manual,
                "ANDROID_ID" => attendance_domain::PunchMethod::AndroidId,
                _ => attendance_domain::PunchMethod::Camera,
            };

            Ok(PunchLog {
                id: Some(row.get(0)?),
                username: row.get(1)?,
                punch_time: row.get(2)?,
                punch_type: p_type,
                punch_method: p_method,
                device_id: row.get(5)?,
                ip_address: row.get(6)?,
                liveness_score: row.get(7)?,
                verified_by_face: v_face == 1,
                created_at: Some(row.get(9)?),
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
