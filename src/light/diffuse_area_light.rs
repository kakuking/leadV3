use std::sync::Arc;

use crate::{core::{Normal3, PI, Point2, Printable, Ray, Transform, Vector3, interaction::{InteractionBase}, light::{Light, LightFlags, LightT, VisibilityTester}, medium::MediumInterface, shape::Shape, spectrum::Spectrum}, registry::Manufacturable};

#[derive(Debug, Clone)]
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

    fn sample_le(&self, _u1: &Point2, _u2: &Point2, _time: f32, _ray: &mut Ray, _n_light: &mut Normal3, _pdf_pos: &mut f32, _pdf_dir: &mut f32) -> Spectrum {
        todo!("DAL::sample_le");
    }

    fn pdf_le(&self, _ray: &Ray, _n_light: &Normal3, _pdf_pos: &mut f32, _pdf_dir: &mut f32) {
        todo!("DAL::pdf_le");
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