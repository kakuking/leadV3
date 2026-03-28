use std::sync::Arc;

use crate::{core::{Printable, Vector3, bxdf::BxDF, interaction::TransportMode, material::{Material, MaterialT}, spectrum::Spectrum, texture::{ConstantTexture, Texture}}, interaction::surface_interaction::SurfaceInteraction, reflection::lambertian::LambertianReflection, registry::Manufacturable};


#[derive(Debug)]
pub struct MatteMaterial {
    kd: Arc<dyn Texture<Spectrum>>,
    sigma: Arc<dyn Texture<f32>>,
    bump_map: Option<Arc<dyn Texture<f32>>>,
}

impl MatteMaterial {
    pub fn init(
        kd: Arc<dyn Texture<Spectrum>>,
        sigma: Arc<dyn Texture<f32>>,
        bump_map: Option<Arc<dyn Texture<f32>>>
    ) -> Self {
        Self {
            kd,
            sigma,
            bump_map
        }
    }
}

impl MatteMaterial {
    pub fn new() -> Self {
        let kd = Arc::new(
            ConstantTexture::new(Spectrum::new(1.0, 1.0, 1.0)),
        );

        let sigma = Arc::new(
            ConstantTexture::new(0.0),
        );

        Self {
            kd, 
            sigma,
            bump_map: None
        }
    }
}

impl MaterialT for MatteMaterial {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, _mode: TransportMode, _allow_multiple_lobes: bool) {
        if let Some(b) = &self.bump_map {
            Self::bump(b, si);
        }

        let r = self.kd.evaluate(si);
        let sig = self.sigma.evaluate(&si).clamp(0.0, 90.0);

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
        let kd = param.get_vector3("color", Some(Spectrum::y()));
        let sigma = param.get_float("sigma", Some(0.0));
        // let bump = param.get_float("bump", Some(0.0));

        let kd_t = ConstantTexture::new(kd);
        let sigma_t = ConstantTexture::new(sigma);

        let mt = Self {
            kd: Arc::new(kd_t),
            sigma: Arc::new(sigma_t),
            bump_map: None
        };

        Material::Matte(mt)
    }
}

impl Printable for MatteMaterial {
    fn to_string(&self) -> String {
        "Matte Material: []".to_string()
    }
}