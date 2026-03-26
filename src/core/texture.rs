use std::{fmt::Debug, sync::Arc};

use crate::{core::{Point2, Vector2, spectrum::{Spectrum, rgb_to_xyz}}, interaction::surface_interaction::SurfaceInteraction, texture::uv_mapping::UVMapping2D};

#[derive(Debug)]
pub enum TextureMapping2D {
    UV(UVMapping2D)
}

impl TextureMapping2D {
    pub fn map(&self, si: &SurfaceInteraction, dsdtx: &mut Vector2, dsdty: &mut Vector2) -> Point2 {
        match self {
            Self::UV(t) => t.map(si, dsdtx, dsdty)
        }
    }
} 
pub trait TextureMapping2DT {
    fn map(&self, si: &SurfaceInteraction, dsdtx: &mut Vector2, dsdty: &mut Vector2) -> Point2;
}

pub trait Texture<T>: Debug + Send + Sync {
    fn evaluate(&self, si: &SurfaceInteraction) -> T;
}

#[derive(Debug, Clone)]
pub struct ConstantTexture<T: Debug> {
    value: T
}

impl<T: Debug + Clone> ConstantTexture<T> {
    pub fn new(value: T) -> Self {
        Self {
            value
        }
    }
}

impl<T: Debug + Clone + Send + Sync> Texture<T> for ConstantTexture<T> {
    fn evaluate(&self, _si: &SurfaceInteraction) -> T {
        self.value.clone()
    }
}

