use crate::{core::{Printable, Ray, camera::Camera, integrator::{Integrator, SamplerIntegrator, uniform_sample_all_lights, uniform_sample_one_light}, interaction::{Interaction, InteractionT, TransportMode}, light::LightStrategy, sampler::Sampler, scene::Scene, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};

pub struct DirectIntegrator {
    max_depth: u32,

    camera: Camera,
    sampler: Sampler,
    strategy: LightStrategy,

    n_light_samples: Vec<usize>,
}

impl SamplerIntegrator for DirectIntegrator {
    fn get_camera(&self) -> &Camera { &self.camera }
    fn get_sampler(&self) -> &Sampler { &self.sampler }
    fn set_camera(&mut self, camera: Camera) { self.camera = camera }
    fn set_sampler(&mut self, sampler: Sampler) { self.sampler = sampler }

    fn get_mut_camera(&mut self) -> &mut Camera { &mut self.camera }
    fn get_mut_sampler(&mut self) -> &mut Sampler { &mut self.sampler }

    fn preprocess(&mut self, scene: &Scene) {
        if self.strategy == LightStrategy::UniformSampleAll {
            for light in &scene.lights {
                self.n_light_samples.push(
                    self.sampler.round_count(light.get_n_samples() as usize)
                );
            }

            for _ in 0..self.max_depth {
                for j in 0..scene.lights.len() {
                    self.sampler.request_2d_array(self.n_light_samples[j]);
                    self.sampler.request_2d_array(self.n_light_samples[j]);
                }
            }
        }
    }

    fn li(&self, ray: &Ray, scene: &Scene, sampler: &mut Sampler, depth: Option<u32>) -> Spectrum {
        let depth = match depth {
            Some(d) => d,
            None => 1
        };

        let mut l = Spectrum::zeros();
        let mut its = SurfaceInteraction::new();

        if !scene.intersect(ray, &mut its) {
            for light in &scene.lights {
                l += light.le(ray);
            }

            return l;
        }

        its.compute_scattering_functions(ray, true, TransportMode::Radiance);

        if its.bsdf.is_none() {
            return self.li(&its.spawn_ray(&ray.d), scene, sampler, Some(depth));
        }

        let wo = its.get_wo();

        l += its.le(&wo);

        if scene.lights.len() > 0 {
            if self.strategy == LightStrategy::UniformSampleAll {
                l += uniform_sample_all_lights(&Interaction::Surface(its.clone()), scene, sampler, &self.n_light_samples, false);
            } else {
                l += uniform_sample_one_light(&Interaction::Surface(its.clone()), scene, sampler, &self.n_light_samples, false);
            }
        }

        if depth + 1 < self.max_depth {
            l += self.specular_reflect(ray, &its, scene, sampler, depth);
            l += self.specular_transmit(ray, &its, scene, sampler, depth);
        }
        
        l
    }
}

impl Manufacturable<Integrator> for DirectIntegrator {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Integrator {
        let it = Self {
            max_depth: 8,

            camera: Camera::Empty,
            sampler: Sampler::Empty,
            strategy: LightStrategy::UniformSampleAll,

            n_light_samples: Vec::new()
        };

        Integrator::Direct(it)
    }
}

impl Printable for DirectIntegrator {
    fn to_string(&self) -> String {
        format!(
            "Direct Integrator [\n
            \tCamera: {}\n
            \tSampler: {}\n
            ]",
            self.camera.to_string(),
            self.sampler.to_string()
        )
    }
}