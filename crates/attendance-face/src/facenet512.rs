use crate::metrics::DistanceMetrics;
use attendance_common::{AppError, Result};
use attendance_domain::FaceMatchResult;
use serde::{Deserialize, Serialize};

/// Standard embedding dimensionality for FaceNet512
pub const FACENET512_DIMENSION: usize = 512;

/// Standard cosine distance threshold for FaceNet512 (behaviorally equivalent to attendance_v8.py line 4816)
pub const FACENET512_MATCH_THRESHOLD: f32 = 0.40;

/// Strict verified FaceNet-512 Embedding representation
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FaceNet512Embedding {
    vector: Vec<f32>,
}

impl FaceNet512Embedding {
    /// Construct a verified FaceNet512 embedding from raw slice
    pub fn new(mut raw: Vec<f32>) -> Result<Self> {
        if raw.len() != FACENET512_DIMENSION {
            return Err(AppError::Validation(format!(
                "Invalid FaceNet512 embedding dimension: expected {}, got {}",
                FACENET512_DIMENSION,
                raw.len()
            )));
        }

        if raw.iter().any(|v| v.is_nan() || v.is_infinite()) {
            return Err(AppError::Validation(
                "FaceNet512 vector contains NaN or infinite values".to_string(),
            ));
        }

        DistanceMetrics::l2_normalize(&mut raw);
        Ok(Self { vector: raw })
    }

    /// Read from binary byte representation (2048 bytes for 512 x IEEE 754 f32)
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let expected_bytes = FACENET512_DIMENSION * std::mem::size_of::<f32>();
        if bytes.len() != expected_bytes {
            return Err(AppError::Validation(format!(
                "FaceNet512 byte stream length mismatch: expected {} bytes, got {}",
                expected_bytes,
                bytes.len()
            )));
        }

        let mut floats = Vec::with_capacity(FACENET512_DIMENSION);
        for chunk in bytes.chunks_exact(4) {
            let arr: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
            floats.push(f32::from_le_bytes(arr));
        }

        if floats.iter().any(|v| v.is_nan() || v.is_infinite()) {
            return Err(AppError::Validation(
                "FaceNet512 vector contains NaN or infinite values".to_string(),
            ));
        }

        Ok(Self { vector: floats })
    }

    /// Serialize to binary byte stream for SQLite BLOB storage
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(FACENET512_DIMENSION * 4);
        for &val in &self.vector {
            bytes.extend_from_slice(&val.to_le_bytes());
        }
        bytes
    }

    pub fn as_slice(&self) -> &[f32] {
        &self.vector
    }

    pub fn into_vec(self) -> Vec<f32> {
        self.vector
    }

    /// Calculate cosine distance (1 - cosine_similarity)
    pub fn cosine_distance(&self, other: &Self) -> f32 {
        DistanceMetrics::cosine_distance(&self.vector, &other.vector)
    }

    /// Calculate cosine similarity
    pub fn cosine_similarity(&self, other: &Self) -> f32 {
        1.0 - self.cosine_distance(other)
    }

    /// Check if distance is within FaceNet512 threshold
    pub fn is_match(&self, other: &Self, threshold_override: Option<f32>) -> bool {
        let threshold = threshold_override.unwrap_or(FACENET512_MATCH_THRESHOLD);
        self.cosine_distance(other) < threshold
    }

    /// Fuse multi-frame embeddings by centroid averaging and unit normalization
    pub fn fuse(embeddings: &[Self]) -> Result<Self> {
        if embeddings.is_empty() {
            return Err(AppError::Validation(
                "Cannot fuse empty list of FaceNet512 embeddings".to_string(),
            ));
        }

        let mut sum = vec![0.0f32; FACENET512_DIMENSION];
        for emb in embeddings {
            for (i, v) in emb.vector.iter().enumerate() {
                sum[i] += v;
            }
        }

        let n = embeddings.len() as f32;
        for v in &mut sum {
            *v /= n;
        }

        Self::new(sum)
    }
}

/// High-level FaceNet512 verification and comparison engine
pub struct FaceNet512Verifier {
    match_threshold: f32,
}

impl Default for FaceNet512Verifier {
    fn default() -> Self {
        Self::new(FACENET512_MATCH_THRESHOLD)
    }
}

impl FaceNet512Verifier {
    pub fn new(match_threshold: f32) -> Self {
        Self { match_threshold }
    }

    pub fn threshold(&self) -> f32 {
        self.match_threshold
    }

    /// Verify 1:1 match between two FaceNet512 embeddings
    pub fn verify_1_to_1(
        &self,
        candidate: &FaceNet512Embedding,
        enrolled: &FaceNet512Embedding,
    ) -> (bool, f32, f32) {
        let dist = candidate.cosine_distance(enrolled);
        let is_match = dist < self.match_threshold;
        let confidence = (1.0 - (dist / self.match_threshold)).clamp(0.0, 1.0);
        (is_match, dist, confidence)
    }

    /// Search candidate 1:N against an enrolled repository
    pub fn verify_1_to_n<'a>(
        &self,
        candidate: &FaceNet512Embedding,
        database: &'a [(String, FaceNet512Embedding)],
    ) -> Option<FaceMatchResult> {
        let mut best_match: Option<(&String, f32)> = None;

        for (username, emb) in database {
            let dist = candidate.cosine_distance(emb);
            if dist < self.match_threshold {
                match best_match {
                    None => best_match = Some((username, dist)),
                    Some((_, current_min)) if dist < current_min => {
                        best_match = Some((username, dist));
                    }
                    _ => {}
                }
            }
        }

        best_match.map(|(user, dist)| {
            let confidence = (1.0 - (dist / self.match_threshold)).clamp(0.0, 1.0) * 100.0;
            FaceMatchResult {
                username: user.clone(),
                distance: dist,
                confidence_percent: confidence,
                is_match: true,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_facenet512_dimension_and_normalization() {
        let raw = vec![0.5f32; 512];
        let emb = FaceNet512Embedding::new(raw).expect("Must succeed");
        assert_eq!(emb.as_slice().len(), 512);

        let norm: f32 = emb.as_slice().iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-4, "Must be unit normalized");
    }

    #[test]
    fn test_facenet512_bytes_roundtrip() {
        let mut raw = vec![0.0f32; 512];
        for i in 0..512 {
            raw[i] = (i as f32).sin();
        }
        let emb = FaceNet512Embedding::new(raw).expect("Must succeed");
        let bytes = emb.to_bytes();
        assert_eq!(bytes.len(), 512 * 4);

        let restored = FaceNet512Embedding::from_bytes(&bytes).expect("Must decode");
        assert_eq!(emb, restored);
    }

    #[test]
    fn test_facenet512_matching_threshold() {
        let mut raw1 = vec![0.0f32; 512];
        raw1[0] = 1.0;
        let emb1 = FaceNet512Embedding::new(raw1).expect("emb1");

        let mut raw2 = vec![0.0f32; 512];
        raw2[0] = 0.95;
        raw2[1] = 0.05;
        let emb2 = FaceNet512Embedding::new(raw2).expect("emb2");

        let verifier = FaceNet512Verifier::default();
        let (is_match, dist, conf) = verifier.verify_1_to_1(&emb1, &emb2);
        assert!(is_match, "Near vectors must match within 0.40 threshold");
        assert!(dist < 0.40);
        assert!(conf > 0.0);
    }
}
