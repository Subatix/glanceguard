mod commands;
mod cv;
mod models;
mod settings;
mod state;
mod storage;

use tauri::Manager;
use state::AppState;

/// Find and configure the ONNX Runtime dynamic library path.
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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    init_ort();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_keyring::init())
        .setup(|app| {
            let state = AppState::initialize(app.handle())?;
            app.manage(state);
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
            commands::enroll_owner_from_live,
            commands::clear_owner,
            commands::get_owner_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
