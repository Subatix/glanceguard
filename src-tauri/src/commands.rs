use image::DynamicImage;
use tauri::{AppHandle, State};

use crate::cv::camera::{self, CameraInfo, CameraSelection};
use crate::cv::config::{load_detector_config, load_embedder_config};
use crate::cv::detector::FaceDetector;
use crate::cv::embedder::FaceEmbedder;
use crate::cv::preprocess::crop_rgb;
use crate::cv::types::{BoundingBox, OwnerModelInfo};
use crate::settings::{Settings, SettingsUpdate};
use crate::state::AppState;
use crate::storage;

#[tauri::command]
pub fn list_cameras() -> Result<Vec<CameraInfo>, String> {
    camera::list_cameras()
}

#[tauri::command]
pub fn set_camera(state: State<'_, AppState>, app: AppHandle, selection: CameraSelection) -> Result<Settings, String> {
    let mut settings = state.settings.lock().map_err(|_| "Settings lock poisoned".to_string())?;
    settings.camera = Some(selection);
    storage::save_settings(&app, &settings)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|_| "Settings lock poisoned".to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_settings(state: State<'_, AppState>, app: AppHandle, update: SettingsUpdate) -> Result<Settings, String> {
    let mut settings = state.settings.lock().map_err(|_| "Settings lock poisoned".to_string())?;
    update.apply(&mut settings)?;
    storage::save_settings(&app, &settings)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn get_owner_status(state: State<'_, AppState>) -> Result<bool, String> {
    let owner = state.owner.lock().map_err(|_| "Owner lock poisoned".to_string())?;
    Ok(owner.is_some())
}

#[tauri::command]
pub fn enroll_owner_from_image(
    state: State<'_, AppState>,
    app: AppHandle,
    image_bytes: Vec<u8>,
) -> Result<OwnerModelInfo, String> {
    let image = image_from_bytes(&image_bytes)?;
    enroll_owner_from_frame(state, app, &image)
}

#[tauri::command]
pub fn enroll_owner_from_live(state: State<'_, AppState>, app: AppHandle) -> Result<OwnerModelInfo, String> {
    let settings = state.settings.lock().map_err(|_| "Settings lock poisoned".to_string())?;
    let selection = settings
        .camera
        .clone()
        .ok_or_else(|| "Select a camera before live enrollment".to_string())?;
    drop(settings);

    let mut camera = crate::cv::camera::open_camera(&selection)?;
    camera.open_stream().map_err(|e| e.to_string())?;
    let frame = camera.frame().map_err(|e| e.to_string())?;
    let image = frame
        .decode_image::<nokhwa::pixel_format::RgbFormat>()
        .map_err(|e| e.to_string())?;
    camera.stop_stream().map_err(|e| e.to_string())?;

    let image = DynamicImage::ImageRgb8(image).to_rgb8();
    enroll_owner_from_frame(state, app, &image)
}

#[tauri::command]
pub fn clear_owner(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    storage::clear_owner_profile(&app)?;
    let mut owner = state.owner.lock().map_err(|_| "Owner lock poisoned".to_string())?;
    *owner = None;
    Ok(())
}

#[tauri::command]
pub fn start_monitoring(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let owner = state.owner.lock().map_err(|_| "Owner lock poisoned".to_string())?;
    if owner.is_none() {
        return Err("Enroll an owner before starting monitoring".to_string());
    }
    drop(owner);

    let mut monitor = state.monitor.lock().map_err(|_| "Monitor lock poisoned".to_string())?;
    if monitor.is_some() {
        return Err("Monitoring is already active".to_string());
    }

    let handle = crate::state::monitor::start_monitoring(
        app,
        state.settings.clone(),
        state.owner.clone(),
    )?;
    *monitor = Some(handle);
    Ok(())
}

#[tauri::command]
pub fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    let mut monitor = state.monitor.lock().map_err(|_| "Monitor lock poisoned".to_string())?;
    if let Some(handle) = monitor.as_mut() {
        handle.stop();
        *monitor = None;
        return Ok(());
    }
    Err("Monitoring is not active".to_string())
}

fn enroll_owner_from_frame(
    state: State<'_, AppState>,
    app: AppHandle,
    image: &image::RgbImage,
) -> Result<OwnerModelInfo, String> {
    let detector_config = load_detector_config(&app)?;
    let embedder_config = load_embedder_config(&app)?;
    let mut detector = FaceDetector::new(&app, detector_config)?;
    let mut embedder = FaceEmbedder::new(&app, embedder_config.clone())?;

    let detections = detector.detect(image)?;
    if detections.is_empty() {
        return Err("No face detected for enrollment".to_string());
    }

    let best = detections
        .iter()
        .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap_or(std::cmp::Ordering::Equal))
        .ok_or_else(|| "No suitable face detected".to_string())?;

    let crop = crop_from_bbox(image, &best.bbox)?;
    let embedding = embedder.embed(&crop)?;
    let model_info = OwnerModelInfo {
        name: embedder_config.model_file,
        input_width: embedder_config.input_width,
        input_height: embedder_config.input_height,
        normalization: format!(
            "mean={:?}, std={:?}, order={}, layout={}",
            embedder_config.mean,
            embedder_config.std,
            embedder_config.channel_order,
            embedder_config.input_layout
        ),
    };

    let profile = storage::new_owner_profile(embedding, model_info.clone());
    storage::save_owner_profile(&app, &profile)?;
    let mut owner = state.owner.lock().map_err(|_| "Owner lock poisoned".to_string())?;
    *owner = Some(profile);
    Ok(model_info)
}

fn crop_from_bbox(image: &image::RgbImage, bbox: &BoundingBox) -> Result<image::RgbImage, String> {
    let x1 = bbox.x.max(0.0) as u32;
    let y1 = bbox.y.max(0.0) as u32;
    let x2 = (bbox.x + bbox.width).min(image.width() as f32) as u32;
    let y2 = (bbox.y + bbox.height).min(image.height() as f32) as u32;

    if x2 <= x1 || y2 <= y1 {
        return Err("Invalid face crop".to_string());
    }

    Ok(crop_rgb(image, x1, y1, x2 - x1, y2 - y1))
}

fn image_from_bytes(bytes: &[u8]) -> Result<image::RgbImage, String> {
    // Try to guess the format from the header
    let format_hint = if bytes.len() >= 4 {
        match &bytes[0..4] {
            [0xFF, 0xD8, 0xFF, _] => " (JPEG detected)",
            [0x89, 0x50, 0x4E, 0x47] => " (PNG detected)",
            [0x66, 0x74, 0x79, 0x70] => " (HEIC/HEIF - NOT SUPPORTED - please convert to JPEG or PNG)",
            _ => " (unknown format)",
        }
    } else {
        " (file too small)"
    };

    if bytes.is_empty() {
        return Err("Empty image data received".to_string());
    }

    let image = image::load_from_memory(bytes).map_err(|e| {
        format!(
            "Failed to load image{}: {}. Supported formats: JPEG, PNG, BMP, GIF, WebP",
            format_hint, e
        )
    })?;
    Ok(image.to_rgb8())
}
