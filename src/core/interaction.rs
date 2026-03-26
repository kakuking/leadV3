use std::{cell::Cell, sync::Arc};

use crate::{core::{EPSILON, INFINITY, Normal3, Point3, Ray, Vector3, medium::{Medium, MediumInterface}, offset_ray_origin}, interaction::surface_interaction::SurfaceInteraction};

pub enum TransportMode {
    Radiance,
    Importance
}

#[derive(Debug, Clone)]
pub enum Interaction {
    Base(InteractionBase),
    Surface(SurfaceInteraction),
    // Medium(MediumInteraction)
}

impl Interaction {
    pub fn get_base(&self) -> &InteractionBase {
        match self {
            Self::Surface(s) => s.get_base(),
            Self::Base(s) => s
        }
    }
    pub fn get_p(&self) -> &Point3 {
        match self {
            Self::Surface(s) => s.get_p(),
            Self::Base(s) => &s.p
        }
    }
    pub fn get_time(&self) -> &f32 {
        match self {
            Self::Surface(s) => s.get_time(),
            Self::Base(s) => &s.time
        }
    }
    pub fn get_p_error(&self) -> &Vector3 {
        match self {
            Self::Surface(s) => s.get_p_error(),
            Self::Base(s) => &s.p_error
        }
    }
    pub fn get_wo(&self) -> &Vector3 {
        match self {
            Self::Surface(s) => s.get_wo(),
            Self::Base(s) => &s.wo
        }
    }
    pub fn get_n(&self) -> &Normal3 {
        match self {
            Self::Surface(s) => s.get_n(),
            Self::Base(s) => &s.n
        }
    }
    pub fn get_medium_interface(&self) -> &MediumInterface {
        match self {
            Self::Surface(s) => s.get_medium_interface(),
            Self::Base(s) => &s.medium_interface
        }
    }

    pub fn is_surface_interaction(&self) -> bool {
        match self {
            Self::Surface(_) => true,
            _ => false
        }
    }
    pub fn is_medium_interaction(&self) -> bool {
        match self {
            Self::Surface(_) => false,
            _ => true
        }
    }

    pub fn get_medium(&self) -> Option<Arc<Medium>> {
        match self {
            Self::Surface(s) => s.get_medium(),
            Self::Base(s) => s.get_medium()
        }
    }
    pub fn get_medium_facing_vector(&self, w: &Vector3) ->Option<Arc<Medium>> {
        match self {
            Self::Surface(s) => s.get_medium_facing_vector(w),
            Self::Base(s) => s.get_medium_facing_vector(w)
        }
    }
    pub fn spawn_ray(&self, d: &Vector3) -> Ray {
        match self {
            Self::Surface(s) => s.spawn_ray(d),
            Self::Base(s) => s.spawn_ray(d)
        }
    }
    pub fn spawn_ray_to(&self, p2: Point3) -> Ray {
        match self {
            Self::Surface(s) => s.spawn_ray_to(p2),
            Self::Base(s) => s.spawn_ray_to(p2)
        }
    }
    pub fn spawn_ray_to_interaction(&self, it: &InteractionBase) -> Ray {
        match self {
            Self::Surface(s) => s.spawn_ray_to_interaction(it),
            Self::Base(s) => s.spawn_ray_to_interaction(it)
        }
    }
}

pub trait InteractionT {
    fn new() -> Self;
    fn get_base(&self) -> &InteractionBase;
    fn init(p: &Point3, n: &Normal3, p_error: &Vector3, wo: &Vector3, time: f32, medium_interface: MediumInterface) -> Self;
    fn init_no_normal(p: &Point3, wo: &Vector3, time: f32, medium_interface: MediumInterface) -> Self;
    fn init_no_wo(p: &Point3, time: f32, medium_interface: MediumInterface) -> Self;

    fn get_p(&self) -> &Point3;
    fn get_time(&self) -> &f32;
    fn get_p_error(&self) -> &Vector3;
    fn get_wo(&self) -> &Vector3;
    fn get_n(&self) -> &Normal3;
    fn get_medium_interface(&self) -> &MediumInterface;

    fn is_surface_interaction(&self) -> bool;
    fn is_medium_interaction(&self) -> bool;

    fn get_medium(&self) -> Option<Arc<Medium>>;
    fn get_medium_facing_vector(&self, w: &Vector3) -> Option<Arc<Medium>>;
    fn spawn_ray(&self, d: &Vector3) -> Ray;
    fn spawn_ray_to(&self, p2: Point3) -> Ray;
    fn spawn_ray_to_interaction(&self, it: &InteractionBase) -> Ray;
}

#[derive(Debug, Clone)]
pub struct InteractionBase {
    pub p: Point3,
    pub time: f32,
    pub p_error: Vector3,
    pub wo: Vector3,
    pub n: Normal3,
    pub medium_interface: MediumInterface
}

impl InteractionBase {
    pub fn new() -> Self {
        Self {
            p: Point3::origin(),
            time: 0.0,
            p_error: Vector3::zeros(),
            wo: Vector3::zeros(),
            n: Normal3::zeros(),
            medium_interface: MediumInterface::new()
        }
    }

    pub fn init(p: &Point3, n: &Normal3, p_error: &Vector3, wo: &Vector3, time: f32, medium_interface: MediumInterface) -> Self {
        Self {
            p: p.clone(),
            time,
            p_error: p_error.clone(),
            wo: wo.clone(),
            n: n.clone(),
            medium_interface
        }
    }

    pub fn init_no_normal(p: &Point3, wo: &Vector3, time: f32, medium_interface: MediumInterface) -> Self {
        Self {
            p: p.clone(),
            time,
            p_error: Vector3::zeros(),
            wo: wo.clone(),
            n: Vector3::zeros(),
            medium_interface
        }
    }
    
    pub fn init_no_wo(p: &Point3, time: f32, medium_interface: MediumInterface) -> Self {
        Self {
            p: p.clone(),
            time,
            p_error: Vector3::zeros(),
            wo: Vector3::zeros(),
            n: Vector3::zeros(),
            medium_interface
        }
    }

    pub fn is_surface_interaction(&self) -> bool {
        self.n != Normal3::zeros()
    }

    pub fn get_medium(&self) -> Option<Arc<Medium>> {
        assert!(self.medium_interface.inside == self.medium_interface.outside);

        self.medium_interface.inside.clone()
    }

    pub fn get_medium_facing_vector(&self, w: &Vector3) -> Option<Arc<Medium>> {
        if self.n.dot(w) > 0.0 {
            self.medium_interface.outside.clone()
        } else {
            self.medium_interface.inside.clone()
        }
    }

    pub fn spawn_ray(&self, d: &Vector3) -> Ray {
        let o = offset_ray_origin(&self.p, &self.p_error, &self.n, d);

        Ray::init(&o, d, INFINITY, self.time, self.get_medium(), None)
    }

    pub fn spawn_ray_to(&self, p2: Point3) -> Ray {
        let origin = offset_ray_origin(&self.p, &self.p_error, &self.n, &(p2 - self.p));
        let d = p2 - origin;

        Ray::init(&origin, &d, 1.0 - EPSILON, self.time, self.get_medium(), None)
    }

    pub fn spawn_ray_to_interaction(&self, it: &InteractionBase) -> Ray {
        let o = offset_ray_origin(&self.p, &self.p_error, &self.n, &(it.p - self.p));
        let t = offset_ray_origin(&it.p, &it.p_error, &it.n, &(o - it.p));
        let d = (t - o).normalize();

        Ray::init(&o, &d, 1.0-EPSILON, self.time, self.get_medium(), None)
    }

}

pub struct MediumInteraction {

}