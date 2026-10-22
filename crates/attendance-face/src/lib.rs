// Reused modules directly from rust_deepFace repository
pub mod alignment;
pub mod errors;
pub mod inference;
pub mod math;
pub mod mtcnn;
pub mod pipeline;
pub mod preprocessing;

pub mod clustering;
pub mod enrollment;
pub mod facenet512;
pub mod hnsw;
pub mod index;
pub mod liveness;
pub mod metrics;
pub mod quality;

// Re-exports from rust_deepFace
pub use errors::AppError as DeepFaceAppError;
pub use pipeline::{
    deepface_router, extract_face_embedding, extract_face_embedding_vec, run_pipeline,
    verify_json_handler, verify_multipart_handler, AppState, AppState as DeepFaceAppState,
    VerifyRequest, VerifyResponse, COSINE_THRESHOLD as DEEPFACE_COSINE_THRESHOLD,
};

pub use clustering::FaceClustering;
pub use enrollment::EnrollmentService;
pub use enrollment::EnrollmentService as FaceEnrollmentPipeline;
pub use facenet512::{
    FaceNet512Embedding, FaceNet512Verifier, FACENET512_DIMENSION, FACENET512_MATCH_THRESHOLD,
};
pub use hnsw::{HnswConfig, PureRustHnswIndex};
pub use index::{FaceVectorIndex, TieredFaceMatchResult};
pub use liveness::LivenessDetector;
pub use metrics::DistanceMetrics;
pub use quality::{FaceQualityAssessor, FaceQualityScore};
