use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceProfile {
    pub id: Option<i64>,
    pub username: String,
    pub embedding_vector: Vec<f32>,
    pub sample_quality_score: f32,
    pub captured_with_camera: bool,
    pub enrolled_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LivenessResult {
    pub is_live: bool,
    pub confidence_score: f32,
    pub eye_blink_detected: bool,
    pub motion_variance: f32,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceMatchResult {
    pub username: String,
    pub distance: f32,
    pub confidence_percent: f32,
    pub is_match: bool,
}
