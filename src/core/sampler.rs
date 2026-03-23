use crate::core::{PI, Point2, Vector2};

pub fn concentric_sample_disk(u: &Point2) -> Point2 {
    let u_offset = 2.0 * u - Vector2::new(1.0, 1.0);

    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2::origin();
    }

    let mut theta = 0.0;
    let mut r = 0.0;

    if u_offset.x.abs() > u_offset.y.abs() {
        r = u_offset.x;
        theta = 0.25 * PI * u_offset.y / u_offset.x;
    } else {
        r = u_offset.y;
        theta = 0.5 * PI - 0.25 * PI * u_offset.x / u_offset.y;
    }

    r * Point2::new(theta.cos(), theta.sin())
}