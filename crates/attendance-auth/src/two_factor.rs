use attendance_common::{AppError, Result};
use sha2::{Digest, Sha256};

pub struct TotpAuthenticator;

impl TotpAuthenticator {
    /// Calculate standard 6-digit TOTP code for a given timestamp and secret
    pub fn generate_totp_code(secret: &str, timestamp_secs: u64, period_secs: u64) -> String {
        let counter = timestamp_secs / period_secs;
        let mut hasher = Sha256::new();
        hasher.update(secret.as_bytes());
        hasher.update(&counter.to_be_bytes());
        let hash = hasher.finalize();

        // Dynamic truncation (RFC 4226)
        let offset = (hash[hash.len() - 1] & 0x0F) as usize;
        let binary = (((hash[offset] & 0x7F) as u32) << 24)
            | (((hash[offset + 1] as u32) & 0xFF) << 16)
            | (((hash[offset + 2] as u32) & 0xFF) << 8)
            | ((hash[offset + 3] as u32) & 0xFF);

        let code = binary % 1_000_000;
        format!("{:06}", code)
    }

    /// Verify TOTP code with time-step drift tolerance (+/- 1 period)
    pub fn verify_code(secret: &str, input_code: &str, period_secs: u64) -> Result<bool> {
        let now = chrono::Utc::now().timestamp() as u64;

        for drift in [-1i64, 0, 1] {
            let eval_time = ((now as i64) + (drift * period_secs as i64)).max(0) as u64;
            let expected = Self::generate_totp_code(secret, eval_time, period_secs);
            if attendance_common::CryptoHelper::constant_time_eq(&expected, input_code) {
                return Ok(true);
            }
        }

        Err(AppError::Unauthorized("Invalid 2FA TOTP authentication code".to_string()))
    }
}
