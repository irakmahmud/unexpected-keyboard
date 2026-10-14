use crate::alignment;
use crate::errors::AppError;
use crate::inference;
use crate::math;
use crate::mtcnn;
use crate::preprocessing;
use anyhow::Result;
use image::DynamicImage;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tract_onnx::prelude::*;
use axum::{
    extract::{Multipart, State},
    routing::post,
    Json, Router,
};

/// Reused AppState from rust_deepFace — models loaded once at startup
#[derive(Clone)]
pub struct AppState {
    pub facenet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    pub pnet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    pub rnet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
    pub onet_model: Arc<SimplePlan<TypedFact, Box<dyn TypedOp>, Graph<TypedFact, Box<dyn TypedOp>>>>,
}

impl AppState {
    /// Load models from a directory containing the 4 ONNX files
    pub fn load_from_dir(models_dir: &str) -> Result<Self> {
        let facenet_path = format!("{}/facenet512.onnx", models_dir);
        let pnet_path = format!("{}/mtcnn_pnet.onnx", models_dir);
        let rnet_path = format!("{}/mtcnn_rnet.onnx", models_dir);
        let onet_path = format!("{}/mtcnn_onet.onnx", models_dir);

        let facenet_model = tract_onnx::onnx()
            .model_for_path(&facenet_path)?
            .into_optimized()?
            .into_runnable()?;

        let pnet_model = tract_onnx::onnx()
            .model_for_path(&pnet_path)?
            .into_optimized()?
            .into_runnable()?;

        let rnet_model = tract_onnx::onnx()
            .model_for_path(&rnet_path)?
            .into_optimized()?
            .into_runnable()?;

        let onet_model = tract_onnx::onnx()
            .model_for_path(&onet_path)?
            .into_optimized()?
            .into_runnable()?;

        Ok(Self {
            facenet_model: Arc::new(facenet_model),
            pnet_model: Arc::new(pnet_model),
            rnet_model: Arc::new(rnet_model),
            onet_model: Arc::new(onet_model),
        })
    }
}

pub const COSINE_THRESHOLD: f32 = 0.30;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyRequest {
    pub image1_b64: String,
    pub image2_b64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyResponse {
    pub verified: bool,
    pub distance: f32,
    pub threshold: f32,
    pub model: &'static str,
    pub detector: &'static str,
    pub metric: &'static str,
}

/// Extract face embedding from raw image bytes using the complete rust_deepFace pipeline
pub fn extract_face_embedding(state: &AppState, img_bytes: &[u8]) -> Result<Array1<f32>, AppError> {
    let img: DynamicImage = image::load_from_memory(img_bytes)
        .map_err(|e| AppError::ImageDecode(e.to_string()))?;
    let det = mtcnn::detect_face(state, &img)?;
    let aligned = alignment::align_face(&img, &det);
    let cropped = preprocessing::crop_and_resize(&aligned, &det, 160, 160);
    let tensor = preprocessing::normalize_facenet(&cropped);
    let raw = inference::get_embedding(&state.facenet_model, tensor)?;
    Ok(math::l2_normalize(&raw))
}

/// Run full 2-image verification pipeline from rust_deepFace
pub fn run_pipeline(
    state: &AppState,
    bytes1: &[u8],
    bytes2: &[u8],
) -> Result<VerifyResponse, AppError> {
    let embed1 = extract_face_embedding(state, bytes1)?;
    let embed2 = extract_face_embedding(state, bytes2)?;
    let distance = math::cosine_distance(&embed1, &embed2);
    let verified = distance <= COSINE_THRESHOLD;

    Ok(VerifyResponse {
        verified,
        distance,
        threshold: COSINE_THRESHOLD,
        model: "Facenet512",
        detector: "MTCNN",
        metric: "cosine",
    })
}

/// JSON handler matching rust_deepFace /api/v1/verify
pub async fn verify_json_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyRequest>,
) -> std::result::Result<Json<VerifyResponse>, AppError> {
    use base64::{engine::general_purpose, Engine as _};

    let bytes1 = general_purpose::STANDARD
        .decode(&req.image1_b64)
        .map_err(|e| AppError::BadRequest(format!("image1 base64 decode failed: {e}")))?;

    let bytes2 = general_purpose::STANDARD
        .decode(&req.image2_b64)
        .map_err(|e| AppError::BadRequest(format!("image2 base64 decode failed: {e}")))?;

    let response = run_pipeline(&state, &bytes1, &bytes2)?;
    Ok(Json(response))
}

/// Multipart handler matching rust_deepFace /api/v1/verify_multipart
pub async fn verify_multipart_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> std::result::Result<Json<VerifyResponse>, AppError> {
    let mut image1_bytes: Option<Vec<u8>> = None;
    let mut image2_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?;

        match name.as_str() {
            "image1" => image1_bytes = Some(data.to_vec()),
            "image2" => image2_bytes = Some(data.to_vec()),
            _ => {}
        }
    }

    let bytes1 = image1_bytes.ok_or_else(|| AppError::BadRequest("missing field: image1".into()))?;
    let bytes2 = image2_bytes.ok_or_else(|| AppError::BadRequest("missing field: image2".into()))?;

    let response = run_pipeline(&state, &bytes1, &bytes2)?;
    Ok(Json(response))
}

/// Create router for rust_deepFace endpoints
pub fn deepface_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/verify", post(verify_json_handler))
        .route("/verify_multipart", post(verify_multipart_handler))
        .with_state(state)
}
