use crate::{core::{Printable, bounds::Bounds3, interaction::InteractionBase}, interaction::surface_interaction::{Shading, SurfaceInteraction}};

use std::{cell::Cell, sync::Arc};

use crate::core::medium::Medium;

pub type Vector3 = nalgebra::Vector3<f32>;
pub type Vector2 = nalgebra::Vector2<f32>;
pub type Point3 = nalgebra::Point3<f32>;
pub type Point2 = nalgebra::Point2<f32>;
pub type Normal3 = nalgebra::Vector3<f32>;
pub type AngleAxis = nalgebra::Vector4<f32>;
pub type Transform = nalgebra::Projective3<f32>;

#[derive(Debug, Clone)]
pub struct Ray {
    pub o: Point3,
    pub d: Vector3,
    pub t_max: Cell<f32>,
    pub time: f32,
    pub medium: Option<Arc<Medium>>,

    pub differential: Option<RayDifferential>    
}

impl Ray {
    pub fn new() -> Self {
        Self {
            o: Point3::origin(),
            d: Vector3::zeros(),
            t_max: Cell::new(f32::INFINITY),
            time: 0.0,
            medium: None,

            differential: None
        }
    }

    pub fn init(o: &Point3, d: &Vector3, t_max: f32, time: f32, medium: Option<Arc<Medium>>, differential: Option<RayDifferential>) -> Self {
        Self {
            o: o.clone(),
            d: d.clone(),
            t_max: Cell::new(t_max),
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
            self.t_max.get(),
            self.time,
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
        Ray::init(&r_o, &r_d, r.t_max.get(), r.time.clone(), r.medium.clone(), Some(rd))
    } else {
        Ray::init(&r_o, &r_d, r.t_max.get(), r.time.clone(), r.medium.clone(), None)
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
        primitive: si.primitive.clone()
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

pub fn permute_v(v: &Vector3, x: usize, y: usize, z: usize) -> Vector3 {
    Vector3::new(v[x], v[y], v[z])
}

pub fn permute_p(p: &Point3, x: usize, y: usize, z: usize) -> Point3 {
    Point3::new(p[x], p[y], p[z])
}

pub fn rotate_angle_axis(aa: AngleAxis) -> Transform {
    use nalgebra::{Vector3, Rotation3, Unit, Projective3};

    let axis = Vector3::new(aa.x, aa.y, aa.z);
    let angle = aa.w.to_radians();

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

pub fn look_at(eye: &Point3, target: &Point3, up: &Vector3) -> Transform {
    let dir = (target - eye).normalize();
    let right = up.normalize().cross(&dir).normalize();
    let new_up = dir.cross(&right);

    let mut m = nalgebra::Matrix4::identity();
    m[(0, 0)] = right.x;   m[(1, 0)] = right.y;   m[(2, 0)] = right.z;
    m[(0, 1)] = new_up.x;  m[(1, 1)] = new_up.y;  m[(2, 1)] = new_up.z;
    m[(0, 2)] = dir.x;     m[(1, 2)] = dir.y;      m[(2, 2)] = dir.z;
    m[(0, 3)] = eye.x;     m[(1, 3)] = eye.y;      m[(2, 3)] = eye.z;

    Transform::from_matrix_unchecked(m)
}

// trait FloorCeil {
//     fn floor(self) -> Self;
//     fn ceil(self) -> Self;
// }

// impl FloorCeil for Vector3 {
//     fn floor(self) -> Self {
//         self.map(|x| x.floor())
//     }

//     fn ceil(self) -> Self {
//         self.map(|x| x.ceil())
//     }
// }

// impl FloorCeil for Point3 {
//     fn floor(self) -> Self {
//         self.map(|x| x.floor())
//     }

//     fn ceil(self) -> Self {
//         self.map(|x| x.ceil())
//     }
// }

// impl FloorCeil for Vector2 {
//     fn floor(self) -> Self {
//         self.map(|x| x.floor())
//     }

//     fn ceil(self) -> Self {
//         self.map(|x| x.ceil())
//     }
// }

// impl FloorCeil for Point2 {
//     fn floor(self) -> Self {
//         self.map(|x| x.floor())
//     }

//     fn ceil(self) -> Self {
//         self.map(|x| x.ceil())
//     }
// }

pub fn spherical_direction(sin_theta: f32, cos_theta: f32, phi: f32) -> Vector3 {
    Vector3::new(
        sin_theta * phi.cos(),
        sin_theta * phi.sin(),
        cos_theta
    )
}

pub fn spherical_direction_with_ref(sin_theta: f32, cos_theta: f32, phi: f32, x: &Vector3, y: &Vector3, z: &Vector3) -> Vector3 {
    sin_theta * phi.cos() * x + sin_theta * phi.sin() * y + cos_theta * z
}