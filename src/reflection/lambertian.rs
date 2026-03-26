use crate::{core::{INV_PI, Printable, Vector3, bxdf::{BxDF, BxDFT, BxDFType}, spectrum::Spectrum}, registry::Manufacturable};

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