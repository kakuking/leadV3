use std::sync::Arc;

use crate::{core::{Printable, bxdf::BxDF, interaction::TransportMode, material::{Material, MaterialT}, spectrum::Spectrum, texture::{Texture}}, interaction::surface_interaction::SurfaceInteraction, reflection::{fresnel::{Fresnel, FresnelNoOp}, specular::SpecularReflection}, registry::Manufacturable};


#[derive(Debug, PartialEq)]
pub struct MirrorMaterial {
    bump_map: Option<Arc<Texture>>,
}

impl MirrorMaterial {
    pub fn init(
        bump_map: Option<Arc<Texture>>
    ) -> Self {
        Self {
            bump_map
        }
    }
}

impl MirrorMaterial {
    pub fn new() -> Self {
        Self {
            bump_map: None
        }
    }
}

impl MaterialT for MirrorMaterial {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, _mode: TransportMode, _allow_multiple_lobes: bool) {
        if let Some(b) = &self.bump_map {
            Self::bump(b, si);
        }

        let bsdf = si.bsdf.as_mut().unwrap();
        // println!("BSDF in mirror: {:?}", bsdf);
        bsdf.add(
            BxDF::SpecRefl(
                SpecularReflection::init(
                    Spectrum::new(1.0, 1.0, 1.0),
                    Arc::new(
                        Fresnel::NoOp(
                            FresnelNoOp::new()
                        )
                    )
                )
            )
        );
    }
}

impl Manufacturable<Material> for MirrorMaterial {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Material {
        let mt = Self {
            bump_map: None
        };

        Material::Mirror(mt)
    }
}

impl Printable for MirrorMaterial {
    fn to_string(&self) -> String {
        "Mirror Material: []".to_string()
    }
}