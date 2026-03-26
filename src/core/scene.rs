use std::sync::Arc;

use crate::{core::{bounds::Bounds3, Printable, Ray, light::Light, primitive::{Primitive}, sampler::Sampler, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, shape::bounding_volume_heirarchy::{BVHAccel, SplitMethod}};

pub struct Scene {
    pub lights: Vec<Arc<Light>>,
    aggregate: Arc<Primitive>,
    world_bound: Bounds3,
    primitives: Vec<Arc<Primitive>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            aggregate: Arc::new(Primitive::Empty),
            lights: Vec::new(),
            world_bound: Bounds3::new(),

            primitives: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.create_aggregate();
        self.world_bound = self.aggregate.world_bounds();

        let lights = self.lights.clone();
        for light in &lights {
            light.preprocess(self);
        }
        self.lights = lights;
    }

    pub fn add_primitives(&mut self, primitives: Vec<Primitive>) {
        for prim in primitives {
            if let Some(area) = prim.get_area_light() {
                self.lights.push(area.clone());
            }
            self.primitives.push(Arc::new(prim));
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(Arc::new(light));
    }

    fn create_aggregate(&mut self) {
        let primitives = &self.primitives;
        let mut accel: BVHAccel = BVHAccel::init(32, SplitMethod::SAH);

        for primitive in primitives {
            accel.add_primitive(
                primitive.clone()
            );
        }

        accel.build();

        self.aggregate = Arc::new(
            Primitive::BVH(
                Arc::new(accel)
            )
        );
    }

    pub fn get_world_bounds(&self) -> Bounds3 { self.world_bound.clone() }

    pub fn intersect(&self, ray: &Ray, si: &mut SurfaceInteraction) -> bool {
        self.aggregate.intersect(ray, si)    
    }
    pub fn intersect_p(&self, ray: &Ray) -> bool {
        self.aggregate.intersect_p(ray)
    }
    pub fn intersect_tr(&self, _ray: &Ray, _sampler: &Sampler, _si: &mut SurfaceInteraction, _transmittance: &mut Spectrum) -> bool {
        panic!("Scene::Intersect_Tr")
    }
}

impl Printable for Scene {
    fn to_string(&self) -> String {
        format!(
            "Scene: [\n
            \tNum lights: {}\n
            \tNum Primitives: {}\n
            ]",
            self.lights.len(),
            self.primitives.len(),
        )
    }
}