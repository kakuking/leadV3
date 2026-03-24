use std::sync::Arc;

use crate::{camera::orthographic::OrthographicCamera, core::{camera::Camera, filter::Filter, sampler::Sampler, shape::Shape}, filter::box_filter::BoxFilter, loader::{Manufacturable, Parameters}, sampler::stratified_sampler::StratifiedSampler};

pub struct Scene {
    pub filters: Vec<Filter>,

    pub shapes: Vec<Arc<Shape>>,
    pub sampler: Sampler,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Self {
        let camera = Camera::Empty;
        let sampler = StratifiedSampler::create_from_parameters(
            Parameters::new()
        );
        let filter = Filter::Bx(
            BoxFilter::new()
        );
        
        Self {
            filters: vec![filter],

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