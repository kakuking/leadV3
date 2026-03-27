use std::thread;
use std::time::Instant;

use crate::core::primitive::GeometricPrimitive;
use crate::light::diffuse_area_light::DiffuseAreaLight;
use crate::registry::{Registry, Manufacturable};
use crate::core::Printable;

// Camera and film
use crate::camera::orthographic::OrthographicCamera;
use crate::core::film::Film;
use crate::filter::box_filter::BoxFilter;
use crate::filter::triangle_filter::TriangleFilter;

// Materials 
use crate::material::matte::MatteMaterial;

// Samplers
use crate::sampler::stratified_sampler::StratifiedSampler;

// Shapes
use crate::shape::Sphere;
use crate::shape::triangle_mesh::TriangleMesh;

// Integrators
use crate::integrator::direct::DirectIntegrator;
use crate::integrator::normal::NormalIntegrator;

// Lights
use crate::light::point_light::PointLight;

pub mod core;
pub mod loader;
pub mod registry;

pub mod interaction;
pub mod shape;
pub mod light;
pub mod camera;
pub mod sampler;
pub mod filter;
pub mod reflection;
pub mod texture;
pub mod integrator;
pub mod material;

fn load_scene_and_render_hit_ppm(registry: &Registry, num_threads: usize) {
    let num_threads = num_threads.min(thread::available_parallelism().unwrap().get());
    
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .unwrap();

    let mut instance = match loader::parse_xml("sample_scene.xml", registry) {
        Some(s) => s,
        _ => panic!("No scene found!"),
    };

    instance.init_scene();
    instance.preprocess();

    let integrator = instance.get_integrator();

    println!("Integrator: {}", integrator.to_string());
    println!("Scene: {}", instance.scene.to_string());
    println!("Rendering...");

    let duration = {
        let start = Instant::now();

        pool.install(|| {
            instance.render();
        });

        start.elapsed()
    };

    println!("Rendered with {} threads in - {:?}", num_threads, duration);
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

    registry.register_integrator(
        "direct".to_string(), 
        Box::new(|params| {
            DirectIntegrator::create_from_parameters(params)
        }),
    );

    registry.register_integrator(
        "normal".to_string(), 
        Box::new(|params| {
            NormalIntegrator::create_from_parameters(params)
        }),
    );

    registry.register_material(
        "matte".to_string(), 
        Box::new(|params| {
        MatteMaterial::create_from_parameters(params)
        }),
    );

    registry.register_primitive(
        "geometric".to_string(),
        Box::new(|params| {
            GeometricPrimitive::create_from_parameters(params)
        })
    );

    registry.register_light(
        "diffuse".to_string(), 
        Box::new(|params| {
            DiffuseAreaLight::create_from_parameters(params)
        })
    );

    load_scene_and_render_hit_ppm(&registry, 20);
}