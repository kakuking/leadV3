use std::{fmt::Debug, sync::Arc};

use crate::{core::{Printable, Ray, bounds::Bounds3, interaction::{InteractionT, TransportMode}, light::Light, material::Material, medium::MediumInterface, shape::Shape}, interaction::surface_interaction::SurfaceInteraction, registry::LeadObject, shape::bounding_volume_heirarchy::BVHAccel};

#[derive(Debug, Clone)]
pub enum Primitive {
    Empty,
    Geometric(Arc<GeometricPrimitive>),
    BVH(Arc<BVHAccel>)
}

impl Primitive {
    pub fn world_bounds(&self) -> Bounds3 {
        match self {
            Self::Geometric(g) => { g.world_bounds() }
            Self::BVH(b) => { b.world_bounds() }
            Self::Empty => panic!("world bounds called on empty primitive")
        }
    }
    pub fn intersect(&self, r: &Ray, isect: &mut SurfaceInteraction) -> bool {
        match self {
            Self::Empty => panic!("intersect called on empty primitive"),
            Self::Geometric(g) => {
                match g.intersect(r, isect) {
                    true => {
                        isect.primitive = Primitive::Geometric(g.clone());
                        isect.shape = Some(g.shape.clone());
                        true
                    }
                    false => false
                }
            
            }
            Self::BVH(b) => { b.intersect(r, isect) }
        }
    }
    pub fn intersect_p(&self, ray: &Ray) -> bool {
        match self {
            Self::Empty => panic!("intersect_p called on empty primitive"),
            Self::Geometric(g) => { g.intersect_p(ray) }
            Self::BVH(b) => { b.intersect_p(ray) }
        }
    }

    pub fn get_shape(&self) -> Arc<Shape> {
        match self {
            Self::Empty => panic!("get_shape called on empty primitive"),
            Self::Geometric(g) => g.shape.clone(),
            Self::BVH(_) => panic!("get_shape on BVH")
        }
    }

    pub fn get_area_light(&self) -> Option<Arc<Light>> {
        match self {
            Self::Empty => panic!("get_area_light called on empty primitive"),
            Self::Geometric(g) => { g.get_area_light().clone() }
            Self::BVH(b) => { b.get_area_light() }
        }
    }
    pub fn get_material(&self) -> Option<Arc<Material>> {
        match self {
            Self::Empty => panic!("get_material called on empty primitive"),
            Self::Geometric(g) => { g.get_material().clone() }
            Self::BVH(b) => { b.get_material() }
        }
    }
    pub fn compute_scattering_function(&self, isect: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_nodes: bool) {
        match self {
            Self::Empty => panic!("compute_scattering_function called on empty primitive"),
            Self::Geometric(g) => { g.compute_scattering_function(isect, mode, allow_multiple_nodes); }
            Self::BVH(b) => { b.compute_scattering_function(isect, mode, allow_multiple_nodes); }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Empty => panic!("to_string called on empty primitive"),
            Self::Geometric(g) => {
                g.to_string()
            }
            Self::BVH(b) => {
                b.to_string()
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeometricPrimitive {
    shape: Arc<Shape>,
    material: Option<Arc<Material>>,
    area_light: Option<Arc<Light>>,
    medium_interface: MediumInterface
}

impl GeometricPrimitive {
    pub fn init(shape: Arc<Shape>, material: Option<Arc<Material>>, area_light: Option<Arc<Light>>, medium_interface: MediumInterface) -> Self {
        Self {
            shape,
            material,
            area_light,
            medium_interface
        }
    }

    pub fn get_shape(&self) -> &Arc<Shape> { &self.shape }
    pub fn get_material(&self) -> &Option<Arc<Material>> { &self.material }
    pub fn get_area_light(&self) -> &Option<Arc<Light>> { &self.area_light }
    pub fn get_medium_interface(&self) -> &MediumInterface { &self.medium_interface }
    
    pub fn intersect(&self, ray: &Ray, isect: &mut SurfaceInteraction) -> bool {
        let mut t_hit = 0.0;
        if !self.get_shape().intersect(ray, &mut t_hit, isect, None) {
            return false
        }
    
        ray.t_max.set(t_hit);
        true
    }
    
    pub fn intersect_p(&self, r: &Ray) -> bool {
        let mut isect = SurfaceInteraction::new();
        self.intersect(r, &mut isect)
    }

    pub fn world_bounds(&self) -> Bounds3 { self.shape.world_bounds() }
    pub fn compute_scattering_function(&self, isect: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_lobes: bool) {
        match &self.material {
            Some(m) => m.compute_scattering_funcitons(isect, mode, allow_multiple_lobes),
            None => panic!("No Material Found"),
        }
    }

    pub fn create_from_parameters(param: crate::loader::Parameters) -> Vec<Primitive> {
        let mut param = param;

        let shapes = match param.get_lead_object("shape") {
            Some(LeadObject::Shape(s)) => s,
            _ => panic!("Primitive requires a shape")
        };

        let mat = match param.get_lead_object("material") {
            Some(LeadObject::Material(m)) => Some(Arc::new(m)),
            _ => None
        };

        let mut primitives: Vec<Primitive> = Vec::new();

        let light = match param.get_lead_object("light") {
            Some(LeadObject::Light(light)) => Some(light),
            _ => None
        };
        
        for shape in shapes {
            let shape_arc = Arc::new(shape);
            let cur_light = match light.clone() {
                Some(mut l) => {
                    l.add_shape(shape_arc.clone()); 
                    Some(Arc::new(l))},
                None => None
            };

            let gp = Self {
                shape: shape_arc,
                material: mat.clone(),
                area_light: cur_light,
                medium_interface: MediumInterface::new()
            };

            primitives.push(
                Primitive::Geometric(Arc::new(gp))
            );
        }

        primitives
    }
}

    

impl Printable for GeometricPrimitive {
    fn to_string(&self) -> String {
        format!(
            "Geometric Primitive: [\n
            \tShape: {}\n
            \tMaterial: {:?}\n
            \tArea Light: {:?}\n
            \tMedium Interface: {:?}\n
            ]",
            self.shape.to_string(),
            self.material,
            self.area_light,
            self.medium_interface
        )
    }
}