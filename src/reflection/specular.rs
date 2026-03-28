use std::sync::Arc;

use crate::{core::{Normal3, Printable, Vector3, abs_cos_theta, bxdf::{BxDF, BxDFT, BxDFType}, cos_theta, face_forward, interaction::TransportMode, spectrum::Spectrum}, reflection::fresnel::{Fresnel, FresnelDielectric, FresnelNoOp}, registry::{Manufacturable}};

pub fn refract(wi: &Vector3, n: &Normal3, eta: f32, wt: &mut Vector3) -> bool {
    let cos_theta_i = n.dot(wi);
    let sin2_theta_i = (1.0 - cos_theta_i*cos_theta_i).max(0.0);
    let sin2_theta_t = eta * eta * sin2_theta_i;

    if sin2_theta_t >= 1.0 {
        return false;
    }

    let cos_theta_t = (1.0 - sin2_theta_t).sqrt();

    *wt = eta * -wi + (eta * cos_theta_i - cos_theta_t) * n;
    true
}

#[derive(Debug, Clone)]
pub struct SpecularReflection {
    r: Spectrum,
    fresnel: Arc<Fresnel>,

    b_type: BxDFType,
}

impl SpecularReflection {
    pub fn new() -> Self {
        Self {
            r: Spectrum::zeros(),
            fresnel: Arc::new(Fresnel::NoOp(FresnelNoOp::new())),

            b_type: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR
        }
    }

    pub fn init(r: Spectrum, fresnel: Arc<Fresnel>) -> Self {
        Self {
            r,
            fresnel,
            b_type: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR
        }
    }
}

impl BxDFT for SpecularReflection {
    fn get_type(&self) -> BxDFType { self.b_type }
    fn set_type(&mut self, typ: BxDFType) { self.b_type = typ; }

    fn f(&self, _wo: &crate::core::Vector3, _wi: &crate::core::Vector3) -> Spectrum {
        Spectrum::zeros()
    }

    fn sample_f(&self, wo: &crate::core::Vector3, wi: &mut crate::core::Vector3, _sample: &crate::core::Point2, pdf: &mut f32, _sampled_type: Option<crate::core::bxdf::BxDFType>) -> Spectrum {
        *wi = Vector3::new(-wo.x, -wo.y, wo.z);
        *pdf = 1.0;

        self.fresnel.evaluate(cos_theta(wi)).component_mul(&self.r) / abs_cos_theta(&wi)
    }

    fn pdf(&self, _wi: &crate::core::Vector3, _wo: &crate::core::Vector3) -> f32 {
        0.0
    }
}

impl Manufacturable<BxDF> for SpecularReflection {
    fn create_from_parameters(param: crate::loader::Parameters) -> BxDF {
        let r = param.get_vector3("r", Some(Vector3::zeros()));
        let fresnel = Arc::new(
            Fresnel::NoOp(
                FresnelNoOp::new()
            )
        );

        BxDF::SpecRefl(
            Self::init(r, fresnel)
        )
    }
}

impl Printable for SpecularReflection {
    fn to_string(&self) -> String {
        format!(
            "Specular Reflection: [\n
            \tr: {}, {}, {}\n
            \tfresnel: {},\n
            \ttype: {:?}\n
            ]",
            self.r.x, self.r.y, self.r.z,
            self.fresnel.to_string(),
            self.b_type
        )
    }
}

#[derive(Debug, Clone)]
pub struct SpecularTransmission {
    t: Spectrum,
    eta_a: f32,
    eta_b: f32,
    fresnel: Arc<Fresnel>,
    mode: TransportMode,

    b_type: BxDFType,
}

impl SpecularTransmission {
    pub fn new() -> Self {
        Self {
            t: Spectrum::zeros(),
            eta_a: 0.0,
            eta_b: 0.0,
            fresnel: Arc::new(Fresnel::Dielectric(FresnelDielectric::new())),
            mode: TransportMode::Radiance,

            b_type: BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR
        }
    }

    pub fn init(t: Spectrum, eta_a: f32, eta_b: f32, fresnel: Arc<Fresnel>, mode: TransportMode) -> Self {
        Self {
            t,
            eta_a,
            eta_b,
            fresnel,
            mode,
            b_type: BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR
        }
    }
}

impl BxDFT for SpecularTransmission {
    fn get_type(&self) -> BxDFType { self.b_type }
    fn set_type(&mut self, typ: BxDFType) { self.b_type = typ; }

    fn f(&self, _wo: &crate::core::Vector3, _wi: &crate::core::Vector3) -> Spectrum {
        Spectrum::zeros()
    }

    fn sample_f(&self, wo: &crate::core::Vector3, wi: &mut crate::core::Vector3, _sample: &crate::core::Point2, pdf: &mut f32, _sampled_type: Option<crate::core::bxdf::BxDFType>) -> Spectrum {
        let entering = cos_theta(wo) > 0.0;
        let (eta_i, eta_t) = if entering {
            (self.eta_a, self.eta_b)
        } else {
            (self.eta_b, self.eta_a)
        };

        if !refract(&wo, &face_forward(&Normal3::z(), wo), eta_i / eta_t, wi) {
            return Spectrum::zeros();
        }

        *pdf = 1.0;
        let mut ft = self.t.component_mul(&(Spectrum::new(1.0, 1.0, 1.0) - self.fresnel.evaluate(cos_theta(&wi))));

        if self.mode == TransportMode::Radiance {
            ft *= eta_i * eta_i / (eta_t * eta_t);
        }

        ft / abs_cos_theta(&wi)
    }

    fn pdf(&self, _wi: &crate::core::Vector3, _wo: &crate::core::Vector3) -> f32 {
        0.0
    }
}

impl Manufacturable<BxDF> for SpecularTransmission {
    fn create_from_parameters(param: crate::loader::Parameters) -> BxDF {
        let t = param.get_vector3("t", Some(Vector3::zeros()));

        let eta_a = param.get_float("eta_a", Some(0.0));
        let eta_b = param.get_float("eta_b", Some(0.0));
        let mode_str = param.get_string("transport_mode", Some("radiance".to_string()));

        let fresnel = Fresnel::Dielectric(
            FresnelDielectric::init(eta_a, eta_b)
        );

        let mode = if mode_str == "radiance" {
            TransportMode::Radiance
        } else {
            TransportMode::Importance
        };

        BxDF::SpecTrans(
            Self::init(
                t,
                eta_a,
                eta_b,
                Arc::new(fresnel),
                mode
            )
        )
    }
}

impl Printable for SpecularTransmission {
    fn to_string(&self) -> String {
        format!(
            "Specular Transmisson: [\n
            \tr: {}, {}, {}\n
            \teta_a: {}\n
            \teta_b: {}\n
            \tfresnel: {},\n
            \ttype: {:?}\n
            ]",
            self.t.x, self.t.y, self.t.z,
            self.eta_a, self.eta_b,
            self.fresnel.to_string(),
            self.b_type
        )
    }
}