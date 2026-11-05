// ============================================================================
// alignment.rs — Face alignment via 2D affine rotation
//
// Replaces Python's:
//   from deepface.commons import image_utils
//   aligned_img = image_utils.align_face(img, left_eye, right_eye)
//
// What alignment does:
//   Given that MTCNN found two eye positions, we rotate the image so those
//   two eyes are on a perfectly horizontal line. This is what bumps FaceNet
//   accuracy from ~98.87% to ~99.63% (reported by Google, cited in DeepFace).
//
// Math:
//   1. Find the angle θ between the eye line and the horizontal axis
//   2. Build a 2D rotation matrix around the midpoint of the two eyes
//   3. Apply the rotation to every pixel (bilinear interpolation)
//
// We implement this WITHOUT OpenCV dependency using pure ndarray math,
// matching the exact logic in DeepFace's align_face() function.
// ============================================================================

use crate::mtcnn::FaceDetection;
use image::{DynamicImage, RgbImage, GenericImageView};

// ============================================================================
// align_face — rotate the image so eyes become horizontal
//
// Returns a new DynamicImage with the face centered and rotated.
// The bounding box in `det` still refers to pre-rotation coordinates,
// but preprocessing::crop_and_resize will handle the crop after rotation.
// ============================================================================
pub fn align_face(img: &DynamicImage, det: &FaceDetection) -> DynamicImage {
    let (img_w, img_h) = img.dimensions();
    let rgb = img.to_rgb8();

    // ── Extract eye coordinates from MTCNN landmarks ───────────────────────
    // Landmark order from O-Net:
    //   [0] = left_eye   (viewer's left, person's right)
    //   [1] = right_eye  (viewer's right, person's left)
    let left_eye  = det.landmarks[0];
    let right_eye = det.landmarks[1];

    // ── Compute rotation angle θ ───────────────────────────────────────────
    // We want the eyes to be on the same horizontal line (y1 == y2).
    // The angle between the current eye line and the horizontal is:
    //   θ = atan2(right_eye.y - left_eye.y, right_eye.x - left_eye.x)
    //
    // Python equivalent (deepface/commons/image_utils.py):
    //   angle = np.degrees(np.arctan2(
    //       right_eye[1] - left_eye[1],
    //       right_eye[0] - left_eye[0]
    //   ))
    let dy = right_eye.1 - left_eye.1;
    let dx = right_eye.0 - left_eye.0;
    let angle_rad = dy.atan2(dx); // radians

    // ── Rotation center = midpoint between the eyes ───────────────────────
    let center_x = (left_eye.0 + right_eye.0) / 2.0;
    let center_y = (left_eye.1 + right_eye.1) / 2.0;

    // ── 2D Affine rotation matrix ─────────────────────────────────────────
    // We rotate by -angle (to correct for the tilt).
    // Standard 2D rotation about a center point (cx, cy):
    //
    //   cos_a  sin_a   cx - cx*cos_a - cy*sin_a
    //  -sin_a  cos_a   cy + cx*sin_a - cy*cos_a
    //
    // This is the formula used by cv2.getRotationMatrix2D(center, angle, 1.0)
    // We replicate it exactly so results match DeepFace.
    let cos_a = (-angle_rad).cos();
    let sin_a = (-angle_rad).sin();

    let tx = center_x - center_x * cos_a - center_y * sin_a;
    let ty = center_y + center_x * sin_a - center_y * cos_a;

    // ── Apply inverse affine map to each output pixel ─────────────────────
    // We use the INVERSE transform to avoid holes:
    // For each output pixel (ox, oy), find the source pixel (sx, sy):
    //   [sx]   [cos_a   sin_a] [ox - tx]
    //   [sy] = [-sin_a  cos_a] [oy - ty]
    //
    // Source pixel lookup = inverse of the forward rotation.
    let inv_cos = cos_a;   // inverse of rotation matrix is its transpose
    let inv_sin = -sin_a;

    let out_w = img_w;
    let out_h = img_h;

    let mut out = RgbImage::new(out_w, out_h);
    let src_raw = rgb.as_raw();
    let dst_raw = out.as_mut();

    // Only rotate pixels within the face bounding box region (+2px margin) that crop_and_resize uses
    let bx1 = (det.bbox.0.floor().max(0.0) as u32).saturating_sub(2);
    let by1 = (det.bbox.1.floor().max(0.0) as u32).saturating_sub(2);
    let bx2 = ((det.bbox.2.ceil().max(0.0) as u32) + 2).min(out_w);
    let by2 = ((det.bbox.3.ceil().max(0.0) as u32) + 2).min(out_h);

    for oy in by1..by2 {
        let dy_t = oy as f32 - ty;
        let row_offset = (oy * out_w) as usize * 3;
        for ox in bx1..bx2 {
            let dx_t = ox as f32 - tx;
            let sx = inv_cos * dx_t + inv_sin * dy_t;
            let sy = -inv_sin * dx_t + inv_cos * dy_t;

            let pixel = bilinear_sample_raw(src_raw, sx, sy, img_w, img_h);
            let idx = row_offset + (ox as usize) * 3;
            dst_raw[idx] = pixel[0];
            dst_raw[idx + 1] = pixel[1];
            dst_raw[idx + 2] = pixel[2];
        }
    }

    DynamicImage::ImageRgb8(out)
}

#[inline]
fn bilinear_sample_raw(raw: &[u8], x: f32, y: f32, w: u32, h: u32) -> [u8; 3] {
    if x < 0.0 || y < 0.0 || x >= (w - 1) as f32 || y >= (h - 1) as f32 {
        return [0u8, 0u8, 0u8];
    }

    let x0 = x as usize;
    let y0 = y as usize;
    let wu = w as usize;
    let x1 = (x0 + 1).min(wu - 1);
    let y1 = (y0 + 1).min((h as usize) - 1);

    let wx = x - (x0 as f32);
    let wy = y - (y0 as f32);
    let w00 = (1.0 - wx) * (1.0 - wy);
    let w10 = wx * (1.0 - wy);
    let w01 = (1.0 - wx) * wy;
    let w11 = wx * wy;

    let i_tl = (y0 * wu + x0) * 3;
    let i_tr = (y0 * wu + x1) * 3;
    let i_bl = (y1 * wu + x0) * 3;
    let i_br = (y1 * wu + x1) * 3;

    [
        (raw[i_tl] as f32 * w00 + raw[i_tr] as f32 * w10 + raw[i_bl] as f32 * w01 + raw[i_br] as f32 * w11).clamp(0.0, 255.0) as u8,
        (raw[i_tl + 1] as f32 * w00 + raw[i_tr + 1] as f32 * w10 + raw[i_bl + 1] as f32 * w01 + raw[i_br + 1] as f32 * w11).clamp(0.0, 255.0) as u8,
        (raw[i_tl + 2] as f32 * w00 + raw[i_tr + 2] as f32 * w10 + raw[i_bl + 2] as f32 * w01 + raw[i_br + 2] as f32 * w11).clamp(0.0, 255.0) as u8,
    ]
}
