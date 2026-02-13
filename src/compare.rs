use anyhow::Result;
use image::{GenericImageView, ImageBuffer, Rgba};
use std::path::Path;

pub struct DiffResult {
    pub score: f64,
    pub diff_pixels: u64,
    pub total_pixels: u64,
    pub diff_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

pub fn compare_images(
    path_a: &Path,
    path_b: &Path,
    threshold: f32,
    generate_diff: bool,
) -> Result<DiffResult> {
    let img_a = image::open(path_a)?;
    let img_b = image::open(path_b)?;

    let (width_a, height_a) = img_a.dimensions();
    let (width_b, height_b) = img_b.dimensions();

    let max_width = width_a.max(width_b);
    let max_height = height_a.max(height_b);

    let mut diff_pixels = 0u64;
    let total_pixels = (max_width as u64) * (max_height as u64);

    let mut diff_buffer = if generate_diff {
        Some(ImageBuffer::new(max_width, max_height))
    } else {
        None
    };

    for y in 0..max_height {
        for x in 0..max_width {
            let pixel_a = if x < width_a && y < height_a {
                img_a.get_pixel(x, y)
            } else {
                Rgba([0, 0, 0, 0])
            };

            let pixel_b = if x < width_b && y < height_b {
                img_b.get_pixel(x, y)
            } else {
                Rgba([0, 0, 0, 0])
            };

            let dist = color_distance(&pixel_a, &pixel_b);
            
            let is_different = dist > (threshold as f64);

            if is_different {
                diff_pixels += 1;
                if let Some(ref mut buffer) = diff_buffer {
                    // Magenta for differences
                    buffer.put_pixel(x, y, Rgba([255, 0, 255, 255]));
                }
            } else if let Some(ref mut buffer) = diff_buffer {
                // Dimmed version of original for non-diff
                let r = (pixel_a[0] as f32 * 0.1) as u8;
                let g = (pixel_a[1] as f32 * 0.1) as u8;
                let b = (pixel_a[2] as f32 * 0.1) as u8;
                buffer.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }

    let score = 1.0 - (diff_pixels as f64 / total_pixels as f64);

    Ok(DiffResult {
        score,
        diff_pixels,
        total_pixels,
        diff_image: diff_buffer,
    })
}

fn color_distance(p1: &Rgba<u8>, p2: &Rgba<u8>) -> f64 {
    let r_diff = (p1[0] as f64 - p2[0] as f64) / 255.0;
    let g_diff = (p1[1] as f64 - p2[1] as f64) / 255.0;
    let b_diff = (p1[2] as f64 - p2[2] as f64) / 255.0;
    let a_diff = (p1[3] as f64 - p2[3] as f64) / 255.0;

    // Euclidean distance in RGBA space
    (r_diff * r_diff + g_diff * g_diff + b_diff * b_diff + a_diff * a_diff).sqrt()
}
