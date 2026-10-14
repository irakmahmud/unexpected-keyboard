use attendance_common::{AppError, CryptoHelper, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtHeader {
    pub alg: String,
    pub typ: String,
}

impl Default for JwtHeader {
    fn default() -> Self {
        Self {
            alg: "HS256".to_string(),
            typ: "JWT".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: String,
    pub username: String,
    pub role: String,
    pub company_id: Option<String>,
    pub exp: u64,
    pub iat: u64,
}

pub struct JwtService {
    secret_key: String,
}

impl JwtService {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret_key: secret.into(),
        }
    }

    /// Sign claims into standard base64url encoded JWT token: header.payload.signature
    pub fn sign(&self, claims: &JwtClaims) -> Result<String> {
        let header = JwtHeader::default();
        let header_json = serde_json::to_string(&header)
            .map_err(AppError::Serialization)?;
        let claims_json = serde_json::to_string(claims)
            .map_err(AppError::Serialization)?;

        let b64_header = CryptoHelper::base64_encode(header_json.as_bytes());
        let b64_payload = CryptoHelper::base64_encode(claims_json.as_bytes());

        let signing_input = format!("{}.{}", b64_header, b64_payload);
        let signature = attendance_common::CryptoHelper::sha256_hex(&format!(
            "{}:{}",
            signing_input, self.secret_key
        ));

        Ok(format!("{}.{}", signing_input, signature))
    }

    /// Verify signature and expiration of JWT token
    pub fn verify(&self, token: &str) -> Result<JwtClaims> {
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(AppError::Unauthorized("Malformed JWT token structure".to_string()));
        }

        let signing_input = format!("{}.{}", parts[0], parts[1]);
        let expected_signature = attendance_common::CryptoHelper::sha256_hex(&format!(
            "{}:{}",
            signing_input, self.secret_key
        ));

        if parts[2] != expected_signature {
            return Err(AppError::Unauthorized("Invalid JWT cryptographic signature".to_string()));
        }

        let payload_bytes = CryptoHelper::base64_decode(parts[1])
            .map_err(|e| AppError::Unauthorized(format!("Invalid base64 payload: {}", e)))?;

        let claims: JwtClaims = serde_json::from_slice(&payload_bytes)
            .map_err(|e| AppError::Unauthorized(format!("Invalid JWT claims JSON: {}", e)))?;

        // Check expiration
        let now = chrono::Utc::now().timestamp() as u64;
        if claims.exp < now {
            return Err(AppError::Unauthorized("JWT token has expired".to_string()));
        }

        Ok(claims)
    }
}
