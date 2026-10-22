use crate::metrics::DistanceMetrics;
use attendance_common::{AppError, Result};
use attendance_domain::FaceMatchResult;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use crate::facenet512::{FACENET512_DIMENSION, FACENET512_MATCH_THRESHOLD};
use std::sync::Arc;

pub const DEFAULT_EMBEDDING_DIM: usize = FACENET512_DIMENSION;
pub const DEFAULT_DISTANCE_THRESHOLD: f32 = FACENET512_MATCH_THRESHOLD;

#[derive(Clone, Default, Serialize, Deserialize)]
pub struct FaceIndexState {
    pub embeddings: HashMap<String, Vec<Vec<f32>>>,
    pub threshold: f32,
    pub dimension: usize,
    pub total_enrolled_faces: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TieredFaceMatchResult {
    pub username: String,
    pub match_tier: String, // "EXACT", "STRONG", "POSSIBLE", "REJECT"
    pub min_distance: f32,
    pub confidence_percent: f32,
    pub total_matches: usize,
    pub exact_matches: usize,
    pub strong_matches: usize,
    pub possible_matches: usize,
    pub is_match: bool,
}

#[derive(Clone)]
pub struct FaceVectorIndex {
    state: Arc<RwLock<FaceIndexState>>,
}

impl FaceVectorIndex {
    pub fn new(threshold: f32) -> Self {
        Self {
            state: Arc::new(RwLock::new(FaceIndexState {
                embeddings: HashMap::new(),
                threshold,
                dimension: DEFAULT_EMBEDDING_DIM,
                total_enrolled_faces: 0,
            })),
        }
    }

    pub fn enroll_user(&self, username: &str, mut vector: Vec<f32>) -> Result<()> {
        let dim = { self.state.read().dimension };
        if vector.len() != dim {
            return Err(AppError::Validation(format!(
                "Vector dimension mismatch: expected {}, got {}",
                dim,
                vector.len()
            )));
        }

        DistanceMetrics::l2_normalize(&mut vector);
        let mut st = self.state.write();
        let list = st.embeddings.entry(username.to_uppercase()).or_default();
        list.push(vector);
        st.total_enrolled_faces += 1;

        tracing::info!("Enrolled face embedding for user {} (total: {})", username, st.total_enrolled_faces);
        Ok(())
    }

    pub fn search(&self, mut query_vector: Vec<f32>) -> Result<Option<FaceMatchResult>> {
        let (dim, threshold) = {
            let st = self.state.read();
            (st.dimension, st.threshold)
        };

        if query_vector.len() != dim {
            return Err(AppError::Validation(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                dim,
                query_vector.len()
            )));
        }

        DistanceMetrics::l2_normalize(&mut query_vector);
        let st = self.state.read();

        let mut best_username = None;
        let mut min_distance = f32::MAX;

        for (user, vectors) in st.embeddings.iter() {
            for v in vectors {
                let dist = DistanceMetrics::euclidean_l2(&query_vector, v);
                if dist < min_distance {
                    min_distance = dist;
                    best_username = Some(user.clone());
                }
            }
        }

        if let Some(user) = best_username {
            let is_match = min_distance <= threshold;
            let confidence = if is_match {
                ((1.0 - (min_distance / threshold)) * 100.0).clamp(50.0, 99.9)
            } else {
                0.0
            };

            Ok(Some(FaceMatchResult {
                username: user,
                distance: min_distance,
                confidence_percent: confidence,
                is_match,
            }))
        } else {
            Ok(None)
        }
    }

    /// Search with multi-tier thresholds exactly matching Python attendance_v8.py lines 1828-1928:
    /// EXACT_MATCH: 0.40, STRONG_MATCH: 0.50, POSSIBLE_MATCH: 0.60, REJECT: 0.70
    pub fn search_tiered(&self, mut query_vector: Vec<f32>) -> Result<Option<TieredFaceMatchResult>> {
        let dim = { self.state.read().dimension };
        if query_vector.len() != dim {
            return Err(AppError::Validation(format!(
                "Query vector dimension mismatch: expected {}, got {}",
                dim, query_vector.len()
            )));
        }

        DistanceMetrics::l2_normalize(&mut query_vector);
        let st = self.state.read();

        const EXACT_THRESHOLD: f32 = 0.40;
        const STRONG_THRESHOLD: f32 = 0.50;
        const POSSIBLE_THRESHOLD: f32 = 0.60;
        const REJECT_THRESHOLD: f32 = 0.70;

        let mut best_result: Option<TieredFaceMatchResult> = None;
        let mut best_confidence = 0.0f32;

        for (user, vectors) in st.embeddings.iter() {
            let mut exact_count = 0usize;
            let mut strong_count = 0usize;
            let mut possible_count = 0usize;
            let mut total_matches = 0usize;
            let mut min_dist = f32::MAX;

            for v in vectors {
                let dist = DistanceMetrics::euclidean_l2(&query_vector, v);
                if dist > REJECT_THRESHOLD {
                    continue;
                }
                total_matches += 1;
                if dist < min_dist {
                    min_dist = dist;
                }

                if dist <= EXACT_THRESHOLD {
                    exact_count += 1;
                } else if dist <= STRONG_THRESHOLD {
                    strong_count += 1;
                } else if dist <= POSSIBLE_THRESHOLD {
                    possible_count += 1;
                }
            }

            if total_matches == 0 {
                continue;
            }

            let (tier, conf) = if vectors.len() >= 8 && exact_count >= 8 && min_dist <= EXACT_THRESHOLD {
                ("EXACT", 95.0 + (exact_count as f32 / 10.0) * 5.0)
            } else if vectors.len() >= 8 && (exact_count + strong_count) >= 7 && min_dist <= STRONG_THRESHOLD {
                ("STRONG", 80.0 + (total_matches as f32 / 10.0) * 15.0)
            } else if vectors.len() >= 8 && (exact_count + strong_count + possible_count) >= 6 && min_dist <= POSSIBLE_THRESHOLD {
                ("POSSIBLE", 60.0 + (total_matches as f32 / 10.0) * 20.0)
            } else if min_dist <= EXACT_THRESHOLD {
                ("EXACT", 95.0 + ((EXACT_THRESHOLD - min_dist) / EXACT_THRESHOLD) * 4.9)
            } else if min_dist <= STRONG_THRESHOLD {
                ("STRONG", 80.0 + ((STRONG_THRESHOLD - min_dist) / (STRONG_THRESHOLD - EXACT_THRESHOLD)) * 14.9)
            } else if min_dist <= POSSIBLE_THRESHOLD {
                ("POSSIBLE", 60.0 + ((POSSIBLE_THRESHOLD - min_dist) / (POSSIBLE_THRESHOLD - STRONG_THRESHOLD)) * 19.9)
            } else {
                ("REJECT", 0.0)
            };

            if conf > best_confidence {
                best_confidence = conf;
                best_result = Some(TieredFaceMatchResult {
                    username: user.clone(),
                    match_tier: tier.to_string(),
                    min_distance: min_dist,
                    confidence_percent: conf.clamp(0.0, 99.9),
                    total_matches,
                    exact_matches: exact_count,
                    strong_matches: strong_count,
                    possible_matches: possible_count,
                    is_match: tier != "REJECT",
                });
            }
        }

        Ok(best_result)
    }

    pub fn count_users(&self) -> usize {
        self.state.read().embeddings.len()
    }

    pub fn count_faces(&self) -> usize {
        self.state.read().total_enrolled_faces
    }

    pub fn clear(&self) {
        let mut st = self.state.write();
        st.embeddings.clear();
        st.total_enrolled_faces = 0;
    }

    pub fn get_all_embeddings(&self) -> HashMap<String, Vec<Vec<f32>>> {
        self.state.read().embeddings.clone()
    }

    /// Check if a face embedding already belongs to another user (matching Python _is_duplicate_user / _compare_face_embeddings lines 4770-4821)
    /// Uses cosine distance < 0.40 (1.0 - cosine_similarity < 0.40)
    pub fn find_duplicate_user(&self, query_vector: &[f32], exclude_username: Option<&str>) -> Option<(String, f32)> {
        if query_vector.is_empty() || query_vector.iter().all(|&x| x == 0.0) {
            return None;
        }
        let excl = exclude_username.map(|u| u.to_uppercase());
        let st = self.state.read();
        let q_norm: f32 = query_vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if q_norm == 0.0 {
            return None;
        }

        for (user, vectors) in st.embeddings.iter() {
            if let Some(ref ex) = excl {
                if user.eq_ignore_ascii_case(ex) {
                    continue;
                }
            }
            for v in vectors {
                if v.len() != query_vector.len() || v.iter().all(|&x| x == 0.0) {
                    continue;
                }
                let v_norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                if v_norm == 0.0 {
                    continue;
                }
                let dot: f32 = query_vector.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                let cosine_similarity = dot / (q_norm * v_norm);
                let distance = 1.0 - cosine_similarity;
                if distance < 0.40 {
                    return Some((user.clone(), distance));
                }
            }
        }
        None
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let st = self.state.read();
        let json = serde_json::to_string_pretty(&*st)?;
        let mut f = File::create(path)?;
        f.write_all(json.as_bytes())?;
        Ok(())
    }

    pub fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        if !path.as_ref().exists() {
            return Ok(());
        }
        let mut f = File::open(path)?;
        let mut contents = String::new();
        f.read_to_string(&mut contents)?;
        let loaded: FaceIndexState = serde_json::from_str(&contents)?;
        let mut st = self.state.write();
        *st = loaded;
        Ok(())
    }
}
