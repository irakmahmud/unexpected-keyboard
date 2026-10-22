use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{User, UserRole, UserStatus};
use rusqlite::{params, Connection, OptionalExtension};

pub struct UserRepo;

impl UserRepo {
    pub fn get_by_username(conn: &Connection, username: &str) -> Result<Option<User>> {
        let upper = username.to_uppercase();
        let mut stmt = conn.prepare(
            "SELECT company_user_uuid, company_name, company_uuid, username, user_uuid,
                    user_designation, user_android_id, face_encoding, password, role, status, email, phone, created_at, last_updated,
                    COALESCE(NULLIF(register_with_camera, ''), CASE WHEN face_encoding IS NOT NULL AND face_encoding != '' THEN 'Y' ELSE 'N' END)
             FROM users WHERE UPPER(username) = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let user = stmt.query_row(params![upper], |row| {
            let role_str: String = row.get(9).unwrap_or_else(|_| "Employee".to_string());
            let status_str: String = row.get(10).unwrap_or_else(|_| "Active".to_string());

            let role = match role_str.as_str() {
                "SuperAdmin" => UserRole::SuperAdmin,
                "CompanyAdmin" => UserRole::CompanyAdmin,
                "HrManager" => UserRole::HrManager,
                "DepartmentHead" => UserRole::DepartmentHead,
                "SecurityGuard" => UserRole::SecurityGuard,
                "KioskTerminal" => UserRole::KioskTerminal,
                _ => UserRole::Employee,
            };

            let status = match status_str.as_str() {
                "Inactive" => UserStatus::Inactive,
                "Suspended" => UserStatus::Suspended,
                "Terminated" => UserStatus::Terminated,
                "PendingApproval" => UserStatus::PendingApproval,
                _ => UserStatus::Active,
            };

            Ok(User {
                company_user_uuid: row.get(0)?,
                company_name: row.get(1)?,
                company_uuid: row.get(2)?,
                username: row.get(3)?,
                user_uuid: row.get(4)?,
                user_designation: row.get(5)?,
                user_android_id: row.get(6)?,
                face_encoding: row.get(7)?,
                register_with_camera: row.get(15).ok(),
                password: row.get(8)?,
                role,
                status,
                email: row.get(11)?,
                phone: row.get(12)?,
                created_at: row.get(13)?,
                last_updated: row.get(14)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(user)
    }

    pub fn get_by_android_id(conn: &Connection, android_id: &str) -> Result<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT company_user_uuid, company_name, company_uuid, username, user_uuid,
                    user_designation, user_android_id, face_encoding, password, role, status, email, phone, created_at, last_updated,
                    COALESCE(NULLIF(register_with_camera, ''), CASE WHEN face_encoding IS NOT NULL AND face_encoding != '' THEN 'Y' ELSE 'N' END)
             FROM users WHERE user_android_id = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let user = stmt.query_row(params![android_id], |row| {
            let role_str: String = row.get(9).unwrap_or_else(|_| "Employee".to_string());
            let status_str: String = row.get(10).unwrap_or_else(|_| "Active".to_string());

            let role = match role_str.as_str() {
                "SuperAdmin" => UserRole::SuperAdmin,
                "CompanyAdmin" => UserRole::CompanyAdmin,
                "HrManager" => UserRole::HrManager,
                "DepartmentHead" => UserRole::DepartmentHead,
                "SecurityGuard" => UserRole::SecurityGuard,
                "KioskTerminal" => UserRole::KioskTerminal,
                _ => UserRole::Employee,
            };

            let status = match status_str.as_str() {
                "Inactive" => UserStatus::Inactive,
                "Suspended" => UserStatus::Suspended,
                "Terminated" => UserStatus::Terminated,
                "PendingApproval" => UserStatus::PendingApproval,
                _ => UserStatus::Active,
            };

            Ok(User {
                company_user_uuid: row.get(0)?,
                company_name: row.get(1)?,
                company_uuid: row.get(2)?,
                username: row.get(3)?,
                user_uuid: row.get(4)?,
                user_designation: row.get(5)?,
                user_android_id: row.get(6)?,
                face_encoding: row.get(7)?,
                register_with_camera: row.get(15).ok(),
                password: row.get(8)?,
                role,
                status,
                email: row.get(11)?,
                phone: row.get(12)?,
                created_at: row.get(13)?,
                last_updated: row.get(14)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(user)
    }

    pub fn get_all(conn: &Connection) -> Result<Vec<User>> {
        let mut stmt = conn.prepare(
            "SELECT company_user_uuid, company_name, company_uuid, username, user_uuid,
                    user_designation, user_android_id, face_encoding, password, role, status, email, phone, created_at, last_updated,
                    COALESCE(NULLIF(register_with_camera, ''), CASE WHEN face_encoding IS NOT NULL AND face_encoding != '' THEN 'Y' ELSE 'N' END)
             FROM users ORDER BY username ASC"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt.query_map([], |row| {
            let role_str: String = row.get(9).unwrap_or_else(|_| "Employee".to_string());
            let status_str: String = row.get(10).unwrap_or_else(|_| "Active".to_string());

            let role = match role_str.as_str() {
                "SuperAdmin" => UserRole::SuperAdmin,
                "CompanyAdmin" => UserRole::CompanyAdmin,
                "HrManager" => UserRole::HrManager,
                _ => UserRole::Employee,
            };

            let status = match status_str.as_str() {
                "Inactive" => UserStatus::Inactive,
                _ => UserStatus::Active,
            };

            Ok(User {
                company_user_uuid: row.get(0)?,
                company_name: row.get(1)?,
                company_uuid: row.get(2)?,
                username: row.get(3)?,
                user_uuid: row.get(4)?,
                user_designation: row.get(5)?,
                user_android_id: row.get(6)?,
                face_encoding: row.get(7)?,
                register_with_camera: row.get(15).ok(),
                password: row.get(8)?,
                role,
                status,
                email: row.get(11)?,
                phone: row.get(12)?,
                created_at: row.get(13)?,
                last_updated: row.get(14)?,
            })
        }).map_err(|e| AppError::Database(e.to_string()))?;

        let mut users = Vec::new();
        for r in rows {
            users.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(users)
    }

    pub fn upsert(conn: &Connection, user: &User) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        let role_str = format!("{:?}", user.role);
        let status_str = format!("{:?}", user.status);
        let reg_cam = user.register_with_camera.clone().unwrap_or_else(|| {
            if user
                .face_encoding
                .as_deref()
                .map(|s| !s.is_empty() && !s.starts_with("[0.0,0.0,"))
                .unwrap_or(false)
            {
                "Y".to_string()
            } else {
                "N".to_string()
            }
        });

        conn.execute(
            "INSERT INTO users (
                company_user_uuid, company_name, company_uuid, username, user_uuid,
                user_designation, user_android_id, face_encoding, register_with_camera, password, role, status, email, phone, created_at, last_updated
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            ON CONFLICT(company_user_uuid) DO UPDATE SET
                company_name = excluded.company_name,
                company_uuid = excluded.company_uuid,
                username = excluded.username,
                user_designation = COALESCE(excluded.user_designation, users.user_designation),
                user_android_id = COALESCE(excluded.user_android_id, users.user_android_id),
                face_encoding = COALESCE(excluded.face_encoding, users.face_encoding),
                register_with_camera = COALESCE(excluded.register_with_camera, users.register_with_camera, 'N'),
                password = COALESCE(excluded.password, users.password),
                role = excluded.role,
                status = excluded.status,
                email = excluded.email,
                phone = excluded.phone,
                last_updated = excluded.last_updated",
            params![
                user.company_user_uuid,
                user.company_name,
                user.company_uuid,
                user.username.to_uppercase(),
                user.user_uuid,
                user.user_designation,
                user.user_android_id,
                user.face_encoding,
                reg_cam,
                user.password,
                role_str,
                status_str,
                user.email,
                user.phone,
                user.created_at.as_deref().unwrap_or(&now),
                now
            ],
        ).map_err(|e| AppError::Database(format!("Failed to upsert user: {}", e)))?;

        // Queue Outbox CDC event
        let event_id = uuid::Uuid::new_v4().to_string();
        let payload = serde_json::to_string(user)?;
        let _ = conn.execute(
            "INSERT INTO bigquery_outbox (event_id, table_name, operation, payload_json, created_at, updated_at)
             VALUES (?1, 'users', 'UPSERT', ?2, ?3, ?3)",
            params![event_id, payload, now],
        );

        Ok(())
    }

    pub fn delete_by_username(conn: &Connection, username: &str) -> Result<bool> {
        let upper = username.trim().to_uppercase();
        let _ = conn.execute(
            "DELETE FROM face_profiles WHERE UPPER(username) = ?",
            params![upper],
        );
        let rows = conn.execute(
            "DELETE FROM users WHERE UPPER(username) = ?",
            params![upper],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(rows > 0)
    }
}
