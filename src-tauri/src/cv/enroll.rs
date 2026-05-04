use std::thread;
use std::time::Duration;

use image::RgbImage;
use tauri::AppHandle;

use crate::cv::camera::{open_camera, CameraSelection};
use crate::cv::config::{load_detector_config, load_embedder_config};
use crate::cv::detector::FaceDetector;
use crate::cv::embedder::{EmbedderConfig, FaceEmbedder};
use crate::cv::matching::{calibrate_personal_threshold, mean_embedding};
use crate::cv::preprocess::{align_face_5pt, maybe_apply_clahe_luminance};
use crate::cv::quality::pass_quality_gate;
use crate::cv::types::{FaceDetection, OwnerModelInfo, OwnerProfile};
use crate::settings::{owner_match_threshold, Settings};
use crate::storage;

/// Matches `POSES` order in `EnrollmentWizard.tsx`.
const ENROLL_WIZARD_STEP_LABELS: [&str; 5] = [
    "Center",
    "Turn left",
    "Turn right",
    "Look up",
    "Look down",
];

const TARGET_FRAME_COUNT: usize = 8;
const ENROLL_MAX_WAIT: Duration = Duration::from_millis(3500);
const FRAME_POLL: Duration = Duration::from_millis(35);

/// Prefer larger faces first so we do not lock onto a tiny high-score false positive when multiple boxes exist.
fn pick_enrollment_face_index(image: &RgbImage, faces: &[FaceDetection]) -> Result<usize, String> {
    let mut order: Vec<usize> = (0..faces.len()).collect();
    order.sort_by(|&i, &j| {
        let ai = faces[i].bbox.width * faces[i].bbox.height;
        let aj = faces[j].bbox.width * faces[j].bbox.height;
        aj.partial_cmp(&ai).unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut last_err: Option<String> = None;
    for &idx in &order {
        match pass_quality_gate(image, &faces[idx]) {
            Ok(()) => return Ok(idx),
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or_else(|| "No suitable face detected for enrollment".into()))
}

/// Detector + quality gate only (no embedder). Matches face picking used by [`embedding_from_rgb_image`].
pub fn validate_enrollment_snapshot_quality(
    app: &AppHandle,
    image: &RgbImage,
) -> Result<(), String> {
    crate::models::ensure_models_verified(app)?;
    let detector_config = load_detector_config(app)?;
    let mut detector = FaceDetector::new(app, detector_config)?;
    let faces = detector.detect(image)?;
    if faces.is_empty() {
        return Err("No face detected for enrollment".into());
    }
    pick_enrollment_face_index(image, &faces)?;
    Ok(())
}

/// One embedding from a still RGB frame using the same detector / embedder / quality path as live enrollment.
pub fn embedding_from_rgb_image(
    image: &RgbImage,
    settings: &Settings,
    detector: &mut FaceDetector,
    embedder: &mut FaceEmbedder,
    embedder_config: &EmbedderConfig,
) -> Result<Vec<f32>, String> {
    let faces = detector.detect(image)?;
    if faces.is_empty() {
        return Err("No face detected for enrollment".into());
    }

    let idx = pick_enrollment_face_index(image, &faces)?;
    let face = &faces[idx];

    let mut aligned = align_face_5pt(
        image,
        &face.landmarks,
        embedder_config.input_width,
        embedder_config.input_height,
    )?;
    maybe_apply_clahe_luminance(&mut aligned, settings.clahe_face_preproc);

    embedder.embed(&aligned)
}

pub fn enroll_owner_from_rgb_image(
    app: &AppHandle,
    settings: &Settings,
    image: &RgbImage,
) -> Result<OwnerProfile, String> {
    crate::models::ensure_models_verified(app)?;
    let detector_config = load_detector_config(app)?;
    let embedder_config = load_embedder_config(app)?;
    let mut detector = FaceDetector::new(app, detector_config)?;
    let mut embedder = FaceEmbedder::new(app, embedder_config.clone())?;

    let embedding = embedding_from_rgb_image(
        image,
        settings,
        &mut detector,
        &mut embedder,
        &embedder_config,
    )?;
    let personal = owner_match_threshold();
    let model_info = owner_model_info(&embedder_config);
    Ok(storage::new_owner_profile(
        embedding.clone(),
        vec![embedding],
        personal,
        model_info,
    ))
}

/// Wizard flow: one JPEG/PNG snapshot per guided pose. Uses the same quality gate and embedder path as live multi-frame enrollment, then mean + personal threshold calibration (Phase 4).
pub fn enroll_owner_from_rgb_images_batch(
    app: &AppHandle,
    settings: &Settings,
    images: &[RgbImage],
) -> Result<OwnerProfile, String> {
    if images.len() != 5 {
        return Err(
            "Enrollment expects exactly five pose images (center, left, right, up, down).".into(),
        );
    }

    crate::models::ensure_models_verified(app)?;

    let detector_config = load_detector_config(app)?;
    let embedder_config = load_embedder_config(app)?;
    let mut detector = FaceDetector::new(app, detector_config)?;
    let mut embedder = FaceEmbedder::new(app, embedder_config.clone())?;

    let mut samples: Vec<Vec<f32>> = Vec::with_capacity(images.len());
    for (i, image) in images.iter().enumerate() {
        let step_label = ENROLL_WIZARD_STEP_LABELS[i];
        let emb =
            embedding_from_rgb_image(image, settings, &mut detector, &mut embedder, &embedder_config)
                .map_err(|e| {
                    format!(
                        "{} (saved pose {} of 5 — validated only after the final capture): {}",
                        step_label,
                        i + 1,
                        e
                    )
                })?;
        samples.push(emb);
    }

    let mean = mean_embedding(&samples)?;
    let personal = calibrate_personal_threshold(&samples)?;
    let model_info = owner_model_info(&embedder_config);
    Ok(storage::new_owner_profile(
        mean, samples, personal, model_info,
    ))
}

pub fn enroll_owner_from_live_capture(
    app: &AppHandle,
    settings: &Settings,
    selection: &CameraSelection,
) -> Result<OwnerProfile, String> {
    crate::models::ensure_models_verified(app)?;
    crate::cv::camera::ensure_camera_video_permission()?;

    let detector_config = load_detector_config(app)?;
    let embedder_config = load_embedder_config(app)?;
    let mut detector = FaceDetector::new(app, detector_config)?;
    let mut embedder = FaceEmbedder::new(app, embedder_config.clone())?;

    let w = embedder_config.input_width;
    let h = embedder_config.input_height;

    let mut camera = open_camera(selection)?;
    camera.open_stream().map_err(|e| e.to_string())?;

    let started = std::time::Instant::now();
    let mut samples: Vec<Vec<f32>> = Vec::with_capacity(TARGET_FRAME_COUNT);

    while samples.len() < TARGET_FRAME_COUNT && started.elapsed() < ENROLL_MAX_WAIT {
        let frame = match camera.frame() {
            Ok(f) => f,
            Err(_) => {
                thread::sleep(FRAME_POLL);
                continue;
            }
        };
        let Ok(image) = frame.decode_image::<nokhwa::pixel_format::RgbFormat>() else {
            thread::sleep(FRAME_POLL);
            continue;
        };

        let rgb: RgbImage = image;
        let Ok(mut faces) = detector.detect(&rgb) else {
            thread::sleep(FRAME_POLL);
            continue;
        };
        if faces.is_empty() {
            thread::sleep(FRAME_POLL);
            continue;
        }
        faces.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        let best = faces.into_iter().next().expect("non-empty");

        if pass_quality_gate(&rgb, &best).is_err() {
            thread::sleep(FRAME_POLL);
            continue;
        }

        let Ok(mut aligned) = align_face_5pt(&rgb, &best.landmarks, w, h) else {
            thread::sleep(FRAME_POLL);
            continue;
        };
        maybe_apply_clahe_luminance(&mut aligned, settings.clahe_face_preproc);

        let Ok(emb) = embedder.embed(&aligned) else {
            thread::sleep(FRAME_POLL);
            continue;
        };
        samples.push(emb);
        thread::sleep(FRAME_POLL);
    }

    camera.stop_stream().map_err(|e| e.to_string())?;

    if samples.len() < 3 {
        return Err(format!(
            "Captured only {} usable frames (need at least 3). Improve lighting, face the camera, and hold still for ~3 seconds.",
            samples.len()
        ));
    }

    let mean = mean_embedding(&samples)?;
    let personal = calibrate_personal_threshold(&samples)?;
    let model_info = owner_model_info(&embedder_config);
    Ok(storage::new_owner_profile(
        mean, samples, personal, model_info,
    ))
}

fn owner_model_info(embedder_config: &crate::cv::embedder::EmbedderConfig) -> OwnerModelInfo {
    OwnerModelInfo {
        name: embedder_config.model_file.clone(),
        input_width: embedder_config.input_width,
        input_height: embedder_config.input_height,
        normalization: format!(
            "mean={:?}, std={:?}, order={}, layout={}",
            embedder_config.mean,
            embedder_config.std,
            embedder_config.channel_order,
            embedder_config.input_layout
        ),
    }
}
