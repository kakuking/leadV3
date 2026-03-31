use std::{fmt::Debug, sync::Arc};

use crate::{core::{Printable, Vector2, spectrum::Spectrum, texture::{Texture, TextureMapping2D, TextureT}}, interaction::surface_interaction::SurfaceInteraction, registry::{LeadObject, Manufacturable}};


#[derive(Debug, Clone, PartialEq)]
pub struct CheckerboardTexture {
    mapping: Arc<TextureMapping2D>,

    tex1: Arc<Texture>,
    tex2: Arc<Texture>
}

impl CheckerboardTexture {
    pub fn init(mapping: Arc<TextureMapping2D>, tex1: Arc<Texture>, tex2: Arc<Texture>) -> Self {
        Self {
            mapping,
            tex1,
            tex2
        }
    }
}

impl TextureT for CheckerboardTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let mut dsdtx = Vector2::zeros();
        let mut dsdty = Vector2::zeros();

        let st = self.mapping.map(si, &mut dsdtx, &mut dsdty);

        if (st[0].floor() + st[1].floor()) as u32 % 2 == 0 {
            self.tex1.evaluate(si)
        } else {
            self.tex2.evaluate(si)
        }
    }
}

impl Printable for CheckerboardTexture {
    fn to_string(&self) -> String {
        format!(
            "Checkerboard Texture: [\n
            \tmapping: {},\n
            \ttex1: {}\n
            \ttex2: {}\n
            ]",
            self.mapping.to_string(),
            self.tex1.to_string(),
            self.tex2.to_string()
        )
    }
}

impl Manufacturable<Texture> for CheckerboardTexture {
    fn create_from_parameters(param: crate::loader::Parameters) -> Texture {
        let mut param = param;

        let mapping = match param.get_lead_object("mapping") {
            Some(LeadObject::TextureMapping(m)) => m,
            _ => panic!("Checkerboard Texture requires a mapping")
        };

        let tex1 = match param.get_lead_object("tex1") {
            Some(LeadObject::Texture(t)) => t,
            _ => panic!("Checkerboard Texture requires tex1")
        };

        let tex2 = match param.get_lead_object("tex2") {
            Some(LeadObject::Texture(t)) => t,
            _ => panic!("Checkerboard Texture requires tex2")
        };

        Texture::Checkerboard(
            CheckerboardTexture::init(
                Arc::new(mapping), 
                Arc::new(tex1), Arc::new(tex2))
        )
    }
}