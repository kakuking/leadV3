use std::sync::Arc;

use crate::{core::{Printable, Vector2, spectrum::Spectrum, texture::{Texture, TextureMapping2D, TextureT}}, interaction::surface_interaction::SurfaceInteraction, registry::{LeadObject, Manufacturable}};


#[derive(Debug, Clone, PartialEq)]
pub struct UVTexture {
    mapping: Arc<TextureMapping2D>
}

impl UVTexture {
    pub fn init(mapping: Arc<TextureMapping2D>) -> Self {
        Self {
            mapping
        }
    }
}

impl TextureT for UVTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let mut dsdtx = Vector2::zeros();
        let mut dsdty = Vector2::zeros();

        let st = self.mapping.map(si, &mut dsdtx, &mut dsdty);

        Spectrum::new(
            st[0] - st[0].floor(),
            st[1] - st[1].floor(),
            0.0
        )
    }
}

impl Manufacturable<Texture> for UVTexture {
    fn create_from_parameters(param: crate::loader::Parameters) -> Texture {
        let mut param = param;
        
        let mapping = match param.get_lead_object("mapping") {
            Some(LeadObject::TextureMapping(m)) => m,
            _ => panic!("UV Texture requires a mapping")
        };

        let tex = UVTexture::init(Arc::new(mapping));

        Texture::UV(tex)
    }
}

impl Printable for UVTexture {
    fn to_string(&self) -> String {
        format!(
            "UV Texture: \n
            \tmapping: {}\n
            ]",
            self.mapping.to_string()
        )
    }
}
