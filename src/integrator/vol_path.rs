use crate::{core::{Printable, Ray, RayDifferential, Vector3, bxdf::BxDFType, camera::Camera, integrator::{Integrator, uniform_sample_one_light}, interaction::{Interaction, InteractionT, MediumInteraction, TransportMode}, sampler::Sampler, scene::Scene, spectrum::Spectrum}, integrator::sampler_integrator::SamplerIntegrator, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};

pub struct VolumePathIntegrator {
    max_depth: usize,

    camera: Camera,
    sampler: Sampler,

}

impl SamplerIntegrator for VolumePathIntegrator {
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

            let mut mi = MediumInteraction::new();

            if let Some(med) = &ray.medium {
                beta.component_mul_assign(&med.sample(&ray, sampler, &mut mi, med.clone()));
            }

            if beta == Spectrum::zeros() {
                break;
            }

            if let Some(phase) = &mi.phase.clone() {
                l += beta.component_mul(&uniform_sample_one_light(&Interaction::Medium(mi.clone()), scene, sampler, true));

                let wo = -ray.d;
                let mut wi = Vector3::zeros();

                phase.sample_p(&wo, &mut wi, &sampler.get_2d());

                ray = mi.spawn_ray(&wi);
            } else {
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

                l += beta.component_mul(&uniform_sample_one_light(&Interaction::Surface(its.clone()), scene, sampler, true));

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
                    if let Some(bssrdf) = &its.bssrdf {
                        let mut pi = SurfaceInteraction::new();
                        let s = bssrdf.sample_s(bssrdf.clone(), scene, sampler.get_1d(), &sampler.get_2d(), &mut pi, &mut pdf);

                        if s == Vector3::zeros() || pdf == 0.0 {
                            break;
                        }

                        beta.component_mul_assign(&(s/pdf));

                        l += beta.component_mul(&uniform_sample_one_light(&Interaction::Surface(pi.clone()), scene, sampler, true));

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

impl Manufacturable<Integrator> for VolumePathIntegrator {
    fn create_from_parameters(param: crate::loader::Parameters) -> Integrator {
        let max_depth = param.get_int("max_depth", Some(8)) as usize;
        
        let it = Self {
            max_depth,

            camera: Camera::Empty,
            sampler: Sampler::Empty,
        };

        Integrator::VolPath(it)
    }
}

impl Printable for VolumePathIntegrator {
    fn to_string(&self) -> String {
        format!(
            "Volume Path Integrator [\n
            \tCamera: {}\n
            \tSampler: {}\n
            ]",
            self.camera.to_string(),
            self.sampler.to_string()
        )
    }
}