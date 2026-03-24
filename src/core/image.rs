use exr::prelude::*;
use std::path::Path;

use crate::core::{Bounds2, Point2};

pub fn write_image(
    filename: &str,
    rgb: &[f32],
    cropped_pixel_bounds: Bounds2,
    _full_resolution: Point2,
) -> exr::error::Result<()> {
    let width =
        (cropped_pixel_bounds.p_max.x - cropped_pixel_bounds.p_min.x) as usize;
    let height =
        (cropped_pixel_bounds.p_max.y - cropped_pixel_bounds.p_min.y) as usize;

    assert_eq!(
        rgb.len(),
        3 * width * height,
        "write_image: rgb buffer size does not match cropped pixel bounds"
    );

    write_rgb_file(Path::new(filename), width, height, |x, y| {
        let i = 3 * (y * width + x);
        (rgb[i], rgb[i + 1], rgb[i + 2])
    })
}