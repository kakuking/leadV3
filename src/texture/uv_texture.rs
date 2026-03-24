use std::sync::Arc;

use crate::{core::{Vector2, spectrum::Spectrum, texture::{Texture, TextureMapping2D}}, interaction::surface_interaction::SurfaceInteraction};


#[derive(Debug, Clone)]
pub struct UVTexture {
    mapping: Arc<TextureMapping2D>
}

impl UVTexture {
    pub fn new(mapping: Arc<TextureMapping2D>) -> Self {
        Self {
            mapping
        }
    }
}

impl Texture<Spectrum> for UVTexture {
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
