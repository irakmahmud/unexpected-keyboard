use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaceQualityScore {
    pub overall_score: f32,
    pub blur_score: f32,
    pub brightness_score: f32,
    pub contrast_score: f32,
    pub resolution_score: f32,
    pub sharpness: f32,
    pub brightness: f32,
    pub contrast: f32,
    pub is_blurry: bool,
    pub is_dark: bool,
    pub is_unclear: bool,
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

        // 1. Resolution check
        let mut is_unclear = false;
        let resolution_score = if face_width >= 112 && face_height >= 112 {
            1.0f32
        } else if face_width >= 80 && face_height >= 80 {
            0.85f32
        } else if face_width >= 36 && face_height >= 36 {
            0.75f32
        } else {
            is_unclear = true;
            failure_reasons.push(format!(
                "Face resolution too small: {}x{} (min 36x36)",
                face_width, face_height
            ));
            0.3f32
        };

        // 2. Blur / sharpness check
        let lap_var = Self::laplacian_variance(gray_pixels, face_width, face_height);
        let is_blurry = lap_var < 35.0;
        let blur_score = if lap_var >= 100.0 {
            1.0f32
        } else if !is_blurry {
            (lap_var / 100.0).clamp(0.55, 0.99)
        } else {
            failure_reasons.push(format!("Image is blurry (sharpness score {:.1} < 35.0)", lap_var));
            0.2f32
        };

        // 3. Brightness & contrast
        let (mean_brightness, std_contrast) = Self::brightness_and_contrast(gray_pixels);
        let is_dark = mean_brightness < 38.0;

        let brightness_score = if is_dark {
            failure_reasons.push(format!("Image is underexposed / too dark ({:.1} < 38.0)", mean_brightness));
            0.2f32
        } else if mean_brightness > 238.0 {
            is_unclear = true;
            failure_reasons.push(format!("Image is overexposed / washed out ({:.1} > 238.0)", mean_brightness));
            0.3f32
        } else {
            1.0f32
        };

        let contrast_score = if std_contrast < 14.0 {
            is_unclear = true;
            failure_reasons.push(format!("Low image contrast / face not clear ({:.1} < 14.0)", std_contrast));
            0.35f32
        } else {
            1.0f32
        };

        let overall = (resolution_score * 0.25)
            + (blur_score * 0.35)
            + (brightness_score * 0.20)
            + (contrast_score * 0.20);

        let passes_quality_gate = !is_blurry && !is_dark && !is_unclear && failure_reasons.is_empty();

        FaceQualityScore {
            overall_score: overall,
            blur_score,
            brightness_score,
            contrast_score,
            resolution_score,
            sharpness: lap_var,
            brightness: mean_brightness,
            contrast: std_contrast,
            is_blurry,
            is_dark,
            is_unclear,
            passes_quality_gate,
            failure_reasons,
        }
    }

    /// Assess image quality directly from encoded image bytes (JPEG/PNG)
    pub fn assess_image_bytes(image_bytes: &[u8]) -> Option<FaceQualityScore> {
        let img = image::load_from_memory(image_bytes).ok()?;
        let gray = img.to_luma8();
        let (w, h) = (gray.width() as usize, gray.height() as usize);
        Some(Self::assess_face_quality(gray.as_raw(), w, h))
    }

    /// Assess both full frame and detected MTCNN face crop for blur, darkness, and clarity
    pub fn assess_image_and_face(
        image_bytes: &[u8],
        bbox: Option<(f32, f32, f32, f32)>,
        detection_confidence: Option<f32>,
    ) -> Option<FaceQualityScore> {
        let img = image::load_from_memory(image_bytes).ok()?;
        let gray = img.to_luma8();
        let (w, h) = (gray.width() as usize, gray.height() as usize);
        let full_raw = gray.as_raw();
        let full_lap = Self::laplacian_variance(full_raw, w, h);
        let (full_bright, full_contrast) = Self::brightness_and_contrast(full_raw);

        if let Some((bx1, by1, bx2, by2)) = bbox {
            let x1 = (bx1.max(0.0) as usize).min(w.saturating_sub(1));
            let y1 = (by1.max(0.0) as usize).min(h.saturating_sub(1));
            let x2 = (bx2.ceil().max(0.0) as usize).min(w);
            let y2 = (by2.ceil().max(0.0) as usize).min(h);
            let cw = x2.saturating_sub(x1);
            let ch = y2.saturating_sub(y1);

            if cw >= 8 && ch >= 8 {
                let mut crop_pixels = Vec::with_capacity(cw * ch);
                for row in y1..y2 {
                    let start = row * w + x1;
                    let end = start + cw;
                    crop_pixels.extend_from_slice(&full_raw[start..end]);
                }
                let crop_lap = Self::laplacian_variance(&crop_pixels, cw, ch);
                let (crop_bright, crop_contrast) = Self::brightness_and_contrast(&crop_pixels);

                let effective_sharpness = if crop_lap < 20.0 {
                    crop_lap
                } else {
                    crop_lap.max(full_lap)
                };
                let effective_brightness = crop_bright.min(full_bright.max(crop_bright * 0.85));
                let effective_contrast = crop_contrast;

                let mut failure_reasons = Vec::new();
                let is_dark = crop_bright < 38.0 || full_bright < 28.0;
                let is_blurry = crop_lap < 20.0 || effective_sharpness < 32.0;
                let conf = detection_confidence.unwrap_or(1.0);
                let mut is_unclear = cw < 36 || ch < 36 || crop_contrast < 13.0 || conf < 0.90;

                if is_dark {
                    failure_reasons.push(format!(
                        "Image/face is too dark or black (brightness {:.1} < 38.0) — Must retake",
                        effective_brightness
                    ));
                }
                if is_blurry {
                    failure_reasons.push(format!(
                        "Face is blurry / out of focus (sharpness {:.1} < 32.0) — Must retake",
                        effective_sharpness
                    ));
                }
                if cw < 36 || ch < 36 {
                    failure_reasons.push(format!(
                        "Face is too small / far from camera ({}x{} < 36x36) — Must retake",
                        cw, ch
                    ));
                }
                if crop_contrast < 13.0 {
                    failure_reasons.push(format!(
                        "Face is not clear / low contrast ({:.1} < 13.0) — Must retake",
                        crop_contrast
                    ));
                }
                if crop_bright > 240.0 && crop_contrast < 18.0 {
                    is_unclear = true;
                    failure_reasons.push(format!(
                        "Face is overexposed / washed out (brightness {:.1}) — Must retake",
                        crop_bright
                    ));
                }
                if conf < 0.90 {
                    failure_reasons.push(format!(
                        "Face detection confidence too low ({:.2} < 0.90) — Must retake",
                        conf
                    ));
                }

                let resolution_score = if cw >= 112 && ch >= 112 {
                    1.0
                } else if cw >= 64 && ch >= 64 {
                    0.85
                } else if cw >= 36 && ch >= 36 {
                    0.75
                } else {
                    0.30
                };
                let blur_score = if effective_sharpness >= 100.0 {
                    1.0
                } else if !is_blurry {
                    (effective_sharpness / 100.0).clamp(0.55, 0.99)
                } else {
                    0.20
                };
                let brightness_score = if !is_dark && crop_bright <= 240.0 { 1.0 } else { 0.20 };
                let contrast_score = if crop_contrast >= 13.0 { 1.0 } else { 0.30 };
                let overall = (resolution_score * 0.25)
                    + (blur_score * 0.35)
                    + (brightness_score * 0.20)
                    + (contrast_score * 0.20);
                let passes_quality_gate = !is_dark && !is_blurry && !is_unclear && failure_reasons.is_empty();

                return Some(FaceQualityScore {
                    overall_score: overall,
                    blur_score,
                    brightness_score,
                    contrast_score,
                    resolution_score,
                    sharpness: effective_sharpness,
                    brightness: effective_brightness,
                    contrast: effective_contrast,
                    is_blurry,
                    is_dark,
                    is_unclear,
                    passes_quality_gate,
                    failure_reasons,
                });
            }
        }

        Some(Self::assess_face_quality(full_raw, w, h))
    }
}


