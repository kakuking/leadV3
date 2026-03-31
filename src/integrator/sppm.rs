// use std::sync::{Arc, atomic::AtomicU32};

// use atomic_float::AtomicF32;

// use crate::{core::{Point2, Point3, Ray, RayDifferential, Vector3, bounds::Bounds2, bsdf::BSDF, bxdf::BxDFType, camera::Camera, distribution::compute_light_power_distribution, integrator::uniform_sample_one_light, interaction::{Interaction, InteractionT, TransportMode}, sampler::{Sampler}, scene::Scene, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, sampler::halton::HaltonSampler};

// pub struct VisiblePoint {
//     pub p: Point3,
//     pub wo: Vector3,
//     pub bsdf: Option<BSDF>,
//     pub beta: Spectrum
// }

// impl VisiblePoint {
//     pub fn new() -> Self {
//         Self {
//             p: Point3::origin(),
//             wo: Vector3::zeros(),
//             bsdf: None,
//             beta: Spectrum::zeros()
//         }
//     }

//     pub fn init(p: &Point3, wo: &Vector3, bsdf: Option<BSDF>, beta: &Spectrum) -> Self {
//         Self {
//             p: p.clone(),
//             wo: wo.clone(),
//             bsdf: bsdf,
//             beta: beta.clone()
//         }
//     }
// }

// struct SPPMPixel {
//     pub radius: f32,
//     pub ld: Spectrum,
//     pub vp: VisiblePoint,
//     pub phi: [AtomicF32; 3],
//     pub m: AtomicU32,
//     pub n: f32,
//     pub tau: Spectrum,
// }

// impl SPPMPixel {
//     pub fn new() -> Self {
//         Self {
//             radius: 0.0,
//             ld: Spectrum::zeros(),
//             vp: VisiblePoint::new(),
//             phi: [AtomicF32::new(0.0), AtomicF32::new(0.0), AtomicF32::new(0.0)],
//             m: AtomicU32::new(0),
//             n: 0.0,
//             tau: Spectrum::zeros()
//         }
//     }
// }

// pub struct SPPMIntegrator {
//     camera: Arc<Camera>,
//     initial_search_radius: f32,
//     n_iterations: usize,
//     max_depth: usize,
//     photons_per_iteration: usize,
//     write_frequency: usize
// }

// impl SPPMIntegrator {
//     pub fn init(
//         camera: Arc<Camera>, 
//         n_iterations: usize,
//         photons_per_iteration: usize,
//         max_depth: usize,
//         initial_search_radius: f32,
//         write_frequency: usize
//     ) -> Self {
//         let photons_per_iteration = if photons_per_iteration > 0 {
//             photons_per_iteration
//         } else {
//             camera.get_film().cropped_pixel_bounds.area() as usize
//         };

//         Self {
//             camera,
//             n_iterations,
//             photons_per_iteration,
//             max_depth,
//             initial_search_radius,
//             write_frequency
//         }
//     }

//     pub fn render(&self, scene: &Scene) {
//         let pixel_bounds = self.camera.get_film().cropped_pixel_bounds.clone();
//         let n_pixels = pixel_bounds.area() as usize;
//         let mut pixels: Vec<SPPMPixel> = Vec::new();

//         for _ in 0..n_pixels {
//             let mut pixel = SPPMPixel::new();
//             pixel.radius = self.initial_search_radius;

//             pixels.push(
//                 pixel
//             );
//         }

//         let light_distr = compute_light_power_distribution(scene);

//         let sampler = Sampler::Halton(
//             HaltonSampler::init(self.n_iterations, pixel_bounds.clone())
//         );
//         let pixel_extent = pixel_bounds.diagonal();
//         let tile_size: f32 = 16.0;

//         let n_tiles = Point2::new(
//             (pixel_extent.x + tile_size - 1.0) / tile_size,
//             (pixel_extent.y + tile_size - 1.0) / tile_size,
//         );

//         let tile_for_bounds = Bounds2::init_two(
//             &Point2::origin(), 
//             &n_tiles
//         );

//         for iter in 0..self.n_iterations {
//             for tile in &tile_for_bounds {
//                 let tile_idx = tile.y * n_tiles.x + tile.x;
//                 let mut tile_sampler = sampler.clone_with_seed(tile_idx as usize);

//                 let x0 = pixel_bounds.p_min.x + tile.x * tile_size;
//                 let x1 = (x0 + tile_size).min(pixel_bounds.p_max.x);

//                 let y0 = pixel_bounds.p_min.y + tile.y * tile_size;
//                 let y1 = (y0 + tile_size).min(pixel_bounds.p_max.y);

//                 let tile_bounds = Bounds2::init_two(
//                     &Point2::new(x0, y0), 
//                     &Point2::new(x1, y1) 
//                 );

//                 for p_pixel in &tile_bounds {
//                     tile_sampler.start_pixel(p_pixel.clone());
//                     tile_sampler.set_sample_number(iter);

//                     let camera_sample = tile_sampler.get_camera_sample(&p_pixel);
//                     let mut ray = Ray::new();
//                     let beta = self.camera.generate_ray_differential(camera_sample, &mut ray);
//                     let mut beta = Spectrum::new(beta, beta, beta);

//                     let p_pixel_o: Point2 = (p_pixel - pixel_bounds.p_min).into();
//                     let pixel_offset = p_pixel_o.x + p_pixel_o.y * (pixel_bounds.p_max.x - pixel_bounds.p_min.x);

//                     let pixel = &mut pixels[pixel_offset as usize];

//                     let mut specular_bounce = false;
//                     let mut depth = 0;
//                     // for depth in 0..self.max_depth {
//                     loop {
//                         if depth >= self.max_depth { break; }
//                         let mut its = SurfaceInteraction::new();


//                         if !scene.intersect(&ray, &mut its) {
//                             for light in &scene.lights {
//                                 pixel.ld += beta.component_mul(&light.le(&ray));
//                             }

//                             break;
//                         }

//                         its.compute_scattering_functions(
//                             &ray, 
//                             true, 
//                             TransportMode::Radiance
//                         );

//                         if let Some(bsdf) = &its.bsdf {
//                             let wo = -ray.d;
//                             if depth == 0 || specular_bounce {
//                                 pixel.ld += beta.component_mul(&its.le(&wo));
//                             }

//                             pixel.ld += beta.component_mul(&uniform_sample_one_light(
//                                 &Interaction::Surface(its.clone()), 
//                                 scene, 
//                                 &mut tile_sampler, 
//                                 false
//                             ));

//                             let is_diffuse = bsdf.num_components(
//                                 BxDFType::BSDF_DIFFUSE | BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION
//                             ) > 0;
//                             let is_glossy = bsdf.num_components(
//                                 BxDFType::BSDF_GLOSSY | BxDFType::BSDF_REFLECTION | BxDFType::BSDF_TRANSMISSION
//                             ) > 0;

//                             if is_diffuse || (is_glossy && depth == self.max_depth-1) {
//                                 pixel.vp = VisiblePoint { 
//                                     p: its.get_p().clone(), 
//                                     wo, 
//                                     bsdf: Some(bsdf.clone()), 
//                                     beta: beta.clone()
//                                 };
//                                 break;
//                             }

//                             if depth < self.max_depth - 1 {
//                                 let mut pdf = 0.0;
//                                 let mut wi = Vector3::zeros();
//                                 let mut typ = BxDFType::empty();

//                                 let f = bsdf.sample_f(&wo, &mut wi, &tile_sampler.get_2d(), &mut pdf, BxDFType::BSDF_ALL, &mut typ);

//                                 if pdf == 0.0 || f == Spectrum::zeros() {
//                                     break;
//                                 }

//                                 specular_bounce = typ.contains(BxDFType::BSDF_SPECULAR);
//                                 beta.component_mul_assign(
//                                     &(f * wi.dot(&its.shading.n).abs() / pdf)
//                                 );

//                                 if beta.y < 0.25 {
//                                     let cont_prob = beta.y.min(1.0);
//                                     if tile_sampler.get_1d() > cont_prob {
//                                         break;
//                                     }

//                                     beta /= cont_prob;
//                                 }

//                                 ray = its.spawn_ray(&wi);
//                                 ray.differential = Some(RayDifferential::new());
//                             }

//                             depth  += 1;
//                         } else {
//                             ray = its.spawn_ray(&ray.d);
//                             // since we continue without increment we dont need to decrement
//                             continue;
//                         }
//                     }
//                 }
//             }
        
//             // create grid of all SPPM visible points onwards
//         }
//     }
// }