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
        .route("/embeddings-summary.csv", get(get_embeddings_summary_csv))
        .route("/embeddings-vectors.csv", get(get_embeddings_vectors_csv))
        .route("/faiss-index/inspect", get(get_faiss_index_inspect))
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

fn build_faiss_index_inspection(state: &AppState) -> serde_json::Value {
    let all_map = state.face_index.get_all_embeddings();
    let mut usernames: Vec<String> = all_map.keys().cloned().collect();
    usernames.sort();

    let mut total_vectors = 0usize;
    let mut user_summaries = Vec::new();
    let mut mappings_list = Vec::new();
    let mut idx_counter = 0usize;

    for uname in &usernames {
        if let Some(vecs) = all_map.get(uname) {
            let start_idx = idx_counter;
            let count = vecs.len();
            total_vectors += count;
            for _ in 0..count {
                mappings_list.push(format!("{}:{}", idx_counter, uname));
                idx_counter += 1;
            }
            let avg_norm = if count > 0 {
                let sum_norm: f32 = vecs
                    .iter()
                    .map(|v| v.iter().map(|x| x * x).sum::<f32>().sqrt())
                    .sum();
                sum_norm / (count as f32)
            } else {
                0.0
            };
            user_summaries.push(serde_json::json!({
                "username": uname,
                "embeddings_count": count,
                "dimension": 512,
                "faiss_index_range": if count > 0 {
                    format!("{}..{}", start_idx, start_idx + count - 1)
                } else {
                    "empty".to_string()
                },
                "avg_l2_norm": (avg_norm * 10000.0).round() / 10000.0,
            }));
        }
    }

    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/opc".to_string());
    let home_index_path = std::path::PathBuf::from(&home).join("db").join("face_encodings.index");
    let data_index_path = std::path::PathBuf::from("data").join("db").join("face_encodings.index");
    let active_path = if home_index_path.exists() {
        home_index_path
    } else {
        data_index_path
    };

    let (file_exists, file_bytes, header_magic, header_dim, header_ntotal) =
        if let Ok(raw) = std::fs::read(&active_path) {
            if raw.len() >= 16 {
                let magic = String::from_utf8_lossy(&raw[0..4]).to_string();
                let dim = u32::from_le_bytes([raw[4], raw[5], raw[6], raw[7]]);
                let ntotal = u64::from_le_bytes([
                    raw[8], raw[9], raw[10], raw[11], raw[12], raw[13], raw[14], raw[15],
                ]);
                (true, raw.len(), magic, dim, ntotal)
            } else {
                (true, raw.len(), "SHORT".to_string(), 0, 0)
            }
        } else {
            (false, 0, "NONE".to_string(), 512, 0)
        };

    serde_json::json!({
        "index_type": "FAISS IndexFlatL2 (512-D L2-Normalized FaceNet512)",
        "file_path": active_path.to_string_lossy(),
        "file_exists": file_exists,
        "file_size_bytes": file_bytes,
        "header_magic": header_magic,
        "dimension": header_dim,
        "ntotal_on_disk": header_ntotal,
        "ntotal_in_ram": total_vectors,
        "total_users": usernames.len(),
        "users": user_summaries,
        "user_mappings": mappings_list
    })
}

async fn get_faiss_index_inspect(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let info = build_faiss_index_inspection(&state);
    Ok(Json(serde_json::json!({
        "success": true,
        "faiss_index": info
    })))
}

async fn get_embeddings_report_json(
    State(state): State<AppState>,
) -> std::result::Result<Json<serde_json::Value>, ApiError> {
    let rows = build_embeddings_analysis(&state);
    let faiss_info = build_faiss_index_inspection(&state);
    Ok(Json(serde_json::json!({
        "success": true,
        "total_records": rows.len(),
        "faiss_index": faiss_info,
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

    let _ = std::fs::write("embeddings_analysis.csv", csv.as_bytes());
    let _ = std::fs::write("data/embeddings_analysis.csv", csv.as_bytes());

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

async fn get_embeddings_summary_csv(
    State(state): State<AppState>,
) -> std::result::Result<Response, ApiError> {
    let rows = build_embeddings_analysis(&state);
    let mut grouped: std::collections::BTreeMap<String, Vec<&serde_json::Value>> =
        std::collections::BTreeMap::new();
    for r in &rows {
        let u = r["Username"].as_str().unwrap_or("").to_string();
        grouped.entry(u).or_default().push(r);
    }

    let mut csv = String::from("Username,Total_Images,Number_of_Embeddings_Generated,Number_of_Errors,Number_of_Unusable_Embeddings,Mean_Nan_Values,Mean_Zero_Values,Mean_Negative_Values,Mean_Positive_Values\n");
    for (uname, list) in grouped {
        let n = (list.len() as f64).max(1.0);
        let sum_emb: u64 = list.iter().map(|r| r["Number_of_Embeddings_Generated"].as_u64().unwrap_or(0)).sum();
        let sum_err: u64 = list.iter().map(|r| r["Number_of_Errors"].as_u64().unwrap_or(0)).sum();
        let sum_unusable: u64 = list.iter().map(|r| r["Number_of_Unusable_Embeddings"].as_u64().unwrap_or(0)).sum();
        let mean_nan: f64 = list.iter().map(|r| r["Number_of_Nan_Values"].as_u64().unwrap_or(0) as f64).sum::<f64>() / n;
        let mean_zero: f64 = list.iter().map(|r| r["Number_of_Zero_Values"].as_u64().unwrap_or(0) as f64).sum::<f64>() / n;
        let mean_neg: f64 = list.iter().map(|r| r["Number_of_Negative_Values"].as_u64().unwrap_or(0) as f64).sum::<f64>() / n;
        let mean_pos: f64 = list.iter().map(|r| r["Number_of_Positive_Values"].as_u64().unwrap_or(0) as f64).sum::<f64>() / n;

        csv.push_str(&format!(
            "{},{},{},{},{},{:.2},{:.2},{:.2},{:.2}\n",
            uname,
            list.len(),
            sum_emb,
            sum_err,
            sum_unusable,
            mean_nan,
            mean_zero,
            mean_neg,
            mean_pos
        ));
    }

    let _ = std::fs::write("embeddings_summary.csv", csv.as_bytes());
    let _ = std::fs::write("data/embeddings_summary.csv", csv.as_bytes());

    let mut response = csv.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"embeddings_summary.csv\""),
    );
    Ok(response)
}

async fn get_embeddings_vectors_csv(
    State(state): State<AppState>,
) -> std::result::Result<Response, ApiError> {
    let all_map = state.face_index.get_all_embeddings();
    let mut usernames: Vec<String> = all_map.keys().cloned().collect();
    usernames.sort();

    let mut header_cols = vec![
        "FAISS_Row_ID".to_string(),
        "Username".to_string(),
        "Image_Name".to_string(),
        "Dimension".to_string(),
        "L2_Norm".to_string(),
    ];
    for d in 0..512 {
        header_cols.push(format!("dim_{}", d));
    }
    let mut csv = header_cols.join(",") + "\n";

    let mut row_id = 0usize;
    for uname in usernames {
        if let Some(vecs) = all_map.get(&uname) {
            for (i, vec) in vecs.iter().enumerate() {
                let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
                let dims_str: Vec<String> = vec.iter().map(|v| format!("{:.6}", v)).collect();
                csv.push_str(&format!(
                    "{},{},image_{},{},{:.6},{}\n",
                    row_id,
                    uname,
                    i + 1,
                    vec.len(),
                    norm,
                    dims_str.join(",")
                ));
                row_id += 1;
            }
        }
    }

    let _ = std::fs::write("embeddings_vectors_512d.csv", csv.as_bytes());
    let _ = std::fs::write("data/embeddings_vectors_512d.csv", csv.as_bytes());

    let mut response = csv.into_response();
    response.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    response.headers_mut().insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_static("attachment; filename=\"embeddings_vectors_512d.csv\""),
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
    #[serde(default)]
    pub pose_index: Option<usize>,
    #[serde(default)]
    pub enforce_pose: Option<bool>,
    #[serde(default)]
    pub is_recapture: Option<bool>,
    #[serde(default)]
    pub baseline_yaw: Option<f32>,
    #[serde(default)]
    pub baseline_pitch: Option<f32>,
    #[serde(default)]
    pub baseline_roll: Option<f32>,
    #[serde(default)]
    pub baseline_area: Option<f32>,
    #[serde(default)]
    pub prev_yaw: Option<f32>,
    #[serde(default)]
    pub prev_pitch: Option<f32>,
    #[serde(default)]
    pub prev_roll: Option<f32>,
    #[serde(default)]
    pub prev_area: Option<f32>,
    #[serde(default)]
    pub attempt: Option<usize>,
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

/// Extract 512-D FaceNet512 embedding from a single camera frame + verify 20-pose head orientation & image quality gate
async fn handle_extract_frame(
    State(state): State<AppState>,
    Json(payload): Json<SingleImagePayload>,
) -> std::result::Result<Response, ApiError> {
    let df_state = state.get_deepface().ok_or_else(|| {
        attendance_common::AppError::Validation(
            "DeepFace ONNX models not loaded in ./models/".to_string(),
        )
    })?;
    let bytes = decode_data_uri(&payload.image_b64)?;

    let (emb, det, dims) = match attendance_face::extract_face_embedding_with_detection(&df_state, &bytes) {
        Ok(res) => res,
        Err(face_err) => {
            let mut img_quality = attendance_face::FaceQualityAssessor::assess_image_bytes(&bytes);
            let (reason_code, msg) = match &face_err {
                attendance_face::FaceError::MultipleFacesDetected(n) => (
                    "MULTIPLE_FACES",
                    format!(
                        "Multiple faces ({}) detected — only one person should be in front of the camera! (একাধিক মুখ শনাক্ত হয়েছে — ক্যামেরার সামনে একজন থাকুন ও পুনরায় ছবি তুলুন)",
                        n
                    ),
                ),
                _ => {
                    if let Some(ref q) = img_quality {
                        if q.is_dark {
                            (
                                "DARK",
                                format!(
                                    "Image is too dark or black (brightness {:.1} < 38.0) — Must retake in proper lighting! (ছবিটি অন্ধকার বা কালো — পর্যাপ্ত আলোতে পুনরায় ছবি তুলুন)",
                                    q.brightness
                                ),
                            )
                        } else if q.is_blurry {
                            (
                                "BLURRY",
                                format!(
                                    "Image is blurry (sharpness {:.1} < 35.0) — Hold camera steady and retake! (ছবিটি ঘোলা বা অস্পষ্ট — ক্যামেরা স্থির রেখে পুনরায় ছবি তুলুন)",
                                    q.sharpness
                                ),
                            )
                        } else if q.is_unclear {
                            (
                                "FACE_NOT_CLEAR",
                                "Face is not clear or low contrast — Keep your face clearly visible inside the camera frame and retake! (চেহারা স্পষ্ট নয় — ক্যামেরার ফ্রেমে স্পষ্টভাবে মুখ রেখে পুনরায় ছবি তুলুন)".to_string(),
                            )
                        } else {
                            (
                                "NO_FACE",
                                "No face detected in frame — Position your face clearly in front of the camera and retake! (কোনো মুখ শনাক্ত হয়নি — ক্যামেরার সামনে মুখ রেখে পুনরায় ছবি তুলুন)".to_string(),
                            )
                        }
                    } else {
                        (
                            "FACE_NOT_CLEAR",
                            "Face is not clear or not detected — Must retake! (চেহারা স্পষ্ট নয় — পুনরায় ছবি তুলুন)".to_string(),
                        )
                    }
                }
            };

            let (bright_val, sharp_val) = if let Some(ref mut q) = img_quality {
                q.quality_ok = false;
                q.passes_quality_gate = false;
                q.face_confidence = 0.0;
                q.quality_reason = Some(reason_code.to_string());
                (q.brightness, q.sharpness)
            } else {
                (0.0_f32, 0.0_f32)
            };

            let body = Json(serde_json::json!({
                "success": false,
                "face_detected": false,
                "pose_verified": false,
                "quality_ok": false,
                "must_retake": true,
                "quality_reason": reason_code,
                "brightness": bright_val,
                "sharpness": sharp_val,
                "face_confidence": 0.0_f32,
                "error": msg,
                "message": msg,
                "quality": img_quality
            }));
            return Ok((axum::http::StatusCode::UNPROCESSABLE_ENTITY, body).into_response());
        }
    };

    let quality = attendance_face::FaceQualityAssessor::assess_image_and_face(
        &bytes,
        Some(det.bbox),
        Some(det.confidence),
    );

    // Compute 3D head pose ratios from MTCNN 5-point landmarks: [left_eye, right_eye, nose, mouth_left, mouth_right]
    let (lx, ly) = det.landmarks[0];
    let (rx, ry) = det.landmarks[1];
    let (nx, ny) = det.landmarks[2];
    let (mlx, mly) = det.landmarks[3];
    let (mrx, mry) = det.landmarks[4];

    let dl = (nx - lx).abs();
    let dr = (rx - nx).abs();
    let yaw_ratio = (dl - dr) / (dl + dr).max(1.0);
    let yaw_deg = (yaw_ratio * 65.0).clamp(-75.0, 75.0);

    let eye_cy = (ly + ry) * 0.5;
    let mouth_cy = (mly + mry) * 0.5;
    let pitch_ratio = (ny - eye_cy) / (mouth_cy - eye_cy).abs().max(1.0);
    let pitch_deg = ((0.55 - pitch_ratio) * 90.0).clamp(-60.0, 60.0);

    let roll_deg = (ry - ly).atan2((rx - lx).abs().max(1.0)).to_degrees();

    let bbox_w = (det.bbox.2 - det.bbox.0).max(1.0);
    let bbox_h = (det.bbox.3 - det.bbox.1).max(1.0);
    let img_area = ((dims.0 as f32) * (dims.1 as f32)).max(1.0);
    let face_area_ratio = (bbox_w * bbox_h) / img_area;

    let eye_dx = rx - lx;
    let eye_dy = ry - ly;
    let eye_dist = (eye_dx * eye_dx + eye_dy * eye_dy).sqrt().max(1.0);
    let mouth_dx = mrx - mlx;
    let mouth_dy = mry - mly;
    let mouth_width_ratio = (mouth_dx * mouth_dx + mouth_dy * mouth_dy).sqrt() / eye_dist;

    let pose_idx = payload.pose_index.unwrap_or(0).min(19);
    let attempt = payload.attempt.unwrap_or(0);
    let relax = if attempt >= 3 { 0.75_f32 } else { 1.0_f32 };

    let base_yaw = payload.baseline_yaw.unwrap_or(0.0);
    let base_pitch = payload.baseline_pitch.unwrap_or(0.535);
    let base_roll = payload.baseline_roll.unwrap_or(0.0);
    let base_area = payload.baseline_area.unwrap_or(face_area_ratio);

    let delta_yaw = yaw_ratio - base_yaw;
    let delta_pitch = pitch_ratio - base_pitch;
    let delta_roll = roll_deg - base_roll;
    let delta_area = (face_area_ratio - base_area) / base_area.max(0.005);

    let detected_pose = if yaw_ratio.abs() >= 0.115 {
        if yaw_ratio > 0.0 { "MORE_LEFT" } else { "MORE_RIGHT" }
    } else if yaw_ratio.abs() >= 0.075 {
        if yaw_ratio > 0.0 { "SLIGHT_LEFT" } else { "SLIGHT_RIGHT" }
    } else if roll_deg.abs() >= 3.2 {
        if roll_deg > 0.0 { "TILT_LEFT" } else { "TILT_RIGHT" }
    } else if pitch_ratio <= 0.512 || (payload.baseline_pitch.is_some() && delta_pitch <= -0.015) {
        "TILT_UP"
    } else if pitch_ratio >= 0.562 || (payload.baseline_pitch.is_some() && delta_pitch >= 0.015) {
        "TILT_DOWN"
    } else {
        "FRONTAL"
    };

    let is_frontal_abs = yaw_ratio.abs() <= 0.115
        && roll_deg.abs() <= 6.5
        && (0.475..=0.585).contains(&pitch_ratio);
    let is_frontal_rel = payload.baseline_yaw.is_some()
        && delta_yaw.abs() <= 0.065
        && delta_pitch.abs() <= 0.035
        && delta_roll.abs() <= 4.5
        && yaw_ratio.abs() <= 0.13
        && roll_deg.abs() <= 8.0;
    let is_frontal = is_frontal_abs || (pose_idx > 0 && is_frontal_rel);

    let (expected_pose, pose_verified, guidance) = match pose_idx {
        0 => (
            "1/20: FRONTAL_NORMAL",
            is_frontal_abs,
            "1/20: Look straight at camera (ক্যামেরার দিকে সরাসরি সোজা হয়ে তাকান)",
        ),
        1 => (
            "2/20: FRONTAL_SLIGHT_SMILE",
            is_frontal,
            "2/20: Look straight with a slight smile (সরাসরি তাকান ও হালকা হাসুন)",
        ),
        2 => (
            "3/20: TILT_HEAD_UP",
            delta_pitch <= -0.015 * relax || pitch_ratio <= 0.512,
            if delta_pitch >= 0.015 || pitch_ratio >= 0.565 {
                "3/20: You tilted DOWN — please tilt your head slightly UP (আপনি নিচে তাকিয়েছেন — মাথা সামান্য উপরে তুলুন)"
            } else {
                "3/20: Please tilt your head slightly UP (মাথা সামান্য উপরে তুলুন)"
            },
        ),
        3 => (
            "4/20: TILT_HEAD_DOWN",
            delta_pitch >= 0.015 * relax || pitch_ratio >= 0.562,
            if delta_pitch <= -0.015 || pitch_ratio <= 0.512 {
                "4/20: You tilted UP — please tilt your head slightly DOWN (আপনি উপরে তাকিয়েছেন — মাথা সামান্য নিচে নামান)"
            } else {
                "4/20: Please tilt your head slightly DOWN (মাথা সামান্য নিচে নামান)"
            },
        ),
        4 => (
            "5/20: TURN_SLIGHT_LEFT",
            delta_yaw >= 0.055 * relax || yaw_ratio >= 0.075 * relax,
            if yaw_ratio <= -0.07 {
                "5/20: You turned RIGHT — please turn your head slightly LEFT (আপনি ডানে তাকিয়েছেন — মাথা সামান্য বামে ঘোরান)"
            } else {
                "5/20: Please turn your head slightly LEFT (মাথা সামান্য বামে ঘোরান)"
            },
        ),
        5 => (
            "6/20: TURN_SLIGHT_RIGHT",
            delta_yaw <= -0.055 * relax || yaw_ratio <= -0.075 * relax,
            if yaw_ratio >= 0.07 {
                "6/20: You turned LEFT — please turn your head slightly RIGHT (আপনি বামে তাকিয়েছেন — মাথা সামান্য ডানে ঘোরান)"
            } else {
                "6/20: Please turn your head slightly RIGHT (মাথা সামান্য ডানে ঘোরান)"
            },
        ),
        6 => (
            "7/20: TILT_LEFT_SHOULDER",
            delta_roll >= 2.8 * relax || roll_deg >= 3.2 * relax,
            if roll_deg <= -3.0 {
                "7/20: You tilted to RIGHT shoulder — please tilt your head to LEFT shoulder (মাথা বাম কাঁধের দিকে কাত করুন)"
            } else {
                "7/20: Please tilt your head to LEFT shoulder (মাথা বাম কাঁধের দিকে কাত করুন)"
            },
        ),
        7 => (
            "8/20: TILT_RIGHT_SHOULDER",
            delta_roll <= -2.8 * relax || roll_deg <= -3.2 * relax,
            if roll_deg >= 3.0 {
                "8/20: You tilted to LEFT shoulder — please tilt your head to RIGHT shoulder (মাথা ডান কাঁধের দিকে কাত করুন)"
            } else {
                "8/20: Please tilt your head to RIGHT shoulder (মাথা ডান কাঁধের দিকে কাত করুন)"
            },
        ),
        8 => (
            "9/20: FRONTAL_SERIOUS",
            is_frontal,
            "9/20: Look straight with a serious expression (সরাসরি তাকান — গম্ভীর অভিব্যক্তি)",
        ),
        9 => (
            "10/20: FRONTAL_EYES_WIDE",
            is_frontal,
            "10/20: Look straight with eyes wide (সরাসরি তাকান — চোখ বড় করে)",
        ),
        10 => (
            "11/20: TURN_MORE_LEFT",
            delta_yaw >= 0.095 * relax || yaw_ratio >= 0.115 * relax,
            if yaw_ratio <= -0.07 {
                "11/20: You turned RIGHT — please turn your head MORE to LEFT (আপনি ডানে তাকিয়েছেন — মাথা আরও একটু বামে ঘোরান)"
            } else {
                "11/20: Please turn your head MORE to LEFT (মাথা আরও একটু বামে ঘোরান)"
            },
        ),
        11 => (
            "12/20: TURN_MORE_RIGHT",
            delta_yaw <= -0.095 * relax || yaw_ratio <= -0.115 * relax,
            if yaw_ratio >= 0.07 {
                "12/20: You turned LEFT — please turn your head MORE to RIGHT (আপনি বামে তাকিয়েছেন — মাথা আরও একটু ডানে ঘোরান)"
            } else {
                "12/20: Please turn your head MORE to RIGHT (মাথা আরও একটু ডানে ঘোরান)"
            },
        ),
        12 => (
            "13/20: CHIN_UP",
            delta_pitch <= -0.012 * relax || pitch_ratio <= 0.515,
            if delta_pitch >= 0.015 || pitch_ratio >= 0.565 {
                "13/20: You lowered chin DOWN — please raise chin UP slightly (থুতনি সামান্য উপরে তুলে ক্যামেরায় তাকান)"
            } else {
                "13/20: Raise chin UP slightly & look at camera (থুতনি সামান্য উপরে তুলে ক্যামেরায় তাকান)"
            },
        ),
        13 => (
            "14/20: CHIN_DOWN",
            delta_pitch >= 0.012 * relax || pitch_ratio >= 0.558,
            if delta_pitch <= -0.015 || pitch_ratio <= 0.512 {
                "14/20: You raised chin UP — please lower chin DOWN slightly (থুতনি সামান্য নিচে নামিয়ে ক্যামেরায় তাকান)"
            } else {
                "14/20: Lower chin DOWN slightly & look at camera (থুতনি সামান্য নিচে নামিয়ে ক্যামেরায় তাকান)"
            },
        ),
        14 => {
            let closer_ok = if payload.baseline_area.is_some() {
                delta_area >= 0.05 * relax || face_area_ratio >= 0.24
            } else {
                face_area_ratio >= 0.14
            };
            (
                "15/20: MOVE_CLOSER",
                closer_ok,
                "15/20: Move slightly CLOSER to the camera (ক্যামেরার সামান্য কাছে আসুন)",
            )
        }
        15 => {
            let moved_back_from_prev = payload
                .prev_area
                .map(|pa| ((face_area_ratio - pa) / pa.max(0.005)) <= -0.06 * relax && delta_area <= 0.05)
                .unwrap_or(false);
            let further_ok = if payload.baseline_area.is_some() {
                delta_area <= -0.05 * relax || moved_back_from_prev || face_area_ratio <= 0.065
            } else {
                face_area_ratio <= 0.075 || moved_back_from_prev
            };
            (
                "16/20: MOVE_FURTHER",
                further_ok,
                "16/20: Move slightly FURTHER from the camera (ক্যামেরা থেকে সামান্য দূরে যান)",
            )
        }
        16 => (
            "17/20: LOOK_UP_LEFT",
            (delta_yaw >= 0.045 * relax || yaw_ratio >= 0.065 * relax)
                && (delta_pitch <= -0.008 * relax || pitch_ratio <= 0.535),
            if yaw_ratio <= -0.06 {
                "17/20: You turned RIGHT — look from UP-LEFT angle towards camera (উপরে-বামে কোণ থেকে ক্যামেরায় তাকান)"
            } else {
                "17/20: Look from UP-LEFT angle towards camera (উপরে-বামে কোণ থেকে ক্যামেরায় তাকান)"
            },
        ),
        17 => (
            "18/20: LOOK_UP_RIGHT",
            (delta_yaw <= -0.045 * relax || yaw_ratio <= -0.065 * relax)
                && (delta_pitch <= -0.008 * relax || pitch_ratio <= 0.535),
            if yaw_ratio >= 0.06 {
                "18/20: You turned LEFT — look from UP-RIGHT angle towards camera (উপরে-ডানে কোণ থেকে ক্যামেরায় তাকান)"
            } else {
                "18/20: Look from UP-RIGHT angle towards camera (উপরে-ডানে কোণ থেকে ক্যামেরায় তাকান)"
            },
        ),
        18 => (
            "19/20: FRONTAL_NATURAL_SMILE",
            is_frontal,
            "19/20: Give a natural smile & look straight (স্বাভাবিক হাসি দিয়ে সরাসরি তাকান)",
        ),
        _ => (
            "20/20: FRONTAL_FINAL_RELAXED",
            is_frontal,
            "20/20: Final straight relaxed pose (চূড়ান্ত সোজা পোজ — স্বাভাবিক থাকুন)",
        ),
    };

    let pose_metrics = serde_json::json!({
        "yaw_ratio": yaw_ratio,
        "yaw_deg": yaw_deg,
        "pitch_ratio": pitch_ratio,
        "pitch_deg": pitch_deg,
        "roll_deg": roll_deg,
        "face_area_ratio": face_area_ratio,
        "mouth_width_ratio": mouth_width_ratio
    });

    let bright_val = quality.as_ref().map(|q| q.brightness).unwrap_or(100.0);
    let sharp_val = quality.as_ref().map(|q| q.sharpness).unwrap_or(100.0);
    let face_conf_val = det.confidence;

    // Hard Quality Gate: If image is blurry, dark/black, or face is not clear, MUST retake!
    if let Some(ref q) = quality {
        if !q.passes_quality_gate {
            let (reason_code, qual_msg) = if q.is_dark {
                (
                    "DARK",
                    format!(
                        "Image/face is too dark or black (brightness {:.1} < 38.0) — Must retake in proper lighting! (ছবিটি অন্ধকার বা কালো — পর্যাপ্ত আলোতে পুনরায় ছবি তুলুন)",
                        q.brightness
                    ),
                )
            } else if q.is_blurry {
                (
                    "BLURRY",
                    format!(
                        "Face is blurry / out of focus (sharpness {:.1} < 32.0) — Hold camera steady and retake! (ছবিটি ঘোলা বা অস্পষ্ট — ক্যামেরা স্থির রেখে পুনরায় ছবি তুলুন)",
                        q.sharpness
                    ),
                )
            } else {
                (
                    "FACE_NOT_CLEAR",
                    format!(
                        "Face is not clear ({}) — Must retake! (চেহারা স্পষ্ট নয় — পুনরায় পরিষ্কার ছবি তুলুন)",
                        q.failure_reasons.join("; ")
                    ),
                )
            };

            let body = Json(serde_json::json!({
                "success": false,
                "face_detected": true,
                "pose_verified": false,
                "quality_ok": false,
                "must_retake": true,
                "quality_reason": reason_code,
                "brightness": bright_val,
                "sharpness": sharp_val,
                "face_confidence": face_conf_val,
                "pose_index": pose_idx,
                "expected_pose": expected_pose,
                "detected_pose": detected_pose,
                "pose_metrics": pose_metrics,
                "error": qual_msg,
                "message": qual_msg,
                "bbox": det.bbox,
                "detection_score": det.confidence,
                "landmarks": det.landmarks,
                "quality": quality
            }));
            return Ok((axum::http::StatusCode::UNPROCESSABLE_ENTITY, body).into_response());
        }
    }

    let enforce_pose = payload.enforce_pose.unwrap_or_else(|| payload.pose_index.is_some());
    let final_pose_verified = if payload.is_recapture.unwrap_or(false) {
        true
    } else if enforce_pose {
        pose_verified
    } else {
        true
    };

    if !final_pose_verified {
        let err_msg = format!(
            "Pose angle skipped (marked ✗) — {} [Current: {}, Yaw: {:+.1}°, Pitch: {:+.1}°, Roll: {:+.1}°]",
            guidance, detected_pose, yaw_deg, pitch_deg, roll_deg
        );
        return Ok(Json(serde_json::json!({
            "success": false,
            "face_detected": true,
            "quality_ok": true,
            "quality_reason": serde_json::Value::Null,
            "brightness": bright_val,
            "sharpness": sharp_val,
            "face_confidence": face_conf_val,
            "must_retake": false,
            "pose_verified": false,
            "pose_status": "CROSS_MARKED_UNVERIFIED",
            "pose_index": pose_idx,
            "expected_pose": expected_pose,
            "detected_pose": detected_pose,
            "pose_metrics": pose_metrics,
            "dimension": emb.len(),
            "model": "Facenet512",
            "detector": "MTCNN",
            "error": err_msg,
            "message": guidance,
            "bbox": det.bbox,
            "detection_score": det.confidence,
            "landmarks": det.landmarks,
            "quality": quality,
            "embedding": emb
        }))
        .into_response());
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "face_detected": true,
        "quality_ok": true,
        "quality_reason": serde_json::Value::Null,
        "brightness": bright_val,
        "sharpness": sharp_val,
        "face_confidence": face_conf_val,
        "must_retake": false,
        "pose_verified": true,
        "pose_status": "TICK_MARKED_VERIFIED",
        "pose_index": pose_idx,
        "expected_pose": expected_pose,
        "detected_pose": detected_pose,
        "pose_metrics": pose_metrics,
        "dimension": emb.len(),
        "model": "Facenet512",
        "detector": "MTCNN",
        "bbox": det.bbox,
        "detection_score": det.confidence,
        "landmarks": det.landmarks,
        "quality": quality,
        "embedding": emb
    }))
    .into_response())
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

