pub mod geometry;
pub mod medium;
pub mod math;
pub mod interaction;
pub mod bsdf;
pub mod shape;

pub use math::*;
pub use geometry::*;

pub type Spectrum = nalgebra::Vector3<f32>;