use std::{cell::Cell, sync::Arc};

use crate::core::{Normal3, Point2, Point3, Ray, Spectrum, Vector3, bsdf::{BSDF, BSSRDF}, face_forward, interaction::{InteractionBase, InteractionT, TransportMode}, medium::{Medium, MediumInterface}, primitive::{GeometricPrimitive, Primitive}, shape::Shape};

pub struct Shading {
    pub n: Normal3,

    pub dpdu: Vector3,
    pub dpdv: Vector3,
    pub dndu: Normal3,
    pub dndv: Normal3
}

impl Shading {
    pub fn new() -> Self {
        Self {
            n: Normal3::zeros(),
            dpdu: Vector3::zeros(),
            dpdv: Vector3::zeros(),
            dndu: Normal3::zeros(),
            dndv: Normal3::zeros()
        }
    }
}

pub struct SurfaceInteraction {
    pub base: InteractionBase,

    pub uv: Point2,
    pub dpdu: Vector3,
    pub dpdv: Vector3,
    pub dndu: Normal3,
    pub dndv: Normal3,

    pub shape: Option<Arc<Shape>>,
    pub shading: Shading,

    pub bsdf: Option<Arc<BSDF>>,
    pub bssrdf: Option<Arc<BSSRDF>>,

    pub dpdx: Cell<Vector3>,
    pub dpdy: Cell<Vector3>,

    pub dudx: Cell<f32>,
    pub dvdx: Cell<f32>,
    pub dudy: Cell<f32>,
    pub dvdy: Cell<f32>,

    pub primitive: Option<Primitive>,
}

impl InteractionT for SurfaceInteraction {
    fn new() -> Self {
        Self {
            base: InteractionBase::new(),

            uv: Point2::origin(),
            dpdu: Vector3::zeros(),
            dpdv: Vector3::zeros(),
            dndu: Normal3::zeros(),
            dndv: Normal3::zeros(),
            shape: None,
            shading: Shading::new(),
            bsdf: None,
            bssrdf: None,
            dpdx: Cell::new(Vector3::zeros()),
            dpdy: Cell::new(Vector3::zeros()),
            dudx: Cell::new(0.0),
            dvdx: Cell::new(0.0),
            dudy: Cell::new(0.0),
            dvdy: Cell::new(0.0),
            primitive: None
        }
    }

    fn init(p: &Point3, n: &Normal3, p_error: &Vector3, wo: &Vector3, time: f32, medium_interface: MediumInterface) -> Self {
        Self {
            base: InteractionBase::init(p, n, p_error, wo, time, medium_interface),

            uv: Point2::origin(),
            dpdu: Vector3::zeros(),
            dpdv: Vector3::zeros(),
            dndu: Normal3::zeros(),
            dndv: Normal3::zeros(),
            shape: None,
            shading: Shading::new(),
            bsdf: None,
            bssrdf: None,
            dpdx: Cell::new(Vector3::zeros()),
            dpdy: Cell::new(Vector3::zeros()),
            dudx: Cell::new(0.0),
            dvdx: Cell::new(0.0),
            dudy: Cell::new(0.0),
            dvdy: Cell::new(0.0),
            primitive: None
        }
    }

    fn init_no_normal(p: &Point3, wo: &Vector3, time: f32, medium_interface: MediumInterface) -> Self {
        Self {
            base: InteractionBase::init_no_normal(p, wo, time, medium_interface),

            uv: Point2::origin(),
            dpdu: Vector3::zeros(),
            dpdv: Vector3::zeros(),
            dndu: Normal3::zeros(),
            dndv: Normal3::zeros(),
            shape: None,
            shading: Shading::new(),
            bsdf: None,
            bssrdf: None,
            dpdx: Cell::new(Vector3::zeros()),
            dpdy: Cell::new(Vector3::zeros()),
            dudx: Cell::new(0.0),
            dvdx: Cell::new(0.0),
            dudy: Cell::new(0.0),
            dvdy: Cell::new(0.0),
            primitive: None
        }
    }
    fn init_no_wo(p: &Point3, time: f32, medium_interface: MediumInterface) -> Self {
        Self {
            base: InteractionBase::init_no_wo(p, time, medium_interface),

            uv: Point2::origin(),
            dpdu: Vector3::zeros(),
            dpdv: Vector3::zeros(),
            dndu: Normal3::zeros(),
            dndv: Normal3::zeros(),
            shape: None,
            shading: Shading::new(),
            bsdf: None,
            bssrdf: None,
            dpdx: Cell::new(Vector3::zeros()),
            dpdy: Cell::new(Vector3::zeros()),
            dudx: Cell::new(0.0),
            dvdx: Cell::new(0.0),
            dudy: Cell::new(0.0),
            dvdy: Cell::new(0.0),
            primitive: None
        }
    }

    fn get_p(&self) -> &Point3 { &self.base.p }
    fn get_time(&self) -> &f32 { &self.base.time }
    fn get_p_error(&self) -> &Vector3 { &self.base.p_error }
    fn get_wo(&self) -> &Vector3 { &self.base.wo }
    fn get_n(&self) -> &Normal3 { &self.base.n }
    fn get_medium_interface(&self) -> &MediumInterface { &self.base.medium_interface }

    fn is_surface_interaction(&self) -> bool { true }
    fn is_medium_interaction(&self) -> bool { false }

    fn get_medium(&self) -> Option<Arc<Medium>> { self.base.get_medium() }
    fn get_medium_facing_vector(&self, w: &Vector3) -> Option<Arc<Medium>> { self.base.get_medium_facing_vector(w) }
    fn spawn_ray(&self, d: &Vector3) -> Ray { self.base.spawn_ray(d) }
    fn spawn_ray_to(&self, p2: Point3) -> Ray { self.base.spawn_ray_to(p2) }
    fn spawn_ray_to_interaction(&self, it: &InteractionBase) -> Ray { self.base.spawn_ray_to_interaction(it) }
}

impl SurfaceInteraction {
    pub fn init(p: &Point3, p_error: &Vector3, uv: &Point2, wo: &Vector3, dpdu: &Vector3, dpdv: &Vector3, dndu: &Normal3, dndv: &Normal3, time: f32, shape: Option<Arc<Shape>>) -> Self {
        let n: Normal3 = dpdu.cross(dpdv).normalize();

        let mut base = InteractionBase::init(p, &n, p_error, wo, time, MediumInterface::new());

        let mut shading = Shading {
            n,
            dpdu: dpdu.clone(),
            dpdv: dpdv.clone(),
            dndu: dndu.clone(),
            dndv: dndv.clone(),
        };

        if let Some(s) = &shape {
            if s.get_reverse_orientation() ^ s.get_transform_swaps_handedness() {
                base.n *= -1.0;
                shading.n *= -1.0;
            }
        }

        Self {
            base,

            uv: *uv,
            dpdu: *dpdu,
            dpdv: *dpdv,
            dndu: *dndu,
            dndv: *dndv,
            shape: shape,
            shading,
            bsdf: None,
            bssrdf: None,
            dpdx: Cell::new(Vector3::zeros()),
            dpdy: Cell::new(Vector3::zeros()),
            dudx: Cell::new(0.0),
            dvdx: Cell::new(0.0),
            dudy: Cell::new(0.0),
            dvdy: Cell::new(0.0),
            primitive: None
        }
    }

    pub fn set_shading_geometry(&mut self, dpdus: &Vector3, dpdvs: &Vector3, dndus: &Normal3, dndvs: &Vector3, orientation_is_auth: bool) {
        self.shading.n = dpdus.cross(dpdvs).normalize();

        if let Some(s) = &self.shape {
            if s.get_reverse_orientation() ^ s.get_transform_swaps_handedness() {
                self.base.n = face_forward(self.get_n(), &self.shading.n);
            }
        }

        if orientation_is_auth {
            self.base.n = face_forward(self.get_n(), &self.shading.n);
        } else {
            self.shading.n = face_forward(&self.shading.n, &self.base.n);
        }

        self.shading.dpdu = dpdus.clone();
        self.shading.dpdv = dpdvs.clone();
        self.shading.dndu = dndus.clone();
        self.shading.dndv = dndvs.clone();
    }

    pub fn compute_scattering_functions(&mut self, _ray: &Ray, _allow_multiple_lobs: Option<bool>, _mode: Option<TransportMode>) {
        todo!("surface_inter::compute_scattering");
    }

    pub fn compute_differentials(&self, _r: &Ray) {
        todo!("surface_inter::compute_diff");
    }
    
    pub fn le(&self, _w: &Vector3) -> Spectrum {
        todo!("surface_inter::le");
    }
}