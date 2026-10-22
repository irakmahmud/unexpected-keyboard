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

    pub fn is_corrupted_constant_vector(vector: &[f32]) -> bool {
        if vector.is_empty() {
            return true;
        }
        let first = vector[0];
        if vector.iter().all(|&x| (x - first).abs() < 1e-5) {
            return true;
        }
        let mean = vector.iter().copied().sum::<f32>() / (vector.len() as f32);
        let var = vector.iter().map(|&x| (x - mean) * (x - mean)).sum::<f32>() / (vector.len() as f32);
        var < 1e-6
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
        if Self::is_corrupted_constant_vector(&vector) {
            tracing::warn!("Skipped corrupted constant face vector for user {}", username);
            return Ok(());
        }

        DistanceMetrics::l2_normalize(&mut vector);
        {
            let mut st = self.state.write();
            let list = st.embeddings.entry(username.to_uppercase()).or_default();
            list.push(vector);
            st.total_enrolled_faces += 1;
            tracing::info!("Enrolled face embedding for user {} (total: {})", username, st.total_enrolled_faces);
        }
        let _ = self.sync_faiss_files();
        Ok(())
    }

    pub fn remove_user(&self, username: &str) -> usize {
        let removed_count = {
            let mut st = self.state.write();
            let key = username.trim().to_uppercase();
            let mut count = 0usize;
            if let Some(vecs) = st.embeddings.remove(&key) {
                count += vecs.len();
            }
            let matching_keys: Vec<String> = st
                .embeddings
                .keys()
                .filter(|k| k.eq_ignore_ascii_case(username.trim()))
                .cloned()
                .collect();
            for k in matching_keys {
                if let Some(vecs) = st.embeddings.remove(&k) {
                    count += vecs.len();
                }
            }
            st.total_enrolled_faces = st.embeddings.values().map(|v| v.len()).sum();
            count
        };
        let _ = self.sync_faiss_files();
        removed_count
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
                if Self::is_corrupted_constant_vector(v) {
                    continue;
                }
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

    /// Search with multi-tier thresholds matching Python attendance_v8.py lines 1828-1928:
    /// EXACT_MATCH: 0.40, STRONG_MATCH: 0.50, POSSIBLE_MATCH: 0.60, REJECT: 0.70
    /// Enforces 40% frame ratio when user has >= 8 enrolled poses.
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
        let mut best_confidence = -1.0f32;

        for (user, vectors) in st.embeddings.iter() {
            let valid_vectors: Vec<&Vec<f32>> = vectors
                .iter()
                .filter(|v| !Self::is_corrupted_constant_vector(v))
                .collect();
            if valid_vectors.is_empty() {
                continue;
            }

            let mut exact_count = 0usize;
            let mut strong_count = 0usize;
            let mut possible_count = 0usize;
            let mut total_matches = 0usize;
            let mut min_dist = f32::MAX;

            for v in &valid_vectors {
                let dist = DistanceMetrics::euclidean_l2(&query_vector, v);
                if dist < min_dist {
                    min_dist = dist;
                }
                if dist > REJECT_THRESHOLD {
                    continue;
                }
                total_matches += 1;

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

            let num_enrolled = valid_vectors.len();
            let tier_matches = exact_count + strong_count + possible_count;
            let match_ratio = (tier_matches as f32) / (num_enrolled as f32);

            let (tier, conf) = if num_enrolled >= 8 {
                if match_ratio < 0.40 {
                    ("REJECT", 0.0)
                } else if exact_count >= 8 && min_dist <= EXACT_THRESHOLD {
                    ("EXACT", 95.0 + (exact_count as f32 / num_enrolled as f32) * 4.9)
                } else if (exact_count + strong_count) >= 7 && min_dist <= STRONG_THRESHOLD {
                    ("STRONG", 80.0 + (match_ratio * 15.0))
                } else if tier_matches >= 6 && min_dist <= POSSIBLE_THRESHOLD {
                    ("POSSIBLE", 60.0 + (match_ratio * 20.0))
                } else if min_dist <= EXACT_THRESHOLD && match_ratio >= 0.40 {
                    ("STRONG", 80.0 + (match_ratio * 14.9))
                } else if min_dist <= STRONG_THRESHOLD && match_ratio >= 0.40 {
                    ("POSSIBLE", 65.0 + (match_ratio * 14.9))
                } else {
                    ("REJECT", 0.0)
                }
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
        {
            let mut st = self.state.write();
            st.embeddings.clear();
            st.total_enrolled_faces = 0;
        }
        let _ = self.sync_faiss_files();
    }

    pub fn get_all_embeddings(&self) -> HashMap<String, Vec<Vec<f32>>> {
        self.state.read().embeddings.clone()
    }

    /// Check if a face embedding already belongs to another user (matching Python _is_duplicate_user / _compare_face_embeddings lines 4770-4821)
    /// Uses cosine distance < 0.30 (FaceNet512 cosine threshold)
    pub fn find_duplicate_user(&self, query_vector: &[f32], exclude_username: Option<&str>) -> Option<(String, f32)> {
        if query_vector.is_empty() || Self::is_corrupted_constant_vector(query_vector) {
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
                if v.len() != query_vector.len() || Self::is_corrupted_constant_vector(v) {
                    continue;
                }
                let v_norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
                if v_norm == 0.0 {
                    continue;
                }
                let dot: f32 = query_vector.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
                let cosine_similarity = dot / (q_norm * v_norm);
                let distance = 1.0 - cosine_similarity;
                if distance < 0.30 {
                    return Some((user.clone(), distance));
                }
            }
        }
        None
    }

    /// Synchronize FAISS-compatible ~/db/face_encodings.index and ~/db/user_mappings.txt (CAP-13)
    pub fn sync_faiss_files(&self) -> Result<()> {
        let st = self.state.read();
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/opc".to_string());
        let dirs = [
            std::path::PathBuf::from(&home).join("db"),
            std::path::PathBuf::from("data").join("db"),
        ];

        let mut sorted_users: Vec<&String> = st.embeddings.keys().collect();
        sorted_users.sort();

        let mut mappings = String::new();
        let mut raw_bytes: Vec<u8> = Vec::new();
        // FAISS IndexFlatL2 4-byte magic ("IxF2") + dim (u32) + ntotal (u64)
        let ntotal: u64 = st.embeddings.values().map(|v| v.len() as u64).sum();
        raw_bytes.extend_from_slice(b"IxF2");
        raw_bytes.extend_from_slice(&(st.dimension as u32).to_le_bytes());
        raw_bytes.extend_from_slice(&ntotal.to_le_bytes());

        let mut idx = 0usize;
        for user in sorted_users {
            if let Some(vecs) = st.embeddings.get(user) {
                for vec in vecs {
                    mappings.push_str(&format!("{}:{}\n", idx, user));
                    for &val in vec {
                        raw_bytes.extend_from_slice(&val.to_le_bytes());
                    }
                    idx += 1;
                }
            }
        }

        for dir in &dirs {
            if std::fs::create_dir_all(dir).is_ok() {
                let index_path = dir.join("face_encodings.index");
                let map_path = dir.join("user_mappings.txt");
                let _ = std::fs::write(&index_path, &raw_bytes);
                let _ = std::fs::write(&map_path, mappings.as_bytes());
            }
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    fn make_basis_vec(idx: usize, sign: f32) -> Vec<f32> {
        let mut v = vec![0.0f32; 512];
        v[idx % 512] = sign;
        v[(idx + 1) % 512] = 0.01;
        DistanceMetrics::l2_normalize(&mut v);
        v
    }

    #[test]
    fn test_corrupted_constant_vector_rejected() {
        let constant_vec = vec![0.044194173f32; 512];
        assert!(FaceVectorIndex::is_corrupted_constant_vector(&constant_vec));
        let valid_vec = make_basis_vec(0, 1.0);
        assert!(!FaceVectorIndex::is_corrupted_constant_vector(&valid_vec));
    }

    #[test]
    fn test_search_tiered_enforces_40_percent_ratio() {
        let index = FaceVectorIndex::new(0.40);
        let target = make_basis_vec(10, 1.0);
        let orthogonal = make_basis_vec(200, 1.0);

        // Enroll 10 poses for USER_OUTLIER where only 1 pose matches target and 9 are orthogonal (10% < 40%)
        index.enroll_user("USER_OUTLIER", target.clone()).unwrap();
        for _ in 0..9 {
            index.enroll_user("USER_OUTLIER", orthogonal.clone()).unwrap();
        }

        let res = index.search_tiered(target.clone()).unwrap();
        assert!(res.is_none(), "1/10 matching frames (10% < 40%) must be rejected");

        // Enroll 10 poses for USER_VALID where 8 poses match target (80% >= 40%)
        for _ in 0..8 {
            index.enroll_user("USER_VALID", target.clone()).unwrap();
        }
        for _ in 0..2 {
            index.enroll_user("USER_VALID", orthogonal.clone()).unwrap();
        }

        let res2 = index.search_tiered(target).unwrap().expect("Should match USER_VALID");
        assert_eq!(res2.username, "USER_VALID");
        assert!(res2.is_match);
        assert_eq!(res2.match_tier, "EXACT");
    }

    #[test]
    fn test_remove_user_clears_ram_vectors() {
        let index = FaceVectorIndex::new(0.40);
        let v = make_basis_vec(5, 1.0);
        index.enroll_user("ALICE", v.clone()).unwrap();
        assert_eq!(index.count_users(), 1);
        let removed = index.remove_user("alice");
        assert_eq!(removed, 1);
        assert_eq!(index.count_users(), 0);
        assert_eq!(index.count_faces(), 0);
    }
}

