use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_keyring::KeyringExt;

use crate::cv::types::{OwnerModelInfo, OwnerProfile};
use crate::settings::Settings;

const OWNER_PROFILE_FILE: &str = "owner_profile.enc.json";
const SETTINGS_FILE: &str = "settings.json";

pub fn load_settings(app: &AppHandle) -> Result<Settings, String> {
    let path = settings_path(app)?;
    if !path.exists() {
        return Ok(Settings::default());
    }
    let data = fs::read(&path).map_err(|e| e.to_string())?;
    let settings: Settings = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    settings.validate()?;
    Ok(settings)
}

pub fn save_settings(app: &AppHandle, settings: &Settings) -> Result<(), String> {
    settings.validate()?;
    let path = settings_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let data = serde_json::to_vec_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

pub fn load_owner_profile(app: &AppHandle) -> Result<Option<OwnerProfile>, String> {
    let path = owner_profile_path(app)?;
    if !path.exists() {
        return Ok(None);
    }
    let data = fs::read(&path).map_err(|e| e.to_string())?;
    let encrypted: EncryptedPayload = serde_json::from_slice(&data).map_err(|e| e.to_string())?;
    let key = get_encryption_key(app)?;
    let plaintext = decrypt(&key, &encrypted)?;
    let profile: OwnerProfile = serde_json::from_slice(&plaintext).map_err(|e| e.to_string())?;
    Ok(Some(profile))
}

pub fn save_owner_profile(app: &AppHandle, profile: &OwnerProfile) -> Result<(), String> {
    let path = owner_profile_path(app)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let key = get_or_create_encryption_key(app)?;
    let plaintext = serde_json::to_vec(profile).map_err(|e| e.to_string())?;
    let encrypted = encrypt(&key, &plaintext)?;
    let data = serde_json::to_vec_pretty(&encrypted).map_err(|e| e.to_string())?;
    fs::write(path, data).map_err(|e| e.to_string())
}

pub fn clear_owner_profile(app: &AppHandle) -> Result<(), String> {
    let path = owner_profile_path(app)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn new_owner_profile(
    embedding: Vec<f32>,
    model_info: OwnerModelInfo,
) -> OwnerProfile {
    let created_at_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    OwnerProfile {
        embedding,
        model: model_info,
        created_at_epoch,
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct EncryptedPayload {
    nonce: Vec<u8>,
    ciphertext: Vec<u8>,
}

fn settings_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?;
    Ok(base.join(SETTINGS_FILE))
}

fn owner_profile_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?;
    Ok(base.join(OWNER_PROFILE_FILE))
}

fn get_or_create_encryption_key(app: &AppHandle) -> Result<Vec<u8>, String> {
    let service = app.config().identifier.clone();
    let user = "owner_profile_key";
    if let Some(secret) = app
        .keyring()
        .get_secret(&service, user)
        .map_err(|e| e.to_string())?
    {
        return Ok(secret);
    }

    let rng = SystemRandom::new();
    let mut key_bytes = vec![0u8; 32];
    rng.fill(&mut key_bytes)
        .map_err(|_| "Failed to generate key".to_string())?;

    app.keyring()
        .set_secret(&service, user, &key_bytes)
        .map_err(|e| e.to_string())?;

    Ok(key_bytes)
}

fn get_encryption_key(app: &AppHandle) -> Result<Vec<u8>, String> {
    let service = app.config().identifier.clone();
    let user = "owner_profile_key";
    let secret = app
        .keyring()
        .get_secret(&service, user)
        .map_err(|e| e.to_string())?;
    secret.ok_or_else(|| "Encryption key missing".to_string())
}

fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<EncryptedPayload, String> {
    if key.len() != 32 {
        return Err("Encryption key must be 32 bytes".to_string());
    }

    let rng = SystemRandom::new();
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rng.fill(&mut nonce_bytes)
        .map_err(|_| "Failed to generate nonce".to_string())?;
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

    let unbound = UnboundKey::new(&AES_256_GCM, key).map_err(|_| "Invalid encryption key".to_string())?;
    let key = LessSafeKey::new(unbound);
    let mut in_out = plaintext.to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Failed to encrypt owner profile".to_string())?;

    Ok(EncryptedPayload {
        nonce: nonce_bytes.to_vec(),
        ciphertext: in_out,
    })
}

fn decrypt(key: &[u8], payload: &EncryptedPayload) -> Result<Vec<u8>, String> {
    if key.len() != 32 {
        return Err("Encryption key must be 32 bytes".to_string());
    }
    if payload.nonce.len() != NONCE_LEN {
        return Err("Invalid nonce length".to_string());
    }

    let nonce = Nonce::try_assume_unique_for_key(&payload.nonce)
        .map_err(|_| "Invalid nonce".to_string())?;
    let unbound = UnboundKey::new(&AES_256_GCM, key).map_err(|_| "Invalid encryption key".to_string())?;
    let key = LessSafeKey::new(unbound);

    let mut in_out = payload.ciphertext.clone();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Failed to decrypt owner profile".to_string())?;
    Ok(plaintext.to_vec())
}
