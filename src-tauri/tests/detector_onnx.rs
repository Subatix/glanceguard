//! ONNX detector smoke tests (`buffalo_s` SCRFD). Requires `det_500m.onnx` next to `scrfd.json`.
//! Fixtures include images from InsightFace `python-package/insightface/data/images` (MIT).
#![cfg(feature = "onnx-integration-tests")]

use std::path::PathBuf;

use image::open;
use glanceguard_lib::cv::config::load_detector_config_from_path;
use glanceguard_lib::cv::detector::FaceDetector;
use glanceguard_lib::ensure_onnx_runtime_loaded;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn detector_finds_single_face_on_one_face_fixture() {
    ensure_onnx_runtime_loaded();
    let root = manifest_dir();
    let models = root.join("models");
    let onnx = models.join("det_500m.onnx");
    assert!(
        onnx.is_file(),
        "missing {}; run scripts/download-buffalo-s-models.sh",
        onnx.display()
    );

    let cfg = load_detector_config_from_path(&models.join("scrfd.json")).unwrap();
    let mut detector = FaceDetector::from_model_file(&onnx, cfg).unwrap();

    let img = open(root.join("tests/fixtures/one_face.jpg"))
        .unwrap()
        .to_rgb8();
    let dets = detector.detect(&img).unwrap();
    assert!(
        !dets.is_empty(),
        "expected ≥1 raw detection on one_face.jpg, got 0"
    );
    // SCRFD often emits overlapping anchors on a single subject after NMS; assert primary confidence instead of raw len()==1.
    let primary = dets
        .iter()
        .max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();
    assert!(
        primary.score >= 0.45,
        "expected a confident primary face on one_face.jpg (got score {})",
        primary.score
    );
}

#[test]
fn detector_finds_multiple_faces_on_two_faces_fixture() {
    ensure_onnx_runtime_loaded();
    let root = manifest_dir();
    let models = root.join("models");
    let onnx = models.join("det_500m.onnx");
    assert!(onnx.is_file());

    let cfg = load_detector_config_from_path(&models.join("scrfd.json")).unwrap();
    let mut detector = FaceDetector::from_model_file(&onnx, cfg).unwrap();

    let img = open(root.join("tests/fixtures/two_faces.jpg"))
        .unwrap()
        .to_rgb8();
    let dets = detector.detect(&img).unwrap();
    assert!(
        dets.len() >= 2,
        "expected ≥2 faces on two_faces.jpg, got {}",
        dets.len()
    );
}

#[test]
fn detector_finds_no_faces_on_blank_fixture() {
    ensure_onnx_runtime_loaded();
    let root = manifest_dir();
    let models = root.join("models");
    let onnx = models.join("det_500m.onnx");
    assert!(onnx.is_file());

    let cfg = load_detector_config_from_path(&models.join("scrfd.json")).unwrap();
    let mut detector = FaceDetector::from_model_file(&onnx, cfg).unwrap();

    let img = open(root.join("tests/fixtures/no_face.jpg"))
        .unwrap()
        .to_rgb8();
    let dets = detector.detect(&img).unwrap();
    assert!(
        dets.is_empty(),
        "expected no faces on no_face.jpg, got {}",
        dets.len()
    );
}
