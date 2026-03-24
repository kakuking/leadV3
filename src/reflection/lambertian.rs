use crate::{core::{INV_PI, Point2, Printable, Vector3, bxdf::{BxDF, BxDFT, BxDFType}, spectrum::Spectrum}, loader::Manufacturable};

#[derive(Debug, Clone)]
pub struct LambertianReflection {
    r: Spectrum,
    b_type: BxDFType
}

impl LambertianReflection {
    pub fn init(r: Spectrum) -> Self {
        Self {
            r,
            b_type: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
        }
    }
}

impl BxDFT for LambertianReflection {
    fn get_type(&self) -> BxDFType { self.b_type }
    fn set_type(&mut self, typ: BxDFType) { self.b_type = typ; }

    fn f(&self, _wo: &Vector3, _wi: &Vector3) -> Spectrum {
        self.r * INV_PI
    }

    fn rho(&self, _wo: &Vector3, _n_samples: usize, _samples: &mut Vec<Point2>) -> Spectrum {
        self.r
    }

    fn rho_2(&self, _n_samples: usize, _samples1: &mut Vec<Point2>, _samples2: &mut Vec<Point2>) -> Spectrum {
        self.r
    }
    
    fn sample_f(&self, _wo: &Vector3, _wi: &mut Vector3, _sample: &Point2, _pdf: &mut f32, _sampled_type: &mut BxDFType) -> Spectrum {
        panic!("Not implemented: Lambertian::Sample_f");
    }

    fn pdf(&self, _wi: &Vector3, _wo: &Vector3) -> f32 {
        panic!("Not implemented: Lambertian::pdf");
    }
}

impl Manufacturable<BxDF> for LambertianReflection {
    fn create_from_parameters(param: crate::loader::Parameters) -> BxDF {
        let r: Spectrum = param.get_vector3("albedo", Some(Vector3::x()));

        BxDF::Lambertian(
            LambertianReflection::init(r)
        )
    }
}

impl Printable for LambertianReflection {
    fn to_string(&self) -> String {
        format!(
            "Lambertian: [\n
            \tR: ({}, {}, {})\n
            \ttype: {:?}\n
            ]",
            self.r.x, self.r.y, self.r.z,
            self.b_type
        )
    }
}