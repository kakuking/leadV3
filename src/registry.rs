use std::collections::HashMap;

use crate::{core::{camera::Camera, film::Film, filter::Filter, integrator::Integrator, lead_instance::Instance, light::Light, material::Material, primitive::Primitive, sampler::Sampler, shape::Shape}, loader::Parameters};


// #[derive(Clone)]
pub enum LeadObject {
    Camera(Camera),
    Shape(Vec<Shape>),
    Primitive(Vec<Primitive>),
    Sampler(Sampler),
    Filter(Filter),
    Film(Film),
    Light(Light),
    Integrator(Integrator),
    Material(Material)
}

pub trait Manufacturable<T> {
    fn create_from_parameters(param: Parameters) -> T;
}

pub type FactoryFn<T> = Box<dyn Fn(Parameters) -> T>;
pub type MultiFactoryFn<T> = Box<dyn Fn(Parameters) -> Vec<T>>;

pub struct Registry {
    pub shape_factories: HashMap<String, MultiFactoryFn<Shape>>,
    pub primitive_factories: HashMap<String, MultiFactoryFn<Primitive>>,
    pub camera_factories: HashMap<String, FactoryFn<Camera>>,
    pub sampler_factories: HashMap<String, FactoryFn<Sampler>>,
    pub filter_factories: HashMap<String, FactoryFn<Filter>>,
    pub film_factories: HashMap<String, FactoryFn<Film>>,
    pub light_factories: HashMap<String, FactoryFn<Light>>,
    pub integrator_factories: HashMap<String, FactoryFn<Integrator>>,
    pub material_factories: HashMap<String, FactoryFn<Material>>,
}

// For everything possible in teh registery, add a register_x, create_x, and add a branch for it in add_to_scene
impl Registry {
    pub fn new() -> Self {
        Self {
            shape_factories: HashMap::new(),
            primitive_factories: HashMap::new(),
            camera_factories: HashMap::new(),
            sampler_factories: HashMap::new(),
            filter_factories: HashMap::new(),
            film_factories: HashMap::new(),
            light_factories: HashMap::new(),
            integrator_factories: HashMap::new(),
            material_factories: HashMap::new()
        }
    }

    pub fn register_shape(&mut self, t: String, function: MultiFactoryFn<Shape>) {
        self.shape_factories.insert(t, function);
    }

    fn create_shape(&self, t: String, parameters: Parameters) -> Vec<Shape> {
        match self.shape_factories.get(&t) {
            Some(s) => s(parameters),
            _ => panic!("NO SHAPE FOUND OF TYPE {}", t),
        }
    }

    pub fn register_primitive(&mut self, t: String, function: MultiFactoryFn<Primitive>) {
        self.primitive_factories.insert(t, function);
    }

    fn create_primitive(&self, t: String, parameters: Parameters) -> Vec<Primitive> {
        match self.primitive_factories.get(&t) {
            Some(s) => s(parameters),
            _ => panic!("NO PRIMITIVE FOUND OF TYPE {}", t),
        }
    }

    pub fn register_camera(&mut self, t: String, function: FactoryFn<Camera>) {
        self.camera_factories.insert(t, function);
    }

    fn create_camera(&self, t: String, parameters: Parameters) -> Camera {
        match self.camera_factories.get(&t) {
            Some(s) => s(parameters),
            _ => panic!("NO SHAPE FOUND OF TYPE {}", t),
        }
    }

    pub fn register_sampler(&mut self, t: String, function: FactoryFn<Sampler>) {
        self.sampler_factories.insert(t, function);
    }

    fn create_sampler(&self, t: String, parameters: Parameters) -> Sampler {
        match self.sampler_factories.get(&t) {
            Some(s) => s(parameters),
            _ => panic!("NO SHAPE FOUND OF TYPE {}", t),
        }
    }

    pub fn register_filter(&mut self, t: String, function: FactoryFn<Filter>) {
        self.filter_factories.insert(t, function);
    }

    fn create_filter(&self, t: String, parameters: Parameters) -> Filter {
        match self.filter_factories.get(&t) {
            Some(f) => f(parameters),
            _ => panic!("NO FILTER FOUND OF TYPE {}", t),
        }
    }

    pub fn register_film(&mut self, t: String, function: FactoryFn<Film>) {
        self.film_factories.insert(t, function);
    }

    fn create_film(&self, t: String, parameters: Parameters) -> Film {
        match self.film_factories.get(&t) {
            Some(f) => f(parameters),
            _ => panic!("NO FILM FOUND OF TYPE {}", t),
        }
    }

    pub fn register_light(&mut self, t: String, function: FactoryFn<Light>) {
        self.light_factories.insert(t, function);
    }

    fn create_light(&self, t: String, parameters: Parameters) -> Light {
        match self.light_factories.get(&t) {
            Some(f) => f(parameters),
            _ => panic!("NO LIGHT FOUND OF TYPE {}", t),
        }
    }

    pub fn register_integrator(&mut self, t: String, function: FactoryFn<Integrator>) {
        self.integrator_factories.insert(t, function);
    }

    fn create_integrator(&self, t: String, parameters: Parameters) -> Integrator {
        match self.integrator_factories.get(&t) {
            Some(f) => f(parameters),
            _ => panic!("NO INTEGRATOR FOUND OF TYPE {}", t),
        }
    }

    pub fn register_material(&mut self, t: String, function: FactoryFn<Material>) {
        self.material_factories.insert(t, function);
    }

    fn create_material(&self, t: String, parameters: Parameters) -> Material {
        match self.material_factories.get(&t) {
            Some(f) => f(parameters),
            _ => panic!("NO MATERIAL FOUND OF TYPE {}", t),
        }
    }

    pub fn create_lead_object(
        &self,
        object: String,
        object_type: String,
        parameters: Parameters,
    ) -> LeadObject {
        match object.as_str() {
            "shape" => LeadObject::Shape(self.create_shape(object_type, parameters)),
            "primitive" => LeadObject::Primitive(self.create_primitive(object_type, parameters)),
            "camera" => LeadObject::Camera(self.create_camera(object_type, parameters)),
            "sampler" => LeadObject::Sampler(self.create_sampler(object_type, parameters)),
            "filter" => LeadObject::Filter(self.create_filter(object_type, parameters)),
            "film" => LeadObject::Film(self.create_film(object_type, parameters)),
            "light" => LeadObject::Light(self.create_light(object_type, parameters)),
            "integrator" => LeadObject::Integrator(self.create_integrator(object_type, parameters)),
            "material" => LeadObject::Material(self.create_material(object_type, parameters)),
            _ => panic!("No lead object found with name {}", object),
        }
    }

    pub fn add_to_instance(
        &self,
        instance: &mut Instance,
        object: String,
        object_type: String,
        parameters: Parameters,
    ) {
        match object.as_str() {
            "camera" => instance.set_camera(self.create_camera(object_type.to_string(), parameters)),
            "sampler" => instance.set_sampler(self.create_sampler(object_type.to_string(), parameters)),
            "light" => instance.scene.add_light(self.create_light(object_type, parameters)),
            "integrator" => instance.set_integrator(self.create_integrator(object_type, parameters)),
            "primitive" => instance.scene.add_primitives(self.create_primitive(object_type.to_string(), parameters)),
            _ => eprintln!("{} should not be added directly to instance", object),
        }
    }
}