use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{Shift, ShiftType};
use rusqlite::{params, Connection, OptionalExtension};

pub struct ShiftRepo;

impl ShiftRepo {
    pub fn get_by_id(conn: &Connection, id: i64) -> Result<Option<Shift>> {
        let mut stmt = conn.prepare(
            "SELECT id, company_id, name, code, shift_type, start_time, end_time,
                    grace_period_minutes, half_day_threshold_hours, min_work_hours_full_day,
                    break_duration_minutes, is_overtime_enabled, overtime_starts_after_minutes,
                    is_active, created_at
             FROM shifts WHERE id = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let shift = stmt.query_row(params![id], |row| {
            let shift_type_str: String = row.get(4).unwrap_or_else(|_| "RegularDay".to_string());
            let shift_type = match shift_type_str.as_str() {
                "Morning" => ShiftType::Morning,
                "Evening" => ShiftType::Evening,
                "NightCrossMidnight" => ShiftType::NightCrossMidnight,
                "Rotational" => ShiftType::Rotational,
                "Flexible" => ShiftType::Flexible,
                _ => ShiftType::RegularDay,
            };

            let is_ot: i32 = row.get(11)?;
            let is_act: i32 = row.get(13)?;

            Ok(Shift {
                id: row.get(0)?,
                company_id: row.get(1)?,
                name: row.get(2)?,
                code: row.get(3)?,
                shift_type,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                grace_period_minutes: row.get(7)?,
                half_day_threshold_hours: row.get(8)?,
                min_work_hours_full_day: row.get(9)?,
                break_duration_minutes: row.get(10)?,
                is_overtime_enabled: is_ot == 1,
                overtime_starts_after_minutes: row.get(12)?,
                is_active: is_act == 1,
                created_at: row.get(14)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(shift)
    }

    pub fn insert(conn: &Connection, shift: &Shift) -> Result<i64> {
        let now = TimeHelper::now_utc_sqlite_string();
        let shift_type_str = format!("{:?}", shift.shift_type);

        conn.execute(
            "INSERT INTO shifts (
                company_id, name, code, shift_type, start_time, end_time,
                grace_period_minutes, half_day_threshold_hours, min_work_hours_full_day,
                break_duration_minutes, is_overtime_enabled, overtime_starts_after_minutes,
                is_active, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            ON CONFLICT(code) DO UPDATE SET
                name = excluded.name,
                shift_type = excluded.shift_type,
                start_time = excluded.start_time,
                end_time = excluded.end_time,
                grace_period_minutes = excluded.grace_period_minutes,
                half_day_threshold_hours = excluded.half_day_threshold_hours,
                min_work_hours_full_day = excluded.min_work_hours_full_day,
                break_duration_minutes = excluded.break_duration_minutes,
                is_overtime_enabled = excluded.is_overtime_enabled,
                overtime_starts_after_minutes = excluded.overtime_starts_after_minutes,
                is_active = excluded.is_active",
            params![
                shift.company_id,
                shift.name,
                shift.code,
                shift_type_str,
                shift.start_time,
                shift.end_time,
                shift.grace_period_minutes,
                shift.half_day_threshold_hours,
                shift.min_work_hours_full_day,
                shift.break_duration_minutes,
                if shift.is_overtime_enabled { 1 } else { 0 },
                shift.overtime_starts_after_minutes,
                if shift.is_active { 1 } else { 0 },
                now
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn list_by_company(conn: &Connection, company_id: &str) -> Result<Vec<Shift>> {
        let mut stmt = conn.prepare(
            "SELECT id, company_id, name, code, shift_type, start_time, end_time,
                    grace_period_minutes, half_day_threshold_hours, min_work_hours_full_day,
                    break_duration_minutes, is_overtime_enabled, overtime_starts_after_minutes,
                    is_active, created_at
             FROM shifts WHERE company_id = ? ORDER BY id ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![company_id], |row| {
            let is_ot: i32 = row.get(11)?;
            let is_act: i32 = row.get(13)?;

            Ok(Shift {
                id: row.get(0)?,
                company_id: row.get(1)?,
                name: row.get(2)?,
                code: row.get(3)?,
                shift_type: ShiftType::RegularDay,
                start_time: row.get(5)?,
                end_time: row.get(6)?,
                grace_period_minutes: row.get(7)?,
                half_day_threshold_hours: row.get(8)?,
                min_work_hours_full_day: row.get(9)?,
                break_duration_minutes: row.get(10)?,
                is_overtime_enabled: is_ot == 1,
                overtime_starts_after_minutes: row.get(12)?,
                is_active: is_act == 1,
                created_at: row.get(14)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
