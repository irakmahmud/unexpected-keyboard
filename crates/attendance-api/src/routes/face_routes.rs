use crate::{ApiError, AppState};
use attendance_domain::*;
use attendance_face::{DistanceMetrics, LivenessDetector};
use axum::{
    extract::{Path, State},
    http::{header, HeaderValue},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LivenessProbeRequest {
    pub variance_scores: Vec<f32>,
    pub eye_aspect_ratios: Vec<f32>,
}

#[derive(Serialize)]
pub struct FaceIndexStatusResponse {
    pub total_enrolled: usize,
    pub dimension: usize,
    pub distance_threshold: f32,
    pub index_mode: String,
}

pub fn face_router() -> Router<AppState> {
    Router::new()
        .route("/enroll", post(handle_enroll))
        .route("/enroll-batch", post(handle_enroll_batch))
        .route("/extract-frame", post(handle_extract_frame))
        .route("/verify-identity", post(handle_verify_identity))
        .route("/verify-2fa", post(handle_verify_identity))
        .route("/match", post(handle_match))
        .route("/identify", post(handle_identify))
        .route("/profiles/:username", get(get_user_profiles).delete(delete_user_profiles))
        .route("/liveness", post(handle_liveness_probe))
        .route("/index/status", get(get_index_status))
        .route("/index/rebuild", post(handle_rebuild_index))
        .route("/index/reset", post(handle_reset_index))
        .route("/reset", post(handle_reset_index))
        .route("/embeddings-report", get(get_embeddings_report_json))
        .route("/embeddings-report.csv", get(get_embeddings_report_csv))
        .route("/upload-model", post(handle_upload_ai_model))
        .route("/verify", post(handle_verify_json))
        .route("/verify_multipart", post(handle_verify_multipart))
}

async fn handle_enroll(
    State(state): State<AppState>,
    Json(payload): Json<FaceEnrollRequest>,
) -> std::result::Result<Json<FaceEnrollResponse>, ApiError> {
    let mut vector = payload.embedding;
    if vector.len() != 512 {
        return Err(attendance_common::AppError::Validation(format!(
            "Invalid embedding dimension: expected 512, got {}",
            vector.len()
        ))
        .into());
    }
    if attendance_face::FaceVectorIndex::is_corrupted_constant_vector(&vector) {
        return Err(attendance_common::AppError::Validation(
            "Corrupted or uniform face embedding rejected".to_string(),
        )
        .into());
    }
    DistanceMetrics::l2_normalize(&mut vector);

    // Add to RAM vector index first
    state.face_index.enroll_user(&payload.username, vector.clone())?;

    // Save to SQLite
    state.db.save_face_profile(
        &payload.username,
        payload.employee_code.as_deref(),
        &vector,
        payload.quality_score.unwrap_or(1.0),
        payload.pose.as_deref().unwrap_or("FRONTAL"),
    )?;

    Ok(Json(FaceEnrollResponse {
        success: true,
        username: payload.username.to_uppercase(),
        embeddings_count: 1,
        message: "Face biometric embedding enrolled and indexed successfully".to_string(),
    }))
}

async fn handle_match(
    State(state): State<AppState>,
    Json(payload): Json<FaceMatchRequest>,
) -> std::result::Result<Json<FaceMatchResponse>, ApiError> {
    let mut query_emb = payload.embedding;
    DistanceMetrics::l2_normalize(&mut query_emb);

    let candidate_trimmed = payload.candidate_username.trim().to_uppercase();
    if candidate_trimmed.is_empty() {
        // 1-to-N search across all enrolled faces when candidate_username is omitted
        if let Ok(Some(tiered)) = state.face_index.search_tiered(query_emb.clone()) {
            let euclidean_dist = (2.0f32 * tiered.min_distance).max(0.0f32).sqrt();
            return Ok(Json(FaceMatchResponse {
                matched: tiered.is_match,
                username: tiered.username,
                distance: euclidean_dist,
                threshold: 0.60,
                confidence: tiered.confidence_percent,
            }));
        }
        return Ok(Json(FaceMatchResponse {
            matched: false,
            username: String::new(),
            distance: 1.0,
            threshold: 0.60,
            confidence: 0.0,
        }));
    }

    let profiles = state.db.get_face_profiles(&candidate_trimmed)?;
    let mut min_dist = f32::MAX;
    if !profiles.is_empty() {
        for p in profiles {
            let mut prof_vec = p.embedding_vector;
            DistanceMetrics::l2_normalize(&mut prof_vec);
            let d = DistanceMetrics::euclidean_l2(&query_emb, &prof_vec);
            if d < min_dist {
                min_dist = d;
            }
        }
    } else if let Ok(Some(u)) = state.db.get_user_by_username(&candidate_trimmed) {
        // Fallback to users table face_encoding (parity with Python reference database)
        if let Some(enc_str) = u.face_encoding {
            if let Ok(mut vec) = serde_json::from_str::<Vec<f32>>(&enc_str) {
                if !vec.is_empty() && vec.iter().any(|&v| v != 0.0) {
                    DistanceMetrics::l2_normalize(&mut vec);
                    min_dist = DistanceMetrics::euclidean_l2(&query_emb, &vec);
                }
            }
        }
    }

    if min_dist == f32::MAX {
        return Ok(Json(FaceMatchResponse {
            matched: false,
            username: candidate_trimmed,
            distance: 1.0,
            threshold: 0.60,
            confidence: 0.0,
        }));
    }

    let threshold = 0.60f32;
    let matched = min_dist <= threshold;
    let confidence = if min_dist <= 0.40 {
        (95.0 + ((0.40 - min_dist) / 0.40) * 4.9).clamp(95.0, 99.9)
    } else if min_dist <= 0.50 {
        (80.0 + ((0.50 - min_dist) / 0.10) * 14.9).clamp(80.0, 94.9)
    } else if matched {
        (60.0 + ((0.60 - min_dist) / 0.10) * 19.9).clamp(60.0, 79.9)
    } else {
        0.0
    };

    Ok(Json(FaceMatchResponse {
        matched,
        username: candidate_trimmed,
        distance: min_dist,
        threshold,
        confidence,
    }))
}

async fn handle_identify(
    State(state): State<AppState>,
    Json(payload): Json<FaceIdentifyRequest>,
) -> std::result::Result<Json<FaceIdentifyResponse>, ApiError> {
    if let Ok(Some(tiered)) = state.face_index.search_tiered(payload.embedding.clone()) {
        if tiered.is_match {
            let candidate = FaceCandidateMatch {
                username: tiered.username,
                distance: tiered.min_distance,
                confidence: tiered.confidence_percent,
            };
            return Ok(Json(FaceIdentifyResponse {
                identified: true,
                top_match: Some(candidate.clone()),
                all_candidates: vec![candidate],
            }));
        }
    }

    let result = state.face_index.search(payload.embedding)?;

    if let Some(matched) = result {
        if matched.is_match {
            let candidate = FaceCandidateMatch {
                username: matched.username,
                distance: matched.distance,
                confidence: matched.confidence_percent,
            };
            return Ok(Json(FaceIdentifyResponse {
                identified: true,
                top_match: Some(candidate.clone()),
                all_candidates: vec![candidate],
            }));
        }
    }

    Ok(Json(FaceIdentifyResponse {
        identified: false,
        top_match: None,
        all_candidates: Vec::new(),
    }))
}

async fn get_user_profiles(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<Vec<FaceProfile>>, ApiError> {
    let profiles = state.db.get_face_profiles(&username)?;
    Ok(Json(profiles))
}

async fn delete_user_profiles(
    State(state): State<AppState>,
    Path(username): Path<String>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let count = state.db.delete_face_profiles(&username)?;
    let ram_removed = state.face_index.remove_user(&username);
    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_count": count,
        "ram_vectors_removed": ram_removed,
        "username": username.to_uppercase()
    })))
}

async fn handle_liveness_probe(
    Json(payload): Json<LivenessProbeRequest>,
) -> std::result::Result<Json<LivenessResult>, ApiError> {
    let res = LivenessDetector::evaluate_frame_sequence(
        &payload.variance_scores,
        &payload.eye_aspect_ratios,
        payload.variance_scores.len(),
    );
    Ok(Json(res))
}

async fn get_index_status(
    State(state): State<AppState>,
) -> std::result::Result<Json<FaceIndexStatusResponse>, ApiError> {
    let count = state.db.count_enrolled_faces()?;
    Ok(Json(FaceIndexStatusResponse {
        total_enrolled: count,
        dimension: 512,
        distance_threshold: 0.60,
        index_mode: "Pure Rust Vector Index + HNSW Accelerated Search".to_string(),
    }))
}

async fn handle_rebuild_index(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    state.face_index.clear();
    let all_profiles = state.db.get_all_face_profiles()?;
    let mut enrolled_count = 0;
    let mut users_with_profiles = std::collections::HashSet::new();

    for p in all_profiles {
        if attendance_face::FaceVectorIndex::is_corrupted_constant_vector(&p.embedding_vector) {
            continue;
        }
        let upper = p.username.trim().to_uppercase();
        if state.face_index.enroll_user(&upper, p.embedding_vector).is_ok() {
            users_with_profiles.insert(upper);
            enrolled_count += 1;
        }
    }

    if let Ok(users) = state.db.get_all_users() {
        for u in users {
            let upper = u.username.trim().to_uppercase();
            if users_with_profiles.contains(&upper) {
                continue;
            }
            if let Some(enc_str) = &u.face_encoding {
                if let Ok(vec) = serde_json::from_str::<Vec<f32>>(enc_str) {
                    if !vec.is_empty()
                        && vec.iter().any(|&v| v != 0.0)
                        && !attendance_face::FaceVectorIndex::is_corrupted_constant_vector(&vec)
                    {
                        if let Ok(()) = state.face_index.enroll_user(&upper, vec) {
                            enrolled_count += 1;
                        }
                    }
                }
            }
        }
    }

    if let Ok(faiss_loaded) = state.face_index.load_faiss_files() {
        enrolled_count += faiss_loaded;
    }
    let _ = state.face_index.sync_faiss_files();

    Ok(Json(serde_json::json!({
        "success": true,
        "indexed_profiles_count": enrolled_count,
        "message": "Face vector index rebuilt from SQLite persistence successfully"
    })))
}

async fn handle_reset_index(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    state.face_index.clear();
    let deleted_profiles = state.db.delete_all_face_profiles().unwrap_or(0);
    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_profiles": deleted_profiles,
        "message": "Database reset successfully"
    })))
}

fn build_embeddings_analysis(state: &AppState) -> Vec<serde_json::Value> {
    let mut all_map = state.face_index.get_all_embeddings();
    if all_map.is_empty() {
        if let Ok(users) = state.db.get_all_users() {
            for u in users {
                if let Some(enc_str) = &u.face_encoding {
                    if let Ok(vec) = serde_json::from_str::<Vec<f32>>(enc_str) {
                        if !vec.is_empty() {
                            all_map.entry(u.username.clone()).or_default().push(vec);
                        }
                    }
                }
            }
        }
    }

    let mut rows = Vec::new();
    let mut usernames: Vec<String> = all_map.keys().cloned().collect();
    usernames.sort();

    for username in usernames {
        if let Some(embeddings) = all_map.get(&username) {
            for (i, embedding) in embeddings.iter().enumerate() {
                let image_name = format!("image_{}", i + 1);
                let nan_count = embedding.iter().filter(|x| x.is_nan()).count();
                let zero_count = embedding.iter().filter(|&&x| x == 0.0).count();
                let negative_count = embedding.iter().filter(|&&x| x < 0.0).count();
                let positive_count = embedding.iter().filter(|&&x| x > 0.0).count();

                let mut non_nan_vals: Vec<f32> = embedding
                    .iter()
                    .copied()
                    .filter(|x| !x.is_nan())
                    .collect();
                non_nan_vals.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

                let (lowest_5, middle_5, highest_5) = if non_nan_vals.len() >= 5 {
                    let low = non_nan_vals[..5]
                        .iter()
                        .map(|x| format!("{:.6}", x))
                        .collect::<Vec<_>>()
                        .join(",");
                    let high = non_nan_vals[non_nan_vals.len() - 5..]
                        .iter()
                        .map(|x| format!("{:.6}", x))
                        .collect::<Vec<_>>()
                        .join(",");
                    let mid_idx = non_nan_vals.len() / 2;
                    let mid = non_nan_vals[mid_idx - 2..mid_idx + 3]
                        .iter()
                        .map(|x| format!("{:.6}", x))
                        .collect::<Vec<_>>()
                        .join(",");
                    (low, mid, high)
                } else {
                    (
                        "Insufficient values".to_string(),
                        "Insufficient values".to_string(),
                        "Insufficient values".to_string(),
                    )
                };

                let unusable = if nan_count == embedding.len() || zero_count == embedding.len() {
                    1
                } else {
                    0
                };

                rows.push(serde_json::json!({
                    "Username": username,
                    "Image_Name": image_name,
                    "Number_of_Embeddings_Generated": embedding.len(),
                    "Number_of_Errors": 0,
                    "Number_of_Unusable_Embeddings": unusable,
                    "Number_of_Nan_Values": nan_count,
                    "Number_of_Zero_Values": zero_count,
                    "Number_of_Negative_Values": negative_count,
                    "Number_of_Positive_Values": positive_count,
                    "Top_5_Lower_Values": lowest_5,
                    "Middle_5_Values": middle_5,
                    "Top_5_Highest_Values": highest_5
                }));
            }
        }
    }
    rows
}

async fn get_embeddings_report_json(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let rows = build_embeddings_analysis(&state);
    Ok(Json(serde_json::json!({
        "success": true,
        "total_records": rows.len(),
        "report": rows
    })))
}

async fn get_embeddings_report_csv(
    State(state): State<AppState>,
) -> std::result::Result<Response, ApiError> {
    let rows = build_embeddings_analysis(&state);
    let mut csv = String::from("Username,Image_Name,Number_of_Embeddings_Generated,Number_of_Errors,Number_of_Unusable_Embeddings,Number_of_Nan_Values,Number_of_Zero_Values,Number_of_Negative_Values,Number_of_Positive_Values,Top_5_Lower_Values,Middle_5_Values,Top_5_Highest_Values\n");
    for r in rows {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},\"{}\",\"{}\",\"{}\"\n",
            r["Username"].as_str().unwrap_or(""),
            r["Image_Name"].as_str().unwrap_or(""),
            r["Number_of_Embeddings_Generated"].as_u64().unwrap_or(0),
            r["Number_of_Errors"].as_u64().unwrap_or(0),
            r["Number_of_Unusable_Embeddings"].as_u64().unwrap_or(0),
            r["Number_of_Nan_Values"].as_u64().unwrap_or(0),
            r["Number_of_Zero_Values"].as_u64().unwrap_or(0),
            r["Number_of_Negative_Values"].as_u64().unwrap_or(0),
            r["Number_of_Positive_Values"].as_u64().unwrap_or(0),
            r["Top_5_Lower_Values"].as_str().unwrap_or(""),
            r["Middle_5_Values"].as_str().unwrap_or(""),
            r["Top_5_Highest_Values"].as_str().unwrap_or("")
        ));
    }

    let mut response = csv.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"embeddings_analysis.csv\""),
    );
    Ok(response)
}

fn map_face_err(e: attendance_face::FaceError) -> attendance_common::AppError {
    match e {
        attendance_face::FaceError::NoFaceDetected => attendance_common::AppError::NoFaceDetected,
        attendance_face::FaceError::MultipleFacesDetected(n) => {
            attendance_common::AppError::MultipleFacesDetected(n)
        }
        other => attendance_common::AppError::Validation(other.to_string()),
    }
}

/// JSON handler exposing verified pure-Rust FaceNet512 + MTCNN pipeline
pub async fn handle_verify_json(
    State(state): State<AppState>,
    Json(req): Json<attendance_face::VerifyRequest>,
) -> std::result::Result<Json<attendance_face::VerifyResponse>, ApiError> {
    let deepface_state = state.get_deepface().ok_or_else(|| {
        attendance_common::AppError::Validation(
            "DeepFace ONNX models not loaded. Place facenet512.onnx, mtcnn_pnet.onnx, mtcnn_rnet.onnx, and mtcnn_onet.onnx in ./models/ to enable neural face verification".to_string()
        )
    })?;

    let bytes1 = decode_data_uri(&req.image1_b64)?;
    let bytes2 = decode_data_uri(&req.image2_b64)?;

    let resp = attendance_face::run_pipeline(&deepface_state, &bytes1, &bytes2)
        .map_err(map_face_err)?;

    Ok(Json(resp))
}

/// Multipart handler exposing verified pure-Rust FaceNet512 + MTCNN pipeline
pub async fn handle_verify_multipart(
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> std::result::Result<Json<attendance_face::VerifyResponse>, ApiError> {
    let deepface_state = state.get_deepface().ok_or_else(|| {
        attendance_common::AppError::Validation(
            "DeepFace ONNX models not loaded. Place facenet512.onnx, mtcnn_pnet.onnx, mtcnn_rnet.onnx, and mtcnn_onet.onnx in ./models/ to enable neural face verification".to_string()
        )
    })?;

    let mut image1_bytes: Option<Vec<u8>> = None;
    let mut image2_bytes: Option<Vec<u8>> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| attendance_common::AppError::Validation(e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();
        let data = field
            .bytes()
            .await
            .map_err(|e| attendance_common::AppError::Validation(e.to_string()))?;

        match name.as_str() {
            "image1" => image1_bytes = Some(data.to_vec()),
            "image2" => image2_bytes = Some(data.to_vec()),
            _ => {}
        }
    }

    let bytes1 = image1_bytes.ok_or_else(|| attendance_common::AppError::Validation("missing field: image1".into()))?;
    let bytes2 = image2_bytes.ok_or_else(|| attendance_common::AppError::Validation("missing field: image2".into()))?;

    let resp = attendance_face::run_pipeline(&deepface_state, &bytes1, &bytes2)
        .map_err(map_face_err)?;

    Ok(Json(resp))
}

#[derive(Deserialize)]
pub struct UploadModelPayload {
    #[serde(alias = "filename")]
    pub model_name: Option<String>,
    #[serde(alias = "model_base64")]
    pub model_b64: Option<String>,
}

async fn handle_upload_ai_model(
    State(state): State<AppState>,
    Json(payload): Json<UploadModelPayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let raw_name = payload
        .model_name
        .unwrap_or_else(|| "facenet512.onnx".to_string());
    let safe_name = attendance_common::sanitize_filename(&raw_name)?;
    if !safe_name.ends_with(".onnx") && !safe_name.ends_with(".h5") {
        return Err(attendance_common::AppError::Validation(
            "Invalid model filename: only .onnx or .h5 model files are permitted".to_string(),
        )
        .into());
    }

    let models_dir_str = std::env::var("MODELS_DIR").unwrap_or_else(|_| "models".to_string());
    let models_dir = std::path::Path::new(&models_dir_str);
    std::fs::create_dir_all(models_dir)
        .map_err(|e| attendance_common::AppError::Internal(format!("Failed to create models dir: {}", e)))?;

    let validated_path = attendance_common::validate_filepath(&safe_name, Some(models_dir), &["onnx", "h5"])?;
    let target_path = validated_path.to_string_lossy().to_string();

    if let Some(ref b64) = payload.model_b64 {
        let bytes = decode_data_uri(b64)?;
        if bytes.len() < 64 {
            return Err(attendance_common::AppError::Validation(
                "Uploaded model file is too small or empty".to_string(),
            )
            .into());
        }
        if safe_name.ends_with(".onnx") {
            attendance_face::validate_onnx_model_bytes(&bytes).map_err(|e| {
                attendance_common::AppError::Validation(format!(
                    "Invalid ONNX model rejected before writing to disk: {}",
                    e
                ))
            })?;
        } else if safe_name.ends_with(".h5") && !bytes.starts_with(b"\x89HDF\r\n\x1a\n") {
            return Err(attendance_common::AppError::Validation(
                "Invalid HDF5 (.h5) model header rejected before writing to disk".to_string(),
            )
            .into());
        }

        std::fs::write(&validated_path, &bytes)
            .map_err(|e| attendance_common::AppError::Internal(format!("Failed to write model file: {}", e)))?;

        if let Ok(home) = std::env::var("HOME") {
            let user_models_dir = std::path::PathBuf::from(home)
                .join("AttendanceSystem_Data")
                .join("models");
            if std::fs::create_dir_all(&user_models_dir).is_ok() {
                let _ = std::fs::write(user_models_dir.join(&safe_name), &bytes);
            }
        }
    }

    let hot_reloaded = state.reload_deepface_from_dir(&models_dir_str).is_ok();

    Ok(Json(serde_json::json!({
        "success": true,
        "target_path": target_path,
        "hot_reloaded": hot_reloaded,
        "message": if hot_reloaded {
            "AI Model file validated, saved, and hot-reloaded in memory!"
        } else {
            "AI Model file saved to models directory."
        }
    })))
}

#[derive(Deserialize)]
pub struct SingleImagePayload {
    pub image_b64: String,
}

#[derive(Deserialize)]
pub struct EnrollBatchPayload {
    pub username: String,
    pub embeddings: Vec<Vec<f32>>,
}

fn decode_data_uri(b64: &str) -> std::result::Result<Vec<u8>, attendance_common::AppError> {
    use base64::{engine::general_purpose, Engine as _};
    let clean = b64
        .trim()
        .split_once(',')
        .map(|(_, data)| data)
        .unwrap_or_else(|| b64.trim());
    general_purpose::STANDARD
        .decode(clean)
        .map_err(|e| attendance_common::AppError::Validation(format!("Invalid base64 image data: {}", e)))
}

/// Extract 512-D FaceNet512 embedding from a single camera frame (Python verify_face_detection lines 1397-1415)
async fn handle_extract_frame(
    State(state): State<AppState>,
    Json(payload): Json<SingleImagePayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let df_state = state.get_deepface().ok_or_else(|| {
        attendance_common::AppError::Validation(
            "DeepFace ONNX models not loaded in ./models/".to_string(),
        )
    })?;
    let bytes = decode_data_uri(&payload.image_b64)?;
    let (emb, det, _dims) = attendance_face::extract_face_embedding_with_detection(&df_state, &bytes)?;
    let quality = attendance_face::FaceQualityAssessor::assess_image_bytes(&bytes);

    Ok(Json(serde_json::json!({
        "success": true,
        "face_detected": true,
        "dimension": emb.len(),
        "model": "Facenet512",
        "detector": "MTCNN",
        "bbox": det.bbox,
        "detection_score": det.confidence,
        "landmarks": det.landmarks,
        "quality": quality,
        "embedding": emb
    })))
}

/// Enroll 20 multi-angle FaceNet512 embeddings for a user (Python _complete_new_user_registration lines 851-947)
async fn handle_enroll_batch(
    State(state): State<AppState>,
    Json(payload): Json<EnrollBatchPayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let upper = payload.username.trim().to_uppercase();
    if payload.embeddings.is_empty() {
        return Err(attendance_common::AppError::Validation("No embeddings provided".to_string()).into());
    }

    // Check duplicate face against other users (cosine distance < 0.40)
    for emb in &payload.embeddings {
        if let Some((dup_user, dist)) = state.face_index.find_duplicate_user(emb, Some(&upper)) {
            return Err(attendance_common::AppError::Validation(format!(
                "Duplicate face detected: already registered to '{}' (distance {:.4} < 0.40)",
                dup_user, dist
            ))
            .into());
        }
    }

    let _ = state.db.delete_face_profiles(&upper);
    state.face_index.remove_user(&upper);
    let mut enrolled = 0usize;
    for (idx, mut emb) in payload.embeddings.into_iter().enumerate() {
        DistanceMetrics::l2_normalize(&mut emb);
        let pose = format!("POSE_{}", idx + 1);
        let _ = state.db.save_face_profile(&upper, Some(&upper), &emb, 1.0, &pose);
        if state.face_index.enroll_user(&upper, emb.clone()).is_ok() {
            enrolled += 1;
        }
        if idx == 0 {
            if let Ok(Some(mut user)) = state.db.get_user_by_username(&upper) {
                user.face_encoding = serde_json::to_string(&emb).ok();
                let _ = state.db.insert_or_update_user(&user);
            }
        }
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "username": upper,
        "embeddings_enrolled": enrolled,
        "message": format!("Enrolled {} L2-normalized FaceNet512 embeddings into FAISS/RAM index & SQLite", enrolled)
    })))
}

/// Verify identity from camera frame and return full Match Analysis (Python verify_identity lines 1617-2436)
async fn handle_verify_identity(
    State(state): State<AppState>,
    Json(payload): Json<SingleImagePayload>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let df_state = state.get_deepface().ok_or_else(|| {
        attendance_common::AppError::Validation(
            "DeepFace ONNX models not loaded in ./models/".to_string(),
        )
    })?;
    let bytes = decode_data_uri(&payload.image_b64)?;
    let (emb, det, _dims) = attendance_face::extract_face_embedding_with_detection(&df_state, &bytes)?;
    let quality = attendance_face::FaceQualityAssessor::assess_image_bytes(&bytes);

    let tiered_opt = state.face_index.search_tiered(emb.clone())?;
    if let Some(tiered) = tiered_opt {
        if tiered.is_match {
            let today = attendance_common::TimeHelper::current_date_dhaka();
            let now_time = attendance_common::TimeHelper::current_time_dhaka();
            let login_type = state
                .engine
                .determine_login_type(&tiered.username, &today)
                .unwrap_or_else(|_| "IN_TIME".to_string());

            return Ok(Json(serde_json::json!({
                "success": true,
                "matched": true,
                "username": tiered.username,
                "login_type": login_type,
                "date": today,
                "time": attendance_common::TimeHelper::format_12hour(&now_time),
                "match_quality": tiered.match_tier,
                "confidence": tiered.confidence_percent,
                "min_distance": tiered.min_distance,
                "exact_matches": tiered.exact_matches,
                "strong_matches": tiered.strong_matches,
                "possible_matches": tiered.possible_matches,
                "total_matches": tiered.total_matches,
                "bbox": det.bbox,
                "landmarks": det.landmarks,
                "quality": quality,
                "embedding": emb
            })));
        }
    }

    Ok(Json(serde_json::json!({
        "success": false,
        "matched": false,
        "error": "No confident match found in face database",
        "bbox": det.bbox,
        "landmarks": det.landmarks,
        "quality": quality,
        "embedding": emb
    })))
}

