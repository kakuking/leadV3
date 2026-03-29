use core::f32;

use crate::core::{Normal3, Point3, Vector3};

pub const INFINITY: f32 = f32::INFINITY;
pub const MIN: f32 = f32::MIN;
pub const MAX: f32 = f32::MAX;
pub const PI: f32 = f32::consts::PI;
pub const INV_PI: f32 = f32::consts::FRAC_1_PI;
pub const INV_4PI: f32 = 0.25 * INV_PI;
pub const EPSILON: f32 = 1e-4;
pub const MACHINE_EPSILON: f32 = f32::EPSILON * 0.5;
pub const ONE_MINUS_EPSILON: f32 = 1.0 - EPSILON;

pub  fn gamma(n: f32) -> f32 {
    n * MACHINE_EPSILON / (1.0 - n * MACHINE_EPSILON)
}

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

pub fn quadratic(a: f32, b: f32, c: f32, t0: &mut f32, t1: &mut f32) -> bool {
    let d2 = b * b - 4.0 * a * c;

    if d2 < 0.0 {
        return false;
    }

    let d = d2.sqrt();

    let q = if b < 0.0 {
        -0.5 * (b - d)
    } else {
        -0.5 * (b + d)
    };

    *t0 = q / a;
    *t1 = c / q;

    if *t0 > *t1 {
        std::mem::swap(t0, t1);
    }

    true
}

pub fn cos_theta(w: &Vector3) -> f32 { w.z }
pub fn cos2_theta(w: &Vector3) -> f32 { w.z * w.z }
pub fn abs_cos_theta(w: &Vector3) -> f32 { w.z.abs() }
pub fn sin2_theta(w: &Vector3) -> f32 { 0f32.max(1.0 - cos2_theta(w)) }
pub fn sin_theta(w: &Vector3) -> f32 { sin2_theta(w).sqrt() }
pub fn tan_theta(w: &Vector3) -> f32 { sin_theta(w) / cos_theta(w) }
pub fn tan2_theta(w: &Vector3) -> f32 { sin2_theta(w) / cos2_theta(w) }
pub fn cos_phi(w: &Vector3) -> f32 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        1.0
    } else {
        (w.x / sin_theta).clamp(-1.0, 1.0)
    }
}
pub fn sin_phi(w: &Vector3) -> f32 {
    let sin_theta = sin_theta(w);
    if sin_theta == 0.0 {
        0.0
    } else {
        (w.y / sin_theta).clamp(-1.0, 1.0)
    }
}
pub fn cos2_phi(w: &Vector3) -> f32 { cos_phi(w) * cos_phi(w) }
pub fn sin2_phi(w: &Vector3) -> f32 { sin_phi(w) * sin_phi(w) }
pub fn cosd_phi(wa: &Vector3, wb: &Vector3) -> f32 {
    let num = wa.x * wb.x + wa.y * wb.y;
    let den = ((wa.x * wa.x + wa.y * wa.y) * (wb.x * wb.x + wb.y * wb.y)).sqrt();

    (num / den).clamp(-1.0, 1.0)
}

pub fn solve_linear_system_2x2(a: [[f32; 2]; 2], b: [f32; 2]) -> Option<(f32, f32)> {
    let det = a[0][0] * a[1][1] - a[0][1] * a[1][0];

    // Same spirit as PBRT: reject nearly singular systems
    if det.abs() < 1e-10 {
        return None;
    }

    let x0 = ( b[0] * a[1][1] - b[1] * a[0][1]) / det;
    let x1 = (-b[0] * a[1][0] + b[1] * a[0][0]) / det;

    if x0.is_nan() || x1.is_nan() {
        None
    } else {
        Some((x0, x1))
    }
}

pub fn multiplicative_inverse(a: usize, n: usize) -> usize {
    let (g, x, _) = extended_gcd(a as i64, n as i64);

    assert!(g == 1, "No modular inverse for {} mod {}", a, n);

    x.rem_euclid(n as i64) as usize
}

pub fn extended_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b == 0 {
        (a, 1, 0)
    } else {
        let (g, x1, y1) = extended_gcd(b, a % b);
        (g, y1, x1 - (a / b) * y1)
    }
}

pub fn mod_i32(a: i32, b: i32) -> i32 {
    a.rem_euclid(b)
}