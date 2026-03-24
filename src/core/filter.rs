use crate::core::{Point2, Printable, Vector2};
use crate::filter::box_filter::BoxFilter;
use crate::filter::triangle_filter::TriangleFilter;

#[derive(Clone)]
pub enum Filter {
    Bx(BoxFilter),
    Triangle(TriangleFilter)
}

impl Filter {
    pub fn get_radius(&self) -> Vector2 {
        match self {
            Self::Bx(f) => f.get_radius(),
            Self::Triangle(f) => f.get_radius(),
        }
    }

    pub fn evaluate(&self, p: &Point2) -> f32 {
        match self {
            Self::Bx(f) => f.evaluate(p),
            Self::Triangle(f) => f.evaluate(p),
        }
    }
}

pub trait FilterT: Printable {
    fn get_radius(&self) -> Vector2;
    fn get_inv_radius(&self) -> Vector2 {
        let rad = self.get_radius();
        Vector2::new(1.0 / rad.x, 1.0 / rad.y)
    }

    fn evaluate(&self, p: &Point2) -> f32;
}