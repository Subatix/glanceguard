use std::path::PathBuf;

use image::RgbImage;
use ndarray::Array4;
use ort::session::builder::GraphOptimizationLevel;
use ort::session::Session;
use ort::value::Tensor;
use serde::Deserialize;
use tauri::{AppHandle, Manager};

use crate::cv::preprocess::{image_to_tensor_f32, resize_rgb, ChannelOrder, TensorLayout};

#[derive(Debug, Clone, Deserialize)]
pub struct EmbedderConfig {
    pub model_file: String,
    pub input_name: String,
    pub output_name: String,
    pub input_width: u32,
    pub input_height: u32,
    pub mean: [f32; 3],
    pub std: [f32; 3],
    pub channel_order: String,
    pub input_layout: String,
}

pub struct FaceEmbedder {
    session: Session,
    config: EmbedderConfig,
}

impl FaceEmbedder {
    pub fn new(app: &AppHandle, config: EmbedderConfig) -> Result<Self, String> {
        let model_path = resolve_model_path(app, &config.model_file)?;
        let session = Session::builder()
            .map_err(|e| e.to_string())?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| e.to_string())?
            .commit_from_file(model_path)
            .map_err(|e| e.to_string())?;

        Ok(Self { session, config })
    }

    pub fn embed(&mut self, image: &RgbImage) -> Result<Vec<f32>, String> {
        let resized = resize_rgb(image, self.config.input_width, self.config.input_height)?;
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
                let array = Array4::from_shape_vec((1, c, h, w), tensor_data)
                    .map_err(|e| e.to_string())?;
                Tensor::from_array(array).map_err(|e| e.to_string())?
            }
            TensorLayout::Nhwc => {
                let array = Array4::from_shape_vec((1, h, w, c), tensor_data)
                    .map_err(|e| e.to_string())?;
                Tensor::from_array(array).map_err(|e| e.to_string())?
            }
        };

        let outputs = self
            .session
            .run(ort::inputs![self.config.input_name.as_str() => input_tensor])
            .map_err(|e| e.to_string())?;

        let output = outputs
            .get(self.config.output_name.as_str())
            .ok_or_else(|| "Missing embedder output".to_string())?;
        let embedding = output
            .try_extract_array::<f32>()
            .map_err(|e| e.to_string())?
            .iter()
            .cloned()
            .collect::<Vec<f32>>();

        Ok(l2_normalize(embedding))
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

fn l2_normalize(mut embedding: Vec<f32>) -> Vec<f32> {
    let norm = embedding.iter().map(|v| v * v).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut embedding {
            *value /= norm;
        }
    }
    embedding
}
