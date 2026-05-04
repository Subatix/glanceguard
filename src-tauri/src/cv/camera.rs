use serde::{Deserialize, Serialize};

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType, Resolution,
};
use nokhwa::{query, Camera};

#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

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

/// Map AVFoundation video authorization to an actionable message before opening the device.
pub fn ensure_camera_video_permission() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = unsafe { avfoundation_video_authorization_status() };
        if status == 1 || status == 2 {
            return Err(
                "Camera access is denied for GlanceGuard. Enable the camera in System Settings → Privacy & Security → Camera, then relaunch the app.".into(),
            );
        }
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[link(name = "AVFoundation", kind = "framework")]
extern "C" {
    static AVMediaTypeVideo: *const objc::runtime::Object;
}

#[cfg(target_os = "macos")]
unsafe fn avfoundation_video_authorization_status() -> i32 {
    let cls = class!(AVCaptureDevice);
    let status: isize = msg_send![cls, authorizationStatusForMediaType: AVMediaTypeVideo];
    status as i32
}

pub fn open_camera(selection: &CameraSelection) -> Result<Camera, String> {
    ensure_camera_video_permission()?;
    let index = resolve_camera_index(selection)?;
    let res = Resolution::new(960, 540);
    let fps = 30u32;
    let requests = [
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            res,
            FrameFormat::MJPEG,
            fps,
        ))),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            res,
            FrameFormat::YUYV,
            fps,
        ))),
        RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(CameraFormat::new(
            res,
            FrameFormat::NV12,
            fps,
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

fn resolve_camera_index(selection: &CameraSelection) -> Result<CameraIndex, String> {
    let backend = nokhwa::native_api_backend()
        .ok_or_else(|| "No native camera backend available".to_string())?;
    let cameras = query(backend).map_err(|e| e.to_string())?;

    for (position, camera) in cameras.iter().enumerate() {
        let camera_index = camera.index();
        if camera_matches_selection(selection, camera_index, position) {
            // On macOS AVFoundation, opening by string ID can resolve to the default
            // camera. Use the enumerated numeric position to ensure the selected device.
            if cfg!(target_os = "macos") && matches!(camera_index, CameraIndex::String(_)) {
                let numeric = u32::try_from(position)
                    .map_err(|_| format!("Camera position {position} is out of range"))?;
                return Ok(CameraIndex::Index(numeric));
            }
            return Ok(camera_index.clone());
        }
    }

    Err(format!(
        "Selected camera is unavailable: {}",
        describe_selection(selection)
    ))
}

fn camera_matches_selection(
    selection: &CameraSelection,
    camera_index: &CameraIndex,
    position: usize,
) -> bool {
    match selection {
        CameraSelection::Index(selected) => match camera_index {
            CameraIndex::Index(index) => index == selected,
            CameraIndex::String(_) => {
                usize::try_from(*selected).map_or(false, |value| value == position)
            }
        },
        CameraSelection::StableId(id) => {
            matches!(camera_index, CameraIndex::String(value) if value == id)
        }
    }
}

fn describe_selection(selection: &CameraSelection) -> String {
    match selection {
        CameraSelection::Index(index) => format!("index:{index}"),
        CameraSelection::StableId(id) => format!("stable:{id}"),
    }
}
