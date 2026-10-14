pub struct DistanceMetrics;

impl DistanceMetrics {
    /// Euclidean (L2) distance: sqrt(sum((a_i - b_i)^2))
    pub fn euclidean_l2(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }

        let mut sum = 0.0f32;
        // Loop unrolling for high-speed cache efficiency (8-way chunking)
        let chunks_a = a.chunks_exact(8);
        let chunks_b = b.chunks_exact(8);
        let remainder_a = chunks_a.remainder();
        let remainder_b = chunks_b.remainder();

        for (ca, cb) in chunks_a.zip(chunks_b) {
            let d0 = ca[0] - cb[0];
            let d1 = ca[1] - cb[1];
            let d2 = ca[2] - cb[2];
            let d3 = ca[3] - cb[3];
            let d4 = ca[4] - cb[4];
            let d5 = ca[5] - cb[5];
            let d6 = ca[6] - cb[6];
            let d7 = ca[7] - cb[7];
            sum += d0 * d0 + d1 * d1 + d2 * d2 + d3 * d3 + d4 * d4 + d5 * d5 + d6 * d6 + d7 * d7;
        }

        for (ra, rb) in remainder_a.iter().zip(remainder_b.iter()) {
            let d = ra - rb;
            sum += d * d;
        }

        sum.sqrt()
    }

    /// Cosine distance: 1.0 - (dot(a, b) / (norm(a) * norm(b)))
    pub fn cosine_distance(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 1.0;
        }

        let mut dot = 0.0f32;
        let mut norm_a = 0.0f32;
        let mut norm_b = 0.0f32;

        let chunks_a = a.chunks_exact(4);
        let chunks_b = b.chunks_exact(4);
        let rem_a = chunks_a.remainder();
        let rem_b = chunks_b.remainder();

        for (ca, cb) in chunks_a.zip(chunks_b) {
            dot += ca[0] * cb[0] + ca[1] * cb[1] + ca[2] * cb[2] + ca[3] * cb[3];
            norm_a += ca[0] * ca[0] + ca[1] * ca[1] + ca[2] * ca[2] + ca[3] * ca[3];
            norm_b += cb[0] * cb[0] + cb[1] * cb[1] + cb[2] * cb[2] + cb[3] * cb[3];
        }

        for (&ra, &rb) in rem_a.iter().zip(rem_b.iter()) {
            dot += ra * rb;
            norm_a += ra * ra;
            norm_b += rb * rb;
        }

        let denominator = (norm_a.sqrt() * norm_b.sqrt()).max(1e-12);
        (1.0 - (dot / denominator)).clamp(0.0, 2.0)
    }

    /// Dot product of two vectors
    pub fn dot_product(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return 0.0;
        }
        let mut sum = 0.0f32;
        let chunks_a = a.chunks_exact(4);
        let chunks_b = b.chunks_exact(4);
        let rem_a = chunks_a.remainder();
        let rem_b = chunks_b.remainder();

        for (ca, cb) in chunks_a.zip(chunks_b) {
            sum += ca[0] * cb[0] + ca[1] * cb[1] + ca[2] * cb[2] + ca[3] * cb[3];
        }
        for (&ra, &rb) in rem_a.iter().zip(rem_b.iter()) {
            sum += ra * rb;
        }
        sum
    }

    /// Manhattan (L1) distance: sum(|a_i - b_i|)
    pub fn manhattan_l1(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }
        a.iter().zip(b.iter()).map(|(x, y)| (x - y).abs()).sum()
    }

    /// Chebyshev (L-infinity) distance: max(|a_i - b_i|)
    pub fn chebyshev(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() {
            return f32::MAX;
        }
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).abs())
            .fold(0.0f32, f32::max)
    }

    /// Pearson correlation coefficient between two vectors
    pub fn pearson_correlation(a: &[f32], b: &[f32]) -> f32 {
        if a.len() != b.len() || a.is_empty() {
            return 0.0;
        }

        let n = a.len() as f32;
        let mean_a = a.iter().sum::<f32>() / n;
        let mean_b = b.iter().sum::<f32>() / n;

        let mut numerator = 0.0f32;
        let mut var_a = 0.0f32;
        let mut var_b = 0.0f32;

        for (&x, &y) in a.iter().zip(b.iter()) {
            let da = x - mean_a;
            let db = y - mean_b;
            numerator += da * db;
            var_a += da * da;
            var_b += db * db;
        }

        let denom = (var_a.sqrt() * var_b.sqrt()).max(1e-12);
        (numerator / denom).clamp(-1.0, 1.0)
    }

    /// Vector angular distance in degrees [0.0, 180.0]
    pub fn angular_distance_degrees(a: &[f32], b: &[f32]) -> f32 {
        let cos_dist = Self::cosine_distance(a, b);
        let cos_sim = (1.0 - cos_dist).clamp(-1.0, 1.0);
        cos_sim.acos().to_degrees()
    }

    /// L2 normalize vector in-place
    pub fn l2_normalize(vec: &mut [f32]) {
        let norm: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 1e-12 {
            for x in vec.iter_mut() {
                *x /= norm;
            }
        }
    }

    /// Compute batch distance matrix between queries and database vectors
    pub fn batch_euclidean_matrix(queries: &[Vec<f32>], database: &[Vec<f32>]) -> Vec<Vec<f32>> {
        let mut matrix = Vec::with_capacity(queries.len());
        for q in queries {
            let mut row = Vec::with_capacity(database.len());
            for d in database {
                row.push(Self::euclidean_l2(q, d));
            }
            matrix.push(row);
        }
        matrix
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0; 512];
        let mut b = vec![0.0; 512];
        b[0] = 3.0;
        b[1] = 4.0;
        let d = DistanceMetrics::euclidean_l2(&a, &b);
        assert!((d - 5.0).abs() < 1e-5);
    }

    #[test]
    fn test_cosine_distance() {
        let a = vec![1.0, 0.0];
        let b = vec![1.0, 0.0];
        assert!(DistanceMetrics::cosine_distance(&a, &b) < 1e-5);

        let c = vec![0.0, 1.0];
        assert!((DistanceMetrics::cosine_distance(&a, &c) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_pearson_correlation() {
        let a = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let b = vec![2.0, 4.0, 6.0, 8.0, 10.0];
        let r = DistanceMetrics::pearson_correlation(&a, &b);
        assert!((r - 1.0).abs() < 1e-5);
    }

    #[test]
    fn test_chebyshev() {
        let a = vec![1.0, 5.0, 3.0];
        let b = vec![2.0, 1.0, 3.0];
        let d = DistanceMetrics::chebyshev(&a, &b);
        assert_eq!(d, 4.0);
    }
}
