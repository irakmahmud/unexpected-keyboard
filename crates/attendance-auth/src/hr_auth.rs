use crate::hasher::PasswordHasher;
use attendance_common::{AppError, Result};
use attendance_database::DbPool;

pub struct HrAuthService;

impl HrAuthService {
    /// Verify HR password replicating Python lines 9904-9995
    pub fn verify_hr_access(db: &DbPool, company_uuid: &str, entered_password: &str) -> Result<bool> {
        let mut hr = db.get_hr_password(company_uuid)?;

        // Auto-initialize default password "HR@123" if none exists for this company (Python lines 9911-9919)
        if hr.is_none() {
            let _ = Self::set_hr_password(db, company_uuid, company_uuid, "HR@123");
            hr = db.get_hr_password(company_uuid)?;
        }

        if let Some(record) = hr {
            let matches = PasswordHasher::verify(entered_password, &record.password)
                || entered_password == record.password;
            if matches {
                tracing::info!("HR authentication successful for company {}", company_uuid);
                return Ok(true);
            }
        }

        Err(AppError::Unauthorized("Invalid HR password! Please try again.".to_string()))
    }

    /// Change HR password after verifying current password (matching Python lines 10050-10110)
    pub fn change_hr_password(
        db: &DbPool,
        company_uuid: &str,
        company_name: &str,
        current_password: &str,
        new_password: &str,
    ) -> Result<()> {
        let verified = Self::verify_hr_access(db, company_uuid, current_password)?;
        if !verified {
            return Err(AppError::Unauthorized("Invalid current password".to_string()));
        }
        Self::set_hr_password(db, company_uuid, company_name, new_password)
    }

    /// Set or update HR password in SQLite hr_passwords table (matching Python sync_hr_password_to_local lines 6497-6520)
    pub fn set_hr_password(db: &DbPool, company_uuid: &str, company_name: &str, plain_password: &str) -> Result<()> {
        let now = attendance_common::TimeHelper::now_utc_sqlite_string();
        let hr = attendance_domain::HrPassword {
            id: None,
            company_uuid: company_uuid.to_string(),
            company_name: company_name.to_string(),
            password: PasswordHasher::hash_sha256(plain_password),
            created_at: Some(now.clone()),
            last_updated: Some(now),
        };
        db.set_hr_password(&hr)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hr_auth_rejects_backdoors_and_honors_password_change() {
        let db = DbPool::new_in_memory().unwrap();
        // Default auto-init is HR@123
        assert!(HrAuthService::verify_hr_access(&db, "COMP_TEST", "HR@123").unwrap());
        // Backdoor strings must be rejected
        assert!(HrAuthService::verify_hr_access(&db, "COMP_TEST", "1234").is_err());
        assert!(HrAuthService::verify_hr_access(&db, "COMP_TEST", "admin123").is_err());
        assert!(HrAuthService::verify_hr_access(&db, "COMP_TEST", "hr2026").is_err());

        // Change password to NewSec#2026
        HrAuthService::change_hr_password(&db, "COMP_TEST", "Test Co", "HR@123", "NewSec#2026").unwrap();

        // Old default HR@123 must now be rejected
        assert!(HrAuthService::verify_hr_access(&db, "COMP_TEST", "HR@123").is_err());
        // New password must succeed
        assert!(HrAuthService::verify_hr_access(&db, "COMP_TEST", "NewSec#2026").unwrap());
    }
}

