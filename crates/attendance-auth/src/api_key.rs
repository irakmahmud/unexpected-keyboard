use attendance_common::{AppError, CryptoHelper, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    pub key_id: String,
    pub key_prefix: String,
    pub client_name: String,
    pub allowed_scopes: Vec<String>,
    pub is_active: bool,
    pub created_at: String,
    pub expires_at: Option<String>,
}

pub struct ApiKeyManager;

impl ApiKeyManager {
    /// Generate a cryptographically secure API key: `ak_<prefix>_<random_secret>`
    pub fn generate_api_key(client_name: &str, scopes: Vec<String>) -> (String, String, ApiKeyInfo) {
        let prefix = CryptoHelper::new_short_token();
        let secret = CryptoHelper::new_uuid().replace('-', "");
        let full_key = format!("ak_{}_{}", prefix, secret);
        let key_hash = CryptoHelper::sha256_hex(&full_key);

        let info = ApiKeyInfo {
            key_id: CryptoHelper::new_uuid(),
            key_prefix: prefix,
            client_name: client_name.to_string(),
            allowed_scopes: scopes,
            is_active: true,
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_at: None,
        };

        (full_key, key_hash, info)
    }

    /// Verify an incoming API key against stored SHA-256 hash
    pub fn verify_key(incoming_key: &str, stored_hash: &str) -> Result<bool> {
        let hash = CryptoHelper::sha256_hex(incoming_key);
        if CryptoHelper::constant_time_eq(&hash, stored_hash) {
            Ok(true)
        } else {
            Err(AppError::Unauthorized("Invalid API key".to_string()))
        }
    }
}
