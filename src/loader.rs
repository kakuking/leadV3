use std::{collections::HashMap, sync::Arc};

use roxmltree::Document;

use crate::{core::{
    AngleAxis, Point2, Point3, Transform, Vector2, Vector3, camera::Camera, film::Film, filter::Filter, integrator::Integrator, lead_instance::Instance, light::Light, material::Material, primitive::Primitive, rotate_angle_axis, sampler::Sampler, scaling, scene::Scene, shape::Shape, translation
}, registry::{LeadObject, Registry}};

// #[derive(Clone)]
pub enum ParamVal {
    Float(f32),
    Int(i32),
    Str(String),
    Bool(bool),
    Vec2(Vector2),
    Pt2(Point2),
    Vec3(Vector3),
    Pt3(Point3),
    AngleAxis(AngleAxis),
    LeadObject(LeadObject)
}

impl ParamVal {
    pub fn new_int(key: &str, value: &str) -> Self {
        Self::Int(
            value.trim().parse::<i32>().expect(&format!("Invalid int for key '{}'", key))
        )
    }

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

    pub fn new_angle_axis(key: &str, value: &str) -> Self {
        let p = Self::parse_floats(key, value, 4);
        Self::AngleAxis(AngleAxis::new(p[0], p[1], p[2], p[3]))
    }

    pub fn new_object(obj: LeadObject) -> Self {
        Self::LeadObject(obj)
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
    transform: Transform
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            transform: Transform::identity()
        }
    }

    pub fn add_translation(&mut self, t_str: String) {
        // println!("Adding translate: {}", t_str);
        let t = match ParamVal::new_vector3("translate", &t_str) {
            ParamVal::Vec3(v) => v,
            _ => panic!("Translate requires vector 3 as input")
        };

        let transform = translation(t);
        self.update_transform(transform);
    }

    pub fn add_scaling(&mut self, t_str: String) {
        // println!("Adding scaling: {}", t_str);
        let t = match ParamVal::new_vector3("scale", &t_str) {
            ParamVal::Vec3(v) => v,
            _ => panic!("Scale requires vector 3 as input")
        };

        let transform = scaling(t);
        self.update_transform(transform);
    }

    pub fn add_rotation(&mut self, t_str: String) {
        // println!("Adding rotation: {}", t_str);
        let t = match ParamVal::new_angle_axis("rotate", &t_str) {
            ParamVal::AngleAxis(v) => v,
            _ => panic!("Rotation requires vector 4 as input")
        };

        let transform = rotate_angle_axis(t);
        self.update_transform(transform);
    }
    
    pub fn add_lead_object(&mut self, key: String, obj: LeadObject) {
        self.map.insert(key, ParamVal::LeadObject(obj));
    }

    pub fn get_lead_object(&mut self, key: &str) -> Option<LeadObject> {
        match self.map.remove(key) {
            Some(ParamVal::LeadObject(obj)) => Some(obj),
            _ => None,
        }
    }

    pub fn update_transform(&mut self, t: Transform) {
        self.transform = t * self.transform;
    }

    pub fn add_int(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_int(&key, &value);
        self.map.insert(key, parameter);
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

    pub fn add_angle_axis(&mut self, key: String, value: String) {
        let parameter = ParamVal::new_angle_axis(&key, &value);
        self.map.insert(key, parameter);
    }

    pub fn get_transform(&self) -> Transform {
        self.transform.clone()
    }

    pub fn get_int(&self, key: &str, default: Option<i32>) -> i32 {
        match self.map.get(key) {
            Some(ParamVal::Int(f)) => *f,
            _ => default.unwrap_or_default(),
        }
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
            Some(ParamVal::AngleAxis(r)) => *r,
            _ => default.unwrap_or_default(),
        }
    }
}

pub fn parse_xml(filename: &str, registry: &Registry) -> Option<Instance> {
    let xml = std::fs::read_to_string(filename).unwrap();
    let doc = Document::parse(&xml).unwrap();
    
    let mut instance = Instance::new();

    for node in doc.root_element().children().filter(|n| n.is_element()) {
        let tag = node.tag_name().name();
        let obj_type = node.attribute("type").unwrap_or("");

        let params = parse_parameters_for_node(node, registry);

        registry.add_to_instance(&mut instance, tag.to_string(), obj_type.to_string(), params);
    }

    Some(instance)
}

fn parse_parameters_for_node(node: roxmltree::Node, registry: &Registry) -> Parameters {
    let mut params = Parameters::new();

    for param in node.children().filter(|n| n.is_element()) {
        let p_tag = param.tag_name().name();
        let p_name = param.attribute("name").unwrap_or("");
        let p_value = param.attribute("value").unwrap_or("");
        let p_type = param.attribute("type").unwrap_or("");

        match p_tag {
            "int"       => params.add_int(p_name.to_string(), p_value.to_string()),
            "float"     => params.add_float(p_name.to_string(), p_value.to_string()),
            "bool"      => params.add_bool(p_name.to_string(), p_value.to_string()),
            "string"    => params.add_string(p_name.to_string(), p_value.to_string()),
            "vector2"   => params.add_vector2(p_name.to_string(), p_value.to_string()),
            "vector3"   => params.add_vector3(p_name.to_string(), p_value.to_string()),
            "point2"    => params.add_point2(p_name.to_string(), p_value.to_string()),
            "point3"    => params.add_point3(p_name.to_string(), p_value.to_string()),
            "angleAxis" => params.add_angle_axis(p_name.to_string(), p_value.to_string()),
            "scale"     => params.add_scaling(p_value.to_string()),
            "translate" => params.add_translation(p_value.to_string()),
            "rotate"    => params.add_rotation(p_value.to_string()),

            _ => {
                // Treat as nested lead object
                let child_params = parse_parameters_for_node(param, registry);
                let lead_object = registry.create_lead_object(
                    p_tag.to_string(),
                    p_type.to_string(),
                    child_params,
                );

                let key = if !p_name.is_empty() {
                    p_name.to_string()
                } else {
                    p_tag.to_string()
                };

                params.add_lead_object(key, lead_object);
            }
        }
    }

    params
}