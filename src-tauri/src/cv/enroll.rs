use std::thread;
use std::time::Duration;

use image::RgbImage;
use tauri::AppHandle;

use crate::cv::camera::{open_camera, CameraSelection};
use crate::cv::config::{load_detector_config, load_embedder_config};
use crate::cv::detector::FaceDetector;
use crate::cv::embedder::FaceEmbedder;
use crate::cv::matching::{calibrate_personal_threshold, mean_embedding};
use crate::cv::preprocess::{align_face_5pt, maybe_apply_clahe_luminance};
use crate::cv::quality::pass_quality_gate;
use crate::cv::types::{OwnerModelInfo, OwnerProfile};
use crate::settings::{owner_match_threshold, Settings};
use crate::storage;

const TARGET_FRAME_COUNT: usize = 8;
const ENROLL_MAX_WAIT: Duration = Duration::from_millis(3500);
const FRAME_POLL: Duration = Duration::from_millis(35);

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

    let mut faces = detector.detect(image)?;
    if faces.is_empty() {
        return Err("No face detected for enrollment".into());
    }
    faces.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let best = faces
        .into_iter()
        .next()
        .ok_or_else(|| "No suitable face detected".to_string())?;

    pass_quality_gate(image, &best)?;

    let mut aligned = align_face_5pt(
        image,
        &best.landmarks,
        embedder_config.input_width,
        embedder_config.input_height,
    )?;
    maybe_apply_clahe_luminance(&mut aligned, settings.clahe_face_preproc);

    let embedding = embedder.embed(&aligned)?;
    let personal = owner_match_threshold();
    let model_info = owner_model_info(&embedder_config);
    Ok(storage::new_owner_profile(
        embedding.clone(),
        vec![embedding],
        personal,
        model_info,
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
