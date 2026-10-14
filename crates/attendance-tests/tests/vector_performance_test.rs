use attendance_face::{
    DistanceMetrics, FaceClustering, FaceQualityAssessor, HnswConfig, PureRustHnswIndex,
};
use attendance_tests::TestFixtures;
use std::time::Instant;

#[test]
fn test_vector_euclidean_l2_throughput_benchmark() {
    let v1 = TestFixtures::generate_synthetic_embedding(100);
    let v2 = TestFixtures::generate_synthetic_embedding(200);

    let iterations = 100_000;
    let start = Instant::now();

    let mut sum = 0.0f32;
    for _ in 0..iterations {
        sum += DistanceMetrics::euclidean_l2(&v1, &v2);
    }

    let elapsed = start.elapsed();
    let nanos_per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!(
        "Euclidean L2 (512-D) Throughput: {:.2} ns/op (Total: {:?}, Sum: {:.2})",
        nanos_per_op, elapsed, sum
    );

    // Unrolled SIMD loop should comfortably execute 512-D L2 in under 50 microseconds even in debug mode
    assert!(
        nanos_per_op < 50_000.0,
        "512-D L2 calculation must execute in under 50 microseconds per pair in debug mode"
    );
}

#[test]
fn test_vector_cosine_distance_throughput_benchmark() {
    let v1 = TestFixtures::generate_synthetic_embedding(300);
    let v2 = TestFixtures::generate_synthetic_embedding(400);

    let iterations = 100_000;
    let start = Instant::now();

    let mut sum = 0.0f32;
    for _ in 0..iterations {
        sum += DistanceMetrics::cosine_distance(&v1, &v2);
    }

    let elapsed = start.elapsed();
    let nanos_per_op = elapsed.as_nanos() as f64 / iterations as f64;

    println!(
        "Cosine Distance (512-D) Throughput: {:.2} ns/op (Total: {:?}, Sum: {:.2})",
        nanos_per_op, elapsed, sum
    );

    assert!(
        nanos_per_op < 50_000.0,
        "512-D Cosine distance must execute in under 50 microseconds per pair in debug mode"
    );
}

#[test]
fn test_hnsw_512d_nearest_neighbor_search_latency() {
    let config = HnswConfig {
        max_elements: 1000,
        m: 16,
        m0: 32,
        ef_construction: 64,
        ef_search: 32,
        ml: 1.0 / (16.0f64).ln(),
    };
    let mut index = PureRustHnswIndex::new(config);

    // Populate index with 100 512-D face vectors
    for i in 1..=100 {
        let vec = TestFixtures::generate_synthetic_embedding(i * 10);
        index.insert(format!("EMP_{:04}", i), vec);
    }

    assert_eq!(index.len(), 100);

    // Perform 1,000 k-NN queries
    let query_vec = TestFixtures::generate_synthetic_embedding(500);
    let queries = 1_000;
    let start = Instant::now();

    for _ in 0..queries {
        let results = index.search_knn(&query_vec, 5);
        assert!(!results.is_empty());
    }

    let elapsed = start.elapsed();
    let micros_per_query = elapsed.as_micros() as f64 / queries as f64;

    println!(
        "HNSW 512-D (100 nodes, k=5) Search Latency: {:.2} µs/query (Total: {:?})",
        micros_per_query, elapsed
    );

    // HNSW graph traversal on 100 elements should take under 5,000 microseconds (5 milliseconds) per query in debug mode
    assert!(
        micros_per_query < 5000.0,
        "HNSW 512-D search must execute within sub-5 millisecond latency"
    );
}

#[test]
fn test_face_quality_laplacian_variance_speed() {
    // 100x100 grayscale face crop (10,000 pixels)
    let gray_image = vec![120u8; 100 * 100];

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let _ = FaceQualityAssessor::laplacian_variance(&gray_image, 100, 100);
    }

    let elapsed = start.elapsed();
    let micros_per_frame = elapsed.as_micros() as f64 / iterations as f64;

    println!(
        "Laplacian Blur Quality (100x100) Speed: {:.2} µs/frame",
        micros_per_frame
    );

    // Should easily process > 200 frames per second even in debug mode (under 5,000 µs per frame)
    assert!(
        micros_per_frame < 5000.0,
        "Quality blur evaluation must execute in under 5000 microseconds per crop in debug mode"
    );
}

#[test]
fn test_multi_frame_centroid_fusion_speed() {
    // 10 video frames of 512-D embeddings
    let mut frames = Vec::new();
    for i in 1..=10 {
        frames.push(TestFixtures::generate_synthetic_embedding(i * 15));
    }

    let iterations = 10_000;
    let start = Instant::now();

    for _ in 0..iterations {
        let centroid = FaceClustering::compute_centroid(&frames);
        assert!(centroid.is_some());
    }

    let elapsed = start.elapsed();
    let micros_per_fusion = elapsed.as_micros() as f64 / iterations as f64;

    println!(
        "Centroid Fusion (10 frames x 512-D) Speed: {:.2} µs/fusion",
        micros_per_fusion
    );

    assert!(
        micros_per_fusion < 200.0,
        "10-frame centroid fusion must complete in under 200 microseconds"
    );
}
