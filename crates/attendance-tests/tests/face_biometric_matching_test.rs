use attendance_face::{
    DistanceMetrics, FaceClustering, FaceQualityAssessor, HnswConfig, LivenessDetector,
    PureRustHnswIndex,
};
use attendance_tests::TestFixtures;

#[test]
fn test_hnsw_512d_vector_index_search() {
    let config = HnswConfig::default();
    let mut index = PureRustHnswIndex::new(config);

    // Insert 10 synthetic 512-D embeddings
    for i in 1..=10 {
        let vec = TestFixtures::generate_synthetic_embedding(i * 100);
        index.insert(format!("EMP_{:03}", i), vec);
    }

    assert_eq!(index.len(), 10);

    // Query with vector identical to EMP_005
    let query_vec = TestFixtures::generate_synthetic_embedding(500);
    let results = index.search_knn(&query_vec, 3);

    assert!(!results.is_empty(), "Search should return neighbors");
    assert_eq!(results[0].0, "EMP_005");
    assert!(results[0].1 < 0.05, "Exact query match should have distance near 0");
}

#[test]
fn test_vector_cosine_and_l2_metrics() {
    let v1 = TestFixtures::generate_synthetic_embedding(42);
    let v2 = TestFixtures::generate_synthetic_embedding(42); // Identical
    let v3 = TestFixtures::generate_synthetic_embedding(999); // Different

    // Identical vectors
    let cos_ident = DistanceMetrics::cosine_distance(&v1, &v2);
    let l2_ident = DistanceMetrics::euclidean_l2(&v1, &v2);

    assert!(cos_ident < 1e-4, "Self-distance (cosine) must be ~0.0");
    assert!(l2_ident < 1e-4, "Self-distance (L2) must be ~0.0");

    // Different vectors
    let cos_diff = DistanceMetrics::cosine_distance(&v1, &v3);
    let l2_diff = DistanceMetrics::euclidean_l2(&v1, &v3);

    assert!(cos_diff > 0.1, "Different embeddings should have higher cosine distance");
    assert!(l2_diff > 0.5, "Different embeddings should have non-zero L2 distance");
}

#[test]
fn test_centroid_fusion_and_clustering() {
    // Generate sample embeddings near base
    let base_a = TestFixtures::generate_synthetic_embedding(10);
    let mut samples = Vec::new();

    for _ in 0..4 {
        let mut sample = base_a.clone();
        sample[0] += 0.001;
        samples.push(sample);
    }

    let centroid = FaceClustering::compute_centroid(&samples).expect("Centroid calculation failed");
    assert_eq!(centroid.len(), 512);

    let variance = FaceClustering::intra_cluster_variance(&samples, &centroid);
    assert!(variance < 0.05, "Tight cluster must have low intra-cluster variance");

    let norm: f32 = centroid.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 1e-3, "Fused centroid must be unit normalized");
}

#[test]
fn test_face_quality_laplacian_sharpness() {
    // Simulated sharp image (high variance checkerboard pattern)
    let mut sharp_gray = vec![0u8; 100 * 100];
    for i in 0..sharp_gray.len() {
        sharp_gray[i] = if (i % 2) == 0 { 200 } else { 50 };
    }

    let sharp_score = FaceQualityAssessor::laplacian_variance(&sharp_gray, 100, 100);
    assert!(sharp_score > 100.0, "Checkerboard pattern must have high sharpness variance");

    // Simulated blurred/flat image (low variance)
    let blurred_gray = vec![128u8; 100 * 100];
    let blur_score = FaceQualityAssessor::laplacian_variance(&blurred_gray, 100, 100);
    assert_eq!(blur_score, 0.0, "Completely uniform image must have zero variance");
}

#[test]
fn test_liveness_detector_sequence() {
    // 5 consecutive frames with adequate normalized motion and an eye-blink dip
    let motion_variances = vec![0.05, 0.08, 0.06, 0.07, 0.05];
    let eye_aspect_ratios = vec![0.30, 0.28, 0.16, 0.29, 0.31]; // Blink dip at frame 3

    let res = LivenessDetector::evaluate_frame_sequence(&motion_variances, &eye_aspect_ratios, 5);
    assert!(res.is_live, "Sequence with motion and blink must pass liveness");
    assert!(res.eye_blink_detected, "Eye blink must be detected in sequence");
}

#[test]
fn test_facenet512_verified_behavioral_equivalence() {
    use attendance_face::{FaceNet512Embedding, FaceNet512Verifier, FACENET512_DIMENSION, FACENET512_MATCH_THRESHOLD};

    assert_eq!(FACENET512_DIMENSION, 512, "FaceNet512 must have 512 dimensions");
    assert_eq!(FACENET512_MATCH_THRESHOLD, 0.40, "FaceNet512 threshold must be 0.40 matching python");

    let v1 = TestFixtures::generate_synthetic_embedding(101);
    let emb1 = FaceNet512Embedding::new(v1).expect("emb1");

    let v2 = TestFixtures::generate_synthetic_embedding(101);
    let emb2 = FaceNet512Embedding::new(v2).expect("emb2");

    let v3 = TestFixtures::generate_synthetic_embedding(999);
    let emb3 = FaceNet512Embedding::new(v3).expect("emb3");

    let verifier = FaceNet512Verifier::default();
    let (match1, dist1, conf1) = verifier.verify_1_to_1(&emb1, &emb2);
    assert!(match1, "Identical FaceNet512 embeddings must match");
    assert!(dist1 < 1e-4, "Distance must be zero");
    assert_eq!(conf1, 1.0, "Confidence must be 1.0");

    let (match2, dist2, _) = verifier.verify_1_to_1(&emb1, &emb3);
    assert!(!match2, "Different synthetic embeddings must not match within 0.40 threshold");
    assert!(dist2 >= 0.40);
}
