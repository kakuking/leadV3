use exr::prelude::*;
use image::{ImageBuffer, Rgb};
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::core::{bounds::Bounds2, Point2};

#[derive(Debug)]
pub enum WriteImageError {
    Io(std::io::Error),
    Exr(exr::error::Error),
    Image(image::ImageError),
    UnsupportedExtension(String),
    MissingExtension,
    BufferSizeMismatch { expected: usize, actual: usize },
}

impl From<std::io::Error> for WriteImageError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<exr::error::Error> for WriteImageError {
    fn from(err: exr::error::Error) -> Self {
        Self::Exr(err)
    }
}

impl From<image::ImageError> for WriteImageError {
    fn from(err: image::ImageError) -> Self {
        Self::Image(err)
    }
}

fn cropped_dimensions(cropped_pixel_bounds: &Bounds2) -> (usize, usize) {
    let width =
        (cropped_pixel_bounds.p_max.x - cropped_pixel_bounds.p_min.x) as usize;
    let height =
        (cropped_pixel_bounds.p_max.y - cropped_pixel_bounds.p_min.y) as usize;

    (width, height)
}

pub fn write_exr(
    filename: &str,
    rgb: &[f32],
    cropped_pixel_bounds: Bounds2,
    _full_resolution: Point2,
) -> Result<()> {
    let (width, height) = cropped_dimensions(&cropped_pixel_bounds);

    write_rgb_file(Path::new(filename), width, height, |x, y| {
        let i = 3 * (y * width + x);
        (rgb[i], rgb[i + 1], rgb[i + 2])
    })?;

    Ok(())
}

pub fn write_ppm(
    filename: &str,
    rgb: &[f32],
    cropped_pixel_bounds: Bounds2,
    _full_resolution: Point2,
) -> Result<()> {
    let (width, height) = cropped_dimensions(&cropped_pixel_bounds);

    let file = File::create(filename)?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "P3")?;
    writeln!(writer, "{} {}", width, height)?;
    writeln!(writer, "255")?;

    for i in 0..(width * height) {
        let r = rgb[3 * i].clamp(0.0, 1.0);
        let g = rgb[3 * i + 1].clamp(0.0, 1.0);
        let b = rgb[3 * i + 2].clamp(0.0, 1.0);

        let ir = (255.0 * r + 0.5) as u32;
        let ig = (255.0 * g + 0.5) as u32;
        let ib = (255.0 * b + 0.5) as u32;

        writeln!(writer, "{} {} {}", ir, ig, ib)?;
    }

    Ok(())
}

pub fn write_png(
    filename: &str,
    rgb: &[f32],
    cropped_pixel_bounds: Bounds2,
    _full_resolution: Point2,
) -> Result<()> {
    let (width, height) = cropped_dimensions(&cropped_pixel_bounds);

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> =
        ImageBuffer::new(width as u32, height as u32);

    for y in 0..height {
        for x in 0..width {
            let i = 3 * (y * width + x);

            let r = (255.0 * rgb[i].clamp(0.0, 1.0) + 0.5) as u8;
            let g = (255.0 * rgb[i + 1].clamp(0.0, 1.0) + 0.5) as u8;
            let b = (255.0 * rgb[i + 2].clamp(0.0, 1.0) + 0.5) as u8;

            img.put_pixel(x as u32, y as u32, Rgb([r, g, b]));
        }
    }

    img.save(filename).unwrap();

    Ok(())
}

pub fn write_image(
    filename: &str,
    rgb: &[f32],
    cropped_pixel_bounds: Bounds2,
    full_resolution: Point2,
) -> Result<()> {
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or(WriteImageError::MissingExtension).unwrap()
        .to_ascii_lowercase();

    match ext.as_str() {
        "exr" => write_exr(filename, rgb, cropped_pixel_bounds, full_resolution),
        "ppm" => write_ppm(filename, rgb, cropped_pixel_bounds, full_resolution),
        "png" => write_png(filename, rgb, cropped_pixel_bounds, full_resolution),
        other => panic!("Unsupported filetype: {}", ext),
    }
}