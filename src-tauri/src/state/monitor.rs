use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use image::codecs::jpeg::JpegEncoder;
use tauri::{AppHandle, Emitter};

use crate::cv::camera::open_camera;
use crate::cv::config::{load_detector_config, load_embedder_config};
use crate::cv::detector::FaceDetector;
use crate::cv::embedder::FaceEmbedder;
use crate::cv::matching::cosine_similarity;
use crate::cv::scoring::compute_observer_score;
use crate::cv::types::{AlertEvent, DebugFace, ErrorEvent, FaceDetection, FrameEvent, OwnerProfile};
use crate::settings::{observer_threshold, owner_match_threshold, Settings};
use crate::state::alert_state::{AlertState, MonitorState};

pub struct MonitorHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

const TARGET_FRAME_INTERVAL: Duration = Duration::from_millis(33);
const MAX_TRACKED_FACES: usize = 2;
const SIMILARITY_CACHE_TTL: Duration = Duration::from_millis(180);
const DEBUG_JPEG_QUALITY: u8 = 35;

#[derive(Clone)]
struct FaceSimilaritySample {
    bbox: crate::cv::types::BoundingBox,
    similarity: f32,
    sampled_at: Instant,
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
    let mut similarity_cache: Vec<FaceSimilaritySample> = Vec::new();
    
    while !stop.load(Ordering::SeqCst) {
        let frame_start = Instant::now();
        let frame = camera.frame().map_err(|e| e.to_string())?;
        let image = frame
            .decode_image::<nokhwa::pixel_format::RgbFormat>()
            .map_err(|e| e.to_string())?;
        let rgb = image::DynamicImage::ImageRgb8(image).to_rgb8();

        let mut faces = detector.detect(&rgb)?;
        faces.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        faces = suppress_nested_detections(faces);
        faces.truncate(MAX_TRACKED_FACES);

        let owner_profile = owner
            .lock()
            .map_err(|_| "Owner lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "Owner profile missing during monitoring".to_string())?;

        let mut similarities = Vec::with_capacity(faces.len());
        let mut next_similarity_cache = Vec::with_capacity(faces.len());
        let mut owner_index: Option<usize> = None;
        let mut owner_similarity: Option<f32> = None;
        let similarity_sampled_at = Instant::now();

        for (idx, face) in faces.iter().enumerate() {
            let similarity = if let Some(cached) =
                lookup_cached_similarity(&similarity_cache, &face.bbox, similarity_sampled_at)
            {
                cached
            } else {
                let crop = crop_face(&rgb, &face.bbox)?;
                let embedding = embedder.embed(&crop)?;
                cosine_similarity(&embedding, &owner_profile.embedding)?
            };

            similarities.push(similarity);
            next_similarity_cache.push(FaceSimilaritySample {
                bbox: face.bbox.clone(),
                similarity,
                sampled_at: similarity_sampled_at,
            });

            if owner_similarity.map_or(true, |best| similarity > best) {
                owner_similarity = Some(similarity);
                owner_index = Some(idx);
            }
        }
        similarity_cache = next_similarity_cache;

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

            let is_owner = Some(idx) == owner_id;
            let resembles_owner = similarity.map_or(false, |sim| sim >= owner_threshold);
            // Suppress duplicate detections for the same physical face so one person
            // is not scored as both owner and observer.
            let duplicate_owner_detection = !is_owner
                && owner_id
                    .and_then(|oid| faces.get(oid))
                    .map(|owner_face| is_duplicate_owner_detection(&face.bbox, &owner_face.bbox))
                    .unwrap_or(false);

            let (label, observer_score) = if is_owner || resembles_owner || duplicate_owner_detection {
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
            // Encode debug frame with lower quality to keep UI throughput high.
            let mut encoder = JpegEncoder::new_with_quality(&mut image_buffer, DEBUG_JPEG_QUALITY);
            encoder
                .encode(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| e.to_string())?;

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
        if elapsed < TARGET_FRAME_INTERVAL {
            thread::sleep(TARGET_FRAME_INTERVAL - elapsed);
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

fn bbox_iou(a: &crate::cv::types::BoundingBox, b: &crate::cv::types::BoundingBox) -> f32 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let union = a.width * a.height + b.width * b.height - inter;
    if union <= 0.0 { 0.0 } else { inter / union }
}

fn suppress_nested_detections(faces: Vec<FaceDetection>) -> Vec<FaceDetection> {
    let mut kept: Vec<FaceDetection> = Vec::with_capacity(faces.len());

    'candidate: for face in faces {
        for existing in &kept {
            if is_duplicate_bbox(&face.bbox, &existing.bbox) {
                continue 'candidate;
            }
        }
        kept.push(face);
    }

    kept
}

fn lookup_cached_similarity(
    cache: &[FaceSimilaritySample],
    bbox: &crate::cv::types::BoundingBox,
    now: Instant,
) -> Option<f32> {
    let mut best_match: Option<(f32, f32)> = None;

    for sample in cache {
        if now.duration_since(sample.sampled_at) > SIMILARITY_CACHE_TTL {
            continue;
        }
        if !is_duplicate_bbox(bbox, &sample.bbox) {
            continue;
        }
        let score = bbox_overlap_score(bbox, &sample.bbox);
        if best_match.map_or(true, |(best, _)| score > best) {
            best_match = Some((score, sample.similarity));
        }
    }

    best_match.map(|(_, similarity)| similarity)
}

fn is_duplicate_owner_detection(
    face_bbox: &crate::cv::types::BoundingBox,
    owner_bbox: &crate::cv::types::BoundingBox,
) -> bool {
    is_duplicate_bbox(face_bbox, owner_bbox)
}

fn is_duplicate_bbox(
    a: &crate::cv::types::BoundingBox,
    b: &crate::cv::types::BoundingBox,
) -> bool {
    let iou = bbox_iou(a, b);
    let ios = bbox_ios(a, b);
    if iou >= 0.18 || ios >= 0.72 {
        return true;
    }

    let center_distance_ratio = bbox_center_distance_ratio(a, b);
    let area_similarity = bbox_area_similarity(a, b);
    center_distance_ratio <= 0.20 && area_similarity >= 0.05
}

fn bbox_area_similarity(
    a: &crate::cv::types::BoundingBox,
    b: &crate::cv::types::BoundingBox,
) -> f32 {
    let area_a = (a.width * a.height).max(1.0);
    let area_b = (b.width * b.height).max(1.0);
    if area_a > area_b {
        area_b / area_a
    } else {
        area_a / area_b
    }
}

fn bbox_center_distance_ratio(
    a: &crate::cv::types::BoundingBox,
    b: &crate::cv::types::BoundingBox,
) -> f32 {
    let (ax, ay) = bbox_center(a);
    let (bx, by) = bbox_center(b);
    let center_distance = ((ax - bx).powi(2) + (ay - by).powi(2)).sqrt();
    let reference_diag =
        (((a.width.powi(2) + a.height.powi(2)).sqrt() + (b.width.powi(2) + b.height.powi(2)).sqrt())
            * 0.5)
            .max(1.0);
    center_distance / reference_diag
}

fn bbox_center(bbox: &crate::cv::types::BoundingBox) -> (f32, f32) {
    (bbox.x + bbox.width * 0.5, bbox.y + bbox.height * 0.5)
}

fn bbox_ios(a: &crate::cv::types::BoundingBox, b: &crate::cv::types::BoundingBox) -> f32 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let min_area = (a.width * a.height).min(b.width * b.height);
    if min_area <= 0.0 { 0.0 } else { inter / min_area }
}

fn bbox_overlap_score(
    a: &crate::cv::types::BoundingBox,
    b: &crate::cv::types::BoundingBox,
) -> f32 {
    bbox_iou(a, b).max(bbox_ios(a, b))
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
