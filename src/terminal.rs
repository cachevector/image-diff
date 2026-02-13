use colored::*;
use image::{DynamicImage, GenericImageView, Rgba};
use terminal_size::{terminal_size, Width};
use viuer::{print, Config};

pub fn print_preview(img: &DynamicImage) {
    let (tw, th) = if let Some((Width(w), h)) = terminal_size() {
        (w as u32, h.0 as u32)
    } else {
        (80, 24)
    };

    // Try to render with high-res graphics protocol (Sixel, Kitty, iTerm2)
    let conf = Config {
        transparent: true,
        absolute_offset: false,
        x: 0,
        y: 0,
        width: Some(tw),
        height: Some(th),
        ..Default::default()
    };

    if print(img, &conf).is_ok() {
        return;
    }

    // Fallback to ANSI half-blocks
    let (width, height) = img.dimensions();
    let aspect_ratio = height as f32 / width as f32;
    
    // We use half blocks, so each character is 2 vertical pixels
    let target_width = tw.min(width).min(80); // Cap width for readability
    let target_height = (target_width as f32 * aspect_ratio) as u32;
    
    let resized = img.resize_exact(target_width, target_height * 2, image::imageops::FilterType::Nearest);
    
    for y in (0..resized.height()).step_by(2) {
        for x in 0..resized.width() {
            let top = resized.get_pixel(x, y);
            let bottom = if y + 1 < resized.height() {
                resized.get_pixel(x, y + 1)
            } else {
                Rgba([0, 0, 0, 0])
            };

            let top_color = to_ansi_color(top);
            let bottom_color = to_ansi_color(bottom);

            print!("{}", "▀".truecolor(top_color.0, top_color.1, top_color.2)
                            .on_truecolor(bottom_color.0, bottom_color.1, bottom_color.2));
        }
        println!();
    }
}

fn to_ansi_color(pixel: Rgba<u8>) -> (u8, u8, u8) {
    let alpha = pixel[3] as f32 / 255.0;
    (
        (pixel[0] as f32 * alpha) as u8,
        (pixel[1] as f32 * alpha) as u8,
        (pixel[2] as f32 * alpha) as u8,
    )
}
