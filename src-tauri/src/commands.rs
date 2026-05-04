use tauri::{AppHandle, State};

use crate::cv::camera::{self, CameraInfo, CameraSelection};
use crate::cv::enroll;
use crate::cv::types::OwnerModelInfo;
use crate::desktop;
use crate::monitoring_ctl;
use crate::settings::{Settings, SettingsUpdate};
use crate::state::AppState;
use crate::storage;

#[tauri::command]
pub fn list_cameras() -> Result<Vec<CameraInfo>, String> {
    camera::list_cameras()
}

#[tauri::command]
pub fn set_camera(
    state: State<'_, AppState>,
    app: AppHandle,
    selection: CameraSelection,
) -> Result<Settings, String> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?;
    settings.camera = Some(selection);
    storage::save_settings(&app, &settings)?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
pub fn set_settings(
    state: State<'_, AppState>,
    app: AppHandle,
    update: SettingsUpdate,
) -> Result<Settings, String> {
    let mut settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?;
    let prev_login = settings.start_at_login;
    update.apply(&mut settings)?;
    storage::save_settings(&app, &settings)?;
    if settings.start_at_login != prev_login {
        let _ = desktop::sync_autostart_with_flag(&app, settings.start_at_login);
    }
    Ok(settings.clone())
}

#[tauri::command]
pub fn models_ready(app: AppHandle) -> Result<bool, String> {
    Ok(crate::models::ensure_models_verified(&app).is_ok())
}

#[tauri::command]
pub fn download_models(app: AppHandle, base_url: String) -> Result<(), String> {
    let trimmed = base_url.trim();
    if trimmed.is_empty() {
        return Err("Pass a non-empty base URL for model downloads (e.g. your updates host `/models` path).".into());
    }
    crate::models::download_all_models_background(&app, trimmed.to_string());
    Ok(())
}

#[tauri::command]
pub fn get_owner_status(state: State<'_, AppState>) -> Result<bool, String> {
    let owner = state
        .owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?;
    Ok(owner.is_some())
}

#[tauri::command(async)]
pub fn enroll_owner_from_image(
    state: State<'_, AppState>,
    app: AppHandle,
    image_bytes: Vec<u8>,
) -> Result<OwnerModelInfo, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?
        .clone();
    let image = image_from_bytes(&image_bytes)?;
    let profile = enroll::enroll_owner_from_rgb_image(&app, &settings, &image)?;
    let model_info = profile.model.clone();
    storage::save_owner_profile(&app, &profile)?;
    let mut owner = state
        .owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?;
    *owner = Some(profile);
    Ok(model_info)
}

#[tauri::command(async)]
pub fn enroll_owner_from_image_batch(
    state: State<'_, AppState>,
    app: AppHandle,
    image_bytes_list: Vec<Vec<u8>>,
) -> Result<OwnerModelInfo, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?
        .clone();

    if image_bytes_list.len() != 5 {
        return Err(
            "Complete all five pose steps in the enrollment wizard before submitting.".into(),
        );
    }

    let mut decoded: Vec<image::RgbImage> = Vec::with_capacity(image_bytes_list.len());
    for (i, bytes) in image_bytes_list.iter().enumerate() {
        let rgb = image_from_bytes(bytes).map_err(|e| format!("Image {}: {}", i + 1, e))?;
        decoded.push(rgb);
    }
    let profile = enroll::enroll_owner_from_rgb_images_batch(&app, &settings, &decoded)?;
    let model_info = profile.model.clone();
    storage::save_owner_profile(&app, &profile)?;
    let mut owner = state
        .owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?;
    *owner = Some(profile);
    Ok(model_info)
}

#[tauri::command(async)]
pub fn enroll_owner_from_live(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<OwnerModelInfo, String> {
    let settings = state
        .settings
        .lock()
        .map_err(|_| "Settings lock poisoned".to_string())?
        .clone();
    let selection = settings
        .camera
        .clone()
        .ok_or_else(|| "Select a camera before live enrollment".to_string())?;

    let profile = enroll::enroll_owner_from_live_capture(&app, &settings, &selection)?;
    let model_info = profile.model.clone();
    storage::save_owner_profile(&app, &profile)?;
    let mut owner = state
        .owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?;
    *owner = Some(profile);
    Ok(model_info)
}

#[tauri::command]
pub fn clear_owner(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    storage::clear_owner_profile(&app)?;
    let mut owner = state
        .owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?;
    *owner = None;
    Ok(())
}

#[tauri::command]
pub fn start_monitoring(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    monitoring_ctl::try_start_monitoring(&app, &state)
}

#[tauri::command]
pub fn stop_monitoring(state: State<'_, AppState>) -> Result<(), String> {
    monitoring_ctl::try_stop_monitoring(&state)
}

fn image_from_bytes(bytes: &[u8]) -> Result<image::RgbImage, String> {
    // Try to guess the format from the header
    let format_hint = if bytes.len() >= 4 {
        match &bytes[0..4] {
            [0xFF, 0xD8, 0xFF, _] => " (JPEG detected)",
            [0x89, 0x50, 0x4E, 0x47] => " (PNG detected)",
            [0x66, 0x74, 0x79, 0x70] => {
                " (HEIC/HEIF - NOT SUPPORTED - please convert to JPEG or PNG)"
            }
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
