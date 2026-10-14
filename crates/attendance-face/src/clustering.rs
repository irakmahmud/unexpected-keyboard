use crate::metrics::DistanceMetrics;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterResult {
    pub cluster_id: usize,
    pub indices: Vec<usize>,
    pub centroid: Vec<f32>,
    pub radius: f32,
}

pub struct FaceClustering;

impl FaceClustering {
    /// Compute centroid (element-wise mean normalized) from multiple face capture embeddings
    pub fn compute_centroid(embeddings: &[Vec<f32>]) -> Option<Vec<f32>> {
        if embeddings.is_empty() {
            return None;
        }

        let dim = embeddings[0].len();
        let mut centroid = vec![0.0f32; dim];

        for emb in embeddings {
            if emb.len() != dim {
                continue;
            }
            for (c, val) in centroid.iter_mut().zip(emb.iter()) {
                *c += val;
            }
        }

        let count = embeddings.len() as f32;
        for c in centroid.iter_mut() {
            *c /= count;
        }

        DistanceMetrics::l2_normalize(&mut centroid);
        Some(centroid)
    }

    /// Check if multi-frame embeddings are consistent (intra-cluster variance within threshold)
    pub fn intra_cluster_variance(embeddings: &[Vec<f32>], centroid: &[f32]) -> f32 {
        if embeddings.is_empty() {
            return 0.0;
        }

        let mut sum_dist = 0.0f32;
        for emb in embeddings {
            sum_dist += DistanceMetrics::euclidean_l2(emb, centroid);
        }

        sum_dist / embeddings.len() as f32
    }

    /// Pure Rust DBSCAN (Density-Based Spatial Clustering of Applications with Noise)
    /// Groups face embeddings into clusters without needing prior knowledge of cluster count.
    pub fn dbscan(
        embeddings: &[Vec<f32>],
        eps: f32,
        min_pts: usize,
    ) -> (Vec<ClusterResult>, Vec<usize>) {
        let n = embeddings.len();
        let mut visited = vec![false; n];
        let mut noise = Vec::new();
        let mut clusters = Vec::new();
        let mut cluster_id = 0;

        for i in 0..n {
            if visited[i] {
                continue;
            }
            visited[i] = true;

            let neighbors = Self::region_query(embeddings, i, eps);

            if neighbors.len() < min_pts {
                noise.push(i);
            } else {
                let mut cluster_indices = Vec::new();
                cluster_indices.push(i);

                let mut queue = VecDeque::new();
                for &nb in &neighbors {
                    if nb != i {
                        queue.push_back(nb);
                    }
                }

                let mut queued_set: HashSet<usize> = neighbors.into_iter().collect();

                while let Some(current_pt) = queue.pop_front() {
                    if !visited[current_pt] {
                        visited[current_pt] = true;
                        let current_neighbors = Self::region_query(embeddings, current_pt, eps);
                        if current_neighbors.len() >= min_pts {
                            for &cn in &current_neighbors {
                                if queued_set.insert(cn) {
                                    queue.push_back(cn);
                                }
                            }
                        }
                    }

                    if !cluster_indices.contains(&current_pt) {
                        cluster_indices.push(current_pt);
                    }
                }

                // Compute centroid and max radius for this cluster
                let cluster_vectors: Vec<Vec<f32>> = cluster_indices
                    .iter()
                    .map(|&idx| embeddings[idx].clone())
                    .collect();

                let centroid = Self::compute_centroid(&cluster_vectors).unwrap_or_default();
                let radius = cluster_vectors
                    .iter()
                    .map(|v| DistanceMetrics::euclidean_l2(v, &centroid))
                    .fold(0.0f32, f32::max);

                clusters.push(ClusterResult {
                    cluster_id,
                    indices: cluster_indices,
                    centroid,
                    radius,
                });
                cluster_id += 1;
            }
        }

        (clusters, noise)
    }

    fn region_query(embeddings: &[Vec<f32>], point_idx: usize, eps: f32) -> Vec<usize> {
        let mut neighbors = Vec::new();
        let target = &embeddings[point_idx];

        for (i, other) in embeddings.iter().enumerate() {
            let dist = DistanceMetrics::euclidean_l2(target, other);
            if dist <= eps {
                neighbors.push(i);
            }
        }

        neighbors
    }
}
