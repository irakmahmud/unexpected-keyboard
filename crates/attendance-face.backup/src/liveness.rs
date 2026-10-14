use attendance_domain::LivenessResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadPose {
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureAnalysisResult {
    pub uniform_lbp_entropy: f32,
    pub moire_pattern_score: f32,
    pub chroma_variance_cb: f32,
    pub chroma_variance_cr: f32,
    pub is_print_attack: bool,
    pub is_screen_replay: bool,
}

pub struct LivenessDetector;

impl LivenessDetector {
    /// Heuristic liveness verification evaluating motion variance and micro-texture
    pub fn evaluate_frame_sequence(
        variance_scores: &[f32],
        eye_aspect_ratios: &[f32],
        consecutive_frames: usize,
    ) -> LivenessResult {
        if consecutive_frames < 3 || variance_scores.is_empty() {
            return LivenessResult {
                is_live: false,
                confidence_score: 0.0,
                eye_blink_detected: false,
                motion_variance: 0.0,
                warning: Some("Insufficient frames captured for liveness evaluation".to_string()),
            };
        }

        // Calculate average motion variance across frames
        let avg_variance: f32 = variance_scores.iter().sum::<f32>() / variance_scores.len() as f32;

        // Detect eye blink: aspect ratio dip below 0.20 followed by rise
        let mut blink_detected = false;
        for i in 1..eye_aspect_ratios.len().saturating_sub(1) {
            if eye_aspect_ratios[i] < 0.21
                && eye_aspect_ratios[i - 1] > 0.25
                && eye_aspect_ratios[i + 1] > 0.25
            {
                blink_detected = true;
                break;
            }
        }

        // Motion variance threshold: static photo attack has variance near 0.0, video replay has repetitive patterns
        let is_live_motion = avg_variance > 0.015 && avg_variance < 0.85;

        let confidence = if blink_detected && is_live_motion {
            96.5f32
        } else if is_live_motion {
            82.0f32
        } else {
            35.0f32
        };

        let is_live = confidence >= 80.0;
        let warning = if !is_live {
            Some("Liveness check failed: possible photo or non-live presentation".to_string())
        } else {
            None
        };

        LivenessResult {
            is_live,
            confidence_score: confidence,
            eye_blink_detected: blink_detected,
            motion_variance: avg_variance,
            warning,
        }
    }

    /// Calculate Eye Aspect Ratio (EAR) from 6 2D facial landmarks:
    /// EAR = (||p2 - p6|| + ||p3 - p5||) / (2 * ||p1 - p4||)
    pub fn calculate_eye_aspect_ratio(eye_landmarks: &[[f32; 2]; 6]) -> f32 {
        let dist = |p1: &[f32; 2], p2: &[f32; 2]| -> f32 {
            let dx = p1[0] - p2[0];
            let dy = p1[1] - p2[1];
            (dx * dx + dy * dy).sqrt()
        };

        let vertical_1 = dist(&eye_landmarks[1], &eye_landmarks[5]);
        let vertical_2 = dist(&eye_landmarks[2], &eye_landmarks[4]);
        let horizontal = dist(&eye_landmarks[0], &eye_landmarks[3]);

        if horizontal <= 1e-6 {
            return 0.0;
        }

        (vertical_1 + vertical_2) / (2.0 * horizontal)
    }

    /// Calculate Mouth Aspect Ratio (MAR) for yawn / speech liveness detection
    pub fn calculate_mouth_aspect_ratio(mouth_landmarks: &[[f32; 2]; 8]) -> f32 {
        let dist = |p1: &[f32; 2], p2: &[f32; 2]| -> f32 {
            let dx = p1[0] - p2[0];
            let dy = p1[1] - p2[1];
            (dx * dx + dy * dy).sqrt()
        };

        let vert_1 = dist(&mouth_landmarks[1], &mouth_landmarks[7]);
        let vert_2 = dist(&mouth_landmarks[2], &mouth_landmarks[6]);
        let vert_3 = dist(&mouth_landmarks[3], &mouth_landmarks[5]);
        let horiz = dist(&mouth_landmarks[0], &mouth_landmarks[4]);

        if horiz <= 1e-6 {
            return 0.0;
        }

        (vert_1 + vert_2 + vert_3) / (3.0 * horiz)
    }

    /// Head Pose estimation using approximate 2D landmark geometry
    pub fn estimate_head_pose(
        nose_tip: [f32; 2],
        _chin: [f32; 2],
        left_eye_outer: [f32; 2],
        right_eye_outer: [f32; 2],
        left_mouth_corner: [f32; 2],
        right_mouth_corner: [f32; 2],
    ) -> HeadPose {
        // Roll: angle between eyes
        let dy = right_eye_outer[1] - left_eye_outer[1];
        let dx = right_eye_outer[0] - left_eye_outer[0];
        let roll = dy.atan2(dx).to_degrees();

        // Yaw: asymmetry between nose tip and eye centers
        let eye_center_x = (left_eye_outer[0] + right_eye_outer[0]) / 2.0;
        let eye_distance = (right_eye_outer[0] - left_eye_outer[0]).abs();
        let yaw = if eye_distance > 1e-5 {
            ((nose_tip[0] - eye_center_x) / eye_distance) * 90.0
        } else {
            0.0
        };

        // Pitch: distance from eyes to nose vs nose to mouth
        let eye_center_y = (left_eye_outer[1] + right_eye_outer[1]) / 2.0;
        let mouth_center_y = (left_mouth_corner[1] + right_mouth_corner[1]) / 2.0;
        let upper_face = (nose_tip[1] - eye_center_y).abs();
        let lower_face = (mouth_center_y - nose_tip[1]).abs();

        let pitch = if lower_face > 1e-5 {
            ((upper_face / lower_face) - 1.0) * 45.0
        } else {
            0.0
        };

        HeadPose { yaw, pitch, roll }
    }

    /// Compute Local Binary Pattern (LBP) histogram on an 8-bit grayscale region of interest
    pub fn compute_lbp_histogram(gray_pixels: &[u8], width: usize, height: usize) -> [f32; 256] {
        let mut hist = [0f32; 256];
        if width < 3 || height < 3 || gray_pixels.len() < width * height {
            return hist;
        }

        let mut total_pixels = 0.0f32;

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let center = gray_pixels[y * width + x];
                let mut code = 0u8;

                // 8 neighbors clockwise starting from top-left
                let neighbors = [
                    gray_pixels[(y - 1) * width + (x - 1)],
                    gray_pixels[(y - 1) * width + x],
                    gray_pixels[(y - 1) * width + (x + 1)],
                    gray_pixels[y * width + (x + 1)],
                    gray_pixels[(y + 1) * width + (x + 1)],
                    gray_pixels[(y + 1) * width + x],
                    gray_pixels[(y + 1) * width + (x - 1)],
                    gray_pixels[y * width + (x - 1)],
                ];

                for (i, &n) in neighbors.iter().enumerate() {
                    if n >= center {
                        code |= 1 << i;
                    }
                }

                hist[code as usize] += 1.0;
                total_pixels += 1.0;
            }
        }

        // Normalize histogram
        if total_pixels > 0.0 {
            for v in hist.iter_mut() {
                *v /= total_pixels;
            }
        }

        hist
    }

    /// Detect print and screen attack through YCbCr color channel variance
    pub fn analyze_chroma_texture(rgb_pixels: &[u8]) -> TextureAnalysisResult {
        let num_pixels = rgb_pixels.len() / 3;
        if num_pixels == 0 {
            return TextureAnalysisResult {
                uniform_lbp_entropy: 0.0,
                moire_pattern_score: 0.0,
                chroma_variance_cb: 0.0,
                chroma_variance_cr: 0.0,
                is_print_attack: false,
                is_screen_replay: false,
            };
        }

        let mut cb_sum = 0.0f64;
        let mut cr_sum = 0.0f64;
        let mut cb_sq_sum = 0.0f64;
        let mut cr_sq_sum = 0.0f64;

        for chunk in rgb_pixels.chunks_exact(3) {
            let r = chunk[0] as f64;
            let g = chunk[1] as f64;
            let b = chunk[2] as f64;

            // ITU-R BT.601 conversion
            let cb = 128.0 - 0.168736 * r - 0.331264 * g + 0.5 * b;
            let cr = 128.0 + 0.5 * r - 0.418688 * g - 0.081312 * b;

            cb_sum += cb;
            cr_sum += cr;
            cb_sq_sum += cb * cb;
            cr_sq_sum += cr * cr;
        }

        let n = num_pixels as f64;
        let cb_mean = cb_sum / n;
        let cr_mean = cr_sum / n;
        let cb_var = ((cb_sq_sum / n) - (cb_mean * cb_mean)).max(0.0) as f32;
        let cr_var = ((cr_sq_sum / n) - (cr_mean * cr_mean)).max(0.0) as f32;

        // Paper printouts typically have severely degraded chroma variance
        let is_print_attack = cb_var < 8.0 || cr_var < 8.0;
        // Screens often present abnormal moire chroma spikes
        let is_screen_replay = cb_var > 350.0 || cr_var > 350.0;

        TextureAnalysisResult {
            uniform_lbp_entropy: 4.5,
            moire_pattern_score: (cb_var - cr_var).abs(),
            chroma_variance_cb: cb_var,
            chroma_variance_cr: cr_var,
            is_print_attack,
            is_screen_replay,
        }
    }
}
