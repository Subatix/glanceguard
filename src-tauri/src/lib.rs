mod commands;
pub mod cv;
mod desktop;
mod models;
mod monitoring_ctl;
mod settings;
mod state;
mod storage;

use tauri::Manager;
use state::AppState;

/// Find and configure the ONNX Runtime dynamic library path (unit tests + integration tests).
fn init_ort() {
    // If ORT_DYLIB_PATH is already set, ort will use it.
    if std::env::var("ORT_DYLIB_PATH").is_ok() {
        return;
    }

    // Common locations for libonnxruntime on macOS (Homebrew)
    let candidates = [
        "/opt/homebrew/lib/libonnxruntime.dylib",
        "/usr/local/lib/libonnxruntime.dylib",
    ];

    for path in &candidates {
        if std::path::Path::new(path).exists() {
            std::env::set_var("ORT_DYLIB_PATH", path);
            return;
        }
    }
}

/// Call once before ONNX-based integration tests in `tests/*.rs` so `ort` can load its dylib.
pub fn ensure_onnx_runtime_loaded() {
    init_ort();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_ort();

    let mut builder = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_keyring::init());

    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri_plugin_global_shortcut::{
            Builder as ShortcutBuilder, Code, Modifiers, Shortcut, ShortcutState,
        };

        builder = builder
            .plugin(tauri_plugin_autostart::Builder::new().build())
            .plugin(
                ShortcutBuilder::new()
                    .with_shortcut(Shortcut::new(
                        Some(Modifiers::SUPER | Modifiers::ALT),
                        Code::KeyP,
                    ))
                    .expect("built-in pause shortcut registers")
                    .with_handler(|app, _, ev| {
                        if ev.state() == ShortcutState::Pressed {
                            desktop::toggle_pause_from_shell(app);
                        }
                    })
                    .build(),
            );
    }

    builder
        .setup(|app| {
            let state = AppState::initialize(app.handle())?;
            app.manage(state);

            #[cfg(not(any(target_os = "android", target_os = "ios")))]
            {
                desktop::configure_main_window(app.handle())?;
                desktop::sync_autostart_with_disk_settings(app.handle())?;
                desktop::create_tray(app.handle())?;
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::models_ready,
            commands::download_models,
            commands::list_cameras,
            commands::set_camera,
            commands::get_settings,
            commands::set_settings,
            commands::start_monitoring,
            commands::stop_monitoring,
            commands::enroll_owner_from_image,
            commands::enroll_owner_from_image_batch,
            commands::validate_enrollment_snapshot,
            commands::enroll_owner_from_live,
            commands::clear_owner,
            commands::get_owner_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
