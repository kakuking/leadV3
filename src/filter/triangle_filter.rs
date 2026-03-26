use crate::{core::{Point2, Printable, Vector2, filter::{Filter, FilterT}}, loader::Parameters, registry::Manufacturable};

#[derive(Clone)]
pub struct TriangleFilter {
    pub radius: Vector2,
    pub inv_radius: Vector2,
}

impl TriangleFilter {
    pub fn new() -> Self {
        Self {
            radius: Vector2::new(1.0, 1.0),
            inv_radius: Vector2::new(1.0, 1.0),
        }
    }

    pub fn init(radius: Vector2) -> Self {
        Self {
            radius,
            inv_radius: Vector2::new(1.0 / radius.x, 1.0 / radius.y)
        }
    }
}

impl FilterT for TriangleFilter {
    fn get_radius(&self) -> Vector2 {
        self.radius
    }

    fn get_inv_radius(&self) -> Vector2 {
        self.inv_radius
    }

    fn evaluate(&self, p: &Point2) -> f32 {
        (self.radius.x -  p.x.abs()).max(0.0) * (self.radius.y - p.y.abs()).max(0.0)
    }
}

impl Printable for TriangleFilter {
    fn to_string(&self) -> String {
        format!(
            "Traingle Filter: [\n
            \tRadius: ({}, {})\n
            ]",
            self.radius.x,
            self.radius.y
        )
    }
}

impl Manufacturable<Filter> for TriangleFilter {
    fn create_from_parameters(param: Parameters) -> Filter {
        let radius = param.get_vector2("radius", Some(Vector2::new(1.0, 1.0)));

        Filter::Triangle(
            TriangleFilter::init(radius)
        )
    }
}