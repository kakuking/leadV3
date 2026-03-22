use std::{fmt::Debug, sync::Arc};

use crate::{core::{Bounds3, Printable, Ray, interaction::{InteractionT, TransportMode}, material::{self, Material}, medium::MediumInterface, shape::Shape}, interaction::surface_interaction::SurfaceInteraction, light::area_light::AreaLight, shape::bounding_volume_heirarchy::BVHAccel};

#[derive(Debug, Clone)]
pub enum Primitive {
    Geometric(Arc<GeometricPrimitive>),
    BVH(Arc<BVHAccel>)
}

impl Primitive {
    pub fn world_bounds(&self) -> Bounds3 {
        match self {
            Self::Geometric(g) => { g.world_bounds() }
            Self::BVH(b) => { b.world_bounds() }
        }
    }
    pub fn intersect(&self, r: &Ray, isect: &mut SurfaceInteraction) -> bool {
        match self {
            Self::Geometric(g) => {
                match g.intersect(r, isect) {
                    true => {
                        isect.primitive = Some(Primitive::Geometric(g.clone()));
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
            Self::Geometric(g) => { g.intersect_p(ray) }
            Self::BVH(b) => { b.intersect_p(ray) }
        }
    }
    pub fn get_area_light(&self) -> Option<Arc<AreaLight>> {
        match self {
            Self::Geometric(g) => { g.get_area_light().clone() }
            Self::BVH(b) => { b.get_area_light() }
        }
    }
    pub fn get_material(&self) -> Option<Arc<Material>> {
        match self {
            Self::Geometric(g) => { g.get_material().clone() }
            Self::BVH(b) => { b.get_material() }
        }
    }
    pub fn compute_scattering_function(&self, isect: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_nodes: bool) {
        match self {
            Self::Geometric(g) => { g.compute_scattering_function(isect, mode, allow_multiple_nodes); }
            Self::BVH(b) => { b.compute_scattering_function(isect, mode, allow_multiple_nodes); }
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Self::Geometric(g) => {
                g.shape.to_string()
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
    area_light: Option<Arc<AreaLight>>,
    medium_interface: MediumInterface
}

impl GeometricPrimitive {
    pub fn init(shape: Arc<Shape>, material: Option<Arc<Material>>, area_light: Option<Arc<AreaLight>>, medium_interface: MediumInterface) -> Self {
        Self {
            shape,
            material,
            area_light,
            medium_interface
        }
    }

    pub fn get_shape(&self) -> &Arc<Shape> { &self.shape }
    pub fn get_material(&self) -> &Option<Arc<Material>> { &self.material }
    pub fn get_area_light(&self) -> &Option<Arc<AreaLight>> { &self.area_light }
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
    pub fn compute_scattering_function(&self, isect: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_nodes: bool) {
        todo!("GeoPrim::comptute_scattering");
    }
}
