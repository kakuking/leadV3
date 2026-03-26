use crate::{core::{Point2, Printable, Vector2, filter::{Filter, FilterT}}, loader::Parameters, registry::Manufacturable};

#[derive(Clone)]
pub struct BoxFilter {
    pub radius: Vector2,
    pub inv_radius: Vector2,
}

impl BoxFilter {
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

impl FilterT for BoxFilter {
    fn get_radius(&self) -> Vector2 {
        self.radius
    }

    fn get_inv_radius(&self) -> Vector2 {
        self.inv_radius
    }

    fn evaluate(&self, _p: &Point2) -> f32 {
        1.0
    }
}

impl Manufacturable<Filter> for BoxFilter {
    fn create_from_parameters(param: Parameters) -> Filter {
        let radius = param.get_vector2("radius", Some(Vector2::new(1.0, 1.0)));

        Filter::Bx(
            BoxFilter::init(radius)
        )
    }
}

impl Printable for BoxFilter {
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