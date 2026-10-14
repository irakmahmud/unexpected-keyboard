use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{Employee, EmploymentType};
use rusqlite::{params, Connection, OptionalExtension};

pub struct EmployeeRepo;

impl EmployeeRepo {
    pub fn get_by_code(conn: &Connection, code: &str) -> Result<Option<Employee>> {
        let mut stmt = conn.prepare(
            "SELECT id, employee_code, user_uuid, company_id, first_name, last_name, full_name,
                    email, phone, department_id, designation_id, shift_id, rfid_card_uid, pin_code,
                    employment_type, join_date, end_date, is_active, avatar_url, emergency_contact_name,
                    emergency_contact_phone, created_at, updated_at
             FROM employees WHERE UPPER(employee_code) = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let emp = stmt.query_row(params![code.to_uppercase()], |row| {
            let emp_type_str: String = row.get(14).unwrap_or_else(|_| "Permanent".to_string());
            let employment_type = match emp_type_str.as_str() {
                "Contractual" => EmploymentType::Contractual,
                "Probationary" => EmploymentType::Probationary,
                "PartTime" => EmploymentType::PartTime,
                "Intern" => EmploymentType::Intern,
                _ => EmploymentType::Permanent,
            };

            let is_active_int: i32 = row.get(17)?;

            Ok(Employee {
                id: row.get(0)?,
                employee_code: row.get(1)?,
                user_uuid: row.get(2)?,
                company_id: row.get(3)?,
                first_name: row.get(4)?,
                last_name: row.get(5)?,
                full_name: row.get(6)?,
                email: row.get(7)?,
                phone: row.get(8)?,
                department_id: row.get(9)?,
                designation_id: row.get(10)?,
                shift_id: row.get(11)?,
                rfid_card_uid: row.get(12)?,
                pin_code: row.get(13)?,
                employment_type,
                join_date: row.get(15)?,
                end_date: row.get(16)?,
                is_active: is_active_int == 1,
                avatar_url: row.get(18)?,
                emergency_contact_name: row.get(19)?,
                emergency_contact_phone: row.get(20)?,
                created_at: row.get(21)?,
                updated_at: row.get(22)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(emp)
    }

    pub fn insert_or_update(conn: &Connection, emp: &Employee) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        let emp_type_str = format!("{:?}", emp.employment_type);

        conn.execute(
            "INSERT INTO employees (
                employee_code, user_uuid, company_id, first_name, last_name, full_name,
                email, phone, department_id, designation_id, shift_id, rfid_card_uid, pin_code,
                employment_type, join_date, end_date, is_active, avatar_url, emergency_contact_name,
                emergency_contact_phone, created_at, updated_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)
            ON CONFLICT(employee_code) DO UPDATE SET
                first_name = excluded.first_name,
                last_name = excluded.last_name,
                full_name = excluded.full_name,
                email = excluded.email,
                phone = excluded.phone,
                department_id = excluded.department_id,
                designation_id = excluded.designation_id,
                shift_id = excluded.shift_id,
                rfid_card_uid = excluded.rfid_card_uid,
                pin_code = excluded.pin_code,
                employment_type = excluded.employment_type,
                is_active = excluded.is_active,
                avatar_url = excluded.avatar_url,
                updated_at = excluded.updated_at",
            params![
                emp.employee_code.to_uppercase(),
                emp.user_uuid,
                emp.company_id,
                emp.first_name,
                emp.last_name,
                emp.full_name,
                emp.email,
                emp.phone,
                emp.department_id,
                emp.designation_id,
                emp.shift_id,
                emp.rfid_card_uid,
                emp.pin_code,
                emp_type_str,
                emp.join_date,
                emp.end_date,
                if emp.is_active { 1 } else { 0 },
                emp.avatar_url,
                emp.emergency_contact_name,
                emp.emergency_contact_phone,
                emp.created_at.as_deref().unwrap_or(&now),
                now
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_by_company(conn: &Connection, company_id: &str) -> Result<Vec<Employee>> {
        let mut stmt = conn.prepare(
            "SELECT id, employee_code, user_uuid, company_id, first_name, last_name, full_name,
                    email, phone, department_id, designation_id, shift_id, rfid_card_uid, pin_code,
                    employment_type, join_date, end_date, is_active, avatar_url, emergency_contact_name,
                    emergency_contact_phone, created_at, updated_at
             FROM employees WHERE company_id = ? ORDER BY employee_code ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![company_id], |row| {
            let is_active_int: i32 = row.get(17)?;
            Ok(Employee {
                id: row.get(0)?,
                employee_code: row.get(1)?,
                user_uuid: row.get(2)?,
                company_id: row.get(3)?,
                first_name: row.get(4)?,
                last_name: row.get(5)?,
                full_name: row.get(6)?,
                email: row.get(7)?,
                phone: row.get(8)?,
                department_id: row.get(9)?,
                designation_id: row.get(10)?,
                shift_id: row.get(11)?,
                rfid_card_uid: row.get(12)?,
                pin_code: row.get(13)?,
                employment_type: EmploymentType::Permanent,
                join_date: row.get(15)?,
                end_date: row.get(16)?,
                is_active: is_active_int == 1,
                avatar_url: row.get(18)?,
                emergency_contact_name: row.get(19)?,
                emergency_contact_phone: row.get(20)?,
                created_at: row.get(21)?,
                updated_at: row.get(22)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }
}
