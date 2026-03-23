use std::sync::Arc;

use crate::{camera::orthographic::OrthographicCamera, core::{camera::Camera, shape::Shape}, loader::{Manufacturable, Parameters}};

pub struct Scene {
    pub shapes: Vec<Arc<Shape>>,
    pub camera: Camera
}

impl Scene {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new(),
            camera: Camera::Orthographic(OrthographicCamera::create_from_parameters(Parameters::new()))
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
}