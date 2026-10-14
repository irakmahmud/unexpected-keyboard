use sha2::{Digest, Sha256};
use uuid::Uuid;

pub struct CryptoHelper;

impl CryptoHelper {
    /// Compute SHA-256 hash string
    pub fn sha256_hex(input: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Compute SHA-256 with salt
    pub fn sha256_salted(input: &str, salt: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(input.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Generate random UUID v4 string
    pub fn new_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// Generate short session token (8 hex characters)
    pub fn new_short_token() -> String {
        let uuid = Uuid::new_v4().to_string();
        uuid.replace('-', "")[..8].to_string()
    }

    /// Constant time comparison to prevent timing attacks
    pub fn constant_time_eq(a: &str, b: &str) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut res = 0;
        for (x, y) in a.bytes().zip(b.bytes()) {
            res |= x ^ y;
        }
        res == 0
    }

    /// URL-safe Base64 encoder without padding
    pub fn base64_encode(input: &[u8]) -> String {
        const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
        let mut out = String::new();
        let mut i = 0;
        while i < input.len() {
            let b0 = input[i] as usize;
            let b1 = if i + 1 < input.len() { input[i + 1] as usize } else { 0 };
            let b2 = if i + 2 < input.len() { input[i + 2] as usize } else { 0 };

            out.push(CHARSET[(b0 >> 2) & 0x3F] as char);
            out.push(CHARSET[((b0 << 4) | (b1 >> 4)) & 0x3F] as char);

            if i + 1 < input.len() {
                out.push(CHARSET[((b1 << 2) | (b2 >> 6)) & 0x3F] as char);
            }
            if i + 2 < input.len() {
                out.push(CHARSET[b2 & 0x3F] as char);
            }
            i += 3;
        }
        out
    }

    /// URL-safe Base64 decoder
    pub fn base64_decode(input: &str) -> Result<Vec<u8>, String> {
        let decode_char = |c: char| -> Result<u8, String> {
            match c {
                'A'..='Z' => Ok(c as u8 - b'A'),
                'a'..='z' => Ok(c as u8 - b'a' + 26),
                '0'..='9' => Ok(c as u8 - b'0' + 52),
                '-' | '+' => Ok(62),
                '_' | '/' => Ok(63),
                '=' => Ok(0),
                _ => Err(format!("Invalid base64 character: {}", c)),
            }
        };

        let chars: Vec<char> = input.chars().filter(|&c| c != '=').collect();
        let mut out = Vec::new();
        let mut i = 0;

        while i < chars.len() {
            let c0 = decode_char(chars[i])? as usize;
            let c1 = if i + 1 < chars.len() { decode_char(chars[i + 1])? as usize } else { 0 };
            let c2 = if i + 2 < chars.len() { decode_char(chars[i + 2])? as usize } else { 0 };
            let c3 = if i + 3 < chars.len() { decode_char(chars[i + 3])? as usize } else { 0 };

            out.push(((c0 << 2) | (c1 >> 4)) as u8);
            if i + 2 < chars.len() {
                out.push(((c1 << 4) | (c2 >> 2)) as u8);
            }
            if i + 3 < chars.len() {
                out.push(((c2 << 6) | c3) as u8);
            }
            i += 4;
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_hashing() {
        let hash1 = CryptoHelper::sha256_hex("admin123");
        let hash2 = CryptoHelper::sha256_hex("admin123");
        assert_eq!(hash1, hash2);
        assert!(CryptoHelper::constant_time_eq(&hash1, &hash2));
        assert!(!CryptoHelper::constant_time_eq(&hash1, "invalid_hash"));
    }
}
