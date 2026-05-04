use fast_image_resize::{images::Image, IntoImageView, Resizer};
use image::DynamicImage;
pub use image::RgbImage;
use nalgebra::{Matrix2, Vector2, SVD};

use crate::cv::types::Point;

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

#[allow(dead_code)]
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

/// ArcFace 112×112 landmark template (InsightFace `utils/face_align.py`, norm_crop).
pub const ARCFACE_REF_112: [[f32; 2]; 5] = [
    [38.2946, 51.6963],
    [73.5318, 51.5014],
    [56.0252, 71.7366],
    [41.5493, 92.3655],
    [70.7299, 92.2041],
];

/// Maps 5-point landmarks into `ARCFACE_REF_112` with a Kabsch similarity transform,
/// then warps into `out_w`×`out_h` with bilinear sampling.
pub fn align_face_5pt(
    image: &RgbImage,
    landmarks: &[Point; 5],
    out_w: u32,
    out_h: u32,
) -> Result<RgbImage, String> {
    let src: [[f32; 2]; 5] = std::array::from_fn(|i| [landmarks[i].x, landmarks[i].y]);
    let (m, t) = kabsch_similarity_2d(&src, &ARCFACE_REF_112)?;
    let m_inv = m
        .try_inverse()
        .ok_or_else(|| "Degenerate face alignment transform".to_string())?;

    let mut out = RgbImage::new(out_w, out_h);
    let iw = image.width() as f32;
    let ih = image.height() as f32;

    for oy in 0..out_h {
        for ox in 0..out_w {
            let p = Vector2::new(ox as f32 + 0.5, oy as f32 + 0.5);
            let s = m_inv * (p - t);
            if s.x < 0.0 || s.y < 0.0 || s.x >= iw - 1.0 || s.y >= ih - 1.0 {
                out.put_pixel(ox, oy, image::Rgb([0, 0, 0]));
            } else {
                let px = sample_rgb_bilinear(image, s.x, s.y);
                out.put_pixel(ox, oy, image::Rgb(px));
            }
        }
    }

    Ok(out)
}

/// Optional CLAHE on luminance; mutates RGB in place (same path for enroll + runtime).
pub fn maybe_apply_clahe_luminance(image: &mut RgbImage, enabled: bool) {
    if !enabled || image.width() == 0 || image.height() == 0 {
        return;
    }
    apply_clahe_luminance_rgb(image);
}

fn kabsch_similarity_2d(
    src: &[[f32; 2]; 5],
    dst: &[[f32; 2]; 5],
) -> Result<(Matrix2<f32>, Vector2<f32>), String> {
    let mut mu_s = Vector2::zeros();
    let mut mu_d = Vector2::zeros();
    for i in 0..5 {
        mu_s += Vector2::new(src[i][0], src[i][1]);
        mu_d += Vector2::new(dst[i][0], dst[i][1]);
    }
    mu_s /= 5.0;
    mu_d /= 5.0;

    let mut h = Matrix2::zeros();
    let mut ss = 0f32;
    for i in 0..5 {
        let p = Vector2::new(src[i][0], src[i][1]) - mu_s;
        let q = Vector2::new(dst[i][0], dst[i][1]) - mu_d;
        h += p * q.transpose();
        ss += p.norm_squared();
    }

    let svd = SVD::new(h, true, true);
    let u = svd
        .u
        .ok_or_else(|| "Face alignment SVD missing U".to_string())?;
    let v_t = svd
        .v_t
        .ok_or_else(|| "Face alignment SVD missing V^T".to_string())?;

    let mut r = v_t.transpose() * u.transpose();
    if r.determinant() < 0.0 {
        let mut u_fix = u;
        u_fix.set_column(1, &-u_fix.column(1));
        r = v_t.transpose() * u_fix.transpose();
    }

    let denom = ss.max(1e-6);
    let scale = (r * h).trace() / denom;
    if !scale.is_finite() || scale.abs() < 1e-6 {
        return Err("Invalid face alignment scale".into());
    }

    let m = scale * r;
    let t = mu_d - m * mu_s;
    Ok((m, t))
}

fn sample_rgb_bilinear(image: &RgbImage, x: f32, y: f32) -> [u8; 3] {
    let x0 = x.floor() as u32;
    let y0 = y.floor() as u32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;

    let w = image.width();
    let h = image.height();
    let c00 = image.get_pixel(x0.min(w - 1), y0.min(h - 1)).0;
    let c10 = image.get_pixel(x1.min(w - 1), y0.min(h - 1)).0;
    let c01 = image.get_pixel(x0.min(w - 1), y1.min(h - 1)).0;
    let c11 = image.get_pixel(x1.min(w - 1), y1.min(h - 1)).0;

    let lerp = |a: u8, b: u8, t: f32| -> u8 {
        (a as f32 * (1.0 - t) + b as f32 * t)
            .round()
            .clamp(0.0, 255.0) as u8
    };

    let i0 = [
        lerp(c00[0], c10[0], fx),
        lerp(c00[1], c10[1], fx),
        lerp(c00[2], c10[2], fx),
    ];
    let i1 = [
        lerp(c01[0], c11[0], fx),
        lerp(c01[1], c11[1], fx),
        lerp(c01[2], c11[2], fx),
    ];
    [
        lerp(i0[0], i1[0], fy),
        lerp(i0[1], i1[1], fy),
        lerp(i0[2], i1[2], fy),
    ]
}

fn rgb_to_y(r: u8, g: u8, b: u8) -> u8 {
    let y = 0.299_f32 * r as f32 + 0.587_f32 * g as f32 + 0.114_f32 * b as f32;
    y.round().clamp(0.0, 255.0) as u8
}

fn y_to_rgb_shift(p: &[u8; 3], y_old: u8, y_new: u8) -> [u8; 3] {
    if y_old == 0 {
        return [y_new, y_new, y_new];
    }
    let gain = y_new as f32 / y_old as f32;
    let shift = |c: u8| -> u8 { (c as f32 * gain).round().clamp(0.0, 255.0) as u8 };
    [shift(p[0]), shift(p[1]), shift(p[2])]
}

fn build_clahe_map(y_vals: &[u8], clip_limit: f32) -> [u8; 256] {
    let mut hist = [0u32; 256];
    for &v in y_vals {
        hist[v as usize] += 1;
    }
    let n = y_vals.len().max(1) as f32;
    let max_bin = (clip_limit * n / 256.0).max(1.0);
    let mut clipped = hist;
    let mut excess: f32 = 0.0;
    for i in 0..256 {
        if clipped[i] as f32 > max_bin {
            excess += clipped[i] as f32 - max_bin;
            clipped[i] = max_bin as u32;
        }
    }
    let add_f = excess / 256.0;
    for c in clipped.iter_mut() {
        *c += add_f.floor() as u32;
    }

    let mut cdf = [0u32; 256];
    cdf[0] = clipped[0];
    for i in 1..256 {
        cdf[i] = cdf[i - 1] + clipped[i];
    }
    let cdf_min = cdf.iter().copied().find(|&v| v > 0).unwrap_or(0);
    let denom = cdf[255].saturating_sub(cdf_min).max(1);
    let mut map = [0u8; 256];
    for i in 0..256 {
        let v = ((cdf[i].saturating_sub(cdf_min) as f64 / denom as f64) * 255.0).round() as u32;
        map[i] = v.min(255) as u8;
    }
    map
}

#[cfg(test)]
mod preprocess_tests {
    use super::{image_to_tensor_f32, ChannelOrder, TensorLayout};
    use image::RgbImage;

    #[test]
    fn image_to_tensor_nchw_rgb_order_mean_std() {
        let mut img = RgbImage::new(2, 2);
        img.put_pixel(0, 0, image::Rgb([255, 0, 0]));
        img.put_pixel(1, 0, image::Rgb([0, 255, 0]));
        img.put_pixel(0, 1, image::Rgb([0, 0, 255]));
        img.put_pixel(1, 1, image::Rgb([10, 20, 30]));

        let mean = [0.0_f32, 0.0_f32, 0.0_f32];
        let std = [1.0_f32, 1.0_f32, 1.0_f32];
        let out =
            image_to_tensor_f32(&img, mean, std, ChannelOrder::Rgb, TensorLayout::Nchw);
        let w = 2usize;
        let px = |x: usize, y: usize| y * w + x;

        assert!((out[0 * 4 + px(0, 0)] - 255.0).abs() < 1e-4);
        assert!((out[1 * 4 + px(1, 0)] - 255.0).abs() < 1e-4);
        assert!((out[2 * 4 + px(0, 1)] - 255.0).abs() < 1e-4);
        assert!((out[0 * 4 + px(1, 1)] - 10.0).abs() < 1e-4);
        assert!((out[1 * 4 + px(1, 1)] - 20.0).abs() < 1e-4);
        assert!((out[2 * 4 + px(1, 1)] - 30.0).abs() < 1e-4);
    }

    #[test]
    fn image_to_tensor_bgr_swap_and_nhwc_layout() {
        let mut img = RgbImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgb([10, 20, 30]));

        let mean = [0.0_f32, 0.0_f32, 0.0_f32];
        let std = [1.0_f32, 1.0_f32, 1.0_f32];
        let out =
            image_to_tensor_f32(&img, mean, std, ChannelOrder::Bgr, TensorLayout::Nhwc);

        assert_eq!(out.len(), 3);
        assert!((out[0] - 30.0).abs() < 1e-4);
        assert!((out[1] - 20.0).abs() < 1e-4);
        assert!((out[2] - 10.0).abs() < 1e-4);
    }
}

fn apply_clahe_luminance_rgb(image: &mut RgbImage) {
    let grid: u32 = 8;
    let w = image.width();
    let h = image.height();
    if w == 0 || h == 0 {
        return;
    }
    let tile_w = (w + grid - 1) / grid;
    let tile_h = (h + grid - 1) / grid;

    let tile_maps: Vec<Vec<[u8; 256]>> = (0..grid)
        .map(|ty| {
            (0..grid)
                .map(|tx| {
                    let mut ys = Vec::new();
                    let y0 = ty * tile_h;
                    let y1 = ((ty + 1) * tile_h).min(h);
                    let x0 = tx * tile_w;
                    let x1 = ((tx + 1) * tile_w).min(w);
                    for row in y0..y1 {
                        for col in x0..x1 {
                            let p = image.get_pixel(col, row).0;
                            ys.push(rgb_to_y(p[0], p[1], p[2]));
                        }
                    }
                    build_clahe_map(&ys, 2.0)
                })
                .collect()
        })
        .collect();

    for row in 0..h {
        for col in 0..w {
            let tx = (col / tile_w).min(grid - 1);
            let ty = (row / tile_h).min(grid - 1);
            let map = &tile_maps[ty as usize][tx as usize];
            let p = image.get_pixel(col, row).0;
            let y0 = rgb_to_y(p[0], p[1], p[2]);
            let y1 = map[y0 as usize];
            let np = y_to_rgb_shift(&p, y0, y1);
            image.put_pixel(col, row, image::Rgb(np));
        }
    }
}
