use std::sync::Arc;

use crate::{core::{Printable, Vector3, bxdf::BxDF, interaction::TransportMode, material::{Material, MaterialT}, texture::Texture}, interaction::surface_interaction::SurfaceInteraction, reflection::lambertian::LambertianReflection, registry::{LeadObject, Manufacturable}};


#[derive(Debug, PartialEq)]
pub struct MatteMaterial {
    kd: Arc<Texture>,
    sigma: Arc<Texture>,
    bump_map: Option<Arc<Texture>>,
}

impl MatteMaterial {
    pub fn init(
        kd: Arc<Texture>,
        sigma: Arc<Texture>,
        bump_map: Option<Arc<Texture>>
    ) -> Self {
        Self {
            kd,
            sigma,
            bump_map
        }
    }
}

impl MaterialT for MatteMaterial {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, _mode: TransportMode, _allow_multiple_lobes: bool) {
        if let Some(b) = &self.bump_map {
            Self::bump(b, si);
        }

        let r = self.kd.evaluate(si);
        let sig = self.sigma.evaluate(&si).x.clamp(0.0, 90.0);

        if r != Vector3::zeros() {
            // let mut bsdf = BSDF::init(&si, 1.0);
            let bsdf = si.bsdf.as_mut().unwrap();
            // println!("BSDF in matte: {:?}", bsdf);
            if sig == 0.0 {
                bsdf.add(
                    BxDF::Lambertian(
                        LambertianReflection::init(r))
                    );
            } else {
                todo!("to make OrenNayar")
                // bsdf.add(
                //     BxDF::OrenNayar(
                //         OrenNayarReflection::init(r, sig))
                //     );
            }
            // si.bsdf = Some(bsdf);
        } else {
            si.bsdf = None;
        }
    }
}

impl Manufacturable<Material> for MatteMaterial {
    fn create_from_parameters(param: crate::loader::Parameters) -> Material {
        let mut param = param;
        let kd = match param.get_lead_object("kd") {
            Some(LeadObject::Texture(t)) => t,
            _ => panic!("Matte Material needs a kd texture")
        };

        let sigma = match param.get_lead_object("sigma") {
            Some(LeadObject::Texture(t)) => t,
            _ => panic!("Matte Material needs a sigma texture")
        };

        let bump_map = match param.get_lead_object("bump") {
            Some(LeadObject::Texture(t)) => Some(Arc::new(t)),
            _ => None
        };

        let mt = Self::init(
            Arc::new(kd), 
            Arc::new(sigma), 
            bump_map
        );

        Material::Matte(mt)
    }
}

impl Printable for MatteMaterial {
    fn to_string(&self) -> String {
        "Matte Material: []".to_string()
    }
}