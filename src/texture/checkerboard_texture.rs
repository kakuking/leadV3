use std::{fmt::Debug, sync::Arc};

use crate::{core::{Vector2, spectrum::Spectrum, texture::{Texture, TextureMapping2D}}, interaction::surface_interaction::SurfaceInteraction};


#[derive(Debug, Clone)]
pub struct CheckerboardTexture<T> {
    mapping: Arc<TextureMapping2D>,

    tex1: Arc<dyn Texture<T>>,
    tex2: Arc<dyn Texture<T>>
}

impl<T: Debug + Clone> CheckerboardTexture<T> {
    pub fn new(mapping: Arc<TextureMapping2D>, tex1: Arc<dyn Texture<T>>, tex2: Arc<dyn Texture<T>>) -> Self {
        Self {
            mapping,
            tex1,
            tex2
        }
    }
}

impl<T: Debug + Clone> Texture<T> for CheckerboardTexture<T> {
    fn evaluate(&self, si: &SurfaceInteraction) -> T {
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
