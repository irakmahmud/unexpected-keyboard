use attendance_common::{AppError, Result};
use attendance_domain::UserRole;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthToken {
    pub token: String,
    pub username: String,
    pub role: UserRole,
    pub company_id: String,
    pub created_at: String,
    pub expires_in_seconds: u64,
}

#[derive(Clone)]
pub struct SessionStore {
    // token -> (username, role, company_id, expiry_instant)
    sessions: Arc<RwLock<HashMap<String, (String, UserRole, String, Instant)>>>,
    default_ttl: Duration,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new(Duration::from_secs(86400)) // 24 hours
    }
}

impl SessionStore {
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_ttl,
        }
    }

    pub fn create_session(&self, username: &str, role: UserRole, company_id: &str) -> AuthToken {
        let token = uuid::Uuid::new_v4().to_string();
        let expiry = Instant::now() + self.default_ttl;

        self.sessions.write().insert(
            token.clone(),
            (username.to_uppercase(), role.clone(), company_id.to_string(), expiry),
        );

        AuthToken {
            token,
            username: username.to_uppercase(),
            role,
            company_id: company_id.to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            expires_in_seconds: self.default_ttl.as_secs(),
        }
    }

    pub fn validate_session(&self, token: &str) -> Result<(String, UserRole, String)> {
        let map = self.sessions.read();
        if let Some((user, role, comp, expiry)) = map.get(token) {
            if Instant::now() < *expiry {
                return Ok((user.clone(), role.clone(), comp.clone()));
            }
        }
        Err(AppError::Unauthorized("Invalid or expired session token".to_string()))
    }

    pub fn revoke_session(&self, token: &str) {
        self.sessions.write().remove(token);
    }
}
