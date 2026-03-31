use crate::{core::{INV_4PI, PI, Printable, Vector3, coordinate_system, medium::{PhaseFunction, PhaseFunctionT}, spherical_direction_with_ref}, registry::Manufacturable};

#[derive(Debug, Clone)]
pub struct HenyeyGreenstein {
    g: f32
}

impl HenyeyGreenstein {
    pub fn init(g: f32) -> Self { Self {g} }
}

impl PhaseFunctionT for HenyeyGreenstein {
    fn p(&self, wo: &crate::core::Vector3, wi: &crate::core::Vector3) -> f32 {
        phase_hg(wo.dot(wi), self.g)
    }

    fn sample_p(&self, wo: &crate::core::Vector3, wi: &mut crate::core::Vector3, u: &crate::core::Point2) -> f32 {
        let cos_theta = if self.g.abs() < 1e-3 {
            1.0 - 2.0 * u.x
        } else {
            let g = self.g;
            let sqr_term = (1.0 - g * g) / (1.0 + g - 2.0 * g * u.x);

            -(1.0 + g * g - sqr_term * sqr_term) / (2.0 * g)
        };

        let sin_theta = (1.0 - cos_theta*cos_theta).max(0.0).sqrt();
        let phi = 2.0 * PI * u.y;

        let mut v1 = Vector3::zeros();
        let mut v2 = Vector3::zeros();
        coordinate_system(wo, &mut v1, &mut v2);

        *wi = spherical_direction_with_ref(sin_theta, cos_theta, phi, &v1, &v2, &wo);

        phase_hg(cos_theta, self.g)
    }
}

impl Manufacturable<PhaseFunction> for HenyeyGreenstein {
    fn create_from_parameters(param: crate::loader::Parameters) -> PhaseFunction {
        let g = param.get_float("g", Some(0.5));

        PhaseFunction::HG(
            Self::init(g)
        )
    }
}

impl Printable for HenyeyGreenstein {
    fn to_string(&self) -> String {
        format!(
            "Henyey Greenstein Phase Function: [\n
            \tg: {}\n
            ]",
            self.g
        )
    }
}


#[inline]
pub fn phase_hg(cos_theta: f32, g: f32) -> f32 {
    let denom = 1.0 + g * g + 2.0 * g * cos_theta;
    INV_4PI * (1.0 - g * g) / (denom * denom.sqrt())
}