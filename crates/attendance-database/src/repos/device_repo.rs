use attendance_common::{AppError, Result, TimeHelper};
use attendance_domain::{DeviceSession, RegisteredDevice};
use rusqlite::{params, Connection, OptionalExtension};

pub struct DeviceRepo;

impl DeviceRepo {
    pub fn upsert_device(conn: &Connection, dev: &RegisteredDevice) -> Result<()> {
        let now = TimeHelper::now_utc_sqlite_string();
        let status_str = format!("{:?}", dev.status);

        conn.execute(
            "INSERT INTO registered_devices (
                device_id, device_name, android_id, mac_address, ip_address, device_type,
                location_name, status, last_heartbeat, app_version, created_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(device_id) DO UPDATE SET
                device_name = excluded.device_name,
                android_id = excluded.android_id,
                ip_address = excluded.ip_address,
                status = excluded.status,
                last_heartbeat = excluded.last_heartbeat,
                app_version = excluded.app_version",
            params![
                dev.device_id,
                dev.device_name,
                dev.android_id,
                dev.mac_address,
                dev.ip_address,
                dev.device_type,
                dev.location_name,
                status_str,
                now,
                dev.app_version,
                dev.created_at.as_deref().unwrap_or(&now)
            ],
        ).map_err(|e| AppError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn save_session(conn: &Connection, sess: &DeviceSession) -> Result<()> {
        conn.execute(
            "INSERT INTO device_sessions (session_id, created_at, registered_android_id, status)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(session_id) DO UPDATE SET
                registered_android_id = excluded.registered_android_id,
                status = excluded.status",
            params![sess.session_id, sess.created_at, sess.registered_android_id, sess.status],
        ).map_err(|e| AppError::Database(e.to_string()))?;
        Ok(())
    }

    pub fn get_session(conn: &Connection, session_id: &str) -> Result<Option<DeviceSession>> {
        let mut stmt = conn.prepare(
            "SELECT session_id, created_at, registered_android_id, status
             FROM device_sessions WHERE session_id = ?"
        ).map_err(|e| AppError::Database(e.to_string()))?;

        let sess = stmt.query_row(params![session_id], |row| {
            Ok(DeviceSession {
                session_id: row.get(0)?,
                created_at: row.get(1)?,
                registered_android_id: row.get(2)?,
                status: row.get(3)?,
            })
        }).optional().map_err(|e| AppError::Database(e.to_string()))?;

        Ok(sess)
    }
}
