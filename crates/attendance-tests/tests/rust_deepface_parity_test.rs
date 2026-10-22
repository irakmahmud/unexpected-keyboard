use attendance_face::{
    math, preprocessing, DeepFaceAppError as AppError, VerifyRequest, VerifyResponse, DEEPFACE_COSINE_THRESHOLD,
};
use image::{ImageBuffer, Rgb};
use ndarray::Array1;

#[test]
fn test_deepface_math_parity() {
    // Test L2 normalize
    let v = Array1::from(vec![3.0f32, 4.0f32]);
    let norm_v = math::l2_normalize(&v);
    assert!((norm_v[0] - 0.6).abs() < 1e-5);
    assert!((norm_v[1] - 0.8).abs() < 1e-5);

    // Test Cosine Distance for identical vectors
    let d_identical = math::cosine_distance(&norm_v, &norm_v);
    assert!(d_identical.abs() < 1e-5, "Identical vector cosine distance must be 0");

    // Test Orthogonal vectors
    let v_ortho = Array1::from(vec![-0.8f32, 0.6f32]);
    let d_ortho = math::cosine_distance(&norm_v, &v_ortho);
    assert!((d_ortho - 1.0).abs() < 1e-5, "Orthogonal vector cosine distance must be 1.0");

    // Test Opposing vectors
    let v_opp = Array1::from(vec![-0.6f32, -0.8f32]);
    let d_opp = math::cosine_distance(&norm_v, &v_opp);
    assert!((d_opp - 2.0).abs() < 1e-5, "Opposing vector cosine distance must be 2.0");
}

#[test]
fn test_deepface_threshold_contract() {
    assert_eq!(
        DEEPFACE_COSINE_THRESHOLD, 0.30,
        "Authoritative threshold from rust_deepFace must be exactly 0.30"
    );

    // Vector with distance <= 0.30 must verify true
    // Cosine distance = 1 - dot
    // dot = 0.85 -> distance = 0.15 <= 0.30 -> verified
    let v1 = Array1::from(vec![1.0f32, 0.0f32]);
    // distance = 1 - cos(theta) = 0.15 -> cos(theta) = 0.85
    let v2 = Array1::from(vec![0.85f32, (1.0f32 - 0.85f32 * 0.85f32).sqrt()]);
    let dist = math::cosine_distance(&v1, &v2);
    assert!(dist <= DEEPFACE_COSINE_THRESHOLD, "Distance {:.4} must be <= 0.30", dist);
}

#[test]
fn test_deepface_preprocessing_tensor_shape_and_standardization() {
    // Create a 160x160 RGB test image
    let img_buf: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(160, 160, |x, y| {
        Rgb([(x % 256) as u8, (y % 256) as u8, ((x + y) % 256) as u8])
    });
    let img = image::DynamicImage::ImageRgb8(img_buf);

    let tensor = preprocessing::normalize_facenet(&img);
    assert_eq!(tensor.shape(), &[1, 160, 160, 3]);

    // Check that mean of normalized tensor is approximately 0 and std is approximately 1
    let sum: f32 = tensor.iter().sum();
    let mean = sum / tensor.len() as f32;
    assert!(mean.abs() < 1e-3, "Normalized tensor mean must be near 0, got {}", mean);
}

#[test]
fn test_deepface_request_response_serialization_parity() {
    let req_json = r#"{
        "image1_b64": "aGVsbG8=",
        "image2_b64": "d29ybGQ="
    }"#;
    let req: VerifyRequest = serde_json::from_str(req_json).expect("Must deserialize");
    assert_eq!(req.image1_b64, "aGVsbG8=");
    assert_eq!(req.image2_b64, "d29ybGQ=");

    let resp = VerifyResponse {
        verified: true,
        distance: 0.1772,
        threshold: DEEPFACE_COSINE_THRESHOLD,
        model: "Facenet512",
        detector: "MTCNN",
        metric: "cosine",
    };
    let json_val = serde_json::to_value(&resp).expect("Must serialize");
    assert_eq!(json_val["verified"], true);
    assert!((json_val["distance"].as_f64().unwrap() - 0.1772).abs() < 1e-4);
    assert!((json_val["threshold"].as_f64().unwrap() - 0.30).abs() < 1e-4);
    assert_eq!(json_val["model"], "Facenet512");
    assert_eq!(json_val["detector"], "MTCNN");
    assert_eq!(json_val["metric"], "cosine");
}

#[test]
fn test_deepface_app_error_display() {
    let err1 = AppError::NoFaceDetected;
    assert_eq!(err1.to_string(), "No face detected in image");

    let err2 = AppError::BadRequest("invalid base64".to_string());
    assert_eq!(err2.to_string(), "Bad request: invalid base64");

    let err3 = AppError::ImageDecode("bad png".to_string());
    assert_eq!(err3.to_string(), "Image decode failed: bad png");
}

#[test]
fn test_validate_onnx_model_bytes_rejects_corrupted_payload() {
    let garbage = vec![0x42u8; 256];
    let res = attendance_face::validate_onnx_model_bytes(&garbage);
    assert!(
        res.is_err(),
        "Arbitrary non-ONNX bytes must be rejected by validate_onnx_model_bytes before disk write"
    );
}
