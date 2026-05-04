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

fn embedding_samples_default() -> Vec<Vec<f32>> {
    Vec::new()
}

fn personal_threshold_default() -> Option<f32> {
    None
}

/// Persisted owner identity. Legacy saves omit `embedding_samples` / `personal_threshold`;
/// those profiles fail validation and are cleared on load (see `storage::load_owner_profile`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OwnerProfile {
    /// Mean embedding (L2-normalized) across passing enrollment frames.
    pub embedding: Vec<f32>,
    #[serde(default = "embedding_samples_default")]
    pub embedding_samples: Vec<Vec<f32>>,
    /// Cosine threshold from enrollment self-pair stats (`μ_self - 3σ_self`, clamped).
    #[serde(default = "personal_threshold_default")]
    pub personal_threshold: Option<f32>,
    pub model: OwnerModelInfo,
    pub created_at_epoch: u64,
}

impl OwnerProfile {
    /// Rejects legacy v0.1 disk format (before Phase 4) and corrupt payloads.
    pub fn validate_enrollment_complete(&self) -> Result<(), String> {
        if self.embedding.is_empty() {
            return Err("Owner embedding is empty.".into());
        }
        if self.embedding_samples.len() < 3 && self.embedding_samples.len() != 1 {
            return Err(
                "Owner enrollment data is incomplete (expected multi-frame enrollment or a single photo). Clear the owner and enroll again."
                    .into(),
            );
        }
        let t = self.personal_threshold.ok_or_else(|| {
            "Owner profile is missing calibrated threshold. Clear the owner and enroll again."
                .to_string()
        })?;
        if !t.is_finite() || t <= 0.0 || t > 1.0 {
            return Err(
                "Owner profile threshold is invalid. Clear the owner and enroll again.".into(),
            );
        }
        Ok(())
    }
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

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorStoppedEvent {
    pub reason: String,
}
