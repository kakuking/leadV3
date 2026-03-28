use crate::{core::{Printable, Vector3, spectrum::Spectrum}, registry::Manufacturable};

pub fn fr_dielectric(cos_theta_i: f32, eta_i: f32, eta_t: f32) -> f32 {
    let mut cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);
    let mut eta_i = eta_i;
    let mut eta_t = eta_t;

    let entering = cos_theta_i > 0.0;

    if !entering {
        std::mem::swap(&mut eta_i,&mut eta_t);
        cos_theta_i = cos_theta_i.abs();
    }

    let sin_theta_i = (1.0 - cos_theta_i*cos_theta_i).max(0.0).sqrt();
    let sin_theta_t = eta_i / eta_t * sin_theta_i;

    if sin_theta_t >= 1.0 {
        return 1.0;
    }

    let cos_theta_t = (1.0 - sin_theta_t*sin_theta_t).max(0.0).sqrt();

    let r_parl = ((eta_t * cos_theta_i) - (eta_i * cos_theta_t)) / ((eta_t * cos_theta_i) + (eta_i * cos_theta_t));
    let r_perp = ((eta_i * cos_theta_i) - (eta_t * cos_theta_t)) / ((eta_i * cos_theta_i) + (eta_t * cos_theta_t));

    (r_parl * r_parl + r_perp * r_perp) / 2.0
}

fn fr_conductor(cos_theta_i: f32, eta_i: &Spectrum, eta_t: &Spectrum, k: &Spectrum) -> Spectrum {
    let cos_theta_i = cos_theta_i.clamp(-1.0, 1.0);
    let eta = eta_t.component_div(eta_i);
    let eta_k = k.component_div(eta_i);

    let cos_theta_i2 = cos_theta_i * cos_theta_i;
    let sin_theta_i2 = 1.0 - cos_theta_i2;
    let eta2 = eta.component_mul(&eta);
    let etak2 = eta_k.component_mul(&eta_k);

    let t0 = eta2 - etak2 - Spectrum::new(sin_theta_i2, sin_theta_i2, sin_theta_i2);
    let a2plusb2: Vector3 = (t0.component_mul(&t0) + 4.0 * eta2.component_mul(&etak2)).map(|x| x.sqrt());
    let t1 = a2plusb2 + Spectrum::new(cos_theta_i2, cos_theta_i2, cos_theta_i2);
    let a: Vector3 = (Spectrum::new(0.5, 0.5, 0.5) + a2plusb2 + t0).map(|x| x.sqrt());
    let t2 = 2.0 * cos_theta_i * a;
    let rs = (t1 - t2).component_div(&(t1 + t2));

    let t3 = cos_theta_i2 * a2plusb2 + sin_theta_i2*sin_theta_i2*Vector3::new(1.0, 1.0, 1.0);
    let t4 = t2 * sin_theta_i2;
    let rp = (t3 - t4).component_div(&(t3 + t4)).component_mul(&rs);

    0.5 * (rp + rs)
}

#[derive(Debug, Clone)]
pub enum Fresnel {
    Conductor(FresnelConductor),
    Dielectric(FresnelDielectric),
    NoOp(FresnelNoOp)
}

impl Fresnel {
    pub fn evaluate(&self, cos_i: f32) -> Spectrum {
        match self {
            Self::Conductor(f) => f.evaluate(cos_i),
            Self::Dielectric(f) => f.evaluate(cos_i),
            Self::NoOp(f) => f.evaluate(cos_i),
        }
    }
}

impl Printable for Fresnel {
    fn to_string(&self) -> String {
        match self {
            Self::Conductor(f) => f.to_string(),
            Self::Dielectric(f) => f.to_string(),
            Self::NoOp(_) => "No Op".to_string(),
        }
    }
}

pub trait FresnelT: Manufacturable<Fresnel> + Printable {
    fn evaluate(&self, cos_i: f32) -> Spectrum;
}

#[derive(Debug, Clone)]
pub struct FresnelConductor {
    eta_i: Spectrum,
    eta_t: Spectrum,
    k: Spectrum,
}

impl FresnelConductor {
    pub fn new() -> Self {
        Self {
            eta_i: Spectrum::zeros(),
            eta_t: Spectrum::zeros(),
            k: Spectrum::zeros(),
        }
    }

    pub fn init(eta_i: Spectrum, eta_t: Spectrum, k: Spectrum) -> Self {
        Self {
            eta_i,
            eta_t,
            k
        }
    }
}

impl FresnelT for FresnelConductor {
    fn evaluate(&self, cos_theta_i: f32) -> Spectrum {
        fr_conductor(cos_theta_i.abs(), &self.eta_i, &self.eta_t, &self.k)
    }
}

impl Manufacturable<Fresnel> for FresnelConductor {
    fn create_from_parameters(param: crate::loader::Parameters) -> Fresnel {
        let eta_i = param.get_vector3("eta_i", Some(Spectrum::zeros()));
        let eta_t = param.get_vector3("eta_t", Some(Spectrum::zeros()));
        let k = param.get_vector3("k", Some(Spectrum::zeros()));

        Fresnel::Conductor(
            Self::init(eta_i, eta_t, k)
        )
    }
}

impl Printable for FresnelConductor {
    fn to_string(&self) -> String {
        format!(
            "Fresnel Conductor: [\n
            \teta_i: {}, {}, {}\n
            \teta_t: {}, {}, {}\n
            \tk: {}, {}, {}\n
            ]",
            self.eta_i.x, self.eta_i.y, self.eta_i.z,
            self.eta_t.x, self.eta_t.y, self.eta_t.z,
            self.k.x, self.k.y, self.k.z,
        )
    }
}

#[derive(Debug, Clone)]
pub struct FresnelDielectric {
    eta_i: f32,
    eta_t: f32,
}

impl FresnelDielectric {
    pub fn new() -> Self {
        Self {
            eta_i: 0.0,
            eta_t: 0.0
        }
    }

    pub fn init(eta_i: f32, eta_t: f32) -> Self {
        Self {
            eta_i,
            eta_t,
        }
    }
}

impl FresnelT for FresnelDielectric {
    fn evaluate(&self, cos_theta_i: f32) -> Spectrum {
        let fr = fr_dielectric(cos_theta_i, self.eta_i, self.eta_t);

        Spectrum::new(fr, fr, fr)
    }
}

impl Manufacturable<Fresnel> for FresnelDielectric {
    fn create_from_parameters(param: crate::loader::Parameters) -> Fresnel {
        let eta_i = param.get_float("eta_i", Some(0.0));
        let eta_t = param.get_float("eta_t", Some(0.0));

        Fresnel::Dielectric(
            Self::init(eta_i, eta_t)
        )
    }
}

impl Printable for FresnelDielectric {
    fn to_string(&self) -> String {
        format!(
            "Fresnel Conductor: [\n
            \teta_i: {}\n
            \teta_t: {}\n
            ]",
            self.eta_i,
            self.eta_t,
        )
    }
}

#[derive(Debug, Clone)]
pub struct FresnelNoOp {

}

impl FresnelNoOp {
    pub fn new() -> Self {
        Self {}
    }
}

impl FresnelT for FresnelNoOp {
    fn evaluate(&self, _cos_i: f32) -> Spectrum {
        Spectrum::new(1.0, 1.0, 1.0)
    }
}

impl Manufacturable<Fresnel> for FresnelNoOp {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Fresnel {
        Fresnel::NoOp(
            Self::new()
        )
    }
}

impl Printable for FresnelNoOp {
    fn to_string(&self) -> String {
        "Fresnel NoOp".to_string()
    }
}

// #[derive(Debug, Clone)]
// pub struct FresnelSpecular {
//     r: Spectrum,
//     t: Spectrum,
//     eta_a: f32,
//     eta_b: f32,
//     fresnel: Arc<Fresnel>,
//     mode: TransportMode,

//     b_type: BxDFType
// }

// impl FresnelSpecular {
//     pub fn new() -> Self {
//         Self {
//             r: Spectrum::zeros(),
//             t: Spectrum::zeros(),
//             eta_a: 0.0,
//             eta_b: 0.0,
//             fresnel: Arc::new(Fresnel::Dielectric(FresnelDielectric::new())),
//             mode: TransportMode::Radiance,

//             b_type: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR
//         }
//     }

//     pub fn init(r: Spectrum, t: Spectrum, eta_a: f32, eta_b: f32, mode: TransportMode) -> Self {
//         let fresnel = Fresnel::Dielectric(
//             FresnelDielectric::init(eta_a, eta_b)
//         );

//         Self {
//             r,
//             t,
//             eta_a,
//             eta_b,
//             fresnel: Arc::new(fresnel),
//             mode,
//             b_type: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR
//         }
//     }
// }

// impl BxDFT for FresnelSpecular {
//     fn get_type(&self) -> BxDFType { self.b_type }
//     fn set_type(&mut self, typ: BxDFType) { self.b_type = typ; }

//     fn f(&self, _wo: &Vector3, _wi: &Vector3) -> Spectrum {
//         Spectrum::zeros()
//     }

//     fn pdf(&self, _wi: &Vector3, _wo: &Vector3) -> f32 {
//         0.0
//     }

//     fn sample_f(&self, wo: &Vector3, wi: &mut Vector3, sample: &crate::core::Point2, pdf: &mut f32, _sampled_type: Option<BxDFType>) -> Spectrum {
        
//     }
// }

// impl Manufacturable<BxDF> for FresnelSpecular {
//     fn create_from_parameters(param: crate::loader::Parameters) -> BxDF {
//         // r: Spectrum, t: Spectrum, eta_a: f32, eta_b: f32, mode: TransportMode

//         let r = param.get_vector3("r", Some(Spectrum::zeros()));
//         let t = param.get_vector3("t", Some(Spectrum::zeros()));
//         let eta_a = param.get_float("eta_a", Some(0.0));
//         let eta_b = param.get_float("eta_b", Some(0.0));
//         let mode_str = param.get_string("mode", Some("radiance".to_string()));

//         let mode = if mode_str == "radiance" {
//             TransportMode::Radiance
//         } else {
//             TransportMode::Importance
//         };

//         BxDF::FresnelSpecular(
//             Self::init(
//                 r, 
//                 t, 
//                 eta_a, 
//                 eta_b, 
//                 mode
//             )
//         )
//     }
// }

// impl Printable for FresnelSpecular {
//     fn to_string(&self) -> String {
//         format!(
//             "Fresnel Specular: [\n
//             \tr: {}, {}, {}\n
//             \tt: {}, {}, {}\n
//             \teta_a: {}\n
//             \teta_b: {}\n
//             \tmode: {:?}\n
//             ]",
//             self.r.x, self.r.y, self.r.z,
//             self.t.x, self.t.y, self.t.z,
//             self.eta_a, self.eta_b,
//             self.mode
//         )
//     }
// }