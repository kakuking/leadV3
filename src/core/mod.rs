pub mod geometry;
pub mod medium;
pub mod math;
pub mod interaction;
pub mod bsdf;
pub mod shape;
pub mod scene;
pub mod primitive;
pub mod material;
pub mod camera;
pub mod light;
pub mod sampler;

pub use math::*;
pub use geometry::*;

pub type Spectrum = nalgebra::Vector3<f32>;

pub trait Printable {
    fn to_string(&self) -> String;
}