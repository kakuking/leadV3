use std::sync::Arc;

use crate::{core::{Normal3, PI, Point2, Printable, Ray, Transform, Vector3, coordinate_system, interaction::InteractionBase, light::{Light, LightFlags, LightT, VisibilityTester}, medium::MediumInterface, random::{cosine_sample_hemisphere, cosine_sample_hemisphere_pdf}, shape::Shape, spectrum::Spectrum}, registry::Manufacturable};

#[derive(Debug, Clone, PartialEq)]
pub struct DiffuseAreaLight {
    flags: LightFlags,
    n_samples: u32,
    medium_interface: MediumInterface,
    light_to_world: Transform,
    world_to_light: Transform,

    l_emit: Spectrum,
    shape: Arc<Shape>,
    area: f32
}

impl DiffuseAreaLight {
    pub fn init(light_to_world: Transform, medium_interface: &MediumInterface, l_emit: Spectrum, n_samples: u32, shape: Arc<Shape>) -> Self {
        let area = shape.area();
        
        Self {
            flags: LightFlags::Area,
            n_samples,
            medium_interface: medium_interface.clone(),
            light_to_world,
            world_to_light: light_to_world.inverse(),

            l_emit,
            area,
            shape,
        }
    }

    pub fn add_shape(&mut self, shape: Arc<Shape>) {
        self.shape = shape;
        self.area = self.shape.area();
    }
}

impl LightT for DiffuseAreaLight {
    fn get_flags(&self) -> LightFlags { self.flags }
    fn get_n_samples(&self) -> u32 { self.n_samples }
    fn get_medium_interface(&self) -> MediumInterface { self.medium_interface.clone() }
    fn get_light_to_world(&self) -> Transform { self.light_to_world }
    fn get_world_to_light(&self) -> Transform { self.world_to_light }

    fn sample_li(&self, re: &InteractionBase, u: &Point2, wi: &mut Vector3, pdf: &mut f32, vis: &mut VisibilityTester) -> Spectrum {
        let mut p_shape = self.shape.sample_interaction(re, u, pdf);
        p_shape.medium_interface = self.medium_interface.clone();
        
        *wi = (p_shape.p - re.p).normalize();
        *vis = VisibilityTester::init(re, &p_shape);

        self.l(&p_shape, &-*wi)
    }

    fn power(&self) -> Spectrum {
        self.l_emit * self.area * PI
    }

    fn le(&self, _r: &Ray) -> Spectrum {
        Spectrum::zeros()
    }

    fn pdf_li(&self, re: &InteractionBase, wi: &Vector3) -> f32{
        self.shape.pdf_interaction(re, wi)
    }

    fn sample_le(&self, u1: &Point2, u2: &Point2, _time: f32, ray: &mut Ray, n_light: &mut Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum {
        let mut p_shape = self.shape.sample(u1, pdf_pos);
        p_shape.medium_interface = self.medium_interface.clone();
        *n_light = p_shape.n;

        let mut w = cosine_sample_hemisphere(*u2);
        *pdf_dir = cosine_sample_hemisphere_pdf(w.z);
        let mut v1 = Vector3::zeros();
        let mut v2 = Vector3::zeros();
        coordinate_system(&p_shape.n, &mut v1, &mut v2);
        
        w = w.x * v1 + w.y * v2 + w.z * p_shape.n;

        *ray = p_shape.spawn_ray(&w);

        self.l(&p_shape, &w)
    }

    fn pdf_le(&self, ray: &Ray, n_light: &Normal3, pdf_pos: &mut f32, pdf_dir: &mut f32) {
        // let its = InteractionBase::init(&ray.o, n_light, &Vector3::zeros(), n_light, ray.time, self.medium_interface.clone());

        *pdf_pos = self.shape.pdf();
        *pdf_dir = cosine_sample_hemisphere_pdf(n_light.dot(&ray.d));
    }

    fn l(&self, its: &InteractionBase, w: &Vector3) -> Spectrum {
        if its.n.dot(w) > 0.0 {
            self.l_emit
        } else {
            Spectrum::zeros()
        }
    }
}

impl Manufacturable<Light> for DiffuseAreaLight {
    fn create_from_parameters(param: crate::loader::Parameters) -> Light {
        let light_to_world = param.get_transform();

        let l_emit: Vector3 = param.get_vector3("l_emit", Some(Vector3::x()));

        let n_samples = param.get_float("n_samples", Some(1.0));
        let medium_interface = MediumInterface::new();

        Light::DiffuseArea(
            Self::init(
                light_to_world, 
                &medium_interface, 
                l_emit, 
                n_samples as u32, 
                Arc::new(Shape::Empty)
            )
        )
    }
}

impl Printable for DiffuseAreaLight {
    fn to_string(&self) -> String {
        format!(
            "Diffuse Area Light: [\n
            \tL emit: {}, {}, {}\n
            \tarea: {}\n
            ]",
            self.l_emit.x,
            self.l_emit.y,
            self.l_emit.z,
            self.area
        )
    }
}