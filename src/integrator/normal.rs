use crate::{core::{Printable, Ray, camera::Camera, integrator::{Integrator, SamplerIntegrator}, interaction::InteractionT, light::LightStrategy, primitive::Primitive, sampler::Sampler, scene::Scene, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};

pub struct NormalIntegrator {
    max_depth: usize,

    camera: Camera,
    sampler: Sampler,
    strategy: LightStrategy,

    n_light_samples: Vec<usize>,
}

impl SamplerIntegrator for NormalIntegrator {
    fn get_camera(&self) -> &Camera { &self.camera }
    fn get_sampler(&self) -> &Sampler { &self.sampler }
    fn set_camera(&mut self, camera: Camera) { self.camera = camera }
    fn set_sampler(&mut self, sampler: Sampler) { self.sampler = sampler }

    fn get_mut_camera(&mut self) -> &mut Camera { &mut self.camera }
    fn get_mut_sampler(&mut self) -> &mut Sampler { &mut self.sampler }

    fn preprocess(&mut self, scene: &Scene, sampler: &mut Sampler) {
        if self.strategy == LightStrategy::UniformSampleAll {
            for light in &scene.lights {
                self.n_light_samples.push(
                    sampler.round_count(light.get_n_samples() as usize)
                );
            }

            for _ in 0..self.max_depth {
                for j in 0..scene.lights.len() {
                    sampler.request_2d_array(self.n_light_samples[j]);
                    sampler.request_2d_array(self.n_light_samples[j]);
                }
            }
        }
    }

    fn li(&self, ray: &Ray, scene: &Scene, sampler: &mut Sampler, depth: Option<u32>) -> Spectrum {
        let mut l = Spectrum::zeros();
        let mut its = SurfaceInteraction::new();

        if !scene.intersect(ray, &mut its) {
            for light in &scene.lights {
                l += light.le(ray);
            }

            return l;
        }


        match &its.bsdf {
            Some(bsdf) => println!("Bsdf: {:?}", bsdf),
            None => println!("Nop BSDF") 
        };

        its.get_n().clone().map(|x| x.abs())
    }

    fn specular_reflect(&self, _ray: &Ray, _its: &SurfaceInteraction, _scene: &Scene, _sampler: &mut Sampler, _depth: u32) -> Spectrum {
        panic!("DirectIntegrator::SpecularReflect");
    }
    
    fn specular_transmit(&self, _ray: &Ray, _its: &SurfaceInteraction, _scene: &Scene, _sampler: &mut Sampler, _depth: u32) -> Spectrum {
        panic!("DirectIntegrator::SpecularTransmit");
    }
}

impl Manufacturable<Integrator> for NormalIntegrator {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Integrator {
        let it = Self {
            max_depth: 8,

            camera: Camera::Empty,
            sampler: Sampler::Empty,
            strategy: LightStrategy::UniformSampleAll,

            n_light_samples: Vec::new()
        };

        Integrator::Normal(it)
    }
}

impl Printable for NormalIntegrator {
    fn to_string(&self) -> String {
        format!(
            "Normal Integrator [\n
            \tCamera: {}\n
            \tSampler: {}\n
            ]",
            self.camera.to_string(),
            self.sampler.to_string()
        )
    }
}