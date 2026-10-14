use crate::clustering::FaceClustering;
use crate::facenet512::{FaceNet512Embedding, FACENET512_DIMENSION};
use crate::metrics::DistanceMetrics;
use attendance_common::{AppError, Result};
use attendance_domain::FaceProfile;

pub struct EnrollmentService;

impl EnrollmentService {
    /// Validate candidate embedding quality and normalize
    pub fn validate_and_normalize(mut embedding: Vec<f32>, expected_dim: usize) -> Result<Vec<f32>> {
        let dim = if expected_dim == 0 { FACENET512_DIMENSION } else { expected_dim };
        if embedding.len() != dim {
            return Err(AppError::Validation(format!(
                "Invalid embedding dimension: expected {}, got {}",
                dim,
                embedding.len()
            )));
        }

        // Check for NaN or Inf
        if embedding.iter().any(|x| x.is_nan() || x.is_infinite()) {
            return Err(AppError::Validation("Embedding contains NaN or Infinite values".to_string()));
        }

        DistanceMetrics::l2_normalize(&mut embedding);
        Ok(embedding)
    }

    /// Validate and normalize candidate FaceNet512 embedding
    pub fn validate_facenet512(embedding: Vec<f32>) -> Result<FaceNet512Embedding> {
        FaceNet512Embedding::new(embedding)
    }

    /// Process multi-frame capture sequence and return high-quality merged profile
    pub fn process_multi_frame_enrollment(
        username: &str,
        frames: Vec<Vec<f32>>,
        min_frames: usize,
    ) -> Result<FaceProfile> {
        if frames.len() < min_frames {
            return Err(AppError::Validation(format!(
                "Insufficient face frames for enrollment: received {}, required minimum {}",
                frames.len(),
                min_frames
            )));
        }

        let centroid = FaceClustering::compute_centroid(&frames)
            .ok_or_else(|| AppError::Validation("Could not compute centroid of frames".to_string()))?;

        let variance = FaceClustering::intra_cluster_variance(&frames, &centroid);
        let quality_score = (1.0 - variance).clamp(0.1, 1.0);

        Ok(FaceProfile {
            id: None,
            username: username.to_uppercase(),
            embedding_vector: centroid,
            sample_quality_score: quality_score,
            captured_with_camera: true,
            enrolled_at: chrono::Utc::now().to_rfc3339(),
        })
    }
}
