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

    pub fn count_users(&self) -> usize {
        self.state.read().embeddings.len()
    }

    pub fn count_faces(&self) -> usize {
        self.state.read().total_enrolled_faces
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
