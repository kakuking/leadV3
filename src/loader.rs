use std::{collections::HashMap};

use roxmltree::Document;

use crate::core::{
    AngleAxis, Point2, Point3, Vector2, Vector3, scene::Scene, shape::Shape
};

pub enum ParamVal {
    Float(f32),
    Str(String),
    Bool(bool),
    Vec2(Vector2),
    Pt2(Point2),
    Vec3(Vector3),
    Pt3(Point3),
    Rotation(AngleAxis),
}

impl ParamVal {
    pub fn new_float(key: &str, value: &str) -> Self {
        Self::Float(
            value.trim().parse::<f32>().expect(&format!("Invalid float for key '{}'", key))
        )
    }

    pub fn new_str(_key: &str, value: &str) -> Self {
        Self::Str(value.trim().to_string())
    }

    pub fn new_bool(key: &str, value: &str) -> Self {
        match value.trim() {
            "true" | "True" | "1"  => Self::Bool(true),
            "false" | "False" | "0" => Self::Bool(false),
            _ => panic!("Invalid bool for key '{}': {}", key, value),
        }
    }

    pub fn new_vector2(key: &str, value: &str) -> Self {
        let p = Self::parse_floats(key, value, 2);
        Self::Vec2(Vector2::new(p[0], p[1]))
    }

    pub fn new_point2(key: &str, value: &str) -> Self {
        let p = Self::parse_floats(key, value, 2);
        Self::Pt2(Point2::new(p[0], p[1]))
    }

    pub fn new_vector3(key: &str, value: &str) -> Self {
        let p = Self::parse_floats(key, value, 3);
        Self::Vec3(Vector3::new(p[0], p[1], p[2]))
    }

    pub fn new_point3(key: &str, value: &str) -> Self {
        let p = Self::parse_floats(key, value, 3);
        Self::Pt3(Point3::new(p[0], p[1], p[2]))
    }

    pub fn new_rotation(key: &str, value: &str) -> Self {
        let p = Self::parse_floats(key, value, 4);
        Self::Rotation(AngleAxis::new(p[0], p[1], p[2], p[3]))
    }

    fn parse_floats(key: &str, value: &str, expected: usize) -> Vec<f32> {
        let parts: Vec<f32> = value
            .split(',')
            .map(|s| {
                s.trim()
                    .parse::<f32>()
                    .expect(&format!("Invalid float in '{}' for key '{}'", value, key))
            })
            .collect();

        assert_eq!(
            parts.len(),
            expected,
            "Expected {} floats for key '{}', got {}",
            expected,
            key,
            parts.len()
        );

        parts
    }
}

pub struct Parameters {
    map: HashMap<String, ParamVal>,
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_float(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_float(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_string(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_str(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_bool(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_bool(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_vector2(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_vector2(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_point2(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_point2(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_vector3(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_vector3(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_point3(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_point3(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn add_rotation(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_rotation(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn get_float(&self, key: &str, default: Option<f32>) -> f32 {
        match self.map.get(key) {
            Some(ParamVal::Float(f)) => *f,
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_string(&self, key: &str, default: Option<String>) -> String {
        match self.map.get(key) {
            Some(ParamVal::Str(s)) => s.clone(),
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_bool(&self, key: &str, default: Option<bool>) -> bool {
        match self.map.get(key) {
            Some(ParamVal::Bool(b)) => *b,
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_vector2(&self, key: &str, default: Option<Vector2>) -> Vector2 {
        match self.map.get(key) {
            Some(ParamVal::Vec2(v)) => *v,
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_point2(&self, key: &str, default: Option<Point2>) -> Point2 {
        match self.map.get(key) {
            Some(ParamVal::Pt2(p)) => *p,
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_vector3(&self, key: &str, default: Option<Vector3>) -> Vector3 {
        match self.map.get(key) {
            Some(ParamVal::Vec3(v)) => *v,
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_point3(&self, key: &str, default: Option<Point3>) -> Point3 {
        match self.map.get(key) {
            Some(ParamVal::Pt3(p)) => *p,
            _ => default.unwrap_or_default(),
        }
    }

    pub fn get_rotation(&self, key: &str, default: Option<AngleAxis>) -> AngleAxis {
        match self.map.get(key) {
            Some(ParamVal::Rotation(r)) => *r,
            _ => default.unwrap_or_default(),
        }
    }
}

pub fn parse_xml(filename: &str, registry: &Registry) -> Option<Scene> {
    let xml = std::fs::read_to_string(filename).unwrap();
    let doc = Document::parse(&xml).unwrap();

    let mut scene = Scene::new();

    for node in doc.root_element().children().filter(|n| n.is_element()) {
        let tag = node.tag_name().name();
        let obj_type = node.attribute("type").unwrap_or("");

        let mut params = Parameters::new();

        for param in node.children().filter(|n| n.is_element()) {
            let p_tag = param.tag_name().name();
            let p_name = param.attribute("name").unwrap_or("");
            let p_value = param.attribute("value").unwrap_or("");

            match p_tag {
                "float"    => params.add_float(p_name.to_string(), p_value.to_string()),
                "bool"     => params.add_bool(p_name.to_string(), p_value.to_string()),
                "string"   => params.add_string(p_name.to_string(), p_value.to_string()),
                "vector2"  => params.add_vector2(p_name.to_string(), p_value.to_string()),
                "vector3"  => params.add_vector3(p_name.to_string(), p_value.to_string()),
                "point2"   => params.add_point2(p_name.to_string(), p_value.to_string()),
                "point3"   => params.add_point3(p_name.to_string(), p_value.to_string()),
                "rotation" => params.add_rotation(p_name.to_string(), p_value.to_string()),
                _ => {
                    eprintln!("Unknown parameter found: {}", p_tag);
                    continue;
                }
            }
        }

        registry.add_to_scene(&mut scene, tag.to_string(), obj_type.to_string(), params);
    }

    Some(scene)
}

pub trait Manufacturable {
    fn create_from_parameters(param: Parameters) -> Self;
}

pub type FactoryFn<T> = Box<dyn Fn(Parameters) -> T>;

pub struct Registry {
    pub shape_factories: HashMap<String, FactoryFn<Shape>>,
}

// For everything possible in teh registery, add a register_x, create_x, and add a branch for it in add_to_scene
impl Registry {
    pub fn new() -> Self {
        Self {
            shape_factories: HashMap::new()
        }
    }

    pub fn register_shape(&mut self, t: String, function: FactoryFn<Shape>) {
        self.shape_factories.insert(t, function);
    }

    fn create_shape(&self, t: String, parameters: Parameters) -> Shape {
        match self.shape_factories.get(&t) {
            Some(s) => s(parameters),
            _ => panic!("NO SHAPE FOUND OF TYPE {}", t),
        }
    }

    pub fn add_to_scene(
        &self,
        scene: &mut Scene,
        object: String,
        object_type: String,
        parameters: Parameters,
    ) {
        match object.as_str() {
            "shape" => scene.add_shape(self.create_shape(object_type.to_string(), parameters)),
            _ => eprintln!("No object found with name {}", object),
        }
    }
}