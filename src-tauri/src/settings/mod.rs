use serde::{Deserialize, Serialize};

use crate::cv::camera::CameraSelection;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sensitivity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub sensitivity: Sensitivity,
    pub cooldown_sec: u64,
    pub debug_overlay: bool,
    pub camera: Option<CameraSelection>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: Sensitivity::Medium,
            cooldown_sec: 30,
            debug_overlay: false,
            camera: None,
        }
    }
}

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.cooldown_sec, 15 | 30 | 60) {
            return Err("Cooldown must be 15, 30, or 60 seconds".to_string());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsUpdate {
    pub sensitivity: Option<Sensitivity>,
    pub cooldown_sec: Option<u64>,
    pub debug_overlay: Option<bool>,
    pub camera: Option<CameraSelection>,
}

impl SettingsUpdate {
    pub fn apply(self, settings: &mut Settings) -> Result<(), String> {
        if let Some(sensitivity) = self.sensitivity {
            settings.sensitivity = sensitivity;
        }
        if let Some(cooldown_sec) = self.cooldown_sec {
            if !matches!(cooldown_sec, 15 | 30 | 60) {
                return Err("Cooldown must be 15, 30, or 60 seconds".to_string());
            }
            settings.cooldown_sec = cooldown_sec;
        }
        if let Some(debug_overlay) = self.debug_overlay {
            settings.debug_overlay = debug_overlay;
        }
        if let Some(camera) = self.camera {
            settings.camera = Some(camera);
        }
        Ok(())
    }
}

pub fn observer_threshold(sensitivity: &Sensitivity) -> f32 {
    match sensitivity {
        Sensitivity::Low => 0.75,
        Sensitivity::Medium => 0.6,
        Sensitivity::High => 0.5,
    }
}

pub fn owner_match_threshold() -> f32 {
    0.62
}
