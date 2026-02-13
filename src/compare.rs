use anyhow::Result;
use image::{GenericImageView, ImageBuffer, Rgba};
use image_compare::Algorithm;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize)]
pub struct DiffResult {
    pub score: f64,
    pub ssim_score: f64,
    pub diff_pixels: u64,
    pub total_pixels: u64,
    #[serde(skip)]
    pub diff_image: Option<ImageBuffer<Rgba<u8>, Vec<u8>>>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Region {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Region {
    pub fn contains(&self, x: u32, y: u32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub fn compare_images(
    path_a: &Path,
    path_b: &Path,
    threshold: f32,
    generate_diff: bool,
    ignore_regions: &[Region],
    mask_path: Option<&Path>,
) -> Result<DiffResult> {
    let img_a = image::open(path_a)?;
    let img_b = image::open(path_b)?;

    let (width_a, height_a) = img_a.dimensions();
    let (width_b, height_b) = img_b.dimensions();

    let max_width = width_a.max(width_b);
    let max_height = height_a.max(height_b);

    let mask_img = if let Some(path) = mask_path {
        Some(image::open(path)?.to_rgba8())
    } else {
        None
    };

    // For SSIM, we need identical dimensions.
    let mut rgba_a = img_a.to_rgba8();
    let mut rgba_b = img_b.to_rgba8();

    if width_a != max_width || height_a != max_height {
        let mut new_a = ImageBuffer::new(max_width, max_height);
        image::imageops::overlay(&mut new_a, &rgba_a, 0, 0);
        rgba_a = new_a;
    }

    if width_b != max_width || height_b != max_height {
        let mut new_b = ImageBuffer::new(max_width, max_height);
        image::imageops::overlay(&mut new_b, &rgba_b, 0, 0);
        rgba_b = new_b;
    }

    let mut diff_pixels = 0u64;
    let total_pixels = (max_width as u64) * (max_height as u64);

    let mut diff_buffer = if generate_diff {
        Some(ImageBuffer::new(max_width, max_height))
    } else {
        None
    };

    for y in 0..max_height {
        for x in 0..max_width {
            let mut is_ignored = ignore_regions.iter().any(|r| r.contains(x, y));
            
            if !is_ignored {
                if let Some(ref mask) = mask_img {
                    if x < mask.width() && y < mask.height() {
                        let mask_pixel = mask.get_pixel(x, y);
                        // Ignore if mask pixel is black or has low alpha
                        if (mask_pixel[0] == 0 && mask_pixel[1] == 0 && mask_pixel[2] == 0) || mask_pixel[3] < 128 {
                            is_ignored = true;
                        }
                    }
                }
            }

            let pixel_a = rgba_a.get_pixel(x, y);
            let pixel_b = rgba_b.get_pixel(x, y);

            let dist = if is_ignored {
                0.0 // Treat as identical
            } else {
                color_distance(pixel_a, pixel_b)
            };
            
            let is_different = dist > (threshold as f64);

            if is_different {
                diff_pixels += 1;
                if let Some(ref mut buffer) = diff_buffer {
                    buffer.put_pixel(x, y, Rgba([255, 0, 255, 255]));
                }
            } else if let Some(ref mut buffer) = diff_buffer {
                let factor = if is_ignored { 0.02 } else { 0.1 };
                let r = (pixel_a[0] as f32 * factor) as u8;
                let g = (pixel_a[1] as f32 * factor) as u8;
                let b = (pixel_a[2] as f32 * factor) as u8;
                buffer.put_pixel(x, y, Rgba([r, g, b, 255]));
            }
        }
    }

    let score = 1.0 - (diff_pixels as f64 / total_pixels as f64);

    // Calculate SSIM using RGB
    let rgb_a = image::DynamicImage::ImageRgba8(rgba_a).to_rgb8();
    let rgb_b = image::DynamicImage::ImageRgba8(rgba_b).to_rgb8();
    let ssim_score = image_compare::rgb_similarity_structure(&Algorithm::MSSIMSimple, &rgb_a, &rgb_b).unwrap().score;

    Ok(DiffResult {
        score,
        ssim_score,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_distance() {
        let p1 = Rgba([0, 0, 0, 255]);
        let p2 = Rgba([255, 255, 255, 255]);
        assert!((color_distance(&p1, &p2) - 1.732).abs() < 0.01);

        let p3 = Rgba([100, 100, 100, 255]);
        assert_eq!(color_distance(&p3, &p3), 0.0);
    }

    #[test]
    fn test_region_contains() {
        let region = Region { x: 10, y: 10, width: 20, height: 20 };
        assert!(region.contains(10, 10));
        assert!(region.contains(29, 29));
        assert!(!region.contains(9, 10));
        assert!(!region.contains(30, 30));
    }

    #[test]
    fn test_compare_identical() -> Result<()> {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(10, 10);
        for p in img.pixels_mut() { *p = Rgba([100, 100, 100, 255]); }
        
        let file_a = tempfile::Builder::new().suffix(".png").tempfile()?;
        let file_b = tempfile::Builder::new().suffix(".png").tempfile()?;
        img.save(file_a.path())?;
        img.save(file_b.path())?;

        let res = compare_images(file_a.path(), file_b.path(), 0.1, false, &[], None)?;
        assert_eq!(res.diff_pixels, 0);
        assert_eq!(res.score, 1.0);
        assert!(res.ssim_score > 0.99);
        Ok(())
    }

    #[test]
    fn test_compare_different_with_ignore() -> Result<()> {
        let mut img_a: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(10, 10);
        for p in img_a.pixels_mut() { *p = Rgba([100, 100, 100, 255]); }
        
        let mut img_b = img_a.clone();
        img_b.put_pixel(5, 5, Rgba([255, 0, 0, 255]));

        let file_a = tempfile::Builder::new().suffix(".png").tempfile()?;
        let file_b = tempfile::Builder::new().suffix(".png").tempfile()?;
        img_a.save(file_a.path())?;
        img_b.save(file_b.path())?;

        // Without ignore
        let res1 = compare_images(file_a.path(), file_b.path(), 0.1, false, &[], None)?;
        assert_eq!(res1.diff_pixels, 1);

        // With ignore
        let ignore = [Region { x: 5, y: 5, width: 1, height: 1 }];
        let res2 = compare_images(file_a.path(), file_b.path(), 0.1, false, &ignore, None)?;
        assert_eq!(res2.diff_pixels, 0);
        assert_eq!(res2.score, 1.0);
        Ok(())
    }

    #[test]
    fn test_compare_with_mask() -> Result<()> {
        let mut img_a: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(10, 10);
        for p in img_a.pixels_mut() { *p = Rgba([100, 100, 100, 255]); }
        
        let mut img_b = img_a.clone();
        img_b.put_pixel(5, 5, Rgba([255, 0, 0, 255]));

        let mut mask: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(10, 10);
        for p in mask.pixels_mut() { *p = Rgba([255, 255, 255, 255]); }
        mask.put_pixel(5, 5, Rgba([0, 0, 0, 255])); // Mask out the difference

        let file_a = tempfile::Builder::new().suffix(".png").tempfile()?;
        let file_b = tempfile::Builder::new().suffix(".png").tempfile()?;
        let file_mask = tempfile::Builder::new().suffix(".png").tempfile()?;
        
        img_a.save(file_a.path())?;
        img_b.save(file_b.path())?;
        mask.save(file_mask.path())?;

        let res = compare_images(file_a.path(), file_b.path(), 0.1, false, &[], Some(file_mask.path()))?;
        assert_eq!(res.diff_pixels, 0);
        Ok(())
    }
}
