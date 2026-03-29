use crate::{core::{Printable, Ray, RayDifferential, Vector3, bxdf::BxDFType, camera::Camera, integrator::{Integrator, uniform_sample_one_light}, interaction::{Interaction, InteractionT, TransportMode}, sampler::Sampler, scene::Scene, spectrum::Spectrum}, integrator::sampler_integrator::SamplerIntegrator, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};

pub struct PathIntegrator {
    max_depth: usize,

    camera: Camera,
    sampler: Sampler,

    n_light_samples: Vec<usize>,
}

impl SamplerIntegrator for PathIntegrator {
    fn get_camera(&self) -> &Camera { &self.camera }
    fn get_sampler(&self) -> &Sampler { &self.sampler }
    fn set_camera(&mut self, camera: Camera) { self.camera = camera }
    fn set_sampler(&mut self, sampler: Sampler) { self.sampler = sampler }

    fn get_mut_camera(&mut self) -> &mut Camera { &mut self.camera }
    fn get_mut_sampler(&mut self) -> &mut Sampler { &mut self.sampler }

    fn preprocess(&mut self, _scene: &Scene) { }

    fn li(&self, r: &Ray, scene: &Scene, sampler: &mut Sampler, _depth: Option<u32>) -> Spectrum {
        let mut l = Spectrum::zeros();
        let mut beta = Spectrum::new(1.0, 1.0, 1.0);
        let mut specular_bounce = false;
        let mut ray = r.clone();
        ray.differential = Some(RayDifferential::new());
        
        let mut bounces = 0;
        loop {
            let mut its = SurfaceInteraction::new();
            let found_its = scene.intersect(&ray, &mut its);

            if bounces == 0 || specular_bounce {
                if found_its {
                    l += beta.component_mul(&(its.le(&-ray.d)));
                } else {
                    for light in &scene.lights {
                        l += beta.component_mul(&(light.le(&ray)));
                    }
                }
            }

            if !found_its || bounces >= self.max_depth {
                break;
            }

            its.compute_scattering_functions(&ray, true, TransportMode::Radiance);

            if its.bsdf.is_none() {
                ray = its.spawn_ray(&ray.d);
                // bounces -= 1;    // since we manually increment we dont need this
                continue;
            }

            l += beta.component_mul(&uniform_sample_one_light(&Interaction::Surface(its.clone()), scene, sampler, &self.n_light_samples, false));

            let wo = -ray.d;
            let mut wi = Vector3::zeros();
            let mut pdf = 0.0;
            let mut flags: BxDFType =  BxDFType::empty();
            let f = match &its.bsdf {
                Some(s) => s.sample_f(&wo, &mut wi, &sampler.get_2d(), &mut pdf, BxDFType::BSDF_ALL, &mut flags),
                None => Spectrum::zeros()
            };

            if f == Vector3::zeros() || pdf == 0.0 {
                break;
            }

            beta.component_mul_assign(&(f * wi.dot(&its.shading.n).abs() / pdf));
            specular_bounce = flags.contains(BxDFType::BSDF_SPECULAR);
            ray = its.spawn_ray(&wi);

            bounces += 1;

            if false && flags.contains(BxDFType::BSDF_TRANSMISSION) {
                if let Some(_bssrdf) = &its.bssrdf {
                    let pi = SurfaceInteraction::new();
                    // let s = bssrdf.sample_s
                    let s = Spectrum::zeros();

                    if s == Vector3::zeros() || pdf == 0.0 {
                        break;
                    }

                    beta.component_mul_assign(&(s/pdf));

                    // l += beta.component_mul(&(uniform_sample_one_light(&Interaction::Medium(pi), scene, sampler, &self.n_light_samples, false)));

                    let f = match &pi.bsdf {
                        Some(bsdf) => bsdf.sample_f(pi.get_wo(), &mut wi, &sampler.get_2d(), &mut pdf, BxDFType::BSDF_ALL, &mut flags),
                        None => Spectrum::zeros()
                    };

                    if f == Vector3::zeros() || pdf == 0.0 {
                        break;
                    }

                    beta.component_mul_assign(&(f * wi.dot(&pi.shading.n).abs() / pdf));
                    specular_bounce = flags.contains(BxDFType::BSDF_SPECULAR);
                    ray = pi.spawn_ray(&wi);
                }
            }

            if bounces > 3 {
                let q = (1.0 - beta.y).max(0.05);
                if sampler.get_1d() < q {
                    break;
                }

                beta /= 1.0 - q;
            }
        }

        l
    }
}

impl Manufacturable<Integrator> for PathIntegrator {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Integrator {
        let it = Self {
            max_depth: 8,

            camera: Camera::Empty,
            sampler: Sampler::Empty,

            n_light_samples: Vec::new()
        };

        Integrator::Path(it)
    }
}

impl Printable for PathIntegrator {
    fn to_string(&self) -> String {
        format!(
            "Path Integrator [\n
            \tCamera: {}\n
            \tSampler: {}\n
            ]",
            self.camera.to_string(),
            self.sampler.to_string()
        )
    }
}