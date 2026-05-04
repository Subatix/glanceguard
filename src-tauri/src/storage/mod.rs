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

const LEGACY_KEYRING_SERVICE: &str = "com.screenpeek.alert";
const KEYRING_OWNER_PROFILE_USER: &str = "owner_profile_key";

fn current_keyring_service(app: &AppHandle) -> String {
    app.config().identifier.clone()
}

/// Copies the owner-profile encryption secret from the pre-rebrand bundle ID if present.
fn migrate_legacy_keyring_secret(app: &AppHandle, service: &str) -> Result<(), String> {
    let keyring = app.keyring();
    if keyring
        .get_secret(service, KEYRING_OWNER_PROFILE_USER)
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(());
    }
    let Some(secret) = keyring
        .get_secret(LEGACY_KEYRING_SERVICE, KEYRING_OWNER_PROFILE_USER)
        .map_err(|e| e.to_string())?
    else {
        return Ok(());
    };
    keyring
        .set_secret(service, KEYRING_OWNER_PROFILE_USER, &secret)
        .map_err(|e| e.to_string())?;
    Ok(())
}

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
    if profile.validate_enrollment_complete().is_err() {
        clear_owner_profile(app)?;
        return Ok(None);
    }
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
    embedding_samples: Vec<Vec<f32>>,
    personal_threshold: f32,
    model_info: OwnerModelInfo,
) -> OwnerProfile {
    let created_at_epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    OwnerProfile {
        embedding,
        embedding_samples,
        personal_threshold: Some(personal_threshold),
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
    let base = app.path().app_config_dir().map_err(|e| e.to_string())?;
    Ok(base.join(SETTINGS_FILE))
}

fn owner_profile_path(app: &AppHandle) -> Result<PathBuf, String> {
    let base = app.path().app_data_dir().map_err(|e| e.to_string())?;
    Ok(base.join(OWNER_PROFILE_FILE))
}

fn get_or_create_encryption_key(app: &AppHandle) -> Result<Vec<u8>, String> {
    let service = current_keyring_service(app);
    migrate_legacy_keyring_secret(app, &service)?;
    if let Some(secret) = app
        .keyring()
        .get_secret(&service, KEYRING_OWNER_PROFILE_USER)
        .map_err(|e| e.to_string())?
    {
        return Ok(secret);
    }

    let rng = SystemRandom::new();
    let mut key_bytes = vec![0u8; 32];
    rng.fill(&mut key_bytes)
        .map_err(|_| "Failed to generate key".to_string())?;

    app.keyring()
        .set_secret(&service, KEYRING_OWNER_PROFILE_USER, &key_bytes)
        .map_err(|e| e.to_string())?;

    Ok(key_bytes)
}

fn get_encryption_key(app: &AppHandle) -> Result<Vec<u8>, String> {
    let service = current_keyring_service(app);
    migrate_legacy_keyring_secret(app, &service)?;
    let secret = app
        .keyring()
        .get_secret(&service, KEYRING_OWNER_PROFILE_USER)
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

    let unbound =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| "Invalid encryption key".to_string())?;
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
    let unbound =
        UnboundKey::new(&AES_256_GCM, key).map_err(|_| "Invalid encryption key".to_string())?;
    let key = LessSafeKey::new(unbound);

    let mut in_out = payload.ciphertext.clone();
    let plaintext = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Failed to decrypt owner profile".to_string())?;
    Ok(plaintext.to_vec())
}

#[cfg(test)]
mod crypto_tests {
    use super::{decrypt, encrypt, EncryptedPayload};
    use crate::cv::types::{OwnerModelInfo, OwnerProfile};

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let key = [9u8; 32];
        let plaintext = b"payload-bytes";
        let enc = encrypt(&key, plaintext).unwrap();
        let out = decrypt(&key, &enc).unwrap();
        assert_eq!(out.as_slice(), plaintext);
    }

    #[test]
    fn decrypt_rejects_tampered_ciphertext() {
        let key = [3u8; 32];
        let mut enc = encrypt(&key, b"secret").unwrap();
        if let Some(b) = enc.ciphertext.first_mut() {
            *b ^= 0xFF;
        }
        assert!(decrypt(&key, &enc).is_err());
    }

    #[test]
    fn owner_profile_roundtrip_through_encrypt_json() {
        let key = [21u8; 32];
        let profile = OwnerProfile {
            embedding: vec![0.1, 0.2, 0.3],
            embedding_samples: vec![
                vec![1.0, 0.0, 0.0],
                vec![0.99, 0.01, 0.0],
                vec![0.98, 0.02, 0.0],
            ],
            personal_threshold: Some(0.55),
            model: OwnerModelInfo {
                name: "w600k_mbf.onnx".into(),
                input_width: 112,
                input_height: 112,
                normalization: "arcface".into(),
            },
            created_at_epoch: 42,
        };
        let plain = serde_json::to_vec(&profile).unwrap();
        let enc = encrypt(&key, &plain).unwrap();
        let json = serde_json::to_vec(&enc).unwrap();
        let loaded: EncryptedPayload = serde_json::from_slice(&json).unwrap();
        let round = decrypt(&key, &loaded).unwrap();
        let back: OwnerProfile = serde_json::from_slice(&round).unwrap();
        assert_eq!(back.embedding, profile.embedding);
        assert_eq!(back.embedding_samples.len(), profile.embedding_samples.len());
        assert_eq!(back.personal_threshold, profile.personal_threshold);
        assert_eq!(back.model.name, profile.model.name);
    }
}
