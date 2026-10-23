use attendance_common::{AppError, Result};
use attendance_domain::FaceProfile;
use rusqlite::{params, Connection};

pub struct BiometricsRepo;

impl BiometricsRepo {
    pub fn save_profile(
        conn: &Connection,
        username: &str,
        employee_code: Option<&str>,
        embedding: &[f32],
        quality: f32,
        pose: &str,
    ) -> Result<i64> {
        let embedding_json = serde_json::to_string(embedding)
            .map_err(AppError::Serialization)?;
        let upper = username.trim().to_uppercase();

        conn.execute(
            "INSERT INTO face_profiles (username, employee_code, embedding_json, sample_quality, pose)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![upper, employee_code, embedding_json, quality, pose],
        )
        .map_err(|e| AppError::Database(format!("Failed to save face profile: {}", e)))?;

        let row_id = conn.last_insert_rowid();
        let _ = conn.execute(
            "UPDATE users SET face_encoding = COALESCE(face_encoding, ?1), register_with_camera = 'Y' WHERE UPPER(username) = ?2",
            params![embedding_json, upper],
        );

        Ok(row_id)
    }

    pub fn get_profiles_by_username(conn: &Connection, username: &str) -> Result<Vec<FaceProfile>> {
        let upper = username.trim().to_uppercase();
        let mut stmt = conn
            .prepare(
                "SELECT id, username, embedding_json, sample_quality, created_at 
                 FROM face_profiles WHERE UPPER(username) = ?1 ORDER BY id DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![upper], |row| {
                let id: i64 = row.get(0)?;
                let uname: String = row.get(1)?;
                let json_str: String = row.get(2)?;
                let quality: f64 = row.get(3)?;
                let created: String = row.get(4)?;

                let vec: Vec<f32> = serde_json::from_str(&json_str).unwrap_or_default();

                Ok(FaceProfile {
                    id: Some(id),
                    username: uname,
                    embedding_vector: vec,
                    sample_quality_score: quality as f32,
                    captured_with_camera: true,
                    enrolled_at: created,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_all_profiles(conn: &Connection) -> Result<Vec<FaceProfile>> {
        let mut stmt = conn
            .prepare(
                "SELECT id, username, embedding_json, sample_quality, created_at 
                 FROM face_profiles ORDER BY username ASC, id DESC",
            )
            .map_err(|e| AppError::Database(e.to_string()))?;

        let rows = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let uname: String = row.get(1)?;
                let json_str: String = row.get(2)?;
                let quality: f64 = row.get(3)?;
                let created: String = row.get(4)?;

                let vec: Vec<f32> = serde_json::from_str(&json_str).unwrap_or_default();

                Ok(FaceProfile {
                    id: Some(id),
                    username: uname,
                    embedding_vector: vec,
                    sample_quality_score: quality as f32,
                    captured_with_camera: true,
                    enrolled_at: created,
                })
            })
            .map_err(|e| AppError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| AppError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn delete_profiles_by_username(conn: &Connection, username: &str) -> Result<usize> {
        let upper = username.trim().to_uppercase();
        let rows = conn
            .execute(
                "DELETE FROM face_profiles WHERE UPPER(username) = ?1",
                params![upper],
            )
            .map_err(|e| AppError::Database(format!("Failed to delete face profiles: {}", e)))?;
        let user_rows = conn
            .execute(
                "UPDATE users SET face_encoding = NULL, register_with_camera = 'N' WHERE UPPER(username) = ?1",
                params![upper],
            )
            .map_err(|e| AppError::Database(format!("Failed to clear user face_encoding: {}", e)))?;
        Ok(rows + user_rows)
    }

    pub fn delete_all_profiles(conn: &Connection) -> Result<usize> {
        let rows = conn
            .execute("DELETE FROM face_profiles", [])
            .map_err(|e| AppError::Database(format!("Failed to reset face profiles: {}", e)))?;
        conn.execute(
            "UPDATE users SET face_encoding = NULL, register_with_camera = 'N'",
            [],
        )
        .map_err(|e| AppError::Database(format!("Failed to clear users face_encoding: {}", e)))?;
        Ok(rows)
    }

    pub fn count_total_enrolled_profiles(conn: &Connection) -> Result<usize> {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM face_profiles", [], |r| r.get(0))
            .unwrap_or(0);
        Ok(count as usize)
    }
}
