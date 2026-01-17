use fast_image_resize::{images::Image, IntoImageView, Resizer};
use image::{DynamicImage, RgbImage};

#[derive(Debug, Clone, Copy)]
pub enum ChannelOrder {
    Rgb,
    Bgr,
}

#[derive(Debug, Clone, Copy)]
pub enum TensorLayout {
    Nchw,
    Nhwc,
}

pub fn resize_rgb(image: &RgbImage, width: u32, height: u32) -> Result<RgbImage, String> {
    if image.width() == width && image.height() == height {
        return Ok(image.clone());
    }

    let src = DynamicImage::ImageRgb8(image.clone());
    let pixel_type = src
        .pixel_type()
        .ok_or_else(|| "Unsupported pixel type for resize".to_string())?;

    let mut dst_image = Image::new(width, height, pixel_type);
    let mut resizer = Resizer::new();
    resizer
        .resize(&src, &mut dst_image, None)
        .map_err(|e| e.to_string())?;

    let buffer = dst_image.buffer().to_vec();
    RgbImage::from_raw(width, height, buffer)
        .ok_or_else(|| "Failed to build resized RGB image".to_string())
}

pub fn crop_rgb(image: &RgbImage, x: u32, y: u32, width: u32, height: u32) -> RgbImage {
    image::imageops::crop_imm(image, x, y, width, height).to_image()
}

pub fn image_to_tensor_f32(
    image: &RgbImage,
    mean: [f32; 3],
    std: [f32; 3],
    order: ChannelOrder,
    layout: TensorLayout,
) -> Vec<f32> {
    let width = image.width() as usize;
    let height = image.height() as usize;
    let pixel_count = width * height;
    let mut output = vec![0.0f32; pixel_count * 3];
    let raw = image.as_raw();

    let channel_indices = match order {
        ChannelOrder::Rgb => [0usize, 1usize, 2usize],
        ChannelOrder::Bgr => [2usize, 1usize, 0usize],
    };

    match layout {
        TensorLayout::Nchw => {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 3;
                    let r = raw[idx] as f32;
                    let g = raw[idx + 1] as f32;
                    let b = raw[idx + 2] as f32;
                    let channels = [r, g, b];
                    for (c, &source) in channel_indices.iter().enumerate() {
                        let value = (channels[source] - mean[source]) / std[source];
                        output[c * pixel_count + y * width + x] = value;
                    }
                }
            }
        }
        TensorLayout::Nhwc => {
            for y in 0..height {
                for x in 0..width {
                    let idx = (y * width + x) * 3;
                    let r = raw[idx] as f32;
                    let g = raw[idx + 1] as f32;
                    let b = raw[idx + 2] as f32;
                    let channels = [r, g, b];
                    let base = (y * width + x) * 3;
                    for (c, &source) in channel_indices.iter().enumerate() {
                        output[base + c] = (channels[source] - mean[source]) / std[source];
                    }
                }
            }
        }
    }

    output
}
