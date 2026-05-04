use std::path::PathBuf;

use tauri::{AppHandle, Manager};

pub fn resolve_onnx_model_path(app: &AppHandle, model_file: &str) -> Result<PathBuf, String> {
    let file_name = std::path::Path::new(model_file)
        .file_name()
        .ok_or_else(|| format!("Invalid model path: {}", model_file))?;

    let app_models = super::models_data_dir(app)?;
    let candidate = app_models.join(file_name);
    if candidate.is_file() {
        return Ok(candidate);
    }

    app.path()
        .resolve(model_file, tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())
}
