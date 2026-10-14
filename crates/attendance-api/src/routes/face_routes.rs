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
        .route("/verify", post(handle_verify_json))
        .route("/verify_multipart", post(handle_verify_multipart))
}

async fn handle_enroll(
    State(state): State<AppState>,
    Json(payload): Json<FaceEnrollRequest>,
) -> std::result::Result<Json<FaceEnrollResponse>, ApiError> {
    let mut vector = payload.embedding;
    DistanceMetrics::l2_normalize(&mut vector);

    // Save to SQLite
    state.db.save_face_profile(
        &payload.username,
        payload.employee_code.as_deref(),
        &vector,
        payload.quality_score.unwrap_or(1.0),
        payload.pose.as_deref().unwrap_or("FRONTAL"),
    )?;

    // Add to RAM vector index
    state.face_index.enroll_user(&payload.username, vector)?;

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
    let profiles = state.db.get_face_profiles(&payload.candidate_username)?;
    let mut min_dist = f32::MAX;
    if !profiles.is_empty() {
        for p in profiles {
            let d = DistanceMetrics::euclidean_l2(&payload.embedding, &p.embedding_vector);
            if d < min_dist {
                min_dist = d;
            }
        }
    } else {
        // Fallback to users table face_encoding (parity with Python reference database)
        if let Ok(Some(u)) = state.db.get_user_by_username(&payload.candidate_username) {
            if let Some(enc_str) = u.face_encoding {
                if let Ok(vec) = serde_json::from_str::<Vec<f32>>(&enc_str) {
                    if !vec.is_empty() && vec.iter().any(|&v| v != 0.0) {
                        min_dist = DistanceMetrics::euclidean_l2(&payload.embedding, &vec);
                    }
                }
            }
        }
    }

    if min_dist == f32::MAX {
        return Ok(Json(FaceMatchResponse {
            matched: false,
            username: payload.candidate_username,
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
        username: payload.candidate_username,
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
    Ok(Json(serde_json::json!({
        "success": true,
        "deleted_count": count,
        "username": username
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
    let all_profiles = state.db.get_all_face_profiles()?;
    let mut enrolled_count = 0;

    for p in all_profiles {
        let _ = state.face_index.enroll_user(&p.username, p.embedding_vector);
        enrolled_count += 1;
    }

    if let Ok(users) = state.db.get_all_users() {
        for u in users {
            if let Some(enc_str) = &u.face_encoding {
                if let Ok(vec) = serde_json::from_str::<Vec<f32>>(enc_str) {
                    if !vec.is_empty() && vec.iter().any(|&v| v != 0.0) {
                        if let Ok(()) = state.face_index.enroll_user(&u.username, vec) {
                            enrolled_count += 1;
                        }
                    }
                }
            }
        }
    }

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

/// JSON handler exposing verified pure-Rust FaceNet512 + MTCNN pipeline
pub async fn handle_verify_json(
    State(state): State<AppState>,
    Json(req): Json<attendance_face::VerifyRequest>,
) -> std::result::Result<Json<attendance_face::VerifyResponse>, ApiError> {
    let deepface_state = state.deepface_state.as_ref().ok_or_else(|| {
        attendance_common::AppError::Validation(
            "DeepFace ONNX models not loaded. Place facenet512.onnx, mtcnn_pnet.onnx, mtcnn_rnet.onnx, and mtcnn_onet.onnx in ./models/ to enable neural face verification".to_string()
        )
    })?;

    use base64::{engine::general_purpose, Engine as _};
    let bytes1 = general_purpose::STANDARD
        .decode(&req.image1_b64)
        .map_err(|e| attendance_common::AppError::Validation(format!("image1 base64 decode failed: {e}")))?;
    let bytes2 = general_purpose::STANDARD
        .decode(&req.image2_b64)
        .map_err(|e| attendance_common::AppError::Validation(format!("image2 base64 decode failed: {e}")))?;

    let resp = attendance_face::run_pipeline(deepface_state, &bytes1, &bytes2)
        .map_err(|e| attendance_common::AppError::Validation(e.to_string()))?;

    Ok(Json(resp))
}

/// Multipart handler exposing verified pure-Rust FaceNet512 + MTCNN pipeline
pub async fn handle_verify_multipart(
    State(state): State<AppState>,
    mut multipart: axum::extract::Multipart,
) -> std::result::Result<Json<attendance_face::VerifyResponse>, ApiError> {
    let deepface_state = state.deepface_state.as_ref().ok_or_else(|| {
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

    let resp = attendance_face::run_pipeline(deepface_state, &bytes1, &bytes2)
        .map_err(|e| attendance_common::AppError::Validation(e.to_string()))?;

    Ok(Json(resp))
}
