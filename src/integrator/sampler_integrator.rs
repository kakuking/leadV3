use indicatif::{ProgressBar, ProgressStyle, ParallelProgressIterator};
use rayon::prelude::*;


use crate::{core::{Point2, Printable, Ray, RayDifferential, Vector3, bounds::Bounds2, bxdf::BxDFType, camera::Camera, integrator::Integrator, interaction::InteractionT, sampler::Sampler, scene::Scene, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};

pub trait SamplerIntegrator: Manufacturable<Integrator> + Printable {
    fn get_camera(&self) -> &Camera;
    fn get_sampler(&self) -> &Sampler;
    fn set_camera(&mut self, camera: Camera);
    fn set_sampler(&mut self, sampler: Sampler);

    fn get_mut_camera(&mut self) -> &mut Camera;
    fn get_mut_sampler(&mut self) -> &mut Sampler;

    // From integrator
    fn render(&mut self, scene: &Scene)
    where
    Self: Sync
    {
        let sample_bounds = self.get_mut_camera().get_film().get_sample_bounds();
        let sample_extent = sample_bounds.diagonal();

        let tile_size = 16.0;

        let n_tiles = Point2::new(
            (sample_extent.x + tile_size - 1.0) / tile_size,
            (sample_extent.y + tile_size - 1.0) / tile_size,
        );

        let tile_for_bounds = Bounds2::init_two(
            &Point2::origin(), 
            &n_tiles
        );

        let tiles: Vec<Point2> = (&tile_for_bounds).into_iter().collect();

        let pb = ProgressBar::new(tiles.len() as u64);
        pb.set_style(ProgressStyle::with_template(
            "[{elapsed_precise}] {wide_bar} {pos}/{len} tiles ({eta})"
        ).unwrap());

        // for tile in &tile_for_bounds{
        tiles
        .par_iter()
        .progress_with(pb.clone())
        .for_each(|tile| {
            let seed = tile.y * n_tiles.x + tile.x;
            let mut tile_sampler = self.get_sampler().clone_with_seed(seed as usize);

            let x0 = sample_bounds.p_min.x + tile.x * tile_size;
            let x1 = (x0 + tile_size).min(sample_bounds.p_max.x);

            let y0 = sample_bounds.p_min.y + tile.y * tile_size;
            let y1 = (y0 + tile_size).min(sample_bounds.p_max.y);

            let tile_bounds = Bounds2::init_two(
                &Point2::new(x0, y0), 
                &Point2::new(x1, y1) 
            );

            let mut film_tile = self.get_camera().get_film().get_film_tile(&tile_bounds);

            for pixel in &tile_bounds{
                tile_sampler.start_pixel(pixel.clone());

                'per_pixel_sample: loop {
                    let camera_sample = tile_sampler.get_camera_sample(&pixel);

                    let mut ray: Ray = Ray::new();

                    let ray_weight = self.get_camera().generate_ray_differential(camera_sample.clone(), &mut ray);

                    ray.scale_differentials(1.0 / (tile_sampler.get_samples_per_pixel() as f32).sqrt());

                    let mut l = Spectrum::zeros();
                    if ray_weight > 0.0 {
                        l = self.li(&ray, scene, &mut tile_sampler, None);
                    }

                    if l.iter().any(|x| x.is_nan()) {
                        eprintln!("NaN radiance returned, setting to black");
                        l = Spectrum::zeros();
                    } else if l.y < -1e-5 {
                        eprintln!("Negative luminance returned, setting to black");
                        l = Spectrum::zeros();
                    } else if l.iter().any(|x| x.is_infinite()) {
                        eprintln!("Infinite luminance returned, setting to black");
                        l = Spectrum::zeros();
                    }

                    film_tile.add_sample(&camera_sample.p_film, &l, ray_weight);

                    if !tile_sampler.start_next_sample() {
                        break 'per_pixel_sample;
                    }
                }
            }
            self.get_camera().get_film().merge_film_tile(&film_tile);
        });

        pb.finish();
        
        self.get_mut_camera().get_mut_film().write_image(1.0);
    }

    fn preprocess(&mut self, _scene: &Scene) {}
    fn li(&self, ray: &Ray, scene: &Scene, sampler: &mut Sampler, depth: Option<u32>) -> Spectrum;

    fn specular_reflect(&self, ray: &Ray, its: &SurfaceInteraction, scene: &Scene, sampler: &mut Sampler, depth: u32) -> Spectrum {
        let wo = its.get_wo();
        let mut wi = Vector3::zeros();
        let mut pdf = 0.0;
        let typ = BxDFType::BSDF_REFLECTION | BxDFType::BSDF_SPECULAR;

        let mut flags = BxDFType::empty();

        let f = match &its.bsdf {
            Some(bsdf) => bsdf.sample_f(wo, &mut wi, &sampler.get_2d(), &mut pdf, typ, &mut flags),
            None => Spectrum::zeros()
        };

        let ns = &its.shading.n;

        if pdf > 0.0 && f != Spectrum::zeros() && wi.dot(ns).abs() != 0.0 {
            let mut rd = its.spawn_ray(&wi);
            if let Some(r_diff) = &ray.differential {
                let mut diff = RayDifferential::new();
                diff.rx_o = its.get_p() - -its.dpdx.get();
                diff.ry_o = its.get_p() - -its.dpdy.get();

                let dndx = its.shading.dndu * its.dudx.get() + its.shading.dndv * its.dvdx.get();
                let dndy = its.shading.dndu * its.dudy.get() + its.shading.dndv * its.dvdy.get();
                let dwodx = -r_diff.rx_d - wo;
                let dwody = -r_diff.ry_d - wo;
                let d_dn_dx = dwodx.dot(ns) + wo.dot(&dndx);
                let d_dn_dy = dwody.dot(ns) + wo.dot(&dndy);

                diff.rx_d = wi - dwodx + 2.0 * (wo.dot(ns) * dndx + d_dn_dx * ns);
                diff.ry_d = wi - dwody + 2.0 * (wo.dot(ns) * dndy + d_dn_dy * ns);

                rd.differential = Some(diff);
            }

            return f.component_mul(&self.li(&rd, scene, sampler, Some(depth + 1))) * wi.dot(ns).abs();
        }

        Spectrum::zeros()
    }
    
    fn specular_transmit(&self, ray: &Ray, its: &SurfaceInteraction, scene: &Scene, sampler: &mut Sampler, depth: u32) -> Spectrum {
        let wo = its.get_wo();
        let mut wi = Vector3::zeros();
        let mut pdf = 0.0;
        let typ = BxDFType::BSDF_TRANSMISSION | BxDFType::BSDF_SPECULAR;

        let mut flags = BxDFType::empty();

        let f = match &its.bsdf {
            Some(bsdf) => bsdf.sample_f(wo, &mut wi, &sampler.get_2d(), &mut pdf, typ, &mut flags),
            None => Spectrum::zeros()
        };

        let ns = &its.shading.n;

        if pdf > 0.0 && f != Spectrum::zeros() && wi.dot(ns).abs() != 0.0 {
            let mut rd = its.spawn_ray(&wi);
            if let Some(r_diff) = &ray.differential {
                let mut diff = RayDifferential::new();
                diff.rx_o = its.get_p() - -its.dpdx.get();
                diff.ry_o = its.get_p() - -its.dpdy.get();

                let dndx = its.shading.dndu * its.dudx.get() + its.shading.dndv * its.dvdx.get();
                let dndy = its.shading.dndu * its.dudy.get() + its.shading.dndv * its.dvdy.get();
                let dwodx = -r_diff.rx_d - wo;
                let dwody = -r_diff.ry_d - wo;
                let d_dn_dx = dwodx.dot(ns) + wo.dot(&dndx);
                let d_dn_dy = dwody.dot(ns) + wo.dot(&dndy);

                diff.rx_d = wi - dwodx + 2.0 * (wo.dot(ns) * dndx + d_dn_dx * ns);
                diff.ry_d = wi - dwody + 2.0 * (wo.dot(ns) * dndy + d_dn_dy * ns);

                rd.differential = Some(diff);
            }

            return f.component_mul(&self.li(&rd, scene, sampler, Some(depth + 1))) * wi.dot(ns).abs();
        }

        Spectrum::zeros()
    }
}