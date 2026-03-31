use std::sync::Arc;

use crate::{core::{Printable, Vector3, bxdf::BxDF, interaction::TransportMode, material::{Material, MaterialT}, spectrum::Spectrum, texture::Texture}, interaction::surface_interaction::SurfaceInteraction, reflection::{fresnel::FresnelSpecular}, registry::Manufacturable};


#[derive(Debug, PartialEq)]
pub struct GlassMaterial {
    r: Spectrum,
    t: Spectrum,
    eta_a: f32,
    eta_b: f32,

    mode: TransportMode,
    bump_map: Option<Arc<Texture>>,
}

impl GlassMaterial {
    pub fn new() -> Self {
        Self {
            r: Spectrum::zeros(),
            t: Spectrum::zeros(),
            eta_a: 0.0,
            eta_b: 0.0,
            mode: TransportMode::Radiance,
            bump_map: None
        }
    }

    pub fn init(r: Spectrum, t: Spectrum, eta_a: f32, eta_b: f32, mode: TransportMode, bump_map: Option<Arc<Texture>>) -> Self {
        Self {
            r,
            t,
            eta_a,
            eta_b,
            mode,
            bump_map
        }
    }
}

impl MaterialT for GlassMaterial {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, _mode: TransportMode, _allow_multiple_lobes: bool) {
        if let Some(b) = &self.bump_map {
            Self::bump(b, si);
        }

        let bsdf = si.bsdf.as_mut().unwrap();
        // println!("BSDF in mirror: {:?}", bsdf);
        bsdf.add(
            BxDF::SpecFresnel(
                FresnelSpecular::init(
                    self.r,
                    self.t,
                    self.eta_a,
                    self.eta_b,
                    self.mode.clone()
                )
            )
        );
    }
}

impl Manufacturable<Material> for GlassMaterial {
    fn create_from_parameters(param: crate::loader::Parameters) -> Material {
        let r = param.get_vector3("r", Some(Vector3::new(1.0, 1.0, 1.0)));
        let t = param.get_vector3("t", Some(Vector3::new(1.0, 1.0, 1.0)));

        let eta_a = param.get_float("eta_a", Some(1.0));
        let eta_b = param.get_float("eta_b", Some(1.5));
        let mode_str = param.get_string("transport_mode", Some("radiance".to_string()));

        let mode = if mode_str == "radiance" {
            TransportMode::Radiance
        } else {
            TransportMode::Importance
        };

        let mt = Self::init(r, t, eta_a, eta_b, mode, None);

        Material::Glass(mt)
    }
}

impl Printable for GlassMaterial {
    fn to_string(&self) -> String {
        "Glass Material: []".to_string()
    }
}