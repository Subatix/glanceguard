use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use ring::digest::{Context, SHA256};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use super::{
    artifact_paths, ensure_models_verified, file_sha256_hex, models_data_dir, ModelArtifact,
};

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub filename: String,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

#[derive(Clone, Serialize)]
pub struct ErrorPayload {
    pub message: String,
}

pub fn download_all_models_background(app: &AppHandle, base_url: String) {
    let app = app.clone();
    std::thread::spawn(move || match download_all_inner(&app, &base_url) {
        Ok(()) => {
            let _ = app.emit("model-download-done", ());
        }
        Err(e) => {
            let _ = app.emit(
                "model-download-error",
                ErrorPayload { message: e },
            );
        }
    });
}

fn download_all_inner(app: &AppHandle, base_url: &str) -> Result<(), String> {
    let dir = models_data_dir(app)?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let base = base_url.trim_end_matches('/');

    for (artifact, dest_path) in artifact_paths(app)? {
        let url = format!("{}/{}.onnx", base, artifact.sha256_hex);
        download_one(app, &url, artifact, &dest_path)?;
    }
    ensure_models_verified(app)
}

fn emit_progress(app: &AppHandle, filename: &str, downloaded: u64, total: Option<u64>) {
    let _ = app.emit(
        "model-download-progress",
        DownloadProgress {
            filename: filename.to_string(),
            downloaded_bytes: downloaded,
            total_bytes: total,
        },
    );
}

fn download_one(
    app: &AppHandle,
    url: &str,
    artifact: ModelArtifact,
    dest_path: &Path,
) -> Result<(), String> {
    let name = dest_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("model.onnx");

    if dest_path.is_file() && file_sha256_hex(dest_path)? == artifact.sha256_hex {
        let len = dest_path.metadata().map(|m| m.len()).unwrap_or(0);
        emit_progress(app, name, len, Some(len));
        return Ok(());
    }

    let part_path = dest_path.with_extension("onnx.part");
    if part_path.exists() {
        fs::remove_file(&part_path).map_err(|e| e.to_string())?;
    }

    let resp = ureq::get(url)
        .call()
        .map_err(|e| format!("GET {}: {}", url, e))?;
    let status = resp.status();
    if status != 200 {
        return Err(format!(
            "GET {} failed with HTTP {} — cannot download model",
            url, status
        ));
    }

    let total = resp
        .header("Content-Length")
        .and_then(|s| s.parse::<u64>().ok());

    let mut reader = resp.into_reader();
    let mut file = File::create(&part_path).map_err(|e| e.to_string())?;
    let mut ctx = Context::new(&SHA256);
    let mut buf = [0u8; 16 * 1024];
    let mut downloaded: u64 = 0;

    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        ctx.update(&buf[..n]);
        file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
        downloaded += n as u64;
        emit_progress(app, name, downloaded, total);
    }

    let digest = ctx.finish();
    let hex_out: String = digest.as_ref().iter().map(|b| format!("{:02x}", b)).collect();
    if hex_out != artifact.sha256_hex {
        fs::remove_file(&part_path).ok();
        return Err(format!(
            "Downloaded file for {} has wrong hash (got {} expected {}). Deleted partial file.",
            name, hex_out, artifact.sha256_hex
        ));
    }

    drop(file);
    if dest_path.exists() {
        fs::remove_file(dest_path).map_err(|e| e.to_string())?;
    }
    fs::rename(&part_path, dest_path).map_err(|e| e.to_string())?;
    Ok(())
}
