pub mod api_key;
pub mod hasher;
pub mod hr_auth;
pub mod jwt;
pub mod rbac;
pub mod session;
pub mod two_factor;

pub use api_key::{ApiKeyInfo, ApiKeyManager};
pub use hasher::PasswordHasher;
pub use hr_auth::HrAuthService;
pub use jwt::{JwtClaims, JwtHeader, JwtService};
pub use rbac::{Permission, RbacPolicy};
pub use session::{AuthToken, SessionStore};
pub use two_factor::TotpAuthenticator;

// Legacy compatibility
pub struct AuthService;
impl AuthService {
    pub fn hash_password(password: &str) -> String {
        PasswordHasher::hash_sha256(password)
    }

    pub fn verify_password(plain: &str, stored: &str) -> bool {
        PasswordHasher::verify(plain, stored)
    }
}
