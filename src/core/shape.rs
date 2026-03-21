pub enum Shape {
    Mesh
}

impl Shape {
    pub fn new() -> Self {
        Shape::Mesh
    }

    pub fn reverse_orientation(&self) -> bool {
        // TO DO
        false
    }

    pub fn transform_swaps_handedness(&self) -> bool {
        // TO DO
        false
    }
}