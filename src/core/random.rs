use nalgebra::ComplexField;

use crate::core::{INV_4PI, INV_PI, PI, Point2, Vector2, Vector3};

pub fn uniform_sample_hemisphere(u: &Point2) -> Vector3 {
    let z = u.x;
    let r = 0.0f32.max(1.0 - z*z).sqrt();
    let phi = 2.0 * PI * u.y;

    Vector3::new(
        r * phi.cos(),
        r * phi.sin(),
        z
    )
}

pub fn uniform_sample_hemisphere_pdf() -> f32 {
    INV_PI / 2.0
}

pub fn uniform_sample_sphere(u: &Point2) -> Vector3 {
    let z = 1.0 - 2.0 * u[0];
    let r = 0.0f32.max(1.0 - z*z).sqrt();
    let phi = 2.0 * PI * u.y;
    Vector3::new(
        r * phi.cos(),
        r * phi.sin(),
        z
    )
}

pub fn uniform_sample_sphere_pdf() -> f32 {
    INV_4PI
}

pub fn concentric_sample_disc(u: &Point2) -> Point2 {
    let u_offset = 2.0 * u - Vector2::new(1.0, 1.0);

    if u_offset.x == 0.0 && u_offset.y == 0.0 {
        return Point2::origin();
    }

    let theta: f32;
    let r: f32;

    if u_offset.x.abs() > u_offset.y.abs() {
        r = u_offset.x;
        theta = 0.25 * PI * (u_offset.y / u_offset.x);
    } else {
        r = u_offset.y;
        theta = 0.5 * PI - 0.25 * PI * (u_offset.x / u_offset.y);
    }

    r * Point2::new(theta.cos(), theta.sin())
}

pub fn cosine_sample_hemisphere(u: Point2) -> Vector3 {
    let d = concentric_sample_disc(&u);
    let z= (1.0 - d.x*d.x - d.y*d.y).max(0.0).sqrt();

    Vector3::new(
        d.x,
        d.y,
        z
    )
}

pub fn cosine_sample_hemisphere_pdf(cos_theta: f32) -> f32 {
    cos_theta * INV_PI
}

pub fn same_hemisphere(w: &Vector3, wp: &Vector3) -> bool {
    w.z * wp.z > 0.0
}

pub fn uniform_sample_triangle(u: &Point2) -> Point2 {
    let su0 = u[0].sqrt();

    Point2::new(1.0 - su0, u[1] * su0)
}

pub fn uniform_sample_cone(u: &Point2, cos_theta_max: f32) -> Vector3 {
    let cos_theta = (1.0 - u.x) + u.x * cos_theta_max;
    let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
    let phi = u.y * 2.0 * PI;

    Vector3::new(
        sin_theta * phi.cos(), 
        sin_theta * phi.sin(), 
        cos_theta
    )
}

pub fn uniform_cone_pdf(cos_theta_max: f32) -> f32 {
    1.0 / (2.0 * PI * (1.0 - cos_theta_max))
}