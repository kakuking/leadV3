use crate::{core::{Printable, interaction::InteractionBase, math::*}, interaction::surface_interaction::{Shading, SurfaceInteraction}};

use std::{cell::Cell, ops::Index, sync::Arc};

use crate::core::medium::Medium;

pub type Vector3 = nalgebra::Vector3<f32>;
pub type Vector2 = nalgebra::Vector2<f32>;
pub type Point3 = nalgebra::Point3<f32>;
pub type Point2 = nalgebra::Point2<f32>;
pub type Normal3 = nalgebra::Vector3<f32>;
pub type AngleAxis = nalgebra::Vector4<f32>;
pub type Transform = nalgebra::Projective3<f32>;

#[derive(Debug)]
pub struct Ray {
    pub o: Point3,
    pub d: Vector3,
    pub t_max: f32,
    pub time: Cell<f32>,
    pub medium: Option<Arc<Medium>>,

    pub differential: Option<RayDifferential>    
}

impl Ray {
    pub fn new() -> Self {
        Self {
            o: Point3::origin(),
            d: Vector3::zeros(),
            t_max: f32::INFINITY,
            time: Cell::new(0.0),
            medium: None,

            differential: None
        }
    }

    pub fn init(o: &Point3, d: &Vector3, t_max: f32, time: Cell<f32>, medium: Option<Arc<Medium>>, differential: Option<RayDifferential>) -> Self {
        Self {
            o: o.clone(),
            d: d.clone(),
            t_max,
            time,
            medium,
            differential
        }
    }

    pub fn at(&self, t: f32) -> Point3 {
        self.o + self.d * t
    }

    pub fn has_differntials(&self) -> bool {
        self.differential.is_some()
    }

    pub fn scale_differentials(&mut self, s: f32) {
        if let Some(rd) = self.differential.as_mut() {
            rd.rx_o = self.o + (rd.rx_o - self.o) * s;
            rd.ry_o = self.o + (rd.ry_o - self.o) * s;

            rd.rx_d = self.d + (rd.rx_d - self.d) * s;
            rd.ry_d = self.d + (rd.ry_d - self.d) * s;
        }
    }
}

impl Printable for Ray {
    fn to_string(&self) -> String {
        let differential_str = match &self.differential {
            Some(rd) => rd.to_string(),
            None => "None".to_string(),
        };

        let medium_str = match &self.medium {
            Some(_) => "Some(Medium)".to_string(),
            None => "None".to_string(),
        };

        format!(
            "Ray [\n\
                o: ({:.6}, {:.6}, {:.6}),\n\
                d: ({:.6}, {:.6}, {:.6}),\n\
                t_max: {:.6},\n\
                time: {:.6},\n\
                medium: {},\n\
                differential: {}\n\
            ]",
            self.o.x, self.o.y, self.o.z,
            self.d.x, self.d.y, self.d.z,
            self.t_max,
            self.time.get(),
            medium_str,
            differential_str
        )
    }
}

#[derive(Debug, Clone)]
pub struct RayDifferential {
    pub rx_o: Point3,
    pub ry_o: Point3,
    pub rx_d: Vector3,
    pub ry_d: Vector3,
}

impl RayDifferential {
    pub fn new() -> Self {
        Self {
            rx_o: Point3::origin(),
            ry_o: Point3::origin(),
            rx_d: Vector3::zeros(),
            ry_d: Vector3::zeros(),
        }
    }
}

impl Printable for RayDifferential {
    fn to_string(&self) -> String {
        format!(
            "RayDifferential [\n\
                rx_o: ({:.6}, {:.6}, {:.6}),\n\
                ry_o: ({:.6}, {:.6}, {:.6}),\n\
                rx_d: ({:.6}, {:.6}, {:.6}),\n\
                ry_d: ({:.6}, {:.6}, {:.6})\n\
            ]",
            self.rx_o.x, self.rx_o.y, self.rx_o.z,
            self.ry_o.x, self.ry_o.y, self.ry_o.z,
            self.rx_d.x, self.rx_d.y, self.rx_d.z,
            self.ry_d.x, self.ry_d.y, self.ry_d.z,
        )
    }
}

#[derive(PartialEq)]
pub struct Bounds2 {
    p_min: Point2,
    p_max: Point2
}

impl Bounds2 {
    pub fn new() -> Self {
        Self {
            p_min: Point2::new(MAX, MAX),
            p_max: Point2::new(MIN, MIN)
        }
    }

    pub fn init_one(p: &Point2) -> Self {
        Self {
            p_min: p.clone(),
            p_max: p.clone()
        }
    }

    pub fn init_two(p1: &Point2, p2: &Point2) -> Self {
        Self {
            p_min: Point2::new(p1.x.min(p2.x), p1.y.min(p2.y)),
            p_max: Point2::new(p1.x.max(p2.x), p1.y.max(p2.y)),
        }
    }

    pub fn diagonal(&self) -> Vector2 {
        self.p_max - self.p_min
    }

    pub fn area(&self) -> f32 {
        let d = self.diagonal();

        d.x * d.y
    }

    // Returns which extent is longer
    pub fn max_extent(&self) -> usize {
        let d = self.diagonal();

        if d.x > d.y {
            0
        } else {
            1
        }
    }

    pub fn lerp(&self, t: &Point2) -> Point2 {
        let x = lerp(t.x, self.p_min.x, self.p_max.x);
        let y = lerp(t.y, self.p_min.y, self.p_max.y);

        Point2::new(x, y)
    }

    pub fn offset(&self, p: Point2) -> Point2 {
        let mut o: Point2 = (p - self.p_min).into();

        if self.p_max.x > self.p_min.y {
            o.x /= self.p_max.x - self.p_min.x;
        }

        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }

        o
    }

    pub fn bounding_sphere(&self, c: &mut Point2, r: &mut f32) {
        *c = (self.p_min -  (-1.0) * self.p_max).into();
        *c /= 2.0;

        *r = if self.inside(&c) {
            (self.p_max - *c).norm()
        } else {
            0.0
        };
    }

    pub fn inside(&self, p: &Point2) -> bool {
        self.p_min.x < p.x && p.x < self.p_max.x &&
        self.p_min.y < p.y && p.y < self.p_max.y
    }
}

impl Index<usize> for Bounds2 {
    type Output = Point2;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index == 0 || index == 1);

        if index == 0 {
            &self.p_min
        } else {
            &self.p_max
        }
    }
}

#[derive(PartialEq)]
pub struct Bounds3 {
    p_min: Point3,
    p_max: Point3
}

impl Bounds3 {
    pub fn new() -> Self {
        Self {
            p_min: Point3::new(MAX, MAX, MAX),
            p_max: Point3::new(MIN, MIN, MIN)
        }
    }

    pub fn init_one(p: &Point3) -> Self {
        Self {
            p_min: p.clone(),
            p_max: p.clone()
        }
    }

    pub fn init_two(p1: &Point3, p2: &Point3) -> Self {
        Self {
            p_min: Point3::new(p1.x.min(p2.x), p1.y.min(p2.y), p1.z.min(p2.z)),
            p_max: Point3::new(p1.x.max(p2.x), p1.y.max(p2.y), p1.z.max(p2.z)),
        }
    }

    pub fn diagonal(&self) -> Vector3 {
        self.p_max - self.p_min
    }

    pub fn corner(&self, corner: usize) -> Point3 {
        let x_idx = corner & 1;
        let y_idx = if (corner & 2) != 0 { 1 } else { 0 };
        let z_idx = if (corner & 4) != 0 { 1 } else { 0 };

        let x = self[x_idx].x;
        let y = self[y_idx].y;
        let z = self[z_idx].z;

        Point3::new(x, y, z)
    }

    pub fn volume(&self) -> f32 {
        let d = self.diagonal();

        d.x * d.y * d.z
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();

        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    // Returns which extent is longer
    pub fn max_extent(&self) -> usize {
        let d = self.diagonal();

        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    pub fn lerp(&self, t: &Point3) -> Point3 {
        let x = lerp(t.x, self.p_min.x, self.p_max.x);
        let y = lerp(t.y, self.p_min.y, self.p_max.y);
        let z = lerp(t.z, self.p_min.z, self.p_max.z);

        Point3::new(x, y, z)
    }

    pub fn offset(&self, p: Point3) -> Point3 {
        let mut o: Point3 = (p - self.p_min).into();

        if self.p_max.x > self.p_min.y {
            o.x /= self.p_max.x - self.p_min.x;
        }

        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }

        if self.p_max.z > self.p_min.z {
            o.z /= self.p_max.z - self.p_min.z;
        }

        o
    }

    pub fn bounding_sphere(&self, c: &mut Point3, r: &mut f32) {
        *c = (self.p_min -  (-1.0) * self.p_max).into();
        *c /= 2.0;

        *r = if self.inside(&c) {
            (self.p_max - *c).norm()
        } else {
            0.0
        };
    }

    pub fn inside(&self, p: &Point3) -> bool {
        self.p_min.x < p.x && p.x < self.p_max.x &&
        self.p_min.y < p.y && p.y < self.p_max.y &&
        self.p_min.z < p.z && p.z < self.p_max.z
    }

    pub fn union_p(&self, p: &Point3) -> Self {
        let p_minx = self.p_min.x.min(p.x);
        let p_miny = self.p_min.y.min(p.y);
        let p_minz = self.p_min.z.min(p.z);

        let p_maxx = self.p_max.x.max(p.x);
        let p_maxy = self.p_max.y.max(p.y);
        let p_maxz = self.p_max.z.max(p.z);

        Self {
            p_min: Point3::new(p_minx, p_miny, p_minz),
            p_max: Point3::new(p_maxx, p_maxy, p_maxz),
        }
    }

    pub fn union(&self, b: &Bounds3) -> Self {
        let p_minx = self.p_min.x.min(b.p_min.x);
        let p_miny = self.p_min.y.min(b.p_min.y);
        let p_minz = self.p_min.z.min(b.p_min.z);

        let p_maxx = self.p_max.x.max(b.p_max.x);
        let p_maxy = self.p_max.y.max(b.p_max.y);
        let p_maxz = self.p_max.z.max(b.p_max.z);

        Self {
            p_min: Point3::new(p_minx, p_miny, p_minz),
            p_max: Point3::new(p_maxx, p_maxy, p_maxz),
        }
    }

    pub fn intersect(&self, b: &Bounds3) -> Self {
        let p_minx = self.p_min.x.max(b.p_min.x);
        let p_miny = self.p_min.y.max(b.p_min.y);
        let p_minz = self.p_min.z.max(b.p_min.z);

        let p_maxx = self.p_max.x.min(b.p_max.x);
        let p_maxy = self.p_max.y.min(b.p_max.y);
        let p_maxz = self.p_max.z.min(b.p_max.z);

        Self {
            p_min: Point3::new(p_minx, p_miny, p_minz),
            p_max: Point3::new(p_maxx, p_maxy, p_maxz),
        }
    }

    pub fn overlaps(&self, b: Bounds3) -> bool {
        let x = self.p_max.x >= b.p_min.x && self.p_min.x <= b.p_max.x;
        let y = self.p_max.y >= b.p_min.y && self.p_min.y <= b.p_max.y;
        let z = self.p_max.z >= b.p_min.z && self.p_min.z <= b.p_max.z;

        x && y && z
    }

    pub fn intersect_p(&self, r: &Ray, hit_t0: &mut f32, hit_t1: &mut f32) -> bool {
        let mut t0 = 0.0f32;
        let mut t1 = r.t_max;

        for i in  0..3 {
            let inv_ray_dir =  1.0 / r.d[i];

            let  mut t_near = (self.p_min[i] - r.o[i]) * inv_ray_dir;
            let  mut t_far = (self.p_max[i] - r.o[i]) * inv_ray_dir;

            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
            }

            t_far *= 1.0 + 2.0 * gamma(3.0);

            t0 = if t_near > t0 { t_near } else { t0 };
            t1 = if  t_far < t1 { t_far } else { t1 };
            if t0 > t1 {
                return false;
            }
        }

        *hit_t0 = t0;
        *hit_t1 = t1;

        true
    }
    
    pub fn intersect_p_with_inv_dir(&self, r: &Ray, inv_dir: &Vector3, dir_is_neg: [u32; 3]) -> bool {
        let mut t_min = (self[dir_is_neg[0] as usize].x - r.o.x) * inv_dir.x;
        let mut t_max = (self[1 - dir_is_neg[0] as usize].x - r.o.x) * inv_dir.x;
        let ty_min = (self[dir_is_neg[1] as usize].y - r.o.y) * inv_dir.y;
        let mut ty_max = (self[1 - dir_is_neg[1] as usize].y - r.o.y) * inv_dir.y;

        t_max *= 1.0 + 2.0 * gamma(3.0);
        ty_max *= 1.0 + 2.0 * gamma(3.0);

        if  t_min > ty_max || ty_min > t_max {
            return false;
        }

        if ty_min > t_min {
            t_min = ty_min;
        }

        if ty_max < t_max {
            t_max = ty_max;
        }

        let tz_min = (self[dir_is_neg[2] as usize].z - r.o.z) * inv_dir.z;
        let mut tz_max = (self[1 - dir_is_neg[2] as usize].z - r.o.z) * inv_dir.z;

        tz_max *= 1.0 + 2.0 * gamma(3.0);

        if t_min > tz_max || tz_min > t_max {
            return false;
        }

        if tz_min > t_min {
            t_min = tz_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
        }

        t_min < r.t_max && t_max > 0.0
    }
}

impl Index<usize> for Bounds3 {
    type Output = Point3;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index == 0 || index == 1);

        if index == 0 {
            &self.p_min
        } else {
            &self.p_max
        }
    }
}

pub fn apply_transform_to_normal(n: &Vector3, t: &Transform) -> Vector3 {
    let mat3 = t.matrix().fixed_view::<3, 3>(0, 0).into_owned();
    let normal_matrix = mat3.try_inverse().unwrap().transpose(); 

    normal_matrix * n
}

pub fn transform_swaps_handedness(t: &Transform) -> bool {
    let lin = t.matrix().fixed_view::<3, 3>(0, 0);
    lin.determinant() < 0.0
}

pub fn apply_transform_to_ray(r: &Ray, t: &Transform) -> Ray {
    let r_o = t.transform_point(&r.o);
    let r_d = t.transform_vector(&r.d);

    let mut rd = RayDifferential::new();
    if let Some(differential) = r.differential.as_ref() {

        rd.rx_o = t.transform_point(&differential.rx_o);
        rd.ry_o = t.transform_point(&differential.ry_o);
        rd.rx_d = t.transform_vector(&differential.rx_d);
        rd.ry_d = t.transform_vector(&differential.ry_d);
    }

    if r.has_differntials() {
        Ray::init(&r_o, &r_d, r.t_max, r.time.clone(), r.medium.clone(), Some(rd))
    } else {
        Ray::init(&r_o, &r_d, r.t_max, r.time.clone(), r.medium.clone(), None)
    }
}

pub fn apply_transform_to_bounds(b: &Bounds3, t: &Transform) -> Bounds3 {
    let mut ret = Bounds3::init_one(&(t * Point3::new(b.p_min.x, b.p_min.y, b.p_min.z)));
    
    ret = ret.union_p(&(t * Point3::new(b.p_max.x, b.p_min.y, b.p_min.z)));
    ret = ret.union_p(&(t * Point3::new(b.p_min.x, b.p_max.y, b.p_min.z)));
    ret = ret.union_p(&(t * Point3::new(b.p_min.x, b.p_min.y, b.p_max.z)));
    ret = ret.union_p(&(t * Point3::new(b.p_min.x, b.p_max.y, b.p_max.z)));
    ret = ret.union_p(&(t * Point3::new(b.p_max.x, b.p_max.y, b.p_min.z)));
    ret = ret.union_p(&(t * Point3::new(b.p_max.x, b.p_min.y, b.p_max.z)));
    ret = ret.union_p(&(t * Point3::new(b.p_max.x, b.p_max.y, b.p_max.z)));
    ret
}

pub fn transform_point_with_error(t: &Transform, p: &Point3, p_error: &Vector3, out_error: &mut Vector3) -> Point3 {
    let p_out = t.transform_point(p);

    let m = t.matrix();
    let abs_error = |row: usize| -> f32 {
        (m[(row, 0)] * p_error.x).abs()
        + (m[(row, 1)] * p_error.y).abs()
        + (m[(row, 2)] * p_error.z).abs()
        + (m[(row, 0)] * p.x + m[(row, 1)] * p.y + m[(row, 2)] * p.z + m[(row, 3)]).abs()
            * f32::EPSILON * 0.5
    };

    *out_error = Vector3::new(abs_error(0), abs_error(1), abs_error(2));
    p_out
}

pub fn apply_transform_to_surface_interaction(si: &SurfaceInteraction, t: &Transform) -> SurfaceInteraction {
    // Transform p and p_error
    let mut p_error_out = Vector3::zeros();
    let p_out = transform_point_with_error(t, &si.base.p, &si.base.p_error, &mut p_error_out);

    // Transform normal and wo
    let n_out = apply_transform_to_normal(&si.base.n, t).normalize();
    let wo_out = t.transform_vector(&si.base.wo);

    // Transform differentials
    let dpdx_out = t.transform_vector(&si.dpdx.get());
    let dpdy_out = t.transform_vector(&si.dpdy.get());

    // Transform shading
    let shading_n = apply_transform_to_normal(&si.shading.n, t).normalize();
    let shading_n = face_forward(&shading_n, &n_out);

    let ret = SurfaceInteraction {
        base: InteractionBase {
            p: p_out,
            p_error: p_error_out,
            n: n_out,
            wo: wo_out,
            time: si.base.time,
            medium_interface: si.base.medium_interface.clone(),
        },
        uv: si.uv,
        dpdu: t.transform_vector(&si.dpdu),
        dpdv: t.transform_vector(&si.dpdv),
        dndu: apply_transform_to_normal(&si.dndu, t),
        dndv: apply_transform_to_normal(&si.dndv, t),
        shape: si.shape.clone(),
        shading: Shading {
            n: shading_n,
            dpdu: t.transform_vector(&si.shading.dpdu),
            dpdv: t.transform_vector(&si.shading.dpdv),
            dndu: apply_transform_to_normal(&si.shading.dndu, t),
            dndv: apply_transform_to_normal(&si.shading.dndv, t),
        },
        bsdf: si.bsdf.clone(),
        bssrdf: si.bssrdf.clone(),
        dpdx: Cell::new(dpdx_out),
        dpdy: Cell::new(dpdy_out),
        dudx: Cell::new(si.dudx.get()),
        dvdx: Cell::new(si.dvdx.get()),
        dudy: Cell::new(si.dudy.get()),
        dvdy: Cell::new(si.dvdy.get()),
    };

    ret
}

pub fn coordinate_system(v1: &Vector3, v2: &mut Vector3, v3: &mut Vector3) {
    if v1.x.abs() > v1.y.abs() {
        *v2 = Vector3::new(-v1.z, 0.0, v1.x) / (v1.x*v1.x + v1.z*v1.z).sqrt();
    } else {
        *v2 = Vector3::new(0.0, v1.z, -v1.y) / (v1.y*v1.y + v1.z*v1.z).sqrt();
    }

    *v3 = v1.cross(v2);
}

pub fn face_forward(n: &Normal3, v: &Vector3) -> Normal3 {

    if n.dot(v) < 0.0 {
        -n
    } else {
        n + Vector3::zeros()
    }
}

pub fn rotate_angle_axis(aa: AngleAxis) -> Transform {
    use nalgebra::{Vector3, Rotation3, Unit, Projective3};

    let axis = Vector3::new(aa.x, aa.y, aa.z);
    let angle = aa.w;

    let axis_unit = Unit::new_normalize(axis);
    let rotation = Rotation3::from_axis_angle(&axis_unit, angle);

    Projective3::from_matrix_unchecked(rotation.to_homogeneous())
}

pub fn translation(t: Vector3) -> Transform {
    use nalgebra::{Translation3, Projective3};

    let translation = Translation3::new(t.x, t.y, t.z);

    Projective3::from_matrix_unchecked(translation.to_homogeneous())
}

pub fn scaling(s: Vector3) -> Transform {
    use nalgebra::{Scale3, Projective3};

    let scale = Scale3::new(s.x, s.y, s.z);

    Projective3::from_matrix_unchecked(scale.to_homogeneous())
}