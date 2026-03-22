use std::sync::Arc;

use crate::core::shape::Shape;

pub struct Scene {
    pub shapes: Vec<Arc<Shape>>
}

impl Scene {
    pub fn new() -> Self {
        Self {
            shapes: Vec::new()
        }
    }

    pub fn add_shapes(&mut self, shapes: Vec<Shape>) {
        for shape in shapes {
            self.shapes.push(Arc::new(shape));
        }
    }
}