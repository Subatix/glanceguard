use serde::{Deserialize, Serialize};

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::{query, Camera};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum CameraSelection {
    Index(u32),
    StableId(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    pub id: CameraSelection,
    pub name: String,
    pub description: String,
}

pub fn list_cameras() -> Result<Vec<CameraInfo>, String> {
    let backend = nokhwa::native_api_backend()
        .ok_or_else(|| "No native camera backend available".to_string())?;
    let cameras = query(backend).map_err(|e| e.to_string())?;

    let results = cameras
        .into_iter()
        .map(|camera| {
            let id = match camera.index().clone() {
                CameraIndex::Index(idx) => CameraSelection::Index(idx),
                CameraIndex::String(value) => CameraSelection::StableId(value),
            };
            CameraInfo {
                id,
                name: camera.human_name(),
                description: camera.description().to_string(),
            }
        })
        .collect();

    Ok(results)
}

fn to_camera_index(selection: &CameraSelection) -> CameraIndex {
    match selection {
        CameraSelection::Index(index) => CameraIndex::Index(*index),
        CameraSelection::StableId(id) => CameraIndex::String(id.clone()),
    }
}

pub fn open_camera(selection: &CameraSelection) -> Result<Camera, String> {
    let index = to_camera_index(selection);
    let requests = [
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            Resolution::new(1280, 720),
            FrameFormat::MJPEG,
            30,
        ))),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::None),
    ];

    for request in requests {
        if let Ok(camera) = Camera::new(index.clone(), request) {
            return Ok(camera);
        }
    }

    Err("Unable to open camera with available formats".to_string())
}
