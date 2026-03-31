use std::sync::Arc;

use crate::{core::{Point2, Printable, Ray, Transform, Vector3, interaction::MediumInteraction, sampler::Sampler, spectrum::Spectrum}, medium::{heterogeneous::HeterogeneousMedium, hg_phase::HenyeyGreenstein, homogeneous::HomogeneousMedium}, registry::Manufacturable};

#[derive(Debug, PartialEq, Clone)]
pub enum Medium {
    Homogeneous(HomogeneousMedium),
    Heterogeneous(HeterogeneousMedium)
}

impl Medium {
    pub fn tr(&self, ray: &Ray, sampler: &mut Sampler) -> Spectrum {
        match self {
            Self::Homogeneous(m) => m.tr(ray, sampler),
            Self::Heterogeneous(m) => m.tr(ray, sampler),
        }
    }

    // if returns true, then add medium to teh medium interaction
    pub fn sample(&self, ray: &Ray, sampler: &mut Sampler, medium_interaction: &mut MediumInteraction, medium: Arc<Medium>) -> Spectrum {
        match self {
            Self::Homogeneous(m) => m.sample(ray, sampler, medium_interaction, medium),
            Self::Heterogeneous(m) => m.sample(ray, sampler, medium_interaction, medium),
        }
    }

    pub fn set_world_to_medium(&mut self, world_to_medium: Transform) {
        match self {
            Self::Heterogeneous(m) => m.set_world_to_medium(world_to_medium),
            _ => {}
        }
    }
}

pub trait MediumT: Manufacturable<Medium> + Printable {
    fn tr(&self, ray: &Ray, sampler: &mut Sampler) -> Spectrum;
    fn sample(&self, ray: &Ray, sampler: &mut Sampler, mi: &mut MediumInteraction, medium: Arc<Medium>) -> Spectrum;
}

#[derive(Debug, Clone)]
pub enum PhaseFunction {
    HG(HenyeyGreenstein)
}

impl PhaseFunction {
    pub fn p(&self, wo: &Vector3, wi: &Vector3) -> f32 {
        match self {
            Self::HG(p) => p.p(wo, wi)
        }
    }

    pub fn sample_p(&self, wo: &Vector3, wi: &mut Vector3, u: &Point2) -> f32 {
        match self {
            Self::HG(p) => p.sample_p(wo, wi, u)
        }
    }
}

pub trait PhaseFunctionT: Manufacturable<PhaseFunction> + Printable {
    fn p(&self, wo: &Vector3, wi: &Vector3) -> f32;
    fn sample_p(&self, wo: &Vector3, wi: &mut Vector3, u: &Point2) -> f32;
}

#[derive(Debug, Clone)]
pub struct MediumInterface {
    pub inside: Option<Arc<Medium>>,
    pub outside: Option<Arc<Medium>>
}

impl MediumInterface {
    pub fn new() -> Self {
        Self {
            inside: None,
            outside: None
        }
    }

    pub fn init(inside: Option<Arc<Medium>>, outside: Option<Arc<Medium>>) -> Self {
        Self {
            inside,
            outside
        }
    }

    pub fn init_one(medium: Option<Arc<Medium>>) -> Self {
        Self {
            inside: medium.clone(),
            outside: medium
        }
    }

    pub fn is_medium_transition(&self) -> bool {
        self.inside != self.outside
    }
}