use crate::{core::{INFINITY, Normal3, PI, Point2, Point3, Printable, Ray, Transform, Vector3, interaction::InteractionBase, light::{Light, LightFlags, LightT, VisibilityTester}, medium::MediumInterface, random::uniform_sample_sphere_pdf, spectrum::Spectrum}, loader::Parameters, registry::Manufacturable};
use crate::core::random::uniform_sample_sphere;

#[derive(Debug, Clone)]
pub struct PointLight {
    flags: LightFlags,
    n_samples: u32,
    medium_interface: MediumInterface,
    light_to_world: Transform,
    world_to_light: Transform,

    p_light: Point3,
    i: Spectrum
}

impl PointLight {
    pub fn init(light_to_world: Transform, mi: MediumInterface, i: Spectrum) -> Self {
        Self {
            flags: LightFlags::DeltaPosition,
            n_samples: 1,
            medium_interface: mi,

            light_to_world,
            world_to_light: light_to_world.inverse(),

            p_light: light_to_world.transform_point(&Point3::origin()),
            i
        }
    }
}

impl LightT for PointLight {
    fn get_flags(&self) -> LightFlags { self.flags }
    fn get_n_samples(&self) -> u32 { self.n_samples }
    fn get_medium_interface(&self) -> MediumInterface { self.medium_interface.clone() }
    fn get_light_to_world(&self) -> Transform { self.light_to_world }
    fn get_world_to_light(&self) -> Transform { self.world_to_light }

    fn sample_li(&self, re: &InteractionBase, _u: &Point2, wi: &mut Vector3, pdf: &mut f32, vis: &mut VisibilityTester) -> Spectrum {
        *wi = (self.p_light - re.p).normalize();
        *pdf = 1.0;

        *vis = VisibilityTester::init(
            re, 
            &InteractionBase::init_no_wo(&self.p_light, re.time, self.medium_interface.clone())
        );

        self.i / (self.p_light - re.p).norm_squared()
    }

    fn power(&self) -> Spectrum { 4.0 * PI * self.i }

    fn le(&self, _r: &Ray) -> Spectrum { Spectrum::zeros() }

    fn pdf_li(&self, _re: &InteractionBase, _wi: &Vector3) -> f32 { 0.0 }
    
    fn sample_le(&self, u1: &Point2, _u2: &Point2, time: f32, ray: &mut Ray, n_light: &mut Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum {
        *ray = Ray::init(&self.p_light, &uniform_sample_sphere(u1), INFINITY, time, self.medium_interface.inside.clone(), None);
        *n_light = ray.d;
        *pdf_pos = 1.0;
        *pdf_dir = uniform_sample_sphere_pdf();

        self.i
    }

    fn pdf_le(&self, _ray: &Ray, _n_light: &Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) {
        *pdf_pos = 0.0;
        *pdf_dir = uniform_sample_sphere_pdf();
    }
}

impl Manufacturable<Light> for PointLight {
    fn create_from_parameters(params: Parameters) -> Light {
        let light_to_world = params.get_transform();
        let i = params.get_vector3("i", Some(Spectrum::new(5.0, 5.0, 5.0)));

        let mi = MediumInterface::new();

        Light::Point(
            PointLight::init(light_to_world, mi, i)
        )
    }
}

impl Printable for PointLight {
    fn to_string(&self) -> String {
        format!(
            "Point Light: [\n
            \tP_light: ({}, {}, {})\n
            \tI: ({}, {}, {})\n
            ]",
            self.p_light.x,
            self.p_light.y,
            self.p_light.z,
            self.i[0],
            self.i[1],
            self.i[2],
        )
    }
}