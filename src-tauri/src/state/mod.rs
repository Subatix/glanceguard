use std::sync::{Arc, Mutex};

use tauri::AppHandle;

use crate::cv::types::OwnerProfile;
use crate::settings::Settings;
use crate::storage;

pub mod monitor;
pub mod alert_state;

pub struct AppState {
    pub settings: Arc<Mutex<Settings>>,
    pub owner: Arc<Mutex<Option<OwnerProfile>>>,
    pub monitor: Mutex<Option<monitor::MonitorHandle>>,
}

impl AppState {
    pub fn initialize(app: &AppHandle) -> Result<Self, String> {
        let settings = storage::load_settings(app)?;
        let owner = storage::load_owner_profile(app)?;

        Ok(Self {
            settings: Arc::new(Mutex::new(settings)),
            owner: Arc::new(Mutex::new(owner)),
            monitor: Mutex::new(None),
        })
    }
}
