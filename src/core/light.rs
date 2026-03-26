use std::sync::Arc;

use crate::{core::{Normal3, Point2, Printable, Ray, Transform, Vector3, interaction::{Interaction, InteractionBase, InteractionT}, medium::MediumInterface, sampler::Sampler, scene::Scene, shape::Shape, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, light::{diffuse_area_light::DiffuseAreaLight, point_light::PointLight}, registry::Manufacturable};

#[derive(PartialEq)]
pub enum LightStrategy {
    UniformSampleAll,
    UniformSampleOne
}

#[derive(Debug, Clone, Copy)]
pub enum LightFlags {
    DeltaPosition=1,
    DeltaDirection=2,
    Area=4,
    Infinite=8,
}

pub fn is_delta_light(flags: u32) -> bool {
    flags & LightFlags::DeltaDirection as u32 != 0 ||
    flags & LightFlags::DeltaPosition as u32 != 0
}

#[derive(Debug, Clone)]
pub struct VisibilityTester {
    pub p0: InteractionBase,
    pub p1: InteractionBase
}

impl VisibilityTester {
    pub fn new() -> Self {
        Self {
            p0: InteractionBase::new(),
            p1: InteractionBase::new()
        }
    }

    pub fn init(p0: &InteractionBase, p1: &InteractionBase) -> Self {
        Self {
            p0: p0.clone(), 
            p1: p1.clone()
        }
    }

    pub fn get_p0(&self) -> &InteractionBase { &self.p0 }
    pub fn get_p1(&self) -> &InteractionBase { &self.p1 }
    pub fn unoccluded(&self, scene: &Scene) -> bool {
        !scene.intersect_p(&self.p0.spawn_ray_to_interaction(&self.p1))
    }

    pub fn tr(&self, scene: &Scene, sampler: &mut Sampler) -> Spectrum {
        let mut ray = self.p0.spawn_ray_to_interaction(&self.p1);
        let mut tr = Spectrum::new(1.0, 1.0, 1.0);

        loop {
            let mut si = SurfaceInteraction::new();
            let hit_surface = scene.intersect(&ray, &mut si);

            if hit_surface && si.primitive.get_material().is_some() {
                return Spectrum::zeros()
            }

            if let Some(m) = &ray.medium {
                tr.component_mul_assign(&m.tr(&ray, sampler));
            };

            if !hit_surface {
                break;
            }

            ray = si.spawn_ray_to_interaction(&self.p1);
        }

        tr
    }
}

pub fn blackbody(lambda: Vec<f32>, n: usize, t: f32, le: &mut Vec<f32>) {
    let c: f64 = 299792458.0;
    let h = 6.62606957e-34;
    let kb = 1.3806488e-23;

    for i in 0..n {
        let l = (lambda[i] * 1e-9) as f64;
        let lambda5: f64 = l * l * l * l * l;

        let denom = lambda5 * (h * c / (l * kb * t as f64) - 1.0).exp();

        le[i] = ((2.0 * h * c * c) / denom) as f32;
    }
}

#[derive(Debug, Clone)]
pub enum Light {
    Point(PointLight),
    DiffuseArea(DiffuseAreaLight)
}

impl Light {
    pub fn get_flags(&self) -> LightFlags {
        match self {
            Self::Point(d) => d.get_flags(),
            Self::DiffuseArea(d) => d.get_flags()
        }
    }

    pub fn get_n_samples(&self) -> u32 {
        match self {
            Self::Point(d) => d.get_n_samples(),
            Self::DiffuseArea(d) => d.get_n_samples()
        }
    }

    pub fn get_medium_interface(&self) -> MediumInterface {
        match self {
            Self::Point(d) => d.get_medium_interface(),
            Self::DiffuseArea(d) => d.get_medium_interface()
        }
    }

    pub fn get_light_to_world(&self) -> Transform {
        match self {
            Self::Point(d) => d.get_light_to_world(),
            Self::DiffuseArea(d) => d.get_light_to_world()
        }
    }

    pub fn get_world_to_light(&self) -> Transform {
        match self {
            Self::Point(d) => d.get_world_to_light(),
            Self::DiffuseArea(d) => d.get_world_to_light()
        }
    }

    pub fn sample_li(&self, re: &InteractionBase, u: &Point2, wi: &mut Vector3, pdf: &mut f32, vis: &mut VisibilityTester) -> Spectrum {
        match self {
            Self::Point(d) => d.sample_li(re, u, wi, pdf, vis),
            Self::DiffuseArea(d) => d.sample_li(re, u, wi, pdf, vis)
        }
    }

    pub fn power(&self) -> Spectrum {
        match self {
            Self::Point(d) => d.power(),
            Self::DiffuseArea(d) => d.power()
        }
    }

    pub fn preprocess(&self, scene: &Scene) {
        match self {
            Self::Point(d) => d.preprocess(scene),
            Self::DiffuseArea(d) => d.preprocess(scene),
        }
    }

    pub fn le(&self, r: &Ray) -> Spectrum {
        match self {
            Self::Point(d) => d.le(r),
            Self::DiffuseArea(d) => d.le(r)
        }
    }

    pub fn pdf_li(&self, re: &InteractionBase, wi: &Vector3) -> f32 {
        match self {
            Self::Point(d) => d.pdf_li(re, wi),
            Self::DiffuseArea(d) => d.pdf_li(re, wi)
        }
    }

    pub fn sample_le(&self, u1: &Point2, u2: &Point2, time: f32, ray: &mut Ray, n_light: &mut Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum {
        match self {
            Self::Point(d) => d.sample_le(u1, u2, time, ray, n_light, pdf_pos, pdf_dir),
            Self::DiffuseArea(d) => d.sample_le(u1, u2, time, ray, n_light, pdf_pos, pdf_dir)
        }
    }

    pub fn pdf_le(&self, ray: &Ray, n_light: &Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) {
        match self {
            Self::Point(d) => d.pdf_le(ray, n_light, pdf_pos, pdf_dir),
            Self::DiffuseArea(d) => d.pdf_le(ray, n_light, pdf_pos, pdf_dir),
        }
    }

    pub fn l(&self, its: &InteractionBase, w: &Vector3) -> Spectrum {
        match self {
            Self::DiffuseArea(d) => d.l(its, w),
            _ => panic!("Calling l on a non-area light")
        }
    }

    pub fn add_shape(&mut self, shape: Arc<Shape>) {
        match self {
            Self::DiffuseArea(d) => d.add_shape(shape),
            _ => panic!("Calling add shape to non-area light")
        }
    }
}

pub trait LightT: Manufacturable<Light> + Printable {
    fn get_flags(&self) -> LightFlags;
    fn get_n_samples(&self) -> u32;
    fn get_medium_interface(&self) -> MediumInterface;
    fn get_light_to_world(&self) -> Transform;
    fn get_world_to_light(&self) -> Transform;

    fn sample_li(&self, re: &InteractionBase, u: &Point2, wi: &mut Vector3, pdf: &mut f32, vis: &mut VisibilityTester) -> Spectrum;
    fn power(&self) -> Spectrum;
    fn preprocess(&self, _scene: &Scene) {}
    fn le(&self, r: &Ray) -> Spectrum;
    fn pdf_li(&self, re: &InteractionBase, wi: &Vector3) -> f32;
    fn sample_le(&self, u1: &Point2, u2: &Point2, time: f32, ray: &mut Ray, n_light: &mut Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum;
    fn pdf_le(&self, ray: &Ray, n_light: &Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32);
    fn l(&self, _its: &InteractionBase, _w: &Vector3) -> Spectrum {
        panic!("Should only be called on Area Lights");
    }

    fn is_delta_light(flags: u32) -> bool {
        (flags & LightFlags::DeltaDirection as u32) != 0 ||
        (flags & LightFlags::DeltaPosition as u32) != 0
    }
}