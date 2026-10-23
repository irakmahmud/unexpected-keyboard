use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{CompanyInfo, Department, HrPassword};
use rusqlite::{params, Connection, OptionalExtension};

pub struct CompanyRepo;

impl CompanyRepo {
    pub fn get_company_by_id(conn: &Connection, company_id: &str) -> Result<Option<CompanyInfo>> {
        let mut stmt = conn.prepare(
            "SELECT company_id, company_name, password, address_road, address_city,
                    contact_person_name, contact_person_designation, contact_person_number,
                    website, email, tax_id, created_at, last_updated
             FROM company_info WHERE company_id = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let company = stmt.query_row(params![company_id], |row| {
            Ok(CompanyInfo {
                company_id: row.get(0)?,
                company_name: row.get(1)?,
                password: row.get(2)?,
                address_road: row.get(3)?,
                address_city: row.get(4)?,
                contact_person_name: row.get(5)?,
                contact_person_designation: row.get(6)?,
                contact_person_number: row.get(7)?,
                website: row.get(8)?,
                email: row.get(9)?,
                tax_id: row.get(10)?,
                created_at: row.get(11)?,
                last_updated: row.get(12)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(company)
    }

    pub fn list_companies(conn: &Connection) -> Result<Vec<CompanyInfo>> {
        let mut stmt = conn.prepare(
            "SELECT company_id, company_name, password, address_road, address_city,
                    contact_person_name, contact_person_designation, contact_person_number,
                    website, email, tax_id, created_at, last_updated
             FROM company_info ORDER BY company_name ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            Ok(CompanyInfo {
                company_id: row.get(0)?,
                company_name: row.get(1)?,
                password: row.get(2)?,
                address_road: row.get(3)?,
                address_city: row.get(4)?,
                contact_person_name: row.get(5)?,
                contact_person_designation: row.get(6)?,
                contact_person_number: row.get(7)?,
                website: row.get(8)?,
                email: row.get(9)?,
                tax_id: row.get(10)?,
                created_at: row.get(11)?,
                last_updated: row.get(12)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn upsert_company(conn: &Connection, info: &CompanyInfo) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        conn.execute(
            "INSERT INTO company_info (
                company_id, company_name, password, address_road, address_city,
                contact_person_name, contact_person_designation, contact_person_number,
                website, email, tax_id, created_at, last_updated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(company_id) DO UPDATE SET
                company_name = excluded.company_name,
                password = excluded.password,
                address_road = COALESCE(excluded.address_road, company_info.address_road),
                address_city = COALESCE(excluded.address_city, company_info.address_city),
                contact_person_name = COALESCE(excluded.contact_person_name, company_info.contact_person_name),
                contact_person_designation = COALESCE(excluded.contact_person_designation, company_info.contact_person_designation),
                contact_person_number = COALESCE(excluded.contact_person_number, company_info.contact_person_number),
                website = COALESCE(excluded.website, company_info.website),
                email = COALESCE(excluded.email, company_info.email),
                tax_id = COALESCE(excluded.tax_id, company_info.tax_id),
                last_updated = excluded.last_updated",
            params![
                info.company_id,
                info.company_name,
                info.password,
                info.address_road,
                info.address_city,
                info.contact_person_name,
                info.contact_person_designation,
                info.contact_person_number,
                info.website,
                info.email,
                info.tax_id,
                info.created_at.as_deref().unwrap_or(&now),
                now
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_hr_password(conn: &Connection, company_uuid: &str) -> Result<Option<HrPassword>> {
        let mut stmt = conn.prepare(
            "SELECT id, company_uuid, company_name, password, created_at, last_updated
             FROM hr_passwords WHERE company_uuid = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let hr = stmt.query_row(params![company_uuid], |row| {
            Ok(HrPassword {
                id: row.get(0)?,
                company_uuid: row.get(1)?,
                company_name: row.get(2)?,
                password: row.get(3)?,
                created_at: row.get(4)?,
                last_updated: row.get(5)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(hr)
    }

    pub fn set_hr_password(conn: &Connection, hr: &HrPassword) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        conn.execute(
            "INSERT INTO hr_passwords (company_uuid, company_name, password, created_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(company_uuid) DO UPDATE SET
                password = excluded.password,
                last_updated = excluded.last_updated",
            params![hr.company_uuid, hr.company_name, hr.password, now, now],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn list_departments(conn: &Connection, company_id: &str) -> Result<Vec<Department>> {
        let mut stmt = conn.prepare(
            "SELECT id, company_id, name, code, description, head_employee_id, created_at
             FROM departments WHERE company_id = ? ORDER BY name ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map(params![company_id], |row| {
            Ok(Department {
                id: row.get(0)?,
                company_id: row.get(1)?,
                name: row.get(2)?,
                code: row.get(3)?,
                description: row.get(4)?,
                head_employee_id: row.get(5)?,
                created_at: row.get(6)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn save_department(conn: &Connection, dept: &Department) -> Result<i64> {
        let now = TimeHelper::now_utc_sqlite_string();
        conn.execute(
            "INSERT INTO departments (company_id, name, code, description, head_employee_id, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)
             ON CONFLICT(company_id, name) DO UPDATE SET
                code = excluded.code,
                description = excluded.description,
                head_employee_id = excluded.head_employee_id",
            params![
                dept.company_id,
                dept.name,
                dept.code,
                dept.description,
                dept.head_employee_id,
                dept.created_at.as_deref().unwrap_or(&now),
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }
}
