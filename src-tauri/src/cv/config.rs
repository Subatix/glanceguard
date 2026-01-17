use serde::de::DeserializeOwned;
use tauri::{AppHandle, Manager};

use crate::cv::detector::DetectorConfig;
use crate::cv::embedder::EmbedderConfig;

pub fn load_detector_config(app: &AppHandle) -> Result<DetectorConfig, String> {
    read_resource_json(app, "models/scrfd.json")
}

pub fn load_embedder_config(app: &AppHandle) -> Result<EmbedderConfig, String> {
    read_resource_json(app, "models/arcface.json")
}

fn read_resource_json<T: DeserializeOwned>(app: &AppHandle, path: &str) -> Result<T, String> {
    let resource_path = app
        .path()
        .resolve(path, tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())?;
    let data = std::fs::read(resource_path).map_err(|e| e.to_string())?;
    serde_json::from_slice(&data).map_err(|e| e.to_string())
}
