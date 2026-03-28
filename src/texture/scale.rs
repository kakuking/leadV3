use std::{fmt::Debug, sync::Arc};

use crate::{core::{Printable, spectrum::Spectrum, texture::{Texture, TextureT}}, interaction::surface_interaction::SurfaceInteraction, registry::{LeadObject, Manufacturable}};


#[derive(Debug, Clone)]
pub struct ScaleTexture {
    tex1: Arc<Texture>,
    tex2: Arc<Texture>
}

impl ScaleTexture {
    pub fn init(tex1: Arc<Texture>, tex2: Arc<Texture>) -> Self {
        Self {
            tex1,
            tex2
        }
    }
}

impl TextureT for ScaleTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        self.tex1.evaluate(si).component_mul(&self.tex2.evaluate(si))
    }
}

impl Printable for ScaleTexture {
    fn to_string(&self) -> String {
        format!(
            "Checkerboard Texture: [\n
            \ttex1: {}\n
            \ttex2: {}\n
            ]",
            self.tex1.to_string(),
            self.tex2.to_string()
        )
    }
}

impl Manufacturable<Texture> for ScaleTexture {
    fn create_from_parameters(param: crate::loader::Parameters) -> Texture {
        let mut param = param;

        let tex1 = match param.get_lead_object("tex1") {
            Some(LeadObject::Texture(t)) => t,
            _ => panic!("Checkerboard Texture requires tex1")
        };

        let tex2 = match param.get_lead_object("tex2") {
            Some(LeadObject::Texture(t)) => t,
            _ => panic!("Checkerboard Texture requires tex2")
        };

        Texture::Scale(
            ScaleTexture::init(
                Arc::new(tex1), 
                Arc::new(tex2)
            )
        )
    }
}