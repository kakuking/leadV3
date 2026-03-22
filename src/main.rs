use std::io::Write;
use std::{fs::File, sync::Arc};

use crate::{core::{INFINITY, Point3, Printable, Ray, Transform, Vector3, interaction::InteractionT, medium::MediumInterface, primitive::{GeometricPrimitive, Primitive}, scene::Scene, shape::Shape, translation}, interaction::surface_interaction::SurfaceInteraction, loader::{Manufacturable, Registry}, shape::{Sphere, bounding_volume_heirarchy::{BVHAccel, SplitMethod}, triangle_mesh::TriangleMesh}};

pub mod core;

pub mod interaction;
pub mod shape;
pub mod light;

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

fn load_scene_and_test_bruteforce(registry: &Registry) {
    let scene = match loader::parse_xml("sample_scene.xml", registry) {
        Some(s) => s,
        _ => panic!("No scene found!"),
    };

    let ray = Ray::init(
        &Point3::new(10.0, 10.0, 10.0),
        &Vector3::new(-1.0, -1.0, -1.0).normalize(),
        100.0,
        0.0,
        None,
        None,
    );

    let shapes = &scene.shapes;
    let mi = MediumInterface::new();

    let mut primitives: Vec<Arc<Primitive>> = Vec::new();
    for shape in shapes {
        let gp = GeometricPrimitive::init(shape.clone(), None, None, mi.clone());
        primitives.push(Arc::new(Primitive::Geometric(Arc::new(gp))));
    }

    println!("Ray: {}\n\n", ray.to_string());
    println!("Brute force intersection testing - ");

    let mut closest_t = ray.t_max.get();
    let mut hit_anything = false;
    let mut best_isect = SurfaceInteraction::new();

    for prim in &primitives {
        let mut ray_local = ray.clone();
        ray_local.t_max.set(closest_t);

        let mut isect = SurfaceInteraction::new();
        if prim.intersect(&mut ray_local, &mut isect) {
            hit_anything = true;
            closest_t = ray_local.t_max.get();
            best_isect = isect;
        }
    }

    if hit_anything {
        let prim = best_isect.primitive.unwrap();
        println!("Intersects?: {}, Primitive: \n{}", true, prim.to_string());
    } else {
        println!("Intersects?: {}\n", false);
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

    let width: usize = 480;
    let height: usize = 480;

    let eye = Point3::new(10.0, 10.0, 10.0);
    let look_at = Point3::new(0.0, 0.0, 0.0);
    let up = Vector3::new(0.0, 1.0, 0.0);

    // Simple pinhole camera basis
    let forward = (look_at - eye).normalize();
    let right = forward.cross(&up).normalize();
    let true_up = right.cross(&forward).normalize();

    // 1:1 aspect ratio, use a simple 45 degree vertical FOV
    let fov_y_deg: f32 = 45.0;
    let fov_y = fov_y_deg.to_radians();
    let half_height = (fov_y * 0.5).tan();
    let half_width = half_height; // aspect ratio = 1

    let mut pixels: Vec<u8> = vec![255; width * height * 3];

    for y in 0..height {
        for x in 0..width {
            // NDC in [-1, 1], sample pixel center
            let u = ((x as f32 + 0.5) / width as f32) * 2.0 - 1.0;
            let v = 1.0 - ((y as f32 + 0.5) / height as f32) * 2.0;

            let dir = (forward
                + right * (u * half_width)
                + true_up * (v * half_height))
                .normalize();

            let ray = Ray::init(&eye, &dir, 1.0e30, 0.0, None, None);

            let mut isect = SurfaceInteraction::new();
            let hit = accel.intersect(&ray, &mut isect);

            let idx = (y * width + x) * 3;
            if hit {
                // black
                pixels[idx] = 0;
                pixels[idx + 1] = 0;
                pixels[idx + 2] = 0;
            } else {
                // white
                pixels[idx] = 255;
                pixels[idx + 1] = 255;
                pixels[idx + 2] = 255;
            }
        }
    }

    let mut file = File::create("hit_test.ppm").expect("Failed to create output PPM");
    writeln!(file, "P3").unwrap();
    writeln!(file, "{} {}", width, height).unwrap();
    writeln!(file, "255").unwrap();

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) * 3;
            writeln!(
                file,
                "{} {} {}",
                pixels[idx],
                pixels[idx + 1],
                pixels[idx + 2]
            )
            .unwrap();
        }
    }

    println!("Wrote hit_test.ppm");
}

fn main() {
    let mut registry = Registry::new();
    registry.register_shape(
        "sphere".to_string(),
        Box::new(|params| vec![Shape::Sphere(Sphere::create_from_parameters(params))]),
    );

    registry.register_shape(
        "mesh".to_string(),
        Box::new(|params| TriangleMesh::create_from_parameters(params))
    );

    // load_scene_and_test(&registry);
    load_scene_and_render_hit_ppm(&registry);
    // load_scene_and_render_hit_ppm_bruteforce(&registry);

    // benchmark_bruteforce_vs_bvh(1000, 10, 10);
}
