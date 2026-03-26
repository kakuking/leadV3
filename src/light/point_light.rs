use crate::{core::{Normal3, PI, Point2, Point3, Printable, Ray, Transform, Vector3, interaction::{Interaction, InteractionBase}, light::{Light, LightFlags, LightT, VisibilityTester}, medium::MediumInterface, spectrum::Spectrum}, loader::Parameters, registry::Manufacturable};

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

    fn sample_li(&self, re: &Interaction, u: &Point2, wi: &mut Vector3, pdf: &mut f32, vis: &mut VisibilityTester) -> Spectrum {
        *wi = (self.p_light - re.get_p()).normalize();
        *pdf = 1.0;

        *vis = VisibilityTester::init(
            re.get_base(), 
            &InteractionBase::init_no_wo(&self.p_light, *re.get_time(), self.medium_interface.clone())
        );

        self.i / (self.p_light - re.get_p()).norm_squared()
    }

    fn power(&self) -> Spectrum { 4.0 * PI * self.i }

    fn le(&self, _r: &Ray) -> Spectrum { Spectrum::zeros() }

    fn pdf_li(&self, _re: &Interaction, _wi: &Vector3) -> f32 { 0.0 }
    
    fn sample_le(&self, u1: &Point2, u2: &Point2, time: f32, ray: &mut Ray, n_light: &mut Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum {
        todo!("PointLight::Sample_Le")
    }

    fn pdf_le(&self, ray: &Ray, n_light: &Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) {
        todo!("PointLight::pdf_le");
    }
}

impl Manufacturable<Light> for PointLight {
    fn create_from_parameters(params: Parameters) -> Light {
        let light_to_world = params.get_transform();
        let i = params.get_vector3("i", Some(Spectrum::new(1.0, 0.0, 0.0)));

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