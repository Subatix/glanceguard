use std::path::PathBuf;

use image::RgbImage;
use ndarray::{ArrayView2, ArrayViewD, Ix2};
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::cv::preprocess::{image_to_tensor_f32, resize_rgb, ChannelOrder, TensorLayout};
use crate::cv::types::{BoundingBox, FaceDetection, Point};

#[derive(Debug, Clone, Deserialize)]
pub struct DetectorConfig {
    pub model_file: String,
    pub input_name: String,
    pub input_width: u32,
    pub input_height: u32,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    pub channel_order: String,
    pub input_layout: String,
    pub score_threshold: f32,
    pub nms_threshold: f32,
    pub outputs: Vec<DetectorOutputSpec>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DetectorOutputSpec {
    pub stride: u32,
    pub score: String,
    pub bbox: String,
    pub kps: Option<String>,
}

pub struct FaceDetector {
    session: Session,
    config: DetectorConfig,
}

impl FaceDetector {
    pub fn new(app: &AppHandle, config: DetectorConfig) -> Result<Self, String> {
        let model_path = resolve_model_path(app, &config.model_file)?;
        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .commit_from_file(model_path)
            .map_err(|e| e.to_string())?;

        Ok(Self { session, config })
    }

    pub fn detect(&mut self, frame: &RgbImage) -> Result<Vec<FaceDetection>, String> {
        let resized = resize_rgb(frame, self.config.input_width, self.config.input_height)?;
        let order = parse_channel_order(&self.config.channel_order)?;
        let layout = parse_layout(&self.config.input_layout)?;
        let tensor_data =
            image_to_tensor_f32(&resized, self.config.mean, self.config.std, order, layout);

        let (c, h, w) = match layout {
            TensorLayout::Nchw => (3usize, resized.height() as usize, resized.width() as usize),
            TensorLayout::Nhwc => (3usize, resized.height() as usize, resized.width() as usize),
        };

        let input_tensor = match layout {
            TensorLayout::Nchw => {
                let array = ndarray::Array4::from_shape_vec((1, c, h, w), tensor_data)
                    .map_err(|e| e.to_string())?;
                Tensor::from_array(array).map_err(|e| e.to_string())?
            }
            TensorLayout::Nhwc => {
                let array = ndarray::Array4::from_shape_vec((1, h, w, c), tensor_data)
                    .map_err(|e| e.to_string())?;
                Tensor::from_array(array).map_err(|e| e.to_string())?
            }
        };

        let outputs = self
            .session
            .run(ort::inputs![self.config.input_name.as_str() => input_tensor])
            .map_err(|e| e.to_string())?;

        let mut detections = Vec::new();
        let scale_x = frame.width() as f32 / self.config.input_width as f32;
        let scale_y = frame.height() as f32 / self.config.input_height as f32;

        for spec in &self.config.outputs {
            let scores_array = outputs[spec.score.as_str()]
                .try_extract_array::<f32>()
                .map_err(|e| e.to_string())?;
            let bboxes_array = outputs[spec.bbox.as_str()]
                .try_extract_array::<f32>()
                .map_err(|e| e.to_string())?;
            let kps_array = match &spec.kps {
                Some(name) => Some(
                    outputs[name.as_str()]
                        .try_extract_array::<f32>()
                        .map_err(|e| e.to_string())?,
                ),
                None => None,
            };

            let stride_detections = if scores_array.ndim() == 2 {
                let scores = scores_array
                    .view()
                    .into_dimensionality::<Ix2>()
                    .map_err(|_| "Scores output is not 2D".to_string())?;
                let bboxes = bboxes_array
                    .view()
                    .into_dimensionality::<Ix2>()
                    .map_err(|_| "Bboxes output is not 2D".to_string())?;
                let kps = match kps_array.as_ref() {
                    Some(array) => Some(
                        array
                            .view()
                            .into_dimensionality::<Ix2>()
                            .map_err(|_| "Keypoints output is not 2D".to_string())?,
                    ),
                    None => None,
                };

                decode_stride_flat(
                    spec.stride,
                    &scores,
                    &bboxes,
                    kps.as_ref(),
                    self.config.input_width,
                    self.config.input_height,
                    self.config.score_threshold,
                    scale_x,
                    scale_y,
                    frame.width(),
                    frame.height(),
                )?
            } else {
                let scores_view = scores_array.view();
                let bboxes_view = bboxes_array.view();
                let scores = tensor_view(&scores_view)?;
                let bboxes = tensor_view(&bboxes_view)?;
                let kps_view = kps_array.as_ref().map(|array| array.view());
                let kps = match kps_view.as_ref() {
                    Some(view) => Some(tensor_view(view)?),
                    None => None,
                };

                decode_stride(
                    spec.stride,
                    &scores,
                    &bboxes,
                    kps.as_ref(),
                    self.config.score_threshold,
                    scale_x,
                    scale_y,
                    frame.width(),
                    frame.height(),
                )?
            };
            detections.extend(stride_detections);
        }

        Ok(nms(detections, self.config.nms_threshold))
    }

}

fn resolve_model_path(app: &AppHandle, file: &str) -> Result<PathBuf, String> {
    app.path()
        .resolve(file, tauri::path::BaseDirectory::Resource)
        .map_err(|e| e.to_string())
}

fn parse_channel_order(value: &str) -> Result<ChannelOrder, String> {
    match value.to_lowercase().as_str() {
        "rgb" => Ok(ChannelOrder::Rgb),
        "bgr" => Ok(ChannelOrder::Bgr),
        _ => Err("Unsupported channel order".to_string()),
    }
}

fn parse_layout(value: &str) -> Result<TensorLayout, String> {
    match value.to_lowercase().as_str() {
        "nchw" => Ok(TensorLayout::Nchw),
        "nhwc" => Ok(TensorLayout::Nhwc),
        _ => Err("Unsupported input layout".to_string()),
    }
}

struct TensorView<'a> {
    data: &'a [f32],
    dims: [usize; 4],
    layout: TensorLayout,
}

impl<'a> TensorView<'a> {
    fn channels(&self) -> usize {
        match self.layout {
            TensorLayout::Nchw => self.dims[1],
            TensorLayout::Nhwc => self.dims[3],
        }
    }

    fn height(&self) -> usize {
        match self.layout {
            TensorLayout::Nchw => self.dims[2],
            TensorLayout::Nhwc => self.dims[1],
        }
    }

    fn width(&self) -> usize {
        match self.layout {
            TensorLayout::Nchw => self.dims[3],
            TensorLayout::Nhwc => self.dims[2],
        }
    }

    fn get(&self, n: usize, y: usize, x: usize, c: usize) -> f32 {
        let idx = match self.layout {
            TensorLayout::Nchw => {
                ((n * self.dims[1] + c) * self.dims[2] + y) * self.dims[3] + x
            }
            TensorLayout::Nhwc => {
                ((n * self.dims[1] + y) * self.dims[2] + x) * self.dims[3] + c
            }
        };
        self.data[idx]
    }
}

fn tensor_view<'a>(array: &'a ArrayViewD<'a, f32>) -> Result<TensorView<'a>, String> {
    let shape = array.shape().to_vec();
    if shape.len() != 4 {
        return Err("Expected 4D output tensor".to_string());
    }

    let dims = [shape[0], shape[1], shape[2], shape[3]];
    let data = array
        .as_slice()
        .ok_or_else(|| "Output tensor is not contiguous".to_string())?;

    let layout = if dims[1] <= 32 {
        TensorLayout::Nchw
    } else if dims[3] <= 32 {
        TensorLayout::Nhwc
    } else {
        return Err("Unable to infer output tensor layout".to_string());
    };

    Ok(TensorView { data, dims, layout })
}

fn decode_stride(
    stride: u32,
    scores: &TensorView<'_>,
    bboxes: &TensorView<'_>,
    kps: Option<&TensorView<'_>>,
    threshold: f32,
    scale_x: f32,
    scale_y: f32,
    frame_width: u32,
    frame_height: u32,
) -> Result<Vec<FaceDetection>, String> {
    if scores.height() != bboxes.height() || scores.width() != bboxes.width() {
        return Err("Mismatched output tensor shapes".to_string());
    }

    let bbox_channels = bboxes.channels();
    if bbox_channels % 4 != 0 {
        return Err("Invalid bbox channel count".to_string());
    }
    let num_anchors = bbox_channels / 4;

    let score_channels = scores.channels();
    let score_mode = if score_channels == num_anchors {
        ScoreMode::Direct
    } else if score_channels == num_anchors * 2 {
        ScoreMode::Foreground
    } else {
        return Err("Unsupported score channel count".to_string());
    };

    if let Some(kps_tensor) = kps {
        let kps_channels = kps_tensor.channels();
        if kps_channels != num_anchors * 10 {
            return Err("Invalid keypoint channel count".to_string());
        }
    }

    let mut detections = Vec::new();
    let height = scores.height();
    let width = scores.width();
    let stride_f = stride as f32;

    for y in 0..height {
        for x in 0..width {
            for anchor in 0..num_anchors {
                let score = match score_mode {
                    ScoreMode::Direct => scores.get(0, y, x, anchor),
                    ScoreMode::Foreground => scores.get(0, y, x, anchor * 2 + 1),
                };
                if score < threshold {
                    continue;
                }

                let cx = (x as f32 + 0.5) * stride_f;
                let cy = (y as f32 + 0.5) * stride_f;
                let base = anchor * 4;
                let l = bboxes.get(0, y, x, base);
                let t = bboxes.get(0, y, x, base + 1);
                let r = bboxes.get(0, y, x, base + 2);
                let b = bboxes.get(0, y, x, base + 3);

                let mut x1 = (cx - l) * scale_x;
                let mut y1 = (cy - t) * scale_y;
                let mut x2 = (cx + r) * scale_x;
                let mut y2 = (cy + b) * scale_y;

                x1 = clamp(x1, 0.0, frame_width as f32);
                y1 = clamp(y1, 0.0, frame_height as f32);
                x2 = clamp(x2, 0.0, frame_width as f32);
                y2 = clamp(y2, 0.0, frame_height as f32);

                let bbox = BoundingBox {
                    x: x1,
                    y: y1,
                    width: (x2 - x1).max(0.0),
                    height: (y2 - y1).max(0.0),
                };

                let landmarks = if let Some(kps_tensor) = kps {
                    let mut points = Vec::with_capacity(5);
                    let kps_base = anchor * 10;
                    for i in 0..5 {
                        let dx = kps_tensor.get(0, y, x, kps_base + i * 2);
                        let dy = kps_tensor.get(0, y, x, kps_base + i * 2 + 1);
                        points.push(Point {
                            x: (cx + dx) * scale_x,
                            y: (cy + dy) * scale_y,
                        });
                    }
                    [points[0].clone(), points[1].clone(), points[2].clone(), points[3].clone(), points[4].clone()]
                } else {
                    [
                        Point { x: bbox.x, y: bbox.y },
                        Point { x: bbox.x + bbox.width, y: bbox.y },
                        Point { x: bbox.x + bbox.width * 0.5, y: bbox.y + bbox.height * 0.5 },
                        Point { x: bbox.x, y: bbox.y + bbox.height },
                        Point { x: bbox.x + bbox.width, y: bbox.y + bbox.height },
                    ]
                };

                detections.push(FaceDetection { bbox, score, landmarks });
            }
        }
    }

    Ok(detections)
}

enum ScoreMode {
    Direct,
    Foreground,
}

fn decode_stride_flat(
    stride: u32,
    scores: &ArrayView2<'_, f32>,
    bboxes: &ArrayView2<'_, f32>,
    kps: Option<&ArrayView2<'_, f32>>,
    input_width: u32,
    input_height: u32,
    threshold: f32,
    scale_x: f32,
    scale_y: f32,
    frame_width: u32,
    frame_height: u32,
) -> Result<Vec<FaceDetection>, String> {
    let num = scores.shape()[0];
    let score_dim = scores.shape()[1];
    let bbox_dim = bboxes.shape()[1];
    if bbox_dim != 4 {
        return Err("Flat bbox output must have 4 columns".to_string());
    }
    if let Some(kps_tensor) = kps {
        if kps_tensor.shape()[1] != 10 {
            return Err("Flat keypoint output must have 10 columns".to_string());
        }
    }

    let width = (input_width / stride) as usize;
    let height = (input_height / stride) as usize;
    let cells = width * height;
    if cells == 0 || num % cells != 0 {
        return Err("Flat output size does not match stride grid".to_string());
    }
    let anchors = num / cells;

    let mut detections = Vec::new();
    let stride_f = stride as f32;

    for idx in 0..num {
        let score = match score_dim {
            1 => scores[[idx, 0]],
            2 => scores[[idx, 1]],
            _ => return Err("Unsupported score dimension".to_string()),
        };
        if score < threshold {
            continue;
        }

        let cell = idx / anchors;
        let x = cell % width;
        let y = cell / width;

        let cx = (x as f32 + 0.5) * stride_f;
        let cy = (y as f32 + 0.5) * stride_f;

        let l = bboxes[[idx, 0]];
        let t = bboxes[[idx, 1]];
        let r = bboxes[[idx, 2]];
        let b = bboxes[[idx, 3]];

        let mut x1 = (cx - l) * scale_x;
        let mut y1 = (cy - t) * scale_y;
        let mut x2 = (cx + r) * scale_x;
        let mut y2 = (cy + b) * scale_y;

        x1 = clamp(x1, 0.0, frame_width as f32);
        y1 = clamp(y1, 0.0, frame_height as f32);
        x2 = clamp(x2, 0.0, frame_width as f32);
        y2 = clamp(y2, 0.0, frame_height as f32);

        let bbox = BoundingBox {
            x: x1,
            y: y1,
            width: (x2 - x1).max(0.0),
            height: (y2 - y1).max(0.0),
        };

        let landmarks = if let Some(kps_tensor) = kps {
            let mut points = Vec::with_capacity(5);
            for i in 0..5 {
                let dx = kps_tensor[[idx, i * 2]];
                let dy = kps_tensor[[idx, i * 2 + 1]];
                points.push(Point {
                    x: (cx + dx) * scale_x,
                    y: (cy + dy) * scale_y,
                });
            }
            [
                points[0].clone(),
                points[1].clone(),
                points[2].clone(),
                points[3].clone(),
                points[4].clone(),
            ]
        } else {
            [
                Point { x: bbox.x, y: bbox.y },
                Point { x: bbox.x + bbox.width, y: bbox.y },
                Point { x: bbox.x + bbox.width * 0.5, y: bbox.y + bbox.height * 0.5 },
                Point { x: bbox.x, y: bbox.y + bbox.height },
                Point { x: bbox.x + bbox.width, y: bbox.y + bbox.height },
            ]
        };

        detections.push(FaceDetection { bbox, score, landmarks });
    }

    Ok(detections)
}

fn clamp(value: f32, min: f32, max: f32) -> f32 {
    value.max(min).min(max)
}

fn nms(mut detections: Vec<FaceDetection>, iou_threshold: f32) -> Vec<FaceDetection> {
    detections.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    let mut kept = Vec::new();
    let mut suppressed = vec![false; detections.len()];

    for i in 0..detections.len() {
        if suppressed[i] {
            continue;
        }
        let current = detections[i].clone();
        kept.push(current.clone());
        for j in (i + 1)..detections.len() {
            if suppressed[j] {
                continue;
            }
            if iou(&current.bbox, &detections[j].bbox) > iou_threshold {
                suppressed[j] = true;
            }
        }
    }

    kept
}

fn iou(a: &BoundingBox, b: &BoundingBox) -> f32 {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);

    let inter_w = (x2 - x1).max(0.0);
    let inter_h = (y2 - y1).max(0.0);
    let inter_area = inter_w * inter_h;
    let area_a = a.width * a.height;
    let area_b = b.width * b.height;
    let union = area_a + area_b - inter_area;
    if union <= 0.0 {
        0.0
    } else {
        inter_area / union
    }
}
