//! Shared monitoring start/stop helpers for IPC commands and tray / shortcuts.

use tauri::AppHandle;

use crate::state::AppState;

pub(crate) fn monitoring_active(state: &AppState) -> Result<bool, String> {
    let monitor = state
        .monitor
        .lock()
        .map_err(|_| "Monitor lock poisoned".to_string())?;
    Ok(monitor.is_some())
}

pub(crate) fn try_start_monitoring(app: &AppHandle, state: &AppState) -> Result<(), String> {
    crate::models::ensure_models_verified(app)?;
    let owner = state
        .owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?;
    let profile = owner
        .as_ref()
        .ok_or_else(|| "Enroll an owner before starting monitoring".to_string())?;
    profile.validate_enrollment_complete()?;
    drop(owner);

    let mut monitor = state
        .monitor
        .lock()
        .map_err(|_| "Monitor lock poisoned".to_string())?;
    if monitor.is_some() {
        return Err("Monitoring is already active".to_string());
    }

    let handle = crate::state::monitor::start_monitoring(
        app.clone(),
        state.settings.clone(),
        state.owner.clone(),
    )?;
    *monitor = Some(handle);
    Ok(())
}

pub(crate) fn try_stop_monitoring(state: &AppState) -> Result<(), String> {
    let mut monitor = state
        .monitor
        .lock()
        .map_err(|_| "Monitor lock poisoned".to_string())?;
    if let Some(handle) = monitor.as_mut() {
        handle.stop();
        *monitor = None;
        return Ok(());
    }
    Err("Monitoring is not active".to_string())
}
