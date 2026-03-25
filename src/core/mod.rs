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
pub mod filter;
pub mod film;
pub mod spectrum;
pub mod image;
pub mod bxdf;
pub mod texture;
pub mod integrator;
pub mod random;

pub mod lead_instance;

pub use math::*;
pub use geometry::*;

pub trait Printable {
    fn to_string(&self) -> String;
}