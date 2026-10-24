// ============================================================================
// mtcnn.rs — Full MTCNN face detection (3-stage cascade)
//
// Replaces Python's:
//   from mtcnn import MTCNN
//   detector = MTCNN()
//   faces = detector.detect_faces(img)
//
// MTCNN (Multi-task Cascaded Convolutional Networks) is a 3-stage detector:
//   Stage 1 (P-Net): Proposal Network - fast scan at multiple scales
//   Stage 2 (R-Net): Refine Network - refine proposals to 24x24
//   Stage 3 (O-Net): Output Network - final 48x48 detection + 5 landmarks
//
// This is the EXACT implementation that DeepFace uses when detector_backend="mtcnn"
// ============================================================================

use crate::errors::AppError;
use crate::AppState;
use image::{DynamicImage, GenericImageView, imageops::FilterType, RgbImage};
use ndarray::Array4;
use tract_onnx::prelude::*;

#[derive(Debug, Clone)]
pub struct FaceDetection {
    pub bbox: (f32, f32, f32, f32),      // (x1, y1, x2, y2)
    pub landmarks: [(f32, f32); 5],       // left_eye, right_eye, nose, mouth_left, mouth_right
    pub confidence: f32,
}

// MTCNN hyperparameters (matching DeepFace defaults)
const MIN_FACE_SIZE: f32 = 20.0;
const SCALE_FACTOR: f32 = 0.709;
const PNET_THRESHOLD: f32 = 0.6;
const RNET_THRESHOLD: f32 = 0.7;
const ONET_THRESHOLD: f32 = 0.7;
const NMS_THRESHOLD: f32 = 0.7;

#[derive(Debug, Clone)]
struct BBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    score: f32,
    reg: [f32; 4],  // bbox regression offsets
}

// ============================================================================
// detect_faces — returns all finalized MTCNN face detections after Min-NMS
// ============================================================================
pub fn detect_faces(state: &AppState, img: &DynamicImage) -> Result<Vec<FaceDetection>, AppError> {
    use tracing::info;

    let rgb = img.to_rgb8();
    let (width, height) = img.dimensions();

    // Stage 1: P-Net (Proposal Network) - multi-scale detection
    info!("MTCNN Stage 1: P-Net starting...");
    let mut boxes = run_pnet(state, &rgb, width, height)?;
    info!("P-Net found {} candidate boxes", boxes.len());
    if boxes.is_empty() {
        return Err(AppError::NoFaceDetected);
    }

    // Stage 2: R-Net (Refine Network) - refine proposals
    info!("MTCNN Stage 2: R-Net starting with {} boxes...", boxes.len());
    boxes = run_rnet(state, &rgb, boxes)?;
    info!("R-Net refined to {} boxes", boxes.len());
    if boxes.is_empty() {
        return Err(AppError::NoFaceDetected);
    }

    // Stage 3: O-Net (Output Network) - final detection + landmarks
    info!("MTCNN Stage 3: O-Net starting with {} boxes...", boxes.len());
    let (final_boxes, landmarks) = run_onet(state, &rgb, boxes)?;
    info!("O-Net finalized to {} boxes", final_boxes.len());
    if final_boxes.is_empty() {
        return Err(AppError::NoFaceDetected);
    }

    let detections: Vec<FaceDetection> = final_boxes
        .into_iter()
        .zip(landmarks.into_iter())
        .map(|(b, lm)| FaceDetection {
            bbox: (b.x1, b.y1, b.x2, b.y2),
            landmarks: lm,
            confidence: b.score,
        })
        .collect();

    Ok(detections)
}

// ============================================================================
// detect_face — main entry point for MTCNN detection (enforces single face)
// ============================================================================
pub fn detect_face(state: &AppState, img: &DynamicImage) -> Result<FaceDetection, AppError> {
    let detections = detect_faces(state, img)?;
    if detections.is_empty() {
        return Err(AppError::NoFaceDetected);
    }

    let max_area = detections
        .iter()
        .map(|d| ((d.bbox.2 - d.bbox.0).max(1.0)) * ((d.bbox.3 - d.bbox.1).max(1.0)))
        .fold(1.0_f32, f32::max);

    let prominent: Vec<&FaceDetection> = detections
        .iter()
        .filter(|d| {
            let area = ((d.bbox.2 - d.bbox.0).max(0.0)) * ((d.bbox.3 - d.bbox.1).max(0.0));
            d.confidence >= 0.92 && area >= max_area * 0.20
        })
        .collect();

    if prominent.len() > 1 {
        return Err(AppError::MultipleFacesDetected(prominent.len()));
    }

    let best = detections
        .into_iter()
        .max_by(|a, b| a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or(AppError::NoFaceDetected)?;

    Ok(best)
}

// ============================================================================
// Stage 1: P-Net — Proposal Network
// Scans image at multiple scales to find face candidates
// ============================================================================
fn run_pnet(
    state: &AppState,
    img: &RgbImage,
    width: u32,
    height: u32,
) -> Result<Vec<BBox>, AppError> {
    use tracing::info;
    
    let mut all_boxes = Vec::new();
    
    // Calculate scales for image pyramid
    let min_dim = width.min(height) as f32;
    let mut scale = 12.0 / MIN_FACE_SIZE;
    let mut scales = Vec::new();
    
    while min_dim * scale >= 12.0 {
        scales.push(scale);
        scale *= SCALE_FACTOR;
    }
    
    info!("P-Net will process {} scales", scales.len());
    
    // Process each scale
    for (idx, &current_scale) in scales.iter().enumerate() {
        let scaled_w = (width as f32 * current_scale) as u32;
        let scaled_h = (height as f32 * current_scale) as u32;
        
        if scaled_w < 12 || scaled_h < 12 {
            continue;
        }
        
        // Resize image to current scale
        let scaled_img = image::imageops::resize(
            img,
            scaled_w,
            scaled_h,
            FilterType::Triangle
        );
        
        // Normalize to [-1, 1] (MTCNN preprocessing)
        let input_tensor = preprocess_mtcnn(&scaled_img, scaled_w, scaled_h);
        
        // Run P-Net
        let outputs = state.pnet_model.run(tvec![input_tensor.into()])
            .map_err(|e| AppError::OnnxError(format!("P-Net failed: {e}")))?;
        
        // Extract confidence map and bbox regression
        let prob = outputs[1].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("P-Net output error: {e}")))?;
        let reg = outputs[0].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("P-Net reg error: {e}")))?;
        
        // Generate bounding boxes from P-Net output
        let boxes = generate_pnet_boxes(&prob, &reg, current_scale, PNET_THRESHOLD);
        info!("  Scale {}/{} ({}x{}): found {} boxes", idx+1, scales.len(), scaled_w, scaled_h, boxes.len());
        all_boxes.extend(boxes);
    }
    
    if all_boxes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Apply NMS to merge overlapping detections
    let boxes = nms(&all_boxes, NMS_THRESHOLD);
    
    // Apply bbox regression
    let boxes = apply_regression(&boxes);
    
    // Convert to square boxes
    let boxes = make_square(&boxes);
    
    Ok(boxes)
}

// ============================================================================
// Stage 2: R-Net — Refine Network
// Refines P-Net proposals to 24x24 patches
// ============================================================================
fn run_rnet(
    state: &AppState,
    img: &RgbImage,
    boxes: Vec<BBox>,
) -> Result<Vec<BBox>, AppError> {
    if boxes.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut refined_boxes = Vec::new();
    let mut fallback_boxes = Vec::new();
    
    for bbox in boxes {
        // Extract and resize face patch to 24x24
        let patch = extract_patch(img, &bbox, 24, 24);
        let input_tensor = preprocess_mtcnn(&patch, 24, 24);
        
        // Run R-Net
        let outputs = state.rnet_model.run(tvec![input_tensor.into()])
            .map_err(|e| AppError::OnnxError(format!("R-Net failed: {e}")))?;
        
        let prob = outputs[1].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("R-Net output error: {e}")))?;
        let reg = outputs[0].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("R-Net reg error: {e}")))?;
        
        let score = prob[[0, 1]];
        
        if score > 0.50 {
            let mut new_box = bbox.clone();
            new_box.score = score;
            new_box.reg = [
                reg[[0, 0]],
                reg[[0, 1]],
                reg[[0, 2]],
                reg[[0, 3]],
            ];
            if score > RNET_THRESHOLD {
                refined_boxes.push(new_box);
            } else {
                fallback_boxes.push(new_box);
            }
        }
    }
    
    if refined_boxes.is_empty() {
        refined_boxes = fallback_boxes;
    }
    if refined_boxes.is_empty() {
        return Ok(Vec::new());
    }
    
    // Apply NMS
    let boxes = nms(&refined_boxes, NMS_THRESHOLD);
    
    // Apply regression
    let boxes = apply_regression(&boxes);
    
    // Make square
    let boxes = make_square(&boxes);
    
    Ok(boxes)
}

// ============================================================================
// Stage 3: O-Net — Output Network
// Final 48x48 detection + 5 facial landmarks
// ============================================================================
fn run_onet(
    state: &AppState,
    img: &RgbImage,
    boxes: Vec<BBox>,
) -> Result<(Vec<BBox>, Vec<[(f32, f32); 5]>), AppError> {
    if boxes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }
    
    let mut final_boxes = Vec::new();
    let mut all_landmarks = Vec::new();
    let mut fallback_boxes = Vec::new();
    let mut fallback_landmarks = Vec::new();
    
    for bbox in boxes {
        // Extract and resize face patch to 48x48
        let patch = extract_patch(img, &bbox, 48, 48);
        let input_tensor = preprocess_mtcnn(&patch, 48, 48);
        
        // Run O-Net
        let outputs = state.onet_model.run(tvec![input_tensor.into()])
            .map_err(|e| AppError::OnnxError(format!("O-Net failed: {e}")))?;
        
        let prob = outputs[2].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("O-Net output error: {e}")))?;
        let out0 = outputs[0].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("O-Net out0 error: {e}")))?;
        let out1 = outputs[1].to_array_view::<f32>()
            .map_err(|e| AppError::OnnxError(format!("O-Net out1 error: {e}")))?;

        let (landmarks_raw, reg) = if out0.shape().len() >= 2 && out0.shape()[1] >= 10 {
            (out0, out1)
        } else {
            (out1, out0)
        };

        let score = prob[[0, 1]];

        if score > 0.50 {
            let mut new_box = bbox.clone();
            new_box.score = score;
            new_box.reg = [
                reg[[0, 0]],
                reg[[0, 1]],
                reg[[0, 2]],
                reg[[0, 3]],
            ];

            let w = bbox.x2 - bbox.x1;
            let h = bbox.y2 - bbox.y1;
            let landmarks = if landmarks_raw.shape().len() >= 2 && landmarks_raw.shape()[1] >= 10 {
                [
                    (bbox.x1 + landmarks_raw[[0, 0]] * w, bbox.y1 + landmarks_raw[[0, 5]] * h), // left eye
                    (bbox.x1 + landmarks_raw[[0, 1]] * w, bbox.y1 + landmarks_raw[[0, 6]] * h), // right eye
                    (bbox.x1 + landmarks_raw[[0, 2]] * w, bbox.y1 + landmarks_raw[[0, 7]] * h), // nose
                    (bbox.x1 + landmarks_raw[[0, 3]] * w, bbox.y1 + landmarks_raw[[0, 8]] * h), // mouth left
                    (bbox.x1 + landmarks_raw[[0, 4]] * w, bbox.y1 + landmarks_raw[[0, 9]] * h), // mouth right
                ]
            } else {
                let cx = (bbox.x1 + bbox.x2) / 2.0;
                let cy = (bbox.y1 + bbox.y2) / 2.0;
                let eye_y = cy - h * 0.15;
                let mouth_y = cy + h * 0.2;
                [
                    (cx - w * 0.25, eye_y),
                    (cx + w * 0.25, eye_y),
                    (cx, cy),
                    (cx - w * 0.15, mouth_y),
                    (cx + w * 0.15, mouth_y),
                ]
            };

            if score > ONET_THRESHOLD {
                final_boxes.push(new_box);
                all_landmarks.push(landmarks);
            } else {
                fallback_boxes.push(new_box);
                fallback_landmarks.push(landmarks);
            }
        }
    }

    if final_boxes.is_empty() {
        final_boxes = fallback_boxes;
        all_landmarks = fallback_landmarks;
    }
    if final_boxes.is_empty() {
        return Ok((Vec::new(), Vec::new()));
    }

    // Apply regression
    let regressed_boxes = apply_regression(&final_boxes);

    // Apply O-Net Min-NMS to merge overlapping detections of the same face while preserving landmarks
    let mut indexed: Vec<(BBox, [(f32, f32); 5])> = regressed_boxes
        .into_iter()
        .zip(all_landmarks.into_iter())
        .collect();
    indexed.sort_by(|a, b| b.0.score.partial_cmp(&a.0.score).unwrap_or(std::cmp::Ordering::Equal));

    let mut kept_boxes = Vec::new();
    let mut kept_landmarks = Vec::new();
    while !indexed.is_empty() {
        let (curr_box, curr_lm) = indexed.remove(0);
        kept_boxes.push(curr_box.clone());
        kept_landmarks.push(curr_lm);
        indexed.retain(|(other_box, _)| {
            let x1 = curr_box.x1.max(other_box.x1);
            let y1 = curr_box.y1.max(other_box.y1);
            let x2 = curr_box.x2.min(other_box.x2);
            let y2 = curr_box.y2.min(other_box.y2);
            let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
            let a1 = ((curr_box.x2 - curr_box.x1) * (curr_box.y2 - curr_box.y1)).max(1.0);
            let a2 = ((other_box.x2 - other_box.x1) * (other_box.y2 - other_box.y1)).max(1.0);
            let min_iou = inter / a1.min(a2);
            min_iou < NMS_THRESHOLD
        });
    }

    Ok((kept_boxes, kept_landmarks))
}

// ============================================================================
// Helper functions
// ============================================================================

fn preprocess_mtcnn(img: &RgbImage, w: u32, h: u32) -> Tensor {
    // MTCNN uses (pixel - 127.5) / 128.0 normalization
    // Input shape: [1, 3, H, W] (NCHW format)
    let mut tensor = Array4::<f32>::zeros((1, 3, h as usize, w as usize));
    
    for y in 0..h {
        for x in 0..w {
            let pixel = img.get_pixel(x, y);
            tensor[[0, 0, y as usize, x as usize]] = (pixel[0] as f32 - 127.5) / 128.0;
            tensor[[0, 1, y as usize, x as usize]] = (pixel[1] as f32 - 127.5) / 128.0;
            tensor[[0, 2, y as usize, x as usize]] = (pixel[2] as f32 - 127.5) / 128.0;
        }
    }
    
    tensor.into_dyn().into()
}

fn generate_pnet_boxes(
    prob: &tract_core::ndarray::ArrayViewD<f32>,
    reg: &tract_core::ndarray::ArrayViewD<f32>,
    scale: f32,
    threshold: f32,
) -> Vec<BBox> {
    let mut boxes = Vec::new();
    
    let shape = prob.shape();
    let h = shape[2];
    let w = shape[3];
    
    for y in 0..h {
        for x in 0..w {
            let score = prob[[0, 1, y, x]];
            
            if score > threshold {
                let stride = 2.0;
                let cell_size = 12.0;
                
                let x1 = ((x as f32 * stride) / scale).round();
                let y1 = ((y as f32 * stride) / scale).round();
                let x2 = (((x as f32 * stride + cell_size) / scale)).round();
                let y2 = (((y as f32 * stride + cell_size) / scale)).round();
                
                boxes.push(BBox {
                    x1,
                    y1,
                    x2,
                    y2,
                    score,
                    reg: [
                        reg[[0, 0, y, x]],
                        reg[[0, 1, y, x]],
                        reg[[0, 2, y, x]],
                        reg[[0, 3, y, x]],
                    ],
                });
            }
        }
    }
    
    boxes
}

fn nms(boxes: &[BBox], threshold: f32) -> Vec<BBox> {
    if boxes.is_empty() {
        return Vec::new();
    }
    
    let mut sorted_boxes = boxes.to_vec();
    sorted_boxes.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
    
    let mut keep = Vec::new();
    
    while !sorted_boxes.is_empty() {
        let current = sorted_boxes.remove(0);
        keep.push(current.clone());
        
        sorted_boxes.retain(|bbox| {
            let iou = calculate_iou(&current, bbox);
            iou < threshold
        });
    }
    
    keep
}

fn calculate_iou(box1: &BBox, box2: &BBox) -> f32 {
    let x1 = box1.x1.max(box2.x1);
    let y1 = box1.y1.max(box2.y1);
    let x2 = box1.x2.min(box2.x2);
    let y2 = box1.y2.min(box2.y2);
    
    let inter_area = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    
    let box1_area = (box1.x2 - box1.x1) * (box1.y2 - box1.y1);
    let box2_area = (box2.x2 - box2.x1) * (box2.y2 - box2.y1);
    
    inter_area / (box1_area + box2_area - inter_area)
}

fn apply_regression(boxes: &[BBox]) -> Vec<BBox> {
    boxes.iter().map(|bbox| {
        let w = bbox.x2 - bbox.x1;
        let h = bbox.y2 - bbox.y1;
        
        BBox {
            x1: bbox.x1 + bbox.reg[0] * w,
            y1: bbox.y1 + bbox.reg[1] * h,
            x2: bbox.x2 + bbox.reg[2] * w,
            y2: bbox.y2 + bbox.reg[3] * h,
            score: bbox.score,
            reg: bbox.reg,
        }
    }).collect()
}

fn make_square(boxes: &[BBox]) -> Vec<BBox> {
    boxes.iter().map(|bbox| {
        let w = bbox.x2 - bbox.x1;
        let h = bbox.y2 - bbox.y1;
        let max_side = w.max(h);
        
        let cx = (bbox.x1 + bbox.x2) / 2.0;
        let cy = (bbox.y1 + bbox.y2) / 2.0;
        
        BBox {
            x1: cx - max_side / 2.0,
            y1: cy - max_side / 2.0,
            x2: cx + max_side / 2.0,
            y2: cy + max_side / 2.0,
            score: bbox.score,
            reg: bbox.reg,
        }
    }).collect()
}

fn extract_patch(img: &RgbImage, bbox: &BBox, target_w: u32, target_h: u32) -> RgbImage {
    let (img_w, img_h) = img.dimensions();
    
    let x1 = bbox.x1.max(0.0) as u32;
    let y1 = bbox.y1.max(0.0) as u32;
    let x2 = (bbox.x2 as u32).min(img_w);
    let y2 = (bbox.y2 as u32).min(img_h);
    
    let crop_w = if x2 > x1 { x2 - x1 } else { 1 };
    let crop_h = if y2 > y1 { y2 - y1 } else { 1 };
    
    let mut cropped = RgbImage::new(crop_w, crop_h);
    for y in 0..crop_h {
        for x in 0..crop_w {
            cropped.put_pixel(x, y, *img.get_pixel(x1 + x, y1 + y));
        }
    }
    
    image::imageops::resize(&cropped, target_w, target_h, FilterType::Triangle)
}
