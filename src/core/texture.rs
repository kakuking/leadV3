use std::fmt::Debug;

use crate::{core::{Point2, Printable, Vector2, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable, texture::{checkerboard::CheckerboardTexture, constant::ConstantTexture, image::ImageTexture, scale::ScaleTexture, uv_mapping::UVMapping2D, uv_texture::UVTexture}};

#[derive(Debug, PartialEq)]
pub enum TextureMapping2D {
    UV(UVMapping2D)
}

impl TextureMapping2D {
    pub fn map(&self, si: &SurfaceInteraction, dsdtx: &mut Vector2, dsdty: &mut Vector2) -> Point2 {
        match self {
            Self::UV(t) => t.map(si, dsdtx, dsdty)
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::UV(t) => t.to_string()
        }
    }
} 

pub trait TextureMapping2DT: Manufacturable<TextureMapping2D> + Printable {
    fn map(&self, si: &SurfaceInteraction, dsdtx: &mut Vector2, dsdty: &mut Vector2) -> Point2;
}

#[derive(Debug, PartialEq)]
pub enum Texture {
    Constant(ConstantTexture),
    UV(UVTexture),
    Checkerboard(CheckerboardTexture),
    Scale(ScaleTexture),
    Image(ImageTexture)
}

impl Texture {
    pub fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        match self {
            Texture::Constant(t) => t.evaluate(si),
            Texture::UV(t) => t.evaluate(si),
            Texture::Checkerboard(t) => t.evaluate(si),
            Texture::Scale(t) => t.evaluate(si),
            Texture::Image(t) => t.evaluate(si),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Texture::Constant(t) => t.to_string(),
            Texture::UV(t) => t.to_string(),
            Texture::Checkerboard(t) => t.to_string(),
            Texture::Scale(t) => t.to_string(),
            Texture::Image(t) => t.to_string(),
        }
    }
}

pub trait TextureT: Manufacturable<Texture> + Printable {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum;
}
