use std::sync::Arc;

use crate::{core::{Point2, Printable, Ray, Vector3, bxdf::BxDFType, camera::Camera, interaction::{Interaction, InteractionT}, light::{Light, VisibilityTester, is_delta_light}, random::power_heuristic, sampler::Sampler, scene::Scene, spectrum::Spectrum}, integrator::{color::ColorIntegrator, direct::DirectIntegrator, normal::NormalIntegrator, path::PathIntegrator, sampler_integrator::SamplerIntegrator}, interaction::surface_interaction::SurfaceInteraction};

pub enum Integrator {
    Direct(DirectIntegrator),
    Normal(NormalIntegrator),
    Color(ColorIntegrator),
    Path(PathIntegrator),
    Empty,
}

impl Integrator {
    pub fn render(&mut self, scene: &Scene) {
        match self {
            Self::Direct(i) => i.render(scene),
            Self::Normal(i) => i.render(scene),
            Self::Color(i) => i.render(scene),
            Self::Path(i) => i.render(scene),
            Self::Empty => panic!("Render called on empty integrator"),
        }
    }

    pub fn set_camera(&mut self, camera: Camera) {
        match self {
            Self::Direct(i) => i.set_camera(camera),
            Self::Normal(i) => i.set_camera(camera),
            Self::Color(i) => i.set_camera(camera),
            Self::Path(i) => i.set_camera(camera),
            Self::Empty => panic!("Set camera called on empty integrator"),
        }
    }

    pub fn set_sampler(&mut self, sampler: Sampler) {
        match self {
            Self::Direct(i) => i.set_sampler(sampler),
            Self::Normal(i) => i.set_sampler(sampler),
            Self::Color(i) => i.set_sampler(sampler),
            Self::Path(i) => i.set_sampler(sampler),
            Self::Empty => panic!("Set sampler called on empty integrator"),
        }
    }

    pub fn preprocess(&mut self, scene: &Scene) {
        match self {
            Self::Direct(i) => i.preprocess(scene),
            Self::Normal(i) => i.preprocess(scene),
            Self::Color(i) => i.preprocess(scene),
            Self::Path(i) => i.preprocess(scene),
            Self::Empty => panic!("preprocess called on empty integrator")
        }
    }
    
    pub fn li(&self, ray: &Ray, scene: &Scene, sampler: &mut Sampler, depth: Option<u32>) -> Spectrum {
        match self {
            Self::Direct(i) => i.li(ray, scene, sampler, depth),
            Self::Normal(i) => i.li(ray, scene, sampler, depth),
            Self::Color(i) => i.li(ray, scene, sampler, depth),
            Self::Path(i) => i.li(ray, scene, sampler, depth),
            Self::Empty => panic!("li called on empty integrator")
        }
    }
}

impl Printable for Integrator {
    fn to_string(&self) -> String {
        match self {
            Self::Direct(i) => i.to_string(),
            Self::Normal(i) => i.to_string(),
            Self::Color(i) => i.to_string(),
            Self::Path(i) => i.to_string(),
            Self::Empty => panic!("to String called on empty integrator"),
        }
    }
}

pub fn uniform_sample_all_lights(it: &Interaction, scene: &Scene, sampler: &mut Sampler, n_light_samples: &Vec<usize>, handle_media: bool) -> Spectrum {
    let mut l = Spectrum::zeros();

    for j in 0..scene.lights.len() {
        let light = &scene.lights[j];

        let n_samples = n_light_samples[j];
        let u_light_array = sampler.get_2d_array(n_samples);
        let u_scattering_array = sampler.get_2d_array(n_samples);

        if u_light_array.len() == 0 || u_scattering_array.len() == 0 {
            let u_light = sampler.get_2d();
            let u_scattering = sampler.get_2d();

            l += estimate_direct(it, &u_scattering, light, &u_light, scene, sampler, handle_media, false);
        } else {
            let mut ld = Spectrum::zeros();
            for k in 0..n_samples {
                ld += estimate_direct(it, &u_scattering_array[k], light, &u_light_array[k], scene, sampler, handle_media, false);
            }

            l += ld / n_samples as f32;
        }
    }

    l
}

pub fn uniform_sample_one_light(it: &Interaction, scene: &Scene, sampler: &mut Sampler, _n_light_samples: &Vec<usize>, handle_media: bool) -> Spectrum {
    let n_lights = scene.lights.len();
    if n_lights == 0 {
        return Spectrum::zeros();
    }

    let light_num = ((sampler.get_1d() * n_lights as f32) as usize).min(n_lights - 1);
    let light = &scene.lights[light_num];

    let u_light = sampler.get_2d();
    let u_scattering = sampler.get_2d();

    n_lights as f32 * estimate_direct(it, &u_scattering, light, &u_light, scene, sampler, handle_media, false)
}

pub fn estimate_direct(it: &Interaction, u_scattering: &Point2, light: &Arc<Light>, u_light: &Point2, scene: &Scene, sampler: &mut Sampler, handle_media: bool, specular: bool) -> Spectrum {
    let bsdf_flags = if specular {
        BxDFType::BSDF_ALL
    } else {
        BxDFType::BSDF_ALL & !BxDFType::BSDF_SPECULAR
    };

    let mut ld = Spectrum::zeros();

    let mut light_pdf = 0.0;
    let mut scattering_pdf = 0.0;

    let mut wi = Vector3::zeros();
    let mut visibility: VisibilityTester = VisibilityTester::new();

    let mut li = light.sample_li(it.get_base(), u_light, &mut wi, &mut light_pdf, &mut visibility);

    // println!("sampled light, li: {:?}", li);

    if light_pdf > 0.0 && li != Vector3::zeros() {
        let f: Spectrum;

        match &it {
            Interaction::Surface(s) => {
                f = match &s.bsdf {
                    Some(bsdf) => {
                        // println!("Found bsdf, calling .f");
                        scattering_pdf = bsdf.pdf(s.get_wo(), &wi, Some(bsdf_flags));
                        bsdf.f(&s.get_wo(), &wi, Some(bsdf_flags)) * wi.dot(&s.shading.n).abs()
                    },
                    None => {
                        eprintln!("No BSDF found in estimate direct");
                        Spectrum::zeros()
                    }
                };
            },
            _ => panic!("Medium Interaction not implemented yet")
        }

        if f != Spectrum::zeros() {
            // println!("F is not zero, checking handle media and allat");

            if handle_media {
                li = li.component_mul(&visibility.tr(scene, sampler));
            } 
            
            else if !visibility.unoccluded(scene) {
                // println!("Scene is occluded, li is black");
                li = Spectrum::zeros();
            }

            if li != Spectrum::zeros() {
                // println!("li is not zero");
                if is_delta_light(light.get_flags() as u32) {
                    ld += f.component_mul(&li) / light_pdf;
                    // println!("Is a delta light ld: {:?}, f: {:?}, li: {:?}, l_pdf: {}", ld,f, li, light_pdf);
                } else {
                    let weight = power_heuristic(1.0, light_pdf, 1.0, scattering_pdf);
                    ld += f.component_mul(&li) * weight / light_pdf;
                    // println!("Is not a delta light,f: {:?}, li: {:?}, weight: {}, ld: {:?}",f, li, weight, ld);
                }
            }
        }
    }

    if !is_delta_light(light.get_flags() as u32) {
        // println!("Not a delta light");
        let mut f: Spectrum;// = Spectrum::zeros();
        let sampled_specular: bool;// = false;

        match &it {
            Interaction::Surface(s) => {
                let mut sampled_type: BxDFType = BxDFType::empty();
                match &s.bsdf {
                    Some(bsdf) => {
                        // println!("Sampling f");
                        f = bsdf.sample_f(&s.get_wo(), &mut wi, u_scattering, &mut scattering_pdf, bsdf_flags, &mut sampled_type);
                        f *= wi.dot(&s.shading.n).abs();
                        sampled_specular = sampled_type.contains(BxDFType::BSDF_SPECULAR);
                        // println!("f: {:?}, scat_pdf: {}", f, scattering_pdf);
                    },
                    _ => panic!("No BSDF found on interaction")
                }
            }
            _ => panic!("Mediunm interaction not implemented")
        }

        if f != Spectrum::zeros() && scattering_pdf > 0.0 {
            // println!("F nonzera and +vs scat_pdf");
            let mut weight = 1.0;

            if !sampled_specular {
                light_pdf = light.pdf_li(it.get_base(), &wi);

                if light_pdf == 0.0 {
                    return ld;
                }

                weight = power_heuristic(1.0, scattering_pdf, 1.0, light_pdf);
            }

            let mut light_its = SurfaceInteraction::new();
            let ray = it.spawn_ray(&wi);
            let mut tr = Spectrum::new(1.0, 1.0, 1.0);

            let found_surface_its = if handle_media {
                scene.intersect_tr(&ray, sampler, &mut light_its, &mut tr)
            } else {
                scene.intersect(&ray, &mut light_its)
            };

            let mut li = Spectrum::zeros();

            // println!("li: {:?}", li);
            
            if found_surface_its {
                if let Some(al) = light_its.primitive.get_area_light() {
                    if Arc::ptr_eq(&al, light) {
                        // println!("light equals light");
                        li = light_its.le(&-wi);
                    }
                }
            } else {
                li = light.le(&ray);
            }
            // println!("li_now: {:?}", li);

            if li != Spectrum::zeros() {
                ld += f.component_mul(&li).component_mul(&tr) * weight / scattering_pdf;
            }
        }
    }

    // println!("Returning ld: {:?}", ld);
    ld
}