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
    pub face_confidence: f32,
    pub is_blurry: bool,
    pub is_dark: bool,
    pub is_unclear: bool,
    pub quality_ok: bool,
    pub quality_reason: Option<String>,
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
        let quality_reason = if passes_quality_gate {
            None
        } else if is_dark {
            Some("DARK".to_string())
        } else if is_blurry {
            Some("BLURRY".to_string())
        } else {
            Some("FACE_NOT_CLEAR".to_string())
        };

        FaceQualityScore {
            overall_score: overall,
            blur_score,
            brightness_score,
            contrast_score,
            resolution_score,
            sharpness: lap_var,
            brightness: mean_brightness,
            contrast: std_contrast,
            face_confidence: if passes_quality_gate { 1.0 } else { 0.0 },
            is_blurry,
            is_dark,
            is_unclear,
            quality_ok: passes_quality_gate,
            quality_reason,
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
        let (full_bright, _full_contrast) = Self::brightness_and_contrast(full_raw);

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

                // Do NOT let a sharp background mask a blurry face crop (crop_lap < 25.0 is always blurry)
                let effective_sharpness = if crop_lap < 25.0 {
                    crop_lap
                } else {
                    crop_lap.max(full_lap * 0.5)
                };
                let effective_brightness = crop_bright.min(full_bright.max(crop_bright * 0.85));
                let effective_contrast = crop_contrast;

                let mut failure_reasons = Vec::new();
                let is_dark = crop_bright < 38.0 || full_bright < 28.0;
                let is_blurry = crop_lap < 25.0 || effective_sharpness < 32.0;
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
                let quality_reason = if passes_quality_gate {
                    None
                } else if is_dark {
                    Some("DARK".to_string())
                } else if is_blurry {
                    Some("BLURRY".to_string())
                } else {
                    Some("FACE_NOT_CLEAR".to_string())
                };

                return Some(FaceQualityScore {
                    overall_score: overall,
                    blur_score,
                    brightness_score,
                    contrast_score,
                    resolution_score,
                    sharpness: effective_sharpness,
                    brightness: effective_brightness,
                    contrast: effective_contrast,
                    face_confidence: conf,
                    is_blurry,
                    is_dark,
                    is_unclear,
                    quality_ok: passes_quality_gate,
                    quality_reason,
                    passes_quality_gate,
                    failure_reasons,
                });
            }
        }

        let mut score = Self::assess_face_quality(full_raw, w, h);
        if let Some(c) = detection_confidence {
            score.face_confidence = c;
        }
        Some(score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{GrayImage, ImageFormat, Luma};
    use std::io::Cursor;

    fn encode_gray_png(img: &GrayImage) -> Vec<u8> {
        let mut buf = Cursor::new(Vec::new());
        image::DynamicImage::ImageLuma8(img.clone())
            .write_to(&mut buf, ImageFormat::Png)
            .unwrap();
        buf.into_inner()
    }

    #[test]
    fn test_dark_black_image_must_retake() {
        let mut img = GrayImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                img.put_pixel(x, y, Luma([((x + y) % 15) as u8]));
            }
        }
        let bytes = encode_gray_png(&img);
        let q = FaceQualityAssessor::assess_image_and_face(&bytes, Some((16.0, 16.0, 112.0, 112.0)), Some(0.99)).unwrap();
        assert!(q.is_dark);
        assert!(!q.quality_ok);
        assert!(!q.passes_quality_gate);
        assert_eq!(q.quality_reason.as_deref(), Some("DARK"));
    }

    #[test]
    fn test_blurry_smooth_image_must_retake() {
        let mut img = GrayImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                let v = (70 + ((x as i32 - 64) / 2) + ((y as i32 - 64) / 2)).clamp(45, 180) as u8;
                img.put_pixel(x, y, Luma([v]));
            }
        }
        let bytes = encode_gray_png(&img);
        let q = FaceQualityAssessor::assess_image_and_face(&bytes, Some((16.0, 16.0, 112.0, 112.0)), Some(0.99)).unwrap();
        assert!(q.is_blurry);
        assert!(!q.quality_ok);
        assert_eq!(q.quality_reason.as_deref(), Some("BLURRY"));
    }

    #[test]
    fn test_blurry_face_crop_on_sharp_background_must_retake() {
        // Sharp checkerboard background, but blurry face region [24..104, 24..104]
        let mut img = GrayImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                if (24..104).contains(&x) && (24..104).contains(&y) {
                    // Smooth gradient inside face bbox (contrast > 15, brightness ~110, but low Laplacian sharpness ~6)
                    let base = 70 + ((x - 24) * 80 / 80) as u8;
                    let wave = if (x + y) % 4 == 0 { 2u8 } else { 0u8 };
                    img.put_pixel(x, y, Luma([base.saturating_add(wave)]));
                } else {
                    // High-frequency sharp background outside face bbox
                    let v = if (x + y) % 2 == 0 { 40u8 } else { 200u8 };
                    img.put_pixel(x, y, Luma([v]));
                }
            }
        }
        let bytes = encode_gray_png(&img);
        let q = FaceQualityAssessor::assess_image_and_face(&bytes, Some((24.0, 24.0, 104.0, 104.0)), Some(0.98)).unwrap();
        assert!(q.is_blurry, "Blurry face crop on sharp background must be flagged blurry, got: {:?}", q);
        assert!(!q.quality_ok);
        assert_eq!(q.quality_reason.as_deref(), Some("BLURRY"));
    }

    #[test]
    fn test_sharp_clear_face_passes_quality_gate() {
        let mut img = GrayImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                let edge = if (x % 7 == 0) || (y % 7 == 0) { 175u8 } else { 85u8 };
                img.put_pixel(x, y, Luma([edge]));
            }
        }
        let bytes = encode_gray_png(&img);
        let q = FaceQualityAssessor::assess_image_and_face(&bytes, Some((10.0, 10.0, 118.0, 118.0)), Some(0.99)).unwrap();
        assert!(q.quality_ok, "Expected sharp clear face to pass quality gate, got: {:?}", q);
        assert!(q.passes_quality_gate);
        assert_eq!(q.quality_reason, None);
        assert!(q.sharpness >= 32.0);
        assert!(q.brightness >= 38.0);
        assert!((q.face_confidence - 0.99).abs() < 1e-4);
    }

    #[test]
    fn test_unclear_low_confidence_or_tiny_face_must_retake() {
        let mut img = GrayImage::new(128, 128);
        for y in 0..128 {
            for x in 0..128 {
                let edge = if (x % 6 == 0) || (y % 6 == 0) { 170u8 } else { 90u8 };
                img.put_pixel(x, y, Luma([edge]));
            }
        }
        let bytes = encode_gray_png(&img);
        // Low detection confidence (0.75 < 0.90)
        let q_low_conf = FaceQualityAssessor::assess_image_and_face(&bytes, Some((10.0, 10.0, 118.0, 118.0)), Some(0.75)).unwrap();
        assert!(q_low_conf.is_unclear);
        assert!(!q_low_conf.quality_ok);
        assert_eq!(q_low_conf.quality_reason.as_deref(), Some("FACE_NOT_CLEAR"));

        // Tiny face bbox (20x20 < 36x36)
        let q_tiny = FaceQualityAssessor::assess_image_and_face(&bytes, Some((50.0, 50.0, 70.0, 70.0)), Some(0.99)).unwrap();
        assert!(q_tiny.is_unclear);
        assert!(!q_tiny.quality_ok);
        assert_eq!(q_tiny.quality_reason.as_deref(), Some("FACE_NOT_CLEAR"));
    }

    #[test]
    fn test_real_face_image_blur_and_dark_rejection() {
        let candidates = [
            "../../rust_deepFace/test_images/Anjelina Jolie/1.jpg",
            "rust_deepFace/test_images/Anjelina Jolie/1.jpg",
        ];
        let Some(path) = candidates.iter().find(|p| std::path::Path::new(p).exists()) else {
            return;
        };
        let raw = std::fs::read(path).unwrap();
        let bbox = Some((401.0, 57.0, 596.0, 316.0));

        // 1. Original clear image passes
        let q_orig = FaceQualityAssessor::assess_image_and_face(&raw, bbox, Some(0.999)).unwrap();
        assert!(q_orig.quality_ok, "Original 1.jpg should pass quality gate: {:?}", q_orig);
        assert_eq!(q_orig.quality_reason, None);

        // 2. Blurred version of real face image fails with BLURRY
        let dyn_img = image::load_from_memory(&raw).unwrap();
        let blurred = dyn_img.blur(6.0);
        let mut blur_buf = Cursor::new(Vec::new());
        blurred.write_to(&mut blur_buf, ImageFormat::Jpeg).unwrap();
        let q_blur = FaceQualityAssessor::assess_image_and_face(blur_buf.get_ref(), bbox, Some(0.99)).unwrap();
        assert!(q_blur.is_blurry, "Blurred 1.jpg must be flagged blurry: {:?}", q_blur);
        assert!(!q_blur.quality_ok);
        assert_eq!(q_blur.quality_reason.as_deref(), Some("BLURRY"));

        // 3. Darkened version of real face image fails with DARK
        let mut dark_luma = dyn_img.to_luma8();
        for p in dark_luma.pixels_mut() {
            p[0] = (p[0] as u16 / 6) as u8;
        }
        let dark_bytes = encode_gray_png(&dark_luma);
        let q_dark = FaceQualityAssessor::assess_image_and_face(&dark_bytes, bbox, Some(0.99)).unwrap();
        assert!(q_dark.is_dark, "Darkened 1.jpg must be flagged dark: {:?}", q_dark);
        assert!(!q_dark.quality_ok);
        assert_eq!(q_dark.quality_reason.as_deref(), Some("DARK"));
    }
}



