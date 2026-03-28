use bitflags::bitflags;

use crate::{core::{INV_PI, Point2, Printable, Vector3, abs_cos_theta, random::{cosine_sample_hemisphere, same_hemisphere, uniform_sample_hemisphere, uniform_sample_hemisphere_pdf}, spectrum::Spectrum}, reflection::{lambertian::LambertianReflection, specular::{SpecularReflection, SpecularTransmission}}, registry::Manufacturable};

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
    SpecRefl(SpecularReflection),
    SpecTrans(SpecularTransmission),
    // FresnelSpecular(FresnelSpecular),
}

impl BxDF {
    pub fn get_type(&self) -> BxDFType {
        match self {
            Self::Lambertian(b) => b.get_type(),
            Self::SpecRefl(b) => b.get_type(),
            Self::SpecTrans(b) => b.get_type(),
            // Self::FresnelSpecular(b) => b.get_type() 
        }
    }

    pub fn set_type(&mut self, typ: BxDFType) {
        match self {
            Self::Lambertian(b) => b.set_type(typ),
            Self::SpecRefl(b) => b.set_type(typ),
            Self::SpecTrans(b) => b.set_type(typ),
            // Self::FresnelSpecular(b) => b.set_type(typ) 
        }
    }

    pub fn f(&self, wo: &Vector3, wi: &Vector3) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.f(wo, wi),
            Self::SpecRefl(b) => b.f(wo, wi),
            Self::SpecTrans(b) => b.f(wo, wi),
            // Self::FresnelSpecular(b) => b.f(wo, wi) 
        }
    }

    pub fn sample_f(&self, wo: &Vector3, wi: &mut Vector3, sample: &Point2, pdf: &mut f32, sampled_type: Option<BxDFType>) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.sample_f(wo, wi, sample, pdf, sampled_type),
            Self::SpecRefl(b) => b.sample_f(wo, wi, sample, pdf, sampled_type),
            Self::SpecTrans(b) => b.sample_f(wo, wi, sample, pdf, sampled_type),
            // Self::FresnelSpecular(b) => b.sample_f(wo, wi, sample, pdf, sampled_type)
        }
    }

    pub fn rho(&self, wo: &Vector3, n_samples: usize, samples: &mut Vec<Point2>) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.rho(wo, n_samples, samples),
            Self::SpecRefl(b) => b.rho(wo, n_samples, samples),
            Self::SpecTrans(b) => b.rho(wo, n_samples, samples),
            // Self::FresnelSpecular(b) => b.rho(wo, n_samples, samples)
        }
    }

    pub fn rho_2(&self, n_samples: usize, samples1: &mut Vec<Point2>, samples2: &mut Vec<Point2>) -> Spectrum {
        match self {
            Self::Lambertian(b) => b.rho_2(n_samples, samples1, samples2),
            Self::SpecRefl(b) => b.rho_2(n_samples, samples1, samples2),
            Self::SpecTrans(b) => b.rho_2(n_samples, samples1, samples2),
            // Self::FresnelSpecular(b) => b.rho_2(n_samples, samples1, samples2)
        }
    }

    pub fn pdf(&self, wo: &Vector3, wi: &Vector3) -> f32 {
        match self {
            Self::Lambertian(b) => b.pdf(wo, wi),
            Self::SpecRefl(b) => b.pdf(wo, wi),
            Self::SpecTrans(b) => b.pdf(wo, wi),
            // Self::FresnelSpecular(b) => b.pdf(wi, wo)
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

    fn sample_f(&self, wo: &Vector3, wi: &mut Vector3, sample: &Point2, pdf: &mut f32, _sampled_type: Option<BxDFType>) -> Spectrum {

        *wi = cosine_sample_hemisphere(*sample);
        if wo.z < 0.0 {
            wi.z *= -1.0;
        }

        *pdf = self.pdf(wo, &wi);
        self.f(wo, &wi)
    }

    fn rho(&self, wo: &Vector3, n_samples: usize, samples: &mut Vec<Point2>) -> Spectrum {
        let mut r = Spectrum::zeros();

        for i in 0..n_samples {
            let mut wi = Vector3::zeros();
            let mut pdf = 0.0;

            let f = self.sample_f(wo, &mut wi, &samples[i], &mut pdf, None);

            if pdf > 0.0 {
                r += f * abs_cos_theta(&wi) / pdf;
            }
        }

        r / n_samples as f32
    }

    fn rho_2(&self, n_samples: usize, samples1: &mut Vec<Point2>, samples2: &mut Vec<Point2>) -> Spectrum {
        let mut r = Spectrum::zeros();

        for i in 0..n_samples {
            let wo;
            let mut wi = Vector3::zeros();

            wo = uniform_sample_hemisphere(&samples1[i]);
            let pdfo = uniform_sample_hemisphere_pdf();
            let mut pdfi = 0.0;
            let f = self.sample_f(&wo, &mut wi, &samples2[i], &mut pdfi, None);

            if pdfi > 0.0 {
                r += f * abs_cos_theta(&wi) * abs_cos_theta(&wo) / (pdfo * pdfi);
            }
        }

        r
    }

    fn pdf(&self, wo: &Vector3, wi: &Vector3) -> f32 {
        if same_hemisphere(wo, wi) {
            abs_cos_theta(wi) * INV_PI
        } else {
            0.0
        }
    }
}