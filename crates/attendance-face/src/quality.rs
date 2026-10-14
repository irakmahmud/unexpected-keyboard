use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceQualityScore {
    pub overall_score: f32,
    pub blur_score: f32,
    pub brightness_score: f32,
    pub contrast_score: f32,
    pub resolution_score: f32,
    pub passes_quality_gate: bool,
    pub failure_reasons: Vec<String>,
}

pub struct FaceQualityAssessor;

impl FaceQualityAssessor {
    /// Calculate blur score using variance of Laplacian on 8-bit grayscale image
    pub fn laplacian_variance(gray_pixels: &[u8], width: usize, height: usize) -> f32 {
        if width < 3 || height < 3 || gray_pixels.len() < width * height {
            return 0.0;
        }

        // Discrete 3x3 Laplacian kernel:
        // [ 0,  1,  0]
        // [ 1, -4,  1]
        // [ 0,  1,  0]
        let mut laplacian_values = Vec::with_capacity((width - 2) * (height - 2));

        for y in 1..height - 1 {
            for x in 1..width - 1 {
                let center = gray_pixels[y * width + x] as i32;
                let top = gray_pixels[(y - 1) * width + x] as i32;
                let bottom = gray_pixels[(y + 1) * width + x] as i32;
                let left = gray_pixels[y * width + (x - 1)] as i32;
                let right = gray_pixels[y * width + (x + 1)] as i32;

                let val = top + bottom + left + right - 4 * center;
                laplacian_values.push(val as f64);
            }
        }

        let n = laplacian_values.len() as f64;
        if n == 0.0 {
            return 0.0;
        }

        let mean = laplacian_values.iter().sum::<f64>() / n;
        let variance = laplacian_values
            .iter()
            .map(|&v| {
                let diff = v - mean;
                diff * diff
            })
            .sum::<f64>()
            / n;

        variance as f32
    }

    /// Calculate brightness and contrast
    pub fn brightness_and_contrast(gray_pixels: &[u8]) -> (f32, f32) {
        if gray_pixels.is_empty() {
            return (0.0, 0.0);
        }

        let n = gray_pixels.len() as f64;
        let sum: f64 = gray_pixels.iter().map(|&p| p as f64).sum();
        let mean = sum / n;

        let var: f64 = gray_pixels
            .iter()
            .map(|&p| {
                let d = p as f64 - mean;
                d * d
            })
            .sum::<f64>()
            / n;

        let std_dev = var.sqrt();
        (mean as f32, std_dev as f32)
    }

    /// Comprehensive face capture quality gate
    pub fn assess_face_quality(
        gray_pixels: &[u8],
        face_width: usize,
        face_height: usize,
    ) -> FaceQualityScore {
        let mut failure_reasons = Vec::new();

        // 1. Resolution check: minimum 80x80 face crop
        let resolution_score = if face_width >= 112 && face_height >= 112 {
            1.0f32
        } else if face_width >= 80 && face_height >= 80 {
            0.8f32
        } else {
            failure_reasons.push(format!(
                "Face resolution too small: {}x{} (min 80x80)",
                face_width, face_height
            ));
            0.3f32
        };

        // 2. Blur / sharpness check
        let lap_var = Self::laplacian_variance(gray_pixels, face_width, face_height);
        let blur_score = if lap_var >= 100.0 {
            1.0f32
        } else if lap_var >= 50.0 {
            (lap_var / 100.0).clamp(0.5, 0.99)
        } else {
            failure_reasons.push(format!("Image is blurry (sharpness score {:.1} < 50.0)", lap_var));
            0.2f32
        };

        // 3. Brightness & contrast
        let (mean_brightness, std_contrast) = Self::brightness_and_contrast(gray_pixels);

        let brightness_score = if mean_brightness < 40.0 {
            failure_reasons.push(format!("Image is underexposed / too dark ({:.1} < 40)", mean_brightness));
            0.2f32
        } else if mean_brightness > 220.0 {
            failure_reasons.push(format!("Image is overexposed / too bright ({:.1} > 220)", mean_brightness));
            0.3f32
        } else {
            1.0f32
        };

        let contrast_score = if std_contrast < 20.0 {
            failure_reasons.push(format!("Low image contrast ({:.1} < 20)", std_contrast));
            0.4f32
        } else {
            1.0f32
        };

        let overall = (resolution_score * 0.25)
            + (blur_score * 0.35)
            + (brightness_score * 0.20)
            + (contrast_score * 0.20);

        let passes_quality_gate = failure_reasons.is_empty() && overall >= 0.70;

        FaceQualityScore {
            overall_score: overall,
            blur_score,
            brightness_score,
            contrast_score,
            resolution_score,
            passes_quality_gate,
            failure_reasons,
        }
    }
}
