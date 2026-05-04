//! ONNX face models: disk layout and integrity verification.

mod paths;

pub use paths::resolve_onnx_model_path;

use std::path::{Path, PathBuf};

use ring::digest::{digest, SHA256};
use tauri::AppHandle;
use tauri::Manager;

/// Manifest entry for each bundled ONNX referenced by `models/*.json`.
#[derive(Debug, Clone, Copy)]
pub struct ModelArtifact {
    /// Path as stored in JSON (`models/det_500m.onnx`).
    pub resource_path: &'static str,
    /// Lowercase SHA-256 of file bytes (integrity check).
    pub sha256_hex: &'static str,
}

pub const MODEL_ARTIFACTS: &[ModelArtifact] = &[
    ModelArtifact {
        resource_path: "models/det_500m.onnx",
        sha256_hex: "5e4447f50245bbd7966bd6c0fa52938c61474a04ec7def48753668a9d8b4ea3a",
    },
    ModelArtifact {
        resource_path: "models/w600k_mbf.onnx",
        sha256_hex: "9cc6e4a75f0e2bf0b1aed94578f144d15175f357bdc05e815e5c4a02b319eb4f",
    },
];

pub fn models_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let root = app
        .path()
        .app_local_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(root.join("models"))
}

pub(crate) fn file_sha256_hex(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let d = digest(&SHA256, &data);
    Ok(d.as_ref().iter().map(|b| format!("{:02x}", b)).collect())
}

/// Verifies each model exists (app data or bundled resource) and matches its SHA-256.
pub fn ensure_models_verified(app: &AppHandle) -> Result<(), String> {
    for artifact in MODEL_ARTIFACTS {
        let path = resolve_onnx_model_path(app, artifact.resource_path)?;
        if !path.is_file() {
            return Err(format!(
                "Missing bundled model `{}`. Reinstall the app or restore `src-tauri/models/` before building.",
                artifact.resource_path
            ));
        }
        let got = file_sha256_hex(&path)?;
        if got != artifact.sha256_hex {
            return Err(format!(
                "Model `{}` failed integrity check (expected {}, got {}). Reinstall from a trusted build.",
                artifact.resource_path, artifact.sha256_hex, got
            ));
        }
    }
    Ok(())
}
