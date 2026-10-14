use sha2::{Digest, Sha256};

pub struct PasswordHasher;

impl PasswordHasher {
    /// SHA-256 hash formatted as hex string
    pub fn hash_sha256(password: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Salted SHA-256 hash (salt:hash)
    pub fn hash_salted(password: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(b"$");
        hasher.update(password.as_bytes());
        format!("{}$sha256${:x}", salt, hasher.finalize())
    }

    /// Verify password against stored hash or plaintext (supporting legacy migration)
    pub fn verify(plain: &str, stored: &str) -> bool {
        // Direct plaintext check for legacy compatibility
        if plain == stored {
            return true;
        }

        // Salted hash check
        if stored.contains("$sha256$") {
            let parts: Vec<&str> = stored.split("$sha256$").collect();
            if parts.len() == 2 {
                let salt = parts[0];
                let expected_hash = parts[1];
                let mut hasher = Sha256::new();
                hasher.update(salt.as_bytes());
                hasher.update(b"$");
                hasher.update(plain.as_bytes());
                let computed = format!("{:x}", hasher.finalize());
                return computed == expected_hash;
            }
        }

        // Standard SHA256 check
        let computed = Self::hash_sha256(plain);
        computed == stored
    }
}
