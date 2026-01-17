use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone)]
pub struct FaceDetection {
    pub bbox: BoundingBox,
    pub score: f32,
    pub landmarks: [Point; 5],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerProfile {
    pub embedding: Vec<f32>,
    pub model: OwnerModelInfo,
    pub created_at_epoch: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OwnerModelInfo {
    pub name: String,
    pub input_width: u32,
    pub input_height: u32,
    pub normalization: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DebugFace {
    pub id: usize,
    pub bbox: BoundingBox,
    pub label: String,
    pub similarity: Option<f32>,
    pub observer_score: Option<f32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FrameEvent {
    pub frame_width: u32,
    pub frame_height: u32,
    pub faces: Vec<DebugFace>,
    pub observer_score: Option<f32>,
    pub state: String,
    pub image: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlertEvent {
    pub score: f32,
    pub reason: String,
    pub cooldown_sec: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorEvent {
    pub message: String,
}
