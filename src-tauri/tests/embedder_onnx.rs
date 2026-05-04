//! ONNX embedder tests (`w600k_mbf.onnx`). Same prerequisite as `detector_onnx.rs`.
#![cfg(feature = "onnx-integration-tests")]

use std::path::PathBuf;

use image::open;
use glanceguard_lib::cv::config::{load_detector_config_from_path, load_embedder_config_from_path};
use glanceguard_lib::cv::detector::FaceDetector;
use glanceguard_lib::cv::embedder::FaceEmbedder;
use glanceguard_lib::cv::matching::cosine_similarity;
use glanceguard_lib::cv::preprocess::align_face_5pt;
use glanceguard_lib::ensure_onnx_runtime_loaded;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn aligned912(
    detector: &mut FaceDetector,
    embedder_cfg: &glanceguard_lib::cv::embedder::EmbedderConfig,
    rgb: &image::RgbImage,
) -> image::RgbImage {
    let faces = detector.detect(rgb).unwrap();
    assert!(
        !faces.is_empty(),
        "detector found no face — cannot run embedder threshold assertions"
    );
    let face = faces
        .into_iter()
        .max_by(|a, b| {
            a.score
                .partial_cmp(&b.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();
    align_face_5pt(
        rgb,
        &face.landmarks,
        embedder_cfg.input_width,
        embedder_cfg.input_height,
    )
    .unwrap()
}

#[test]
fn embedder_is_deterministic_on_same_aligned_crop() {
    ensure_onnx_runtime_loaded();
    let root = manifest_dir();
    let models = root.join("models");
    let det_onnx = models.join("det_500m.onnx");
    let emb_onnx = models.join("w600k_mbf.onnx");
    assert!(det_onnx.is_file() && emb_onnx.is_file());

    let det_cfg = load_detector_config_from_path(&models.join("scrfd.json")).unwrap();
    let emb_cfg = load_embedder_config_from_path(&models.join("arcface.json")).unwrap();

    let mut detector = FaceDetector::from_model_file(&det_onnx, det_cfg).unwrap();
    let mut embedder = FaceEmbedder::from_model_file(&emb_onnx, emb_cfg.clone()).unwrap();

    let rgb = open(root.join("tests/fixtures/person_a_1.jpg"))
        .unwrap()
        .to_rgb8();
    let aligned = aligned912(&mut detector, &emb_cfg, &rgb);

    let e1 = embedder.embed(&aligned).unwrap();
    let e2 = embedder.embed(&aligned).unwrap();
    let sim = cosine_similarity(&e1, &e2).unwrap();
    assert!(
        sim >= 0.99,
        "same crop should yield cosine ≥ 0.99, got {}",
        sim
    );
}

#[test]
fn embedder_same_person_cosine_above_cross_person() {
    ensure_onnx_runtime_loaded();
    let root = manifest_dir();
    let models = root.join("models");
    let det_onnx = models.join("det_500m.onnx");
    let emb_onnx = models.join("w600k_mbf.onnx");
    assert!(det_onnx.is_file() && emb_onnx.is_file());

    let det_cfg = load_detector_config_from_path(&models.join("scrfd.json")).unwrap();
    let emb_cfg = load_embedder_config_from_path(&models.join("arcface.json")).unwrap();

    let mut detector = FaceDetector::from_model_file(&det_onnx, det_cfg).unwrap();
    let mut embedder = FaceEmbedder::from_model_file(&emb_onnx, emb_cfg.clone()).unwrap();

    let a1 = open(root.join("tests/fixtures/person_a_1.jpg"))
        .unwrap()
        .to_rgb8();
    let a2 = open(root.join("tests/fixtures/person_a_2.jpg"))
        .unwrap()
        .to_rgb8();
    let b1 = open(root.join("tests/fixtures/person_b_1.jpg"))
        .unwrap()
        .to_rgb8();

    let aligned_a1 = aligned912(&mut detector, &emb_cfg, &a1);
    let aligned_a2 = aligned912(&mut detector, &emb_cfg, &a2);
    let aligned_b1 = aligned912(&mut detector, &emb_cfg, &b1);

    let ea1 = embedder.embed(&aligned_a1).unwrap();
    let ea2 = embedder.embed(&aligned_a2).unwrap();
    let eb1 = embedder.embed(&aligned_b1).unwrap();

    let same = cosine_similarity(&ea1, &ea2).unwrap();
    let cross = cosine_similarity(&ea1, &eb1).unwrap();

    assert!(
        same >= 0.45,
        "same-person cosine expected ≥ 0.45 for buffalo_s fixtures, got {}",
        same
    );
    assert!(
        same > cross,
        "same-person cosine must exceed cross-person (same={}, cross={})",
        same,
        cross
    );
}
