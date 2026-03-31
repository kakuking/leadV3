use std::fmt::Debug;

use crate::{core::{Printable, Vector3, spectrum::Spectrum, texture::{Texture, TextureT}}, interaction::surface_interaction::SurfaceInteraction, loader::Parameters, registry::Manufacturable};


#[derive(Debug, Clone, PartialEq)]
pub struct ConstantTexture {
    value: Spectrum
}

impl ConstantTexture {
    pub fn new(value: Spectrum) -> Self {
        Self {
            value
        }
    }
}

impl TextureT for ConstantTexture {
    fn evaluate(&self, _si: &SurfaceInteraction) -> Spectrum {
        self.value.clone()
    }
}

impl Manufacturable<Texture> for ConstantTexture {
    fn create_from_parameters(param: Parameters) -> Texture {
        let value = param.get_vector3("value", Some(Vector3::zeros()));

        let tex = Self {
            value
        };

        Texture::Constant(tex)
    }
}

impl Printable for ConstantTexture {
    fn to_string(&self) -> String {
        format!(
            "Constant Texture: [\n
            \tT: {}, {}, {}n
            ]",
            self.value.x, self.value.y, self.value.z
        )
    }
}