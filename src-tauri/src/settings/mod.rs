use serde::{Deserialize, Serialize};

use crate::cv::camera::CameraSelection;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppTheme {
    System,
    Light,
    Dark,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::System
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NotificationStyle {
    /// Native dialog + standard title via Tauri notification plugin.
    Native,
    /// Shorter title for less intrusion (same delivery path).
    Compact,
}

impl Default for NotificationStyle {
    fn default() -> Self {
        Self::Native
    }
}

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
    #[serde(default)]
    pub clahe_face_preproc: bool,
    pub camera: Option<CameraSelection>,
    #[serde(default)]
    pub theme: AppTheme,
    #[serde(default)]
    pub start_at_login: bool,
    #[serde(default)]
    pub notification_style: NotificationStyle,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sensitivity: Sensitivity::Medium,
            cooldown_sec: 30,
            debug_overlay: false,
            clahe_face_preproc: false,
            camera: None,
            theme: AppTheme::default(),
            start_at_login: false,
            notification_style: NotificationStyle::default(),
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
    pub clahe_face_preproc: Option<bool>,
    pub camera: Option<CameraSelection>,
    pub theme: Option<AppTheme>,
    pub start_at_login: Option<bool>,
    pub notification_style: Option<NotificationStyle>,
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
        if let Some(clahe) = self.clahe_face_preproc {
            settings.clahe_face_preproc = clahe;
        }
        if let Some(camera) = self.camera {
            settings.camera = Some(camera);
        }
        if let Some(theme) = self.theme {
            settings.theme = theme;
        }
        if let Some(start_at_login) = self.start_at_login {
            settings.start_at_login = start_at_login;
        }
        if let Some(notification_style) = self.notification_style {
            settings.notification_style = notification_style;
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
