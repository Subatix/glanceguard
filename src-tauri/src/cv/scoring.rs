use crate::cv::types::{BoundingBox, FaceDetection};
use crate::settings::Sensitivity;

fn clamp01(value: f32) -> f32 {
    value.max(0.0).min(1.0)
}

fn bbox_center(bbox: &BoundingBox) -> (f32, f32) {
    (bbox.x + bbox.width * 0.5, bbox.y + bbox.height * 0.5)
}

pub fn estimate_yaw_score(face: &FaceDetection) -> f32 {
    let left_eye = &face.landmarks[0];
    let right_eye = &face.landmarks[1];
    let nose = &face.landmarks[2];

    let eye_mid_x = (left_eye.x + right_eye.x) * 0.5;
    let eye_dist = (left_eye.x - right_eye.x).abs().max(1.0);
    let yaw_ratio = (nose.x - eye_mid_x) / eye_dist;

    clamp01(1.0 - (yaw_ratio.abs() / 0.5))
}

pub fn compute_observer_score(
    face: &FaceDetection,
    owner_bbox: Option<&BoundingBox>,
    similarity: Option<f32>,
    frame_width: u32,
    frame_height: u32,
    sensitivity: Sensitivity,
) -> f32 {
    let frame_area = (frame_width * frame_height) as f32;
    let face_area = face.bbox.width * face.bbox.height;
    let size_ratio = if frame_area > 0.0 { face_area / frame_area } else { 0.0 };
    let size_score = clamp01(size_ratio * 8.0);

    let orientation_score = estimate_yaw_score(face);

    let proximity_score = match owner_bbox {
        Some(owner) => {
            let (ox, oy) = bbox_center(owner);
            let (fx, fy) = bbox_center(&face.bbox);
            let dx = fx - ox;
            let dy = fy - oy;
            let max_dist = ((frame_width * frame_width + frame_height * frame_height) as f32).sqrt() * 0.5;
            if max_dist > 0.0 {
                clamp01(1.0 - ((dx * dx + dy * dy).sqrt() / max_dist))
            } else {
                0.0
            }
        }
        None => 0.5,
    };

    let mismatch_score = match similarity {
        Some(sim) => clamp01(1.0 - sim),
        None => 0.6,
    };

    let (w_size, w_orient, w_prox, w_mismatch) = match sensitivity {
        Sensitivity::Low => (0.20, 0.35, 0.20, 0.25),
        Sensitivity::Medium => (0.25, 0.30, 0.20, 0.25),
        Sensitivity::High => (0.30, 0.25, 0.15, 0.30),
    };

    clamp01(
        size_score * w_size
            + orientation_score * w_orient
            + proximity_score * w_prox
            + mismatch_score * w_mismatch,
    )
}

#[cfg(test)]
mod tests {
    use super::{compute_observer_score, estimate_yaw_score};
    use crate::cv::types::{BoundingBox, FaceDetection, Point};
    use crate::settings::Sensitivity;

    fn sample_face() -> FaceDetection {
        FaceDetection {
            bbox: BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 120.0,
                height: 140.0,
            },
            score: 0.9,
            landmarks: [
                Point { x: 120.0, y: 130.0 },
                Point { x: 180.0, y: 130.0 },
                Point { x: 150.0, y: 150.0 },
                Point { x: 130.0, y: 190.0 },
                Point { x: 170.0, y: 190.0 },
            ],
        }
    }

    #[test]
    fn yaw_score_is_high_when_nose_centered() {
        let face = sample_face();
        let score = estimate_yaw_score(&face);
        assert!(score > 0.7);
    }

    #[test]
    fn observer_score_increases_with_mismatch() {
        let face = sample_face();
        let owner_bbox = BoundingBox {
            x: 90.0,
            y: 90.0,
            width: 120.0,
            height: 140.0,
        };

        let low_mismatch = compute_observer_score(
            &face,
            Some(&owner_bbox),
            Some(0.9),
            1280,
            720,
            Sensitivity::Medium,
        );
        let high_mismatch = compute_observer_score(
            &face,
            Some(&owner_bbox),
            Some(0.2),
            1280,
            720,
            Sensitivity::Medium,
        );

        assert!(high_mismatch > low_mismatch);
    }
}
