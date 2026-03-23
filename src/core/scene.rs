use std::sync::Arc;

use crate::{camera::orthographic::OrthographicCamera, core::{camera::Camera, sampler::Sampler, shape::Shape}, loader::{Manufacturable, Parameters}, sampler::stratified_sampler::StratifiedSampler};

pub struct Scene {
    pub shapes: Vec<Arc<Shape>>,
    pub sampler: Sampler,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Self {
        let camera = OrthographicCamera::create_from_parameters(
            Parameters::new()
            );
        let sampler = StratifiedSampler::create_from_parameters(
            Parameters::new()
        );

        Self {
            shapes: Vec::new(),
            camera,
            sampler
        }
    }

    pub fn add_shapes(&mut self, shapes: Vec<Shape>) {
        for shape in shapes {
            self.shapes.push(Arc::new(shape));
        }
    }

    pub fn add_camera(&mut self, camera: Camera) {
        self.camera = camera;
    }

    pub fn add_sampler(&mut self, sampler: Sampler) {
        self.sampler = sampler;
    }
}