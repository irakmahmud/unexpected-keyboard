use attendance_common::{AppError, Result};
use attendance_domain::{Holiday, HolidayType};
use rusqlite::{params, Connection};

pub struct HolidayRepo;

impl HolidayRepo {
    pub fn insert(conn: &Connection, holiday: &Holiday) -> Result<i64> {
        let type_str = format!("{:?}", holiday.holiday_type);
        conn.execute(
            "INSERT INTO holidays (company_id, name, holiday_date, holiday_type, is_recurring_yearly, description)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(company_id, holiday_date) DO UPDATE SET
                name = excluded.name,
                holiday_type = excluded.holiday_type,
                description = excluded.description",
            params![
                holiday.company_id,
                holiday.name,
                holiday.holiday_date,
                type_str,
                if holiday.is_recurring_yearly { 1 } else { 0 },
                holiday.description
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn is_holiday(conn: &Connection, company_id: &str, date: &str) -> Result<bool> {
        let mut stmt = conn.prepare(
            "SELECT COUNT(*) FROM holidays WHERE company_id = ? AND holiday_date = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let count: i64 = stmt.query_row(params![company_id, date], |row| row.get(0))
            .map_err(|e| AppError::Database(e.to_string()))?;

        Ok(count > 0)
    }

    pub fn list_holidays_in_year(conn: &Connection, company_id: &str, year: &str) -> Result<Vec<Holiday>> {
        let pattern = format!("{}-%", year);
        let mut stmt = conn.prepare(
            "SELECT id, company_id, name, holiday_date, holiday_type, is_recurring_yearly, description
             FROM holidays WHERE company_id = ? AND holiday_date LIKE ? ORDER BY holiday_date ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![company_id, pattern], |row| {
            let is_rec: i32 = row.get(5)?;
            Ok(Holiday {
                id: row.get(0)?,
                company_id: row.get(1)?,
                name: row.get(2)?,
                holiday_date: row.get(3)?,
                holiday_type: HolidayType::NationalPublic,
                is_recurring_yearly: is_rec == 1,
                description: row.get(6)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn list_all_holidays(conn: &Connection, company_id: &str) -> Result<Vec<Holiday>> {
        let mut stmt = conn.prepare(
            "SELECT id, company_id, name, holiday_date, holiday_type, is_recurring_yearly, description
             FROM holidays WHERE company_id = ? ORDER BY holiday_date ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![company_id], |row| {
            let is_rec: i32 = row.get(5)?;
            Ok(Holiday {
                id: row.get(0)?,
                company_id: row.get(1)?,
                name: row.get(2)?,
                holiday_date: row.get(3)?,
                holiday_type: HolidayType::NationalPublic,
                is_recurring_yearly: is_rec == 1,
                description: row.get(6)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
