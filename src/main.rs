use std::cell::Cell;

use nalgebra::{Point3, Vector3};

use crate::{core::{INFINITY, Printable, Ray, shape::Shape}, loader::{Manufacturable, Registry}, shape::Sphere};

pub mod core;

pub mod interaction;
pub mod shape;

pub mod loader;

fn main() {
    let mut registry = Registry::new();
    registry.register_shape(
        "sphere".to_string(),
        Box::new(|params| Shape::Sphere(Sphere::create_from_parameters(params))),
    );

    let scene = match loader::parse_xml("sample_scene.xml", registry) {
        Some(s) => s,
        _ => panic!("No scene found!")
    };

    let ray = Ray::init(&Point3::new(10.0, 10.0, 10.0), &Vector3::new(-1.0, -1.0, -1.0).normalize(), 17.3, Cell::new(0.0), None, None);

    let shapes = &scene.shapes;

    println!("Ray: {}\n\n", ray.to_string());

    for shape in shapes {
        println!("Shape: {}", shape.to_string());
        let its = shape.intersect_p(&ray, None);
        println!("Intersects?: {}\n", its);
    }


}
