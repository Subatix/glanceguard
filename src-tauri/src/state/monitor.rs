use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use image::codecs::jpeg::JpegEncoder;
use image::RgbImage;
use tauri::{AppHandle, Emitter};

use crate::cv::camera::open_camera;
use crate::cv::config::{load_detector_config, load_embedder_config};
use crate::cv::detector::FaceDetector;
use crate::cv::embedder::FaceEmbedder;
use crate::cv::matching::{max_cosine_vs_samples, owner_cosine_threshold};
use crate::cv::preprocess::{align_face_5pt, maybe_apply_clahe_luminance};
use crate::cv::quality::pass_quality_gate;
use crate::cv::scoring::compute_observer_score;
use crate::cv::tracker::{FaceTracker, StableFaceLabel};
use crate::cv::types::{
    AlertEvent, DebugFace, ErrorEvent, FaceDetection, FrameEvent, MonitorStoppedEvent, OwnerProfile,
};
use crate::settings::{observer_threshold, owner_match_threshold, Settings};
use crate::state::alert_state::{AlertState, MonitorState};

pub struct MonitorHandle {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
}

const TARGET_FRAME_INTERVAL: Duration = Duration::from_millis(33);
const MAX_TRACKED_FACES: usize = 2;
const DEBUG_JPEG_QUALITY: u8 = 35;
const CONSECUTIVE_ERROR_STOP: u32 = 5;

fn require_matching_embedder_model(
    profile: &OwnerProfile,
    embedder_model_file: &str,
) -> Result<(), String> {
    if profile.model.name != embedder_model_file {
        return Err(format!(
            "Owner was enrolled with embedder `{}`, but the app is configured for `{}`. Clear the owner profile and enroll again.",
            profile.model.name, embedder_model_file
        ));
    }
    Ok(())
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
        monitor_worker(app_clone, settings_clone, owner_clone, stop_clone);
    });

    Ok(MonitorHandle {
        stop,
        join: Some(join),
    })
}

fn monitor_worker(
    app: AppHandle,
    settings: Arc<Mutex<Settings>>,
    owner: Arc<Mutex<Option<OwnerProfile>>>,
    stop: Arc<AtomicBool>,
) {
    let session = (|| -> Result<(), String> {
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
        let mut embedder = FaceEmbedder::new(&app, embedder_config.clone())?;

        {
            let owner_guard = owner
                .lock()
                .map_err(|_| "Owner lock poisoned".to_string())?;
            let profile = owner_guard
                .as_ref()
                .ok_or_else(|| "Owner profile missing during monitoring".to_string())?;
            profile.validate_enrollment_complete()?;
            require_matching_embedder_model(profile, &embedder_config.model_file)?;
        }

        let mut camera = open_camera(&selection)?;
        camera.open_stream().map_err(|e| e.to_string())?;

        let mut alert_state = AlertState::new();
        let mut tracker = FaceTracker::new();
        let w = embedder_config.input_width;
        let h = embedder_config.input_height;

        let mut consecutive_failures = 0u32;

        while !stop.load(Ordering::SeqCst) {
            let frame_start = Instant::now();

            let step = (|| -> Result<(), String> {
                let frame = camera
                    .frame()
                    .map_err(|e| format!("Camera frame error: {}", e))?;
                let image = frame
                    .decode_image::<nokhwa::pixel_format::RgbFormat>()
                    .map_err(|e| format!("Camera decode error: {}", e))?;
                let rgb: RgbImage = image;

                let settings_snapshot = settings
                    .lock()
                    .map_err(|_| "Settings lock poisoned".to_string())?
                    .clone();
                let clahe = settings_snapshot.clahe_face_preproc;

                let owner_profile = owner
                    .lock()
                    .map_err(|_| "Owner lock poisoned".to_string())?
                    .clone()
                    .ok_or_else(|| "Owner profile missing during monitoring".to_string())?;

                let owner_threshold =
                    owner_cosine_threshold(&owner_profile, owner_match_threshold());

                let mut faces = detector.detect(&rgb)?;
                faces.sort_by(|a, b| {
                    b.score
                        .partial_cmp(&a.score)
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                faces = suppress_nested_detections(faces);
                faces.truncate(MAX_TRACKED_FACES);

                let mut similarities: Vec<Option<f32>> = vec![None; faces.len()];
                for (idx, face) in faces.iter().enumerate() {
                    if pass_quality_gate(&rgb, face).is_err() {
                        continue;
                    }
                    let mut aligned = align_face_5pt(&rgb, &face.landmarks, w, h)?;
                    maybe_apply_clahe_luminance(&mut aligned, clahe);
                    let embedding = embedder.embed(&aligned)?;
                    similarities[idx] = Some(max_cosine_vs_samples(
                        &embedding,
                        &owner_profile.embedding_samples,
                    )?);
                }

                let now = Instant::now();
                let track_outputs = tracker.update(now, &faces, &similarities, owner_threshold);

                let mut owner_index: Option<usize> = None;
                let mut owner_similarity: Option<f32> = None;
                for (idx, out) in track_outputs.iter().enumerate() {
                    let Some(to) = out else { continue };
                    if to.stable_label == StableFaceLabel::Owner {
                        let sim = to.similarity_this_frame.or(to.ema_similarity);
                        if let Some(s) = sim {
                            if owner_similarity.map_or(true, |best| s > best) {
                                owner_similarity = Some(s);
                                owner_index = Some(idx);
                            }
                        }
                    }
                }
                if owner_index.is_none() {
                    for (idx, sim_opt) in similarities.iter().enumerate() {
                        let Some(sim) = sim_opt else { continue };
                        if *sim >= owner_threshold {
                            if owner_similarity.map_or(true, |best| *sim > best) {
                                owner_similarity = Some(*sim);
                                owner_index = Some(idx);
                            }
                        }
                    }
                }

                let owner_id = match owner_similarity {
                    Some(sim) if sim >= owner_threshold => owner_index,
                    _ => None,
                };

                let sensitivity = settings_snapshot.sensitivity.clone();
                let cooldown_sec = settings_snapshot.cooldown_sec;
                let debug_overlay = settings_snapshot.debug_overlay;

                let mut max_score = None;
                let mut debug_faces = Vec::new();

                for (idx, face) in faces.iter().enumerate() {
                    let similarity = similarities.get(idx).copied().flatten();

                    let stable = track_outputs
                        .get(idx)
                        .and_then(|o| o.as_ref())
                        .map(|t| t.stable_label)
                        .unwrap_or(StableFaceLabel::Uncertain);

                    let is_owner = Some(idx) == owner_id;
                    let resembles_owner = similarity.map_or(false, |sim| sim >= owner_threshold);
                    let duplicate_owner_detection = !is_owner
                        && owner_id
                            .and_then(|oid| faces.get(oid))
                            .map(|owner_face| {
                                is_duplicate_owner_detection(&face.bbox, &owner_face.bbox)
                            })
                            .unwrap_or(false);

                    let label_str = match stable {
                        StableFaceLabel::Owner => "owner",
                        StableFaceLabel::Observer => "observer",
                        StableFaceLabel::Uncertain => "uncertain",
                    };

                    let observer_like = stable == StableFaceLabel::Observer;

                    let (label, observer_score) =
                        if is_owner || resembles_owner || duplicate_owner_detection {
                            ("owner".to_string(), None)
                        } else if observer_like {
                            let owner_bbox =
                                owner_id.and_then(|oid| faces.get(oid)).map(|f| &f.bbox);
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
                        } else {
                            (label_str.to_string(), None)
                        };

                    debug_faces.push(DebugFace {
                        id: idx,
                        bbox: face.bbox.clone(),
                        label,
                        similarity,
                        observer_score,
                    });
                }

                let now_ts = Instant::now();
                let threshold = observer_threshold(&sensitivity);
                let update = alert_state.update(
                    max_score,
                    threshold,
                    now_ts,
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
                    let mut encoder =
                        JpegEncoder::new_with_quality(&mut image_buffer, DEBUG_JPEG_QUALITY);
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

                Ok(())
            })();

            match step {
                Ok(()) => {
                    consecutive_failures = 0;
                }
                Err(message) => {
                    consecutive_failures += 1;
                    let _ = emit_error(&app, &message);
                    if consecutive_failures >= CONSECUTIVE_ERROR_STOP {
                        let _ = emit_monitor_stopped(&app, &message);
                        break;
                    }
                }
            }

            let elapsed = frame_start.elapsed();
            if elapsed < TARGET_FRAME_INTERVAL {
                thread::sleep(TARGET_FRAME_INTERVAL - elapsed);
            }
        }

        camera.stop_stream().map_err(|e| e.to_string())?;
        Ok(())
    })();

    if let Err(e) = session {
        let _ = emit_error(&app, &e);
    }
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

fn emit_monitor_stopped(app: &AppHandle, reason: &str) -> Result<(), String> {
    let event = MonitorStoppedEvent {
        reason: reason.to_string(),
    };
    app.emit("cv:monitor-stopped", event)
        .map_err(|e| e.to_string())
}

fn bbox_iou(a: &crate::cv::types::BoundingBox, b: &crate::cv::types::BoundingBox) -> f32 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    let inter = (x2 - x1).max(0.0) * (y2 - y1).max(0.0);
    let union = a.width * a.height + b.width * b.height - inter;
    if union <= 0.0 {
        0.0
    } else {
        inter / union
    }
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

fn is_duplicate_owner_detection(
    face_bbox: &crate::cv::types::BoundingBox,
    owner_bbox: &crate::cv::types::BoundingBox,
) -> bool {
    is_duplicate_bbox(face_bbox, owner_bbox)
}

fn is_duplicate_bbox(a: &crate::cv::types::BoundingBox, b: &crate::cv::types::BoundingBox) -> bool {
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
    let reference_diag = (((a.width.powi(2) + a.height.powi(2)).sqrt()
        + (b.width.powi(2) + b.height.powi(2)).sqrt())
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
    if min_area <= 0.0 {
        0.0
    } else {
        inter / min_area
    }
}

#[cfg(test)]
mod duplicate_bbox_tests {
    use crate::cv::types::{BoundingBox, FaceDetection, Point};

    fn landmarks() -> [Point; 5] {
        std::array::from_fn(|i| Point {
            x: 10.0 + i as f32,
            y: 10.0 + i as f32,
        })
    }

    #[test]
    fn is_duplicate_bbox_detects_heavy_overlap() {
        let outer = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
        };
        let inner = BoundingBox {
            x: 10.0,
            y: 10.0,
            width: 80.0,
            height: 80.0,
        };
        assert!(super::is_duplicate_bbox(&inner, &outer));
    }

    #[test]
    fn is_duplicate_bbox_false_for_far_apart() {
        let a = BoundingBox {
            x: 0.0,
            y: 0.0,
            width: 50.0,
            height: 50.0,
        };
        let b = BoundingBox {
            x: 400.0,
            y: 400.0,
            width: 50.0,
            height: 50.0,
        };
        assert!(!super::is_duplicate_bbox(&a, &b));
    }

    #[test]
    fn suppress_nested_detections_keeps_highest_score_first() {
        let faces = vec![
            FaceDetection {
                bbox: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 100.0,
                    height: 100.0,
                },
                score: 0.95,
                landmarks: landmarks(),
            },
            FaceDetection {
                bbox: BoundingBox {
                    x: 5.0,
                    y: 5.0,
                    width: 90.0,
                    height: 90.0,
                },
                score: 0.50,
                landmarks: landmarks(),
            },
        ];
        let kept = super::suppress_nested_detections(faces);
        assert_eq!(kept.len(), 1);
        assert!((kept[0].score - 0.95).abs() < 1e-6);
    }
}
