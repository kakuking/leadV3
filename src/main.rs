use std::io::Write;
use std::{fs::File, sync::Arc};

use crate::camera::orthographic::OrthographicCamera;
use crate::core::Point2;
use crate::core::camera::{CameraSample};
use crate::core::film::Film;
use crate::filter::box_filter::BoxFilter;
use crate::filter::triangle_filter::TriangleFilter;
use crate::light::point_light::PointLight;
use crate::sampler::stratified_sampler::StratifiedSampler;
use crate::{core::{INFINITY, Point3, Printable, Ray, Transform, Vector3, interaction::InteractionT, medium::MediumInterface, primitive::{GeometricPrimitive, Primitive}, scene::Scene, shape::Shape, translation}, interaction::surface_interaction::SurfaceInteraction, loader::{Manufacturable, Registry}, shape::{Sphere, bounding_volume_heirarchy::{BVHAccel, SplitMethod}, triangle_mesh::TriangleMesh}};

pub mod core;

pub mod interaction;
pub mod shape;
pub mod light;
pub mod camera;
pub mod sampler;
pub mod filter;
pub mod reflection;
pub mod texture;

pub mod loader;

fn load_scene_and_render_hit_ppm(registry: &Registry) {
    let mut scene = match loader::parse_xml("sample_scene.xml", registry) {
        Some(s) => s,
        _ => panic!("No scene found!"),
    };

    // let shapes = &scene.shapes;
    // let mut accel: BVHAccel = BVHAccel::init(32, SplitMethod::SAH);
    // let mi = MediumInterface::new();
    // for shape in shapes {
    //     let gp = GeometricPrimitive::init(shape.clone(), None, None, mi.clone());
    //     accel.add_primitive(Arc::new(Primitive::Geometric(Arc::new(gp))));
    // }
    // accel.build();

    scene.init();

    println!("{}", scene.to_string());

    let camera = scene.get_camera();
    let mut sampler = scene.get_sampler();

    let film = camera.get_film();
    let width = film.full_resolution.x as usize;
    let height = film.full_resolution.y as usize;

    let mut pixels: Vec<u8> = vec![255; width * height * 3];

    for y in 0..height {
        for x in 0..width {
            let pixel = Point2::new(x as f32, y as f32);
            sampler.start_pixel(pixel);

            let mut hit_count = 0usize;
            let mut num_samples = 0usize;

            loop {
                num_samples += 1;

                let p_film_offset = sampler.get_2d();
                let p_lens = sampler.get_2d();

                let sample = CameraSample {
                    p_film: Point2::new(
                        x as f32 + p_film_offset.x,
                        y as f32 + p_film_offset.y,
                    ),
                    p_lens,
                    time: sampler.get_1d(),
                };

                let mut ray = Ray::new();
                let wt = camera.generate_ray(sample, &mut ray);

                if wt != 0.0 {
                    let mut isect = SurfaceInteraction::new();
                    if scene.intersect(&ray, &mut isect) {
                        hit_count += 1;
                    }
                }

                if !sampler.start_next_sample() {
                    break;
                }
            }

            // println!("{:4} {:4} -> {:4} samples", x, y, num_samples);

            let idx = (y * width + x) * 3;

            let coverage = hit_count as f32 / num_samples as f32;
            let value = ((1.0 - coverage) * 255.0) as u8;

            pixels[idx] = value;
            pixels[idx + 1] = value;
            pixels[idx + 2] = value;
        }
    }

    let mut file = File::create("output.ppm").expect("Failed to create output PPM");
    writeln!(file, "P3").unwrap();
    writeln!(file, "{} {}", width, height).unwrap();
    writeln!(file, "255").unwrap();
    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            writeln!(file, "{} {} {}", pixels[idx], pixels[idx + 1], pixels[idx + 2]).unwrap();
        }
    }

    println!("Wrote hit_test.ppm");
}

fn main() {
    let mut registry = Registry::new();

    registry.register_shape(
        "sphere".to_string(),
        Box::new(|params| {
            vec![Sphere::create_from_parameters(params)]
        }),
    );

    registry.register_shape(
        "mesh".to_string(),
        Box::new(|params| {
            TriangleMesh::create_from_parameters(params)
        }),
    );

    registry.register_camera(
        "orthographic".to_string(),
        Box::new(|params| {
            OrthographicCamera::create_from_parameters(params)
        }),
    );

    registry.register_sampler(
        "stratified".to_string(),
        Box::new(|params| {
            StratifiedSampler::create_from_parameters(params)
        }),
    );

    registry.register_filter(
        "box".to_string(),
        Box::new(|params| {
            BoxFilter::create_from_parameters(params)
        }),
    );

    registry.register_filter(
        "triangle".to_string(),
        Box::new(|params| {
            TriangleFilter::create_from_parameters(params)
        }),
    );

    registry.register_film(
        "film".to_string(),
        Box::new(|params| {
            Film::create_from_parameters(params)
        }),
    );

    registry.register_light(
        "point".to_string(), 
        Box::new(|params| {
            PointLight::create_from_parameters(params)
        }),
    );

    load_scene_and_render_hit_ppm(&registry);
}