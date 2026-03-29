use std::sync::{Arc, atomic::AtomicU32};

use atomic_float::AtomicF32;

use crate::{core::{Point2, Point3, Vector3, bsdf::BSDF, camera::Camera, distribution::compute_light_power_distribution, scene::Scene, spectrum::Spectrum}, sampler::halton::HaltonSampler};

pub struct VisiblePoint {
    pub p: Point3,
    pub wo: Vector3,
    pub bsdf: Option<Arc<BSDF>>,
    pub beta: Spectrum
}

impl VisiblePoint {
    pub fn new() -> Self {
        Self {
            p: Point3::origin(),
            wo: Vector3::zeros(),
            bsdf: None,
            beta: Spectrum::zeros()
        }
    }

    pub fn init(p: &Point3, wo: &Vector3, bsdf: Option<Arc<BSDF>>, beta: &Spectrum) -> Self {
        Self {
            p: p.clone(),
            wo: wo.clone(),
            bsdf: bsdf,
            beta: beta.clone()
        }
    }
}

struct SPPMPixel {
    pub radius: f32,
    pub ld: Spectrum,
    pub vp: VisiblePoint,
    pub phi: [AtomicF32; 3],
    pub m: AtomicU32,
    pub n: f32,
    pub tau: Spectrum,
}

impl SPPMPixel {
    pub fn new() -> Self {
        Self {
            radius: 0.0,
            ld: Spectrum::zeros(),
            vp: VisiblePoint::new(),
            phi: [AtomicF32::new(0.0), AtomicF32::new(0.0), AtomicF32::new(0.0)],
            m: AtomicU32::new(0),
            n: 0.0,
            tau: Spectrum::zeros()
        }
    }
}

pub struct SPPMIntegrator {
    camera: Arc<Camera>,
    initial_search_radius: f32,
    n_iterations: usize,
    max_depth: usize,
    photons_per_iteration: usize,
    write_frequency: usize
}

impl SPPMIntegrator {
    pub fn init(
        camera: Arc<Camera>, 
        n_iterations: usize,
        photons_per_iteration: usize,
        max_depth: usize,
        initial_search_radius: f32,
        write_frequency: usize
    ) -> Self {
        let photons_per_iteration = if photons_per_iteration > 0 {
            photons_per_iteration
        } else {
            camera.get_film().cropped_pixel_bounds.area() as usize
        };

        Self {
            camera,
            n_iterations,
            photons_per_iteration,
            max_depth,
            initial_search_radius,
            write_frequency
        }
    }

    pub fn render(&self, scene: &Scene) {
        let pixel_bounds = self.camera.get_film().cropped_pixel_bounds.clone();
        let n_pixels = pixel_bounds.area() as usize;
        let mut pixels: Vec<SPPMPixel> = Vec::new();

        for _ in 0..n_pixels {
            let mut pixel = SPPMPixel::new();
            pixel.radius = self.initial_search_radius;

            pixels.push(
                pixel
            );
        }

        let light_distr = compute_light_power_distribution(scene);

        let sampler = HaltonSampler::init(self.n_iterations, pixel_bounds.clone());
        let pixel_extent = pixel_bounds.diagonal();
        let tile_size: usize = 16;

        let n_tiles = Point2::new(
            (pixel_extent.x + tile_size as f32 - 1.0) / tile_size as f32,
            (pixel_extent.y + tile_size as f32 - 1.0) / tile_size as f32,
        );

    }
}