//! Menu bar tray, hide-on-close, autostart sync (desktop targets only).

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use serde::Serialize;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri::{
    image::Image,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use tauri_plugin_autostart::ManagerExt;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::monitoring_ctl;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::state::AppState;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::storage;

#[cfg(not(any(target_os = "android", target_os = "ios")))]
#[derive(Clone, Serialize)]
struct GlanceGuardMonitorPayload {
    idle: bool,
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn tauri_err(e: tauri::Error) -> String {
    e.to_string()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn emit_monitor_state(app: &AppHandle, idle: bool) {
    let payload = GlanceGuardMonitorPayload { idle };
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit("glanceguard-monitor-state", &payload);
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn emit_tray_error(app: &AppHandle, message: String) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit(
            "glanceguard-tray-error",
            &serde_json::json!({ "message": message }),
        );
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn sync_autostart_with_disk_settings(app: &AppHandle) -> Result<(), String> {
    let settings = storage::load_settings(app)?;
    sync_autostart_with_flag(app, settings.start_at_login)
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn sync_autostart_with_flag(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(|e| e.to_string())?;
    } else {
        manager.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn toggle_pause_from_shell(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let active = match monitoring_ctl::monitoring_active(&state) {
        Ok(v) => v,
        Err(e) => {
            emit_tray_error(app, e);
            return;
        }
    };
    let result = if active {
        monitoring_ctl::try_stop_monitoring(&state).map(|_| true)
    } else {
        monitoring_ctl::try_start_monitoring(app, &state).map(|_| false)
    };
    match result {
        Ok(now_idle) => emit_monitor_state(app, now_idle),
        Err(e) => emit_tray_error(app, e),
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn navigate_main(app: &AppHandle, screen: &str) {
    show_main_window(app);
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.emit(
            "glanceguard-navigate",
            &serde_json::json!({ "screen": screen }),
        );
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn configure_main_window(app: &AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "main window missing".to_string())?;
    let window_clone = window.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window_clone.hide();
        }
    });
    Ok(())
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
fn menu_event_id(event: &MenuEvent) -> &str {
    event.id.as_ref()
}

#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub(crate) fn create_tray(app: &AppHandle) -> Result<(), String> {
    let status = MenuItem::with_id(
        app,
        "tray_status",
        "Monitoring: see Pause / resume below",
        false,
        None::<&str>,
    )
    .map_err(tauri_err)?;
    let toggle = MenuItem::with_id(
        app,
        "toggle_pause",
        "Pause / resume monitoring",
        true,
        None::<&str>,
    )
    .map_err(tauri_err)?;
    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>).map_err(tauri_err)?;
    let settings_m =
        MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>).map_err(tauri_err)?;
    let sep = PredefinedMenuItem::separator(app).map_err(tauri_err)?;
    let quit =
        MenuItem::with_id(app, "quit", "Quit GlanceGuard", true, None::<&str>).map_err(tauri_err)?;

    let menu = Menu::with_items(app, &[&status, &toggle, &open, &settings_m, &sep, &quit])
        .map_err(tauri_err)?;

    let icon_bytes = include_bytes!("../icons/tray-Template.png");
    let icon = Image::from_bytes(icon_bytes).map_err(|e| e.to_string())?;

    let app_click = app.clone();
    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("GlanceGuard")
        .show_menu_on_left_click(false)
        .on_tray_icon_event(move |_tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                show_main_window(&app_click);
            }
        })
        .on_menu_event(|app, event: MenuEvent| {
            match menu_event_id(&event) {
                "toggle_pause" => toggle_pause_from_shell(app),
                "open" => show_main_window(app),
                "settings" => navigate_main(app, "settings"),
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .icon_as_template(cfg!(target_os = "macos"))
        .build(app)
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) fn sync_autostart_with_disk_settings(_app: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) fn sync_autostart_with_flag(
    _app: &tauri::AppHandle,
    _enabled: bool,
) -> Result<(), String> {
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) fn configure_main_window(_app: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[cfg(any(target_os = "android", target_os = "ios"))]
pub(crate) fn create_tray(_app: &tauri::AppHandle) -> Result<(), String> {
    Ok(())
}
