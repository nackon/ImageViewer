use image::{ImageBuffer, Rgb, RgbImage};
use std::fs;
use std::path::Path;

fn main() {
    let output_dir = "test_images";

    // Create output directory if it doesn't exist
    fs::create_dir_all(output_dir).expect("Failed to create output directory");

    println!("Generating 100 test images...");

    for i in 0..100 {
        let width = 800;
        let height = 600;

        // Create a colorful gradient image
        let mut img: RgbImage = ImageBuffer::new(width, height);

        // Generate different patterns for variety
        let pattern = i % 5;

        for (x, y, pixel) in img.enumerate_pixels_mut() {
            let (r, g, b) = match pattern {
                0 => {
                    // Horizontal gradient
                    let r = (x as f32 / width as f32 * 255.0) as u8;
                    let g = (y as f32 / height as f32 * 255.0) as u8;
                    let b = ((i * 17) % 256) as u8;
                    (r, g, b)
                }
                1 => {
                    // Vertical gradient
                    let r = ((i * 13) % 256) as u8;
                    let g = (y as f32 / height as f32 * 255.0) as u8;
                    let b = (x as f32 / width as f32 * 255.0) as u8;
                    (r, g, b)
                }
                2 => {
                    // Diagonal pattern
                    let r = ((x + y) % 256) as u8;
                    let g = ((x * 2 + y) % 256) as u8;
                    let b = ((i * 23) % 256) as u8;
                    (r, g, b)
                }
                3 => {
                    // Checkerboard
                    let size = 50;
                    let checker = ((x / size) + (y / size)) % 2;
                    if checker == 0 {
                        (((i * 7) % 256) as u8, 100, 200)
                    } else {
                        (200, ((i * 11) % 256) as u8, 100)
                    }
                }
                _ => {
                    // Circular gradient
                    let cx = width as f32 / 2.0;
                    let cy = height as f32 / 2.0;
                    let dx = x as f32 - cx;
                    let dy = y as f32 - cy;
                    let dist = (dx * dx + dy * dy).sqrt();
                    let max_dist = (cx * cx + cy * cy).sqrt();
                    let intensity = (dist / max_dist * 255.0) as u8;
                    (intensity, ((i * 19) % 256) as u8, 255 - intensity)
                }
            };

            *pixel = Rgb([r, g, b]);
        }

        // Add some text-like patterns
        if i % 3 == 0 {
            // Add some "text" bars
            for y in 50..70 {
                for x in 50..750 {
                    if (x / 10) % 2 == 0 {
                        img.put_pixel(x, y, Rgb([255, 255, 255]));
                    }
                }
            }
        }

        // Determine file format (mix PNG and JPEG)
        let (filename, format) = if i % 2 == 0 {
            (format!("{}/image_{:03}.png", output_dir, i), "png")
        } else {
            (format!("{}/image_{:03}.jpg", output_dir, i), "jpg")
        };

        // Save the image
        if format == "png" {
            img.save_with_format(&filename, image::ImageFormat::Png)
                .expect(&format!("Failed to save {}", filename));
        } else {
            img.save_with_format(&filename, image::ImageFormat::Jpeg)
                .expect(&format!("Failed to save {}", filename));
        }

        if (i + 1) % 10 == 0 {
            println!("Generated {} images...", i + 1);
        }
    }

    println!("Successfully generated 100 test images in '{}'", output_dir);
    println!("50 PNG files and 50 JPEG files");
}
