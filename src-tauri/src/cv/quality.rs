use image::RgbImage;

use crate::cv::types::{BoundingBox, FaceDetection};

pub const MIN_FACE_WIDTH: f32 = 80.0;
pub const MIN_DETECTION_SCORE: f32 = 0.6;
pub const MIN_LAPLACIAN_VARIANCE: f32 = 50.0;
/// Landmark-derived yaw ratio; |ratio| ≤ ~0.5 matches ~30° practical bound.
pub const MAX_YAW_RATIO_ABS: f32 = 0.5;

pub fn yaw_ratio(face: &FaceDetection) -> f32 {
    let left_eye = &face.landmarks[0];
    let right_eye = &face.landmarks[1];
    let nose = &face.landmarks[2];

    let eye_mid_x = (left_eye.x + right_eye.x) * 0.5;
    let eye_dist = (left_eye.x - right_eye.x).abs().max(1.0);
    (nose.x - eye_mid_x) / eye_dist
}

pub fn pass_quality_gate(image: &RgbImage, face: &FaceDetection) -> Result<(), String> {
    if face.bbox.width < MIN_FACE_WIDTH {
        return Err(format!(
            "face width {:.1} below minimum {:.0}",
            face.bbox.width, MIN_FACE_WIDTH
        ));
    }
    if face.score < MIN_DETECTION_SCORE {
        return Err(format!(
            "detection score {:.2} below minimum {:.2}",
            face.score, MIN_DETECTION_SCORE
        ));
    }
    let y = yaw_ratio(face);
    if y.abs() > MAX_YAW_RATIO_ABS {
        return Err(format!(
            "|yaw ratio| {:.2} exceeds {:.2}",
            y.abs(),
            MAX_YAW_RATIO_ABS
        ));
    }
    let lap_var = laplacian_variance_crop(image, &face.bbox)?;
    if lap_var < MIN_LAPLACIAN_VARIANCE {
        return Err(format!(
            "Laplacian variance {:.1} below {:.0} (too blurry)",
            lap_var, MIN_LAPLACIAN_VARIANCE
        ));
    }
    Ok(())
}

fn rgb_to_luma(r: u8, g: u8, b: u8) -> f32 {
    0.299_f32 * r as f32 + 0.587_f32 * g as f32 + 0.114_f32 * b as f32
}

fn laplacian_variance_crop(image: &RgbImage, bbox: &BoundingBox) -> Result<f32, String> {
    let x1 = bbox.x.max(0.0).floor() as u32;
    let y1 = bbox.y.max(0.0).floor() as u32;
    let x2 = (bbox.x + bbox.width).min(image.width() as f32).ceil() as u32;
    let y2 = (bbox.y + bbox.height).min(image.height() as f32).ceil() as u32;
    if x2 <= x1 + 2 || y2 <= y1 + 2 {
        return Err("Face crop too small for blur check".into());
    }

    let w = x2 - x1;
    let h = y2 - y1;
    let mut gray = vec![0f32; (w * h) as usize];
    for row in 0..h {
        for col in 0..w {
            let px = image.get_pixel(x1 + col, y1 + row);
            let idx = (row * w + col) as usize;
            gray[idx] = rgb_to_luma(px[0], px[1], px[2]);
        }
    }

    let idx = |row: u32, col: u32| -> usize { (row * w + col) as usize };

    let mut responses = Vec::with_capacity(((w - 2) * (h - 2)) as usize);
    for row in 1..h - 1 {
        for col in 1..w - 1 {
            let c = gray[idx(row, col)];
            let l = gray[idx(row - 1, col)]
                + gray[idx(row + 1, col)]
                + gray[idx(row, col - 1)]
                + gray[idx(row, col + 1)]
                - 4.0 * c;
            responses.push(l);
        }
    }

    if responses.is_empty() {
        return Err("Laplacian region empty".into());
    }

    let mean = responses.iter().copied().sum::<f32>() / responses.len() as f32;
    let var = responses.iter().map(|r| (r - mean).powi(2)).sum::<f32>() / responses.len() as f32;
    Ok(var)
}
