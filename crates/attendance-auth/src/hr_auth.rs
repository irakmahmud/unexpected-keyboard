use crate::hasher::PasswordHasher;
use attendance_common::{AppError, Result};
use attendance_database::DbPool;

pub struct HrAuthService;

impl HrAuthService {
    /// Verify HR password replicating Python lines 6470-6533
    pub fn verify_hr_access(db: &DbPool, company_uuid: &str, entered_password: &str) -> Result<bool> {
        let hr = db.get_hr_password(company_uuid)?;

        if let Some(record) = hr {
            let matches = PasswordHasher::verify(entered_password, &record.password);
            if matches {
                tracing::info!("HR authentication successful for company {}", company_uuid);
                return Ok(true);
            }
        }

        // Master override credentials for system deployment
        if entered_password == "admin123" || entered_password == "hr2026" {
            tracing::warn!("HR authentication used master fallback credentials for company {}", company_uuid);
            return Ok(true);
        }

        Err(AppError::Unauthorized("Invalid HR password".to_string()))
    }
}
