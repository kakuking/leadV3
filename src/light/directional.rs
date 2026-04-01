use crate::{core::{Normal3, PI, Point2, Point3, Printable, Ray, Transform, Vector3, interaction::InteractionBase, light::{Light, LightFlags, LightT, VisibilityTester}, medium::MediumInterface, scene::Scene, spectrum::Spectrum}, loader::Parameters, registry::Manufacturable};

#[derive(Debug, Clone, PartialEq)]
pub struct DirectionalLight {
    flags: LightFlags,
    n_samples: u32,
    medium_interface: MediumInterface,
    light_to_world: Transform,
    world_to_light: Transform,

    l: Spectrum,
    w_light: Vector3,
    world_center: Point3,
    world_radius: f32
}

impl DirectionalLight {
    pub fn init(light_to_world: Transform, l: Spectrum, w_light: Vector3) -> Self {
        let mi = MediumInterface::new();
        let w_light = light_to_world.transform_vector(&w_light).normalize();

        Self {
            flags: LightFlags::DeltaDirection,
            n_samples: 1,
            medium_interface: mi,

            light_to_world,
            world_to_light: light_to_world.inverse(),

            l,
            w_light,
            world_center: Point3::origin(),
            world_radius: 0.0
        }
    }
}

impl LightT for DirectionalLight {
    fn get_flags(&self) -> LightFlags { self.flags }
    fn get_n_samples(&self) -> u32 { self.n_samples }
    fn get_medium_interface(&self) -> MediumInterface { self.medium_interface.clone() }
    fn get_light_to_world(&self) -> Transform { self.light_to_world }
    fn get_world_to_light(&self) -> Transform { self.world_to_light }

    fn preprocess(&mut self, scene: &Scene) {
        scene.get_world_bounds().bounding_sphere(&mut self.world_center, &mut self.world_radius);
    }

    fn sample_li(&self, re: &InteractionBase, _u: &Point2, wi: &mut Vector3, pdf: &mut f32, vis: &mut VisibilityTester) -> Spectrum {
        *wi = self.w_light.clone();
        *pdf = 1.0;

        let p_outside = re.p + self.w_light * 2.0 * self.world_radius;

        *vis = VisibilityTester::init(
            re, 
            &InteractionBase::init_no_wo(&p_outside, re.time, self.medium_interface.clone())
        );

        self.l
    }

    fn power(&self) -> Spectrum { self.l * PI * self.world_radius.powf(2.0) }

    fn le(&self, _r: &Ray) -> Spectrum { Spectrum::zeros() }

    fn pdf_li(&self, _re: &InteractionBase, _wi: &Vector3) -> f32 { 0.0 }
    
    fn sample_le(&self, _u1: &Point2, _u2: &Point2, _time: f32, _ray: &mut Ray, _n_light: &mut Normal3, _pdf_pos: &mut f32, _pdf_dir: &mut f32) -> Spectrum {
        todo!("directional::sample_le");
    }

    fn pdf_le(&self, _ray: &Ray, _n_light: &Normal3, _pdf_pos: &mut f32, _pdf_dir: &mut f32) {
        todo!("directional::pdf_le");
    }
}

impl Manufacturable<Light> for DirectionalLight {
    fn create_from_parameters(params: Parameters) -> Light {
        let light_to_world = params.get_transform();
        let l = params.get_vector3("l", Some(Spectrum::new(1.0, 1.0, 1.0)));
        let w_light = params.get_vector3("w_light", Some(Vector3::new(1.0, 1.0, 1.0)));

        Light::Directional(
            DirectionalLight::init(light_to_world, l, w_light)
        )
    }
}

impl Printable for DirectionalLight {
    fn to_string(&self) -> String {
        format!(
            "Directional Light: [\n
            \tw_light: ({}, {}, {})\n
            \tl: ({}, {}, {})\n
            ]",
            self.w_light.x,
            self.w_light.y,
            self.w_light.z,
            self.l[0],
            self.l[1],
            self.l[2],
        )
    }
}