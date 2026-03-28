use crate::{core::{Printable, Ray, Vector3, bxdf::BxDFType, camera::Camera, integrator::{Integrator, SamplerIntegrator}, interaction::{InteractionT, TransportMode}, light::LightStrategy, primitive::Primitive, sampler::Sampler, scene::Scene, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};

pub struct ColorIntegrator {
    max_depth: usize,

    camera: Camera,
    sampler: Sampler,
    strategy: LightStrategy,

    n_light_samples: Vec<usize>,
}

impl SamplerIntegrator for ColorIntegrator {
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

        if depth > self.max_depth as u32 {
            return Spectrum::zeros();
        }

        let mut its = SurfaceInteraction::new();

        if !scene.intersect(ray, &mut its) {
            return Spectrum::zeros();
        }

        its.compute_scattering_functions(ray, true, TransportMode::Radiance);

        if its.bsdf.is_none() {
            return self.li(&its.spawn_ray(&ray.d), scene, sampler, Some(depth));
        }

        // let wo = its.get_wo().clone();

        match &its.bsdf {
            Some(bsdf) => 
                {
                    let mut wi = Vector3::zeros();
                    let mut pdf = 0.0;
                    let mut sampled_type = BxDFType::empty();

                    let f = bsdf.sample_f(
                        its.get_wo(),
                        &mut wi,
                        &sampler.get_2d(),
                        &mut pdf,
                        BxDFType::BSDF_ALL,
                        &mut sampled_type,
                    );

                    if sampled_type.contains(BxDFType::BSDF_SPECULAR) {
                        if pdf == 0.0 {
                            return Spectrum::zeros();
                        }

                        let new_ray = its.spawn_ray(&wi);
                        let li = self.li(&new_ray, scene, sampler, Some(depth + 1));

                        return f.component_mul(&li) * wi.dot(&its.shading.n).abs() / pdf;
                    }
                    
                    return f;
                },
            None => return Spectrum::zeros()
        };        
    }

    fn specular_reflect(&self, _ray: &Ray, _its: &SurfaceInteraction, _scene: &Scene, _sampler: &mut Sampler, _depth: u32) -> Spectrum {
        panic!("DirectIntegrator::SpecularReflect");
    }
    
    fn specular_transmit(&self, _ray: &Ray, _its: &SurfaceInteraction, _scene: &Scene, _sampler: &mut Sampler, _depth: u32) -> Spectrum {
        panic!("DirectIntegrator::SpecularTransmit");
    }
}

impl Manufacturable<Integrator> for ColorIntegrator {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Integrator {
        let it = Self {
            max_depth: 8,

            camera: Camera::Empty,
            sampler: Sampler::Empty,
            strategy: LightStrategy::UniformSampleAll,

            n_light_samples: Vec::new()
        };

        Integrator::Color(it)
    }
}

impl Printable for ColorIntegrator {
    fn to_string(&self) -> String {
        format!(
            "Color Integrator [\n
            \tCamera: {}\n
            \tSampler: {}\n
            ]",
            self.camera.to_string(),
            self.sampler.to_string()
        )
    }
}