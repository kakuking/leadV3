use std::io::Write;
use std::{fs::File, sync::Arc};

use crate::camera::orthographic::OrthographicCamera;
use crate::core::Point2;
use crate::core::camera::{CameraSample};
use crate::sampler::stratified_sampler::StratifiedSampler;
use crate::{core::{INFINITY, Point3, Printable, Ray, Transform, Vector3, interaction::InteractionT, medium::MediumInterface, primitive::{GeometricPrimitive, Primitive}, scene::Scene, shape::Shape, translation}, interaction::surface_interaction::SurfaceInteraction, loader::{Manufacturable, Registry}, shape::{Sphere, bounding_volume_heirarchy::{BVHAccel, SplitMethod}, triangle_mesh::TriangleMesh}};

pub mod core;

pub mod interaction;
pub mod shape;
pub mod light;
pub mod camera;
pub mod sampler;

pub mod loader;

use std::time::{Instant};

use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

pub fn benchmark_bruteforce_vs_bvh(
    num_spheres: usize,
    num_rand_rays: usize,
    num_pointed_rays: usize,
) {
    let mut rng = StdRng::seed_from_u64(42);

    // Create random scene
    let mut scene = Scene::new();

    let mut sphere_data: Vec<(Point3, f32)> = Vec::with_capacity(num_spheres);

    for _ in 0..num_spheres {
        let radius = rng.random_range(0.25_f32..2.0_f32);

        let center = Point3::new(
            rng.random_range(-100.0_f32..100.0_f32),
            rng.random_range(-100.0_f32..100.0_f32),
            rng.random_range(-100.0_f32..100.0_f32),
        );

        let object_to_world: Transform = translation(center.coords);
        let world_to_object: Transform = object_to_world.inverse();

        let reverse_orientation = false;
        let z_min = -radius;
        let z_max = radius;
        let phi_max = 360.0_f32;

        let sphere = Shape::Sphere(Sphere::init(
            object_to_world,
            world_to_object,
            reverse_orientation,
            radius,
            z_min,
            z_max,
            phi_max,
        ));

        scene.shapes.push(Arc::new(sphere));
        sphere_data.push((center, radius));
    }

    // Create random rays
    let mut random_rays = Vec::with_capacity(num_rand_rays);

    for _ in 0..num_rand_rays {
        let origin = Point3::new(
            rng.random_range(-150.0_f32..150.0_f32),
            rng.random_range(-150.0_f32..150.0_f32),
            rng.random_range(-150.0_f32..150.0_f32),
        );

        let mut dir = Vector3::new(
            rng.random_range(-1.0_f32..1.0_f32),
            rng.random_range(-1.0_f32..1.0_f32),
            rng.random_range(-1.0_f32..1.0_f32),
        );

        while dir.norm_squared() < 1e-8 {
            dir = Vector3::new(
                rng.random_range(-1.0_f32..1.0_f32),
                rng.random_range(-1.0_f32..1.0_f32),
                rng.random_range(-1.0_f32..1.0_f32),
            );
        }

        random_rays.push(Ray::init(
            &origin,
            &dir.normalize(),
            INFINITY,
            0.0,
            None,
            None,
        ));
    }

    // Create pointed rays
    let mut pointed_rays = Vec::with_capacity(num_pointed_rays);

    for _ in 0..num_pointed_rays {
        let sphere_idx = rng.random_range(0..sphere_data.len());
        let (center, radius) = sphere_data[sphere_idx];

        let mut dir_from_center = Vector3::new(
            rng.random_range(-1.0_f32..1.0_f32),
            rng.random_range(-1.0_f32..1.0_f32),
            rng.random_range(-1.0_f32..1.0_f32),
        );

        while dir_from_center.norm_squared() < 1e-8 {
            dir_from_center = Vector3::new(
                rng.random_range(-1.0_f32..1.0_f32),
                rng.random_range(-1.0_f32..1.0_f32),
                rng.random_range(-1.0_f32..1.0_f32),
            );
        }

        let dir_from_center = dir_from_center.normalize();

        let distance_from_center = radius + rng.random_range(5.0_f32..25.0_f32);
        let origin = center + dir_from_center * distance_from_center;
        let ray_dir = (center - origin).normalize();

        pointed_rays.push(Ray::init(
            &origin,
            &ray_dir,
            INFINITY,
            0.0,
            None,
            None,
        ));
    }

    // Combine all rays
    let mut rays = Vec::with_capacity(num_rand_rays + num_pointed_rays);
    rays.extend(random_rays);
    rays.extend(pointed_rays);

    // Build BVH
    let mut accel = BVHAccel::init(32, SplitMethod::SAH);
    let mi = MediumInterface::new();

    for shape in &scene.shapes {
        let gp = GeometricPrimitive::init(shape.clone(), None, None, mi.clone());
        accel.add_primitive(Arc::new(Primitive::Geometric(Arc::new(gp))));
    }

    accel.build();

    // Brute force benchmark
    let brute_start = Instant::now();

    let mut brute_hits = 0usize;
    for ray in &rays {
        let mut ray_hit = false;
        for shape in &scene.shapes {
            if shape.intersect_p(ray, None) {
                ray_hit = true;
                break;
            }
        }
        if ray_hit {
            brute_hits += 1;
        }
    }

    let brute_elapsed = brute_start.elapsed();

    // BVH traversal benchmark
    let bvh_start = Instant::now();

    let mut bvh_hits = 0usize;
    for ray in &rays {
        let mut isect = SurfaceInteraction::new();
        if accel.intersect(ray, &mut isect) {
            bvh_hits += 1;
        }
    }

    let bvh_elapsed = bvh_start.elapsed();

    // Results
    let total_rays = rays.len();

    println!("=== Benchmark Results ===");
    println!("Spheres: {}", num_spheres);
    println!("Random rays: {}", num_rand_rays);
    println!("Pointed rays: {}", num_pointed_rays);
    println!("Total rays: {}", total_rays);
    println!("Brute-force hits: {}", brute_hits);
    println!("BVH hits: {}", bvh_hits);
    println!("Brute-force time: {:?}", brute_elapsed);
    println!("BVH traversal time: {:?}", bvh_elapsed);

    if bvh_elapsed.as_nanos() > 0 {
        let speedup = brute_elapsed.as_secs_f64() / bvh_elapsed.as_secs_f64();
        println!("Speedup (brute / BVH): {:.3}x", speedup);
    }
}

fn load_scene_and_test(registry: &Registry) {
    let scene = match loader::parse_xml("sample_scene.xml", registry) {
        Some(s) => s,
        _ => panic!("No scene found!")
    };

    let ray = Ray::init(&Point3::new(10.0, 10.0, 10.0), &Vector3::new(-1.0, -1.0, -1.0).normalize(), 100.0, 0.0, None, None);

    let shapes = &scene.shapes;
    let mut accel = BVHAccel::init(32, SplitMethod::SAH);

    let mi = MediumInterface::new();
    for shape in shapes {
        let gp = GeometricPrimitive::init(shape.clone(), None, None, mi.clone());

        accel.add_primitive(Arc::new(Primitive::Geometric(Arc::new(gp))));
    }

    accel.build();

    println!("Ray: {}\n\n", ray.to_string());

    println!("BVH accel intersection testing - ");
    let mut isect = SurfaceInteraction::new();
    if accel.intersect(&ray, &mut isect) {
        let prim = isect.primitive.unwrap();
        println!("Intersects?: {}, Primitive: \n{}", true, prim.to_string());
    } else {
        println!("Intersects?: {}\n", false);
    }
}

fn load_scene_and_render_hit_ppm(registry: &Registry) {
    let scene = match loader::parse_xml("sample_scene.xml", registry) {
        Some(s) => s,
        _ => panic!("No scene found!"),
    };

    let shapes = &scene.shapes;
    let mut accel = BVHAccel::init(32, SplitMethod::SAH);
    let mi = MediumInterface::new();
    for shape in shapes {
        let gp = GeometricPrimitive::init(shape.clone(), None, None, mi.clone());
        accel.add_primitive(Arc::new(Primitive::Geometric(Arc::new(gp))));
    }
    accel.build();

    let camera = &scene.camera;
    let mut sampler = scene.sampler;

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
                    if accel.intersect(&ray, &mut isect) {
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

    let mut file = File::create("hit_test.ppm").expect("Failed to create output PPM");
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
        Box::new(
            |params| {
                vec![Sphere::create_from_parameters(params)]
            }
        ),
    );

    registry.register_shape(
        "mesh".to_string(),
        Box::new(
            |params| {
                TriangleMesh::create_from_parameters(params)
            }
        )
    );

    registry.register_camera(
        "orthographic".to_string(), 
        Box::new(
            |params| {
                OrthographicCamera::create_from_parameters(params)
            }
        )
    );

    registry.register_sampler(
        "stratified".to_string(), 
        Box::new(
            |params| {
                StratifiedSampler::create_from_parameters(params)
            }
        )
    );

    // load_scene_and_test(&registry);
    load_scene_and_render_hit_ppm(&registry);
    // load_scene_and_render_hit_ppm_bruteforce(&registry);

    // benchmark_bruteforce_vs_bvh(1000, 10, 10);
}
