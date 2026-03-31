use std::sync::Arc;

use crate::{core::{MAX, Printable, Ray, interaction::{MediumInteraction}, medium::{Medium, MediumInterface, MediumT, PhaseFunction}, sampler::Sampler, spectrum::Spectrum}, medium::hg_phase::HenyeyGreenstein, registry::Manufacturable};

#[derive(Debug, PartialEq, Clone)]
pub struct HomogeneousMedium {
    sigma_a: Spectrum,
    sigma_s: Spectrum,
    sigma_t: Spectrum,
    g: f32
}

impl HomogeneousMedium {
    pub fn new() -> Self {
        Self {
            sigma_a: Spectrum::zeros(),
            sigma_s: Spectrum::zeros(),
            sigma_t: Spectrum::zeros(),
            g: 0.0
        }
    }

    pub fn init(sigma_a: Spectrum, sigma_s: Spectrum, sigma_t: Spectrum, g: f32) -> Self {
        Self {
            sigma_a,
            sigma_s,
            sigma_t,
            g
        }
    }
}

impl MediumT for HomogeneousMedium {
    fn sample(&self, ray: &Ray, sampler: &mut Sampler, mi: &mut MediumInteraction, medium: Arc<Medium>) -> Spectrum {
        let channel = ((sampler.get_1d() * 3.0) as usize).min(2);
        let dist = -(1.0 - sampler.get_1d()).ln() / self.sigma_t[channel];
        let t = (dist * ray.d.norm()).min(ray.t_max.get());
        let sampled_medium = t < ray.t_max.get();

        let phase = Arc::new(
            PhaseFunction::HG(
                HenyeyGreenstein::init(self.g)
            )
        );

        if sampled_medium {
            *mi = MediumInteraction::init_no_normal_one_medium(&ray.at(t), &-ray.d, ray.time, MediumInterface::init_one(Some(medium)), Some(phase));
        }

        let tr = (-self.sigma_t * t.min(MAX) * ray.d.norm()).map(|x| x.exp());

        let density = if sampled_medium {
            self.sigma_t.component_mul(&tr)
        } else {
            tr
        };

        let mut pdf = 0.0;
        for i in 0..3 {
            pdf += density[i];
        }

        pdf *= 1.0 / 3.0;

        if sampled_medium {
            tr.component_mul(&tr.component_mul(&self.sigma_s)) / pdf
        } else {
            tr / pdf
        }
    }

    fn tr(&self, ray: &Ray, _sampler: &mut Sampler) -> Spectrum {
        (-self.sigma_t * (ray.t_max.get() * ray.d.norm()).min(MAX)).map(|x| x.exp())
    }
}

impl Manufacturable<Medium> for HomogeneousMedium {
    fn create_from_parameters(param: crate::loader::Parameters) -> Medium {
        let sigma_a = param.get_vector3("sigma_a", Some(Spectrum::zeros()));
        let sigma_s = param.get_vector3("sigma_s", Some(Spectrum::zeros()));
        let sigma_t = sigma_a + sigma_s;

        let g = param.get_float("g", Some(0.0));

        Medium::Homogeneous(
            Self::init(sigma_a, sigma_s, sigma_t, g)
        )
    }
}

impl Printable for HomogeneousMedium {
    fn to_string(&self) -> String {
        format!(
            "Homogeneous Medium: [\n
            \tsigma_a: {}, {}, {}\n
            \tsigma_s: {}, {}, {}\n
            \tsigma_t: {}, {}, {}\n
            \tg: {}
            ",
            self.sigma_a.x, self.sigma_a.y, self.sigma_a.z, 
            self.sigma_s.x, self.sigma_s.y, self.sigma_s.z, 
            self.sigma_t.x, self.sigma_t.y, self.sigma_t.z, 
            self.g
        )
    }
}