use crate::metrics::DistanceMetrics;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashSet};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswConfig {
    pub max_elements: usize,
    pub m: usize,              // number of established connections
    pub m0: usize,             // maximum number of connections for layer 0
    pub ef_construction: usize, // size of the dynamic candidate list
    pub ef_search: usize,       // search candidate list size
    pub ml: f64,               // normalization factor for level generation
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            max_elements: 10_000,
            m: 16,
            m0: 32,
            ef_construction: 64,
            ef_search: 32,
            ml: 1.0 / (16.0f64).ln(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HnswNode {
    pub id: usize,
    pub username: String,
    pub vector: Vec<f32>,
    pub level: usize,
    pub neighbors: Vec<Vec<usize>>, // neighbors per level
}

#[derive(Clone, Debug)]
struct NeighborCandidate {
    id: usize,
    distance: f32,
}

impl PartialEq for NeighborCandidate {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for NeighborCandidate {}

impl PartialOrd for NeighborCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NeighborCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse for min-heap
        other.distance.partial_cmp(&self.distance).unwrap_or(Ordering::Equal)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PureRustHnswIndex {
    pub config: HnswConfig,
    pub nodes: Vec<HnswNode>,
    pub enter_node: Option<usize>,
    pub max_level: usize,
}

impl PureRustHnswIndex {
    pub fn new(config: HnswConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            enter_node: None,
            max_level: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    fn random_level(&self) -> usize {
        let r: f64 = rand_float();
        let level = (-r.ln() * self.config.ml).floor() as usize;
        level.min(16)
    }

    pub fn insert(&mut self, username: String, mut vector: Vec<f32>) -> usize {
        DistanceMetrics::l2_normalize(&mut vector);
        let id = self.nodes.len();
        let level = self.random_level();

        let node = HnswNode {
            id,
            username,
            vector,
            level,
            neighbors: vec![Vec::new(); level + 1],
        };

        self.nodes.push(node);

        if self.enter_node.is_none() {
            self.enter_node = Some(id);
            self.max_level = level;
            return id;
        }

        let mut curr_entry = self.enter_node.unwrap();
        let query_vec = self.nodes[id].vector.clone();
        let mut curr_dist = DistanceMetrics::euclidean_l2(&self.nodes[curr_entry].vector, &query_vec);

        // 1. Greedily traverse down to insertion level
        for l in (level + 1..=self.max_level).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                if l < self.nodes[curr_entry].neighbors.len() {
                    for &neighbor_id in &self.nodes[curr_entry].neighbors[l] {
                        let d = DistanceMetrics::euclidean_l2(&self.nodes[neighbor_id].vector, &query_vec);
                        if d < curr_dist {
                            curr_dist = d;
                            curr_entry = neighbor_id;
                            changed = true;
                        }
                    }
                }
            }
        }

        // 2. Connect neighbors on all levels from min(level, max_level) down to 0
        let top_connect = level.min(self.max_level);
        let mut candidates = vec![curr_entry];

        for l in (0..=top_connect).rev() {
            let neighbors = self.search_layer_internal(&query_vec, &candidates, self.config.ef_construction, l);
            let m_max = if l == 0 { self.config.m0 } else { self.config.m };

            // Pick closest M neighbors
            let selected_neighbors: Vec<usize> = neighbors
                .iter()
                .filter(|c| c.id != id)
                .take(m_max)
                .map(|c| c.id)
                .collect();

            self.nodes[id].neighbors[l] = selected_neighbors.clone();

            // Bidirectional connection
            for &nbr_id in &selected_neighbors {
                if l < self.nodes[nbr_id].neighbors.len() {
                    self.nodes[nbr_id].neighbors[l].push(id);
                    if self.nodes[nbr_id].neighbors[l].len() > m_max {
                        self.shrink_neighbors(nbr_id, l, m_max);
                    }
                }
            }

            candidates = selected_neighbors;
        }

        if level > self.max_level {
            self.max_level = level;
            self.enter_node = Some(id);
        }

        id
    }

    fn search_layer_internal(
        &self,
        query: &[f32],
        entry_points: &[usize],
        ef: usize,
        level: usize,
    ) -> Vec<NeighborCandidate> {
        let mut visited = HashSet::new();
        let mut candidates = BinaryHeap::new();
        let mut results = Vec::new();

        for &ep in entry_points {
            let dist = DistanceMetrics::euclidean_l2(&self.nodes[ep].vector, query);
            visited.insert(ep);
            candidates.push(NeighborCandidate { id: ep, distance: dist });
            results.push(NeighborCandidate { id: ep, distance: dist });
        }

        while let Some(curr) = candidates.pop() {
            let worst_result_dist = results.iter().map(|c| c.distance).fold(0.0f32, f32::max);
            if curr.distance > worst_result_dist && results.len() >= ef {
                break;
            }

            if level < self.nodes[curr.id].neighbors.len() {
                for &neighbor_id in &self.nodes[curr.id].neighbors[level] {
                    if visited.insert(neighbor_id) {
                        let d = DistanceMetrics::euclidean_l2(&self.nodes[neighbor_id].vector, query);
                        if d < worst_result_dist || results.len() < ef {
                            candidates.push(NeighborCandidate { id: neighbor_id, distance: d });
                            results.push(NeighborCandidate { id: neighbor_id, distance: d });
                            results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Ordering::Equal));
                            if results.len() > ef {
                                results.pop();
                            }
                        }
                    }
                }
            }
        }

        results.sort_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(Ordering::Equal));
        results
    }

    fn shrink_neighbors(&mut self, node_id: usize, level: usize, max_neighbors: usize) {
        let query = &self.nodes[node_id].vector.clone();
        let mut nbr_with_dist: Vec<(usize, f32)> = self.nodes[node_id].neighbors[level]
            .iter()
            .map(|&nbr| {
                let d = DistanceMetrics::euclidean_l2(&self.nodes[nbr].vector, query);
                (nbr, d)
            })
            .collect();

        nbr_with_dist.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Ordering::Equal));
        self.nodes[node_id].neighbors[level] = nbr_with_dist
            .into_iter()
            .take(max_neighbors)
            .map(|(nbr, _)| nbr)
            .collect();
    }

    pub fn search_knn(&self, query: &[f32], k: usize) -> Vec<(String, f32)> {
        if self.nodes.is_empty() || self.enter_node.is_none() {
            return Vec::new();
        }

        let mut curr_entry = self.enter_node.unwrap();
        let mut curr_dist = DistanceMetrics::euclidean_l2(&self.nodes[curr_entry].vector, query);

        for l in (1..=self.max_level).rev() {
            let mut changed = true;
            while changed {
                changed = false;
                if l < self.nodes[curr_entry].neighbors.len() {
                    for &nbr in &self.nodes[curr_entry].neighbors[l] {
                        let d = DistanceMetrics::euclidean_l2(&self.nodes[nbr].vector, query);
                        if d < curr_dist {
                            curr_dist = d;
                            curr_entry = nbr;
                            changed = true;
                        }
                    }
                }
            }
        }

        let ef = self.config.ef_search.max(k);
        let candidates = self.search_layer_internal(query, &[curr_entry], ef, 0);

        candidates
            .into_iter()
            .take(k)
            .map(|c| (self.nodes[c.id].username.clone(), c.distance))
            .collect()
    }
}

// Pseudo random generator for level generation without heavy external dependencies
fn rand_float() -> f64 {
    use std::time::SystemTime;
    let nanos = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();
    let val = ((nanos % 99991) as f64) / 99991.0;
    val.clamp(0.0001, 0.9999)
}
