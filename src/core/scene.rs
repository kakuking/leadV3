use std::sync::Arc;

use crate::{core::{Bounds3, Printable, Ray, camera::Camera, light::Light, medium::MediumInterface, primitive::{GeometricPrimitive, Primitive}, sampler::Sampler, shape::Shape, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, loader::{Manufacturable, Parameters}, sampler::stratified_sampler::StratifiedSampler, shape::bounding_volume_heirarchy::{BVHAccel, SplitMethod}};

pub struct Scene {
    pub lights: Vec<Arc<Light>>,
    aggregate: Arc<Primitive>,
    world_bound: Bounds3,
    shapes: Vec<Arc<Shape>>,
}

impl Scene {
    pub fn new() -> Self {
        Self {
            aggregate: Arc::new(Primitive::Empty),
            lights: Vec::new(),
            world_bound: Bounds3::new(),

            shapes: Vec::new(),
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

    pub fn add_shapes(&mut self, shapes: Vec<Shape>) {
        for shape in shapes {
            self.shapes.push(Arc::new(shape));
        }
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(Arc::new(light));
    }

    fn create_aggregate(&mut self) {
        let shapes = &self.shapes;
        let mut accel: BVHAccel = BVHAccel::init(32, SplitMethod::SAH);
        let mi = MediumInterface::new();

        for shape in shapes {
            let gp = GeometricPrimitive::init(
                shape.clone(), 
                None, 
                None, 
                mi.clone()
            );

            accel.add_primitive(
                Arc::new(
                    Primitive::Geometric(
                        Arc::new(
                            gp
                        )
                    )
                )
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
            \tNum Shapes: {}\n
            ]",
            self.lights.len(),
            self.shapes.len(),
        )
    }
}