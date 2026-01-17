use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use tauri::{AppHandle, Emitter};

use crate::cv::camera::open_camera;
use crate::cv::config::{load_detector_config, load_embedder_config};
use crate::cv::detector::FaceDetector;
use crate::cv::embedder::FaceEmbedder;
use crate::cv::matching::cosine_similarity;
use crate::cv::scoring::compute_observer_score;
use crate::cv::types::{AlertEvent, DebugFace, ErrorEvent, FrameEvent, OwnerProfile};
use crate::settings::{observer_threshold, owner_match_threshold, Settings};
use crate::state::alert_state::{AlertState, MonitorState};

pub struct MonitorHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

impl MonitorHandle {
    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::SeqCst);
        if let Some(handle) = self.join.take() {
            let _ = handle.join();
        }
    }
}

pub fn start_monitoring(
    app: AppHandle,
    settings: Arc<Mutex<Settings>>,
    owner: Arc<Mutex<Option<OwnerProfile>>>,
) -> Result<MonitorHandle, String> {
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let app_clone = app.clone();
    let settings_clone = settings.clone();
    let owner_clone = owner.clone();

    let join = thread::spawn(move || {
        if let Err(err) = run_loop(app_clone, settings_clone, owner_clone, stop_clone) {
            let _ = emit_error(&app, &err);
        }
    });

    Ok(MonitorHandle {
        stop,
        join: Some(join),
    })
}

fn run_loop(
    app: AppHandle,
    settings: Arc<Mutex<Settings>>,
    owner: Arc<Mutex<Option<OwnerProfile>>>,
    stop: Arc<AtomicBool>,
) -> Result<(), String> {
    let selection = {
        let settings_guard = settings
            .lock()
            .map_err(|_| "Settings lock poisoned".to_string())?;
        settings_guard
            .camera
            .clone()
            .ok_or_else(|| "Select a camera before monitoring".to_string())?
    };
    let detector_config = load_detector_config(&app)?;
    let embedder_config = load_embedder_config(&app)?;
    let mut detector = FaceDetector::new(&app, detector_config)?;
    let mut embedder = FaceEmbedder::new(&app, embedder_config)?;

    let mut camera = open_camera(&selection)?;
    camera.open_stream().map_err(|e| e.to_string())?;

    let mut alert_state = AlertState::new();

    let target_interval = Duration::from_millis(66);
    
    while !stop.load(Ordering::SeqCst) {
        let frame_start = Instant::now();
        let frame = camera.frame().map_err(|e| e.to_string())?;
        let image = frame
            .decode_image::<nokhwa::pixel_format::RgbFormat>()
            .map_err(|e| e.to_string())?;
        let rgb = image::DynamicImage::ImageRgb8(image).to_rgb8();

        let detections = detector.detect(&rgb)?;
        let mut faces = detections;
        faces.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        faces.truncate(3);

    let owner_profile = owner
        .lock()
        .map_err(|_| "Owner lock poisoned".to_string())?
        .clone()
        .ok_or_else(|| "Owner profile missing during monitoring".to_string())?;

        let mut similarities = Vec::with_capacity(faces.len());
        let mut owner_index: Option<usize> = None;
        let mut owner_similarity: Option<f32> = None;

        for (idx, face) in faces.iter().enumerate() {
            let crop = crop_face(&rgb, &face.bbox)?;
            let embedding = embedder.embed(&crop)?;
            let similarity = cosine_similarity(&embedding, &owner_profile.embedding)?;
            similarities.push(similarity);
            if owner_similarity.map_or(true, |best| similarity > best) {
                owner_similarity = Some(similarity);
                owner_index = Some(idx);
            }
        }

        let owner_threshold = owner_match_threshold();
        let owner_id = match owner_similarity {
            Some(sim) if sim >= owner_threshold => owner_index,
            _ => None,
        };

        let settings_guard = settings
            .lock()
            .map_err(|_| "Settings lock poisoned".to_string())?;
        let sensitivity = settings_guard.sensitivity.clone();
        let cooldown_sec = settings_guard.cooldown_sec;
        let debug_overlay = settings_guard.debug_overlay;
        drop(settings_guard);

        let mut max_score = None;
        let mut debug_faces = Vec::new();

        for (idx, face) in faces.iter().enumerate() {
            let similarity = similarities.get(idx).copied();

            let (label, observer_score) = if Some(idx) == owner_id {
                ("owner".to_string(), None)
            } else {
                let owner_bbox = owner_id.and_then(|oid| faces.get(oid)).map(|f| &f.bbox);
                let score = compute_observer_score(
                    face,
                    owner_bbox,
                    similarity,
                    rgb.width(),
                    rgb.height(),
                    sensitivity.clone(),
                );
                if max_score.map_or(true, |best| score > best) {
                    max_score = Some(score);
                }
                ("observer".to_string(), Some(score))
            };

            debug_faces.push(DebugFace {
                id: idx,
                bbox: face.bbox.clone(),
                label,
                similarity,
                observer_score,
            });
        }

        let now = Instant::now();
        let threshold = observer_threshold(&sensitivity);
        let update = alert_state.update(
            max_score,
            threshold,
            now,
            Duration::from_secs(cooldown_sec),
            !faces.is_empty(),
        );
        if update.triggered {
            if let Some(score) = max_score {
                emit_alert(&app, score, cooldown_sec)?;
            }
        }

        let state_label = match update.state {
            MonitorState::Idle => "idle",
            MonitorState::Monitoring => "monitoring",
            MonitorState::Alert => "alert",
            MonitorState::Cooldown => "cooldown",
        };

        if debug_overlay {
            let mut image_buffer = Vec::new();
            // Encode as JPEG with quality 50 to save bandwidth
            let _ = image::DynamicImage::ImageRgb8(rgb.clone())
                .write_to(&mut std::io::Cursor::new(&mut image_buffer), image::ImageFormat::Jpeg);

            let frame_event = FrameEvent {
                frame_width: rgb.width(),
                frame_height: rgb.height(),
                faces: debug_faces,
                observer_score: max_score,
                state: state_label.to_string(),
                image: Some(image_buffer),
            };
            let _ = app.emit("cv:frame", frame_event);
        } else {
            let frame_event = FrameEvent {
                frame_width: rgb.width(),
                frame_height: rgb.height(),
                faces: Vec::new(),
                observer_score: max_score,
                state: state_label.to_string(),
                image: None,
            };
            let _ = app.emit("cv:frame", frame_event);
        }

        let elapsed = frame_start.elapsed();
        if elapsed < target_interval {
            thread::sleep(target_interval - elapsed);
        }
    }

    camera.stop_stream().map_err(|e| e.to_string())?;
    Ok(())
}

fn emit_alert(app: &AppHandle, score: f32, cooldown_sec: u64) -> Result<(), String> {
    let event = AlertEvent {
        score,
        reason: "Observer score exceeded threshold".to_string(),
        cooldown_sec,
    };
    app.emit("cv:alert", event).map_err(|e| e.to_string())
}

fn emit_error(app: &AppHandle, message: &str) -> Result<(), String> {
    let event = ErrorEvent {
        message: message.to_string(),
    };
    app.emit("cv:error", event).map_err(|e| e.to_string())
}

fn crop_face(image: &image::RgbImage, bbox: &crate::cv::types::BoundingBox) -> Result<image::RgbImage, String> {
    let x1 = bbox.x.max(0.0).floor() as u32;
    let y1 = bbox.y.max(0.0).floor() as u32;
    let x2 = (bbox.x + bbox.width).min(image.width() as f32).ceil() as u32;
    let y2 = (bbox.y + bbox.height).min(image.height() as f32).ceil() as u32;

    if x2 <= x1 || y2 <= y1 {
        return Err("Invalid face crop".to_string());
    }

    Ok(crate::cv::preprocess::crop_rgb(image, x1, y1, x2 - x1, y2 - y1))
}
