use core::f32;

use crate::core::{Normal3, Point3, Vector3};

pub const INFINITY: f32 = f32::INFINITY;
pub const MIN: f32 = f32::MIN;
pub const MAX: f32 = f32::MAX;
pub const PI: f32 = f32::consts::PI;
pub const INV_PI: f32 = f32::consts::FRAC_1_PI;
pub const INV_4PI: f32 = 0.25 * INV_PI;
pub const EPSILON: f32 = 0.0001;

pub fn lerp(t: f32, a: f32, b: f32) -> f32 {
    (1.0 - t) * a + t * b 
}

pub fn next_float_up(v: f32) -> f32 {
    if v.is_infinite() && v > 0.0 {
        return v;
    }
    let v = if v == -0.0 { 0.0 } else { v };
    let bits = v.to_bits();
    let bits = if v >= 0.0 { bits + 1 } else { bits - 1 };
    f32::from_bits(bits)
}

pub fn next_float_down(v: f32) -> f32 {
    if v.is_infinite() && v < 0.0 {
        return v;
    }
    let v = if v == 0.0 { -0.0 } else { v };
    let bits = v.to_bits();
    let bits = if v > 0.0 { bits - 1 } else { bits + 1 };
    f32::from_bits(bits)
}

pub fn offset_ray_origin(p: &Point3, p_error: &Vector3, n: &Normal3, w: &Vector3) -> Point3 {
    let n_abs = Vector3::new(n.x.abs(), n.y.abs(), n.z.abs());
    let d = n_abs.dot(p_error);

    let mut offset = d * Vector3::new(n.x, n.y, n.z);
    if w.dot(&Vector3::new(n.x, n.y, n.z)) < 0.0 {
        offset = -offset;
    }

    let mut po = p + offset;

    for i in 0..3 {
        if offset[i] > 0.0 {
            po[i] = next_float_up(po[i]);
        } else if offset[i] < 0.0 {
            po[i] = next_float_down(po[i]);
        }
    }

    po
}