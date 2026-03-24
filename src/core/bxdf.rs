use bitflags::bitflags;

use crate::{core::{Point2, Printable, Vector3, spectrum::Spectrum}, loader::Manufacturable, reflection::lambertian::LambertianReflection};

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct BxDFType: u32 {
        const BSDF_REFLECTION   = 1 << 0;
        const BSDF_TRANSMISSION = 1 << 1;
        const BSDF_DIFFUSE      = 1 << 2;
        const BSDF_GLOSSY       = 1 << 3;
        const BSDF_SPECULAR     = 1 << 4;

        const BSDF_ALL = Self::BSDF_REFLECTION.bits()
                       | Self::BSDF_TRANSMISSION.bits()
                       | Self::BSDF_DIFFUSE.bits()
                       | Self::BSDF_GLOSSY.bits()
                       | Self::BSDF_SPECULAR.bits();
    }
}

#[derive(Debug, Clone)]
pub enum BxDF {
    Lambertian(LambertianReflection),
}

impl BxDF {
    pub fn get_type(&self) -> BxDFType {
        match self {
            Self::Lambertian(b) => b.get_type() 
        }
    }

    pub fn set_type(&mut self, typ: BxDFType) {
        match self {
            Self::Lambertian(b) => b.set_type(typ) 
        }
    }

    pub fn f(&self, wo: &Vector3, wi: &Vector3) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.f(wo, wi) 
        }
    }

    pub fn sample_f(&self, wo: &Vector3, wi: &mut Vector3, sample: &Point2, pdf: &mut f32, sampled_type: &mut BxDFType) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.sample_f(wo, wi, sample, pdf, sampled_type)
        }
    }

    pub fn rho(&self, wo: &Vector3, n_samples: usize, samples: &mut Vec<Point2>) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.rho(wo, n_samples, samples)
        }
    }

    pub fn rho_2(&self, n_samples: usize, samples1: &mut Vec<Point2>, samples2: &mut Vec<Point2>) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.rho_2(n_samples, samples1, samples2)
        }
    }

    pub fn pdf(&self, wi: &Vector3, wo: &Vector3) -> f32 {
        match self {
            Self::Lambertian(b) => b.pdf(wi, wo)
        }
    }

    pub fn matches_flags(&self, t: BxDFType) -> bool {
        self.get_type() & t == self.get_type()
    }
}

pub trait BxDFT: Manufacturable<BxDF> + Printable {
    fn get_type(&self) -> BxDFType;
    fn set_type(&mut self, typ: BxDFType);

    fn matches_flags(&self, t: BxDFType) -> bool {
        self.get_type() & t == self.get_type()
    }

    fn f(&self, wo: &Vector3, wi: &Vector3) -> Spectrum;
    fn sample_f(&self, wo: &Vector3, wi: &mut Vector3, sample: &Point2, pdf: &mut f32, sampled_type: &mut BxDFType) -> Spectrum;
    fn rho(&self, wo: &Vector3, n_samples: usize, samples: &mut Vec<Point2>) -> Spectrum;
    fn rho_2(&self, n_samples: usize, samples1: &mut Vec<Point2>, samples2: &mut Vec<Point2>) -> Spectrum;
    fn pdf(&self, wi: &Vector3, wo: &Vector3) -> f32;
}