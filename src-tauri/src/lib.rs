mod commands;
mod cv;
mod settings;
mod state;
mod storage;

use tauri::Manager;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
