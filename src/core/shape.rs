use crate::{core::{Bounds3, Point2, Printable, Ray, Transform, Vector3, apply_transform_to_bounds, interaction::{Interaction, InteractionT}, transform_swaps_handedness}, interaction::surface_interaction::SurfaceInteraction, loader::Manufacturable, shape::triangle_mesh::Triangle};

use crate::shape::{Sphere};

#[derive(Debug)]
pub enum Shape {
    Sphere(Sphere),
    Triangle(Triangle)
}

impl Shape {
    pub fn get_object_to_world(&self) -> &Transform {
        match self {
            Self::Sphere(s) => s.get_object_to_world(),
            Self::Triangle(s) => s.get_object_to_world(),
        }
    }

    pub fn get_world_to_object(&self) -> &Transform {
        match self {
            Self::Sphere(s) => s.get_world_to_object(),
            Self::Triangle(s) => s.get_world_to_object()
        }
    }

    pub fn get_reverse_orientation(&self) -> bool {
        match self {
            Self::Sphere(s) => s.get_reverse_orientation(),
            Self::Triangle(s) => s.get_reverse_orientation()
        }
    }

    pub fn get_transform_swaps_handedness(&self) -> bool {
        match self {
            Self::Sphere(s) => s.get_transform_swaps_handedness(),
            Self::Triangle(s) => s.get_transform_swaps_handedness()
        }
    }

    pub fn object_bounds(&self) -> Bounds3 {
        match self {
            Self::Sphere(s) => s.object_bounds(),
            Self::Triangle(s) => s.object_bounds()
        }
    }

    pub fn world_bounds(&self) -> Bounds3 {
        apply_transform_to_bounds(&self.object_bounds(), self.get_object_to_world())
    }

    pub fn intersect(&self, ray: &Ray, t_hit: &mut f32, isect: &mut SurfaceInteraction, test_alpha_texture: Option<bool>) -> bool {
        match self {
            Self::Sphere(s) => s.intersect(ray, t_hit, isect, test_alpha_texture),
            Self::Triangle(s) => s.intersect(ray, t_hit, isect, test_alpha_texture)
        }
    }

    pub fn intersect_p(&self, ray: &Ray, test_alpha_texture: Option<bool>) -> bool {
        let mut t_hit = ray.t_max.get();
        let mut isect = SurfaceInteraction::new();

        self.intersect(ray, &mut t_hit, &mut isect, test_alpha_texture)
    }

    pub fn area(&self) -> f32 {
        match self {
            Self::Sphere(s) => s.area(),
            Self::Triangle(s) => s.area()
        }
    }

    pub fn pdf(&self) -> f32 { 1.0 / self.area() }

    pub fn sample(&self, u: &Point2) -> Interaction {
        match self {
            Self::Sphere(s) => s.sample(u),
            Self::Triangle(s) => s.sample(u)
        }
    }

    pub fn sample_interaction(&self, _re: &Interaction, u: &Point2) -> Interaction {
        self.sample(u)
    }

    pub fn pdf_interaction(&self, re: &Interaction, wi: &Vector3) -> f32 {
        match self {
            Self::Sphere(s) => s.pdf_interaction(re, wi),
            Self::Triangle(s) => s.pdf_interaction(re, wi)
        }
    }
}

impl Printable for Shape {
    fn to_string(&self) -> String {
        match self {
            Shape::Sphere(s) => s.to_string(),
            Shape::Triangle(s) => s.to_string(),
        }
    }
}

pub trait ShapeT: Manufacturable<Shape> + Printable {
    fn get_object_to_world(&self) -> &Transform;
    fn get_world_to_object(&self) -> &Transform;
    fn get_reverse_orientation(&self) -> bool;
    fn get_transform_swaps_handedness(&self) -> bool {
        transform_swaps_handedness(self.get_object_to_world())
    }

    fn object_bounds(&self) -> Bounds3;
    fn world_bounds(&self) -> Bounds3 {
        apply_transform_to_bounds(&self.object_bounds(), self.get_object_to_world())
    }

    fn intersect(&self, ray: &Ray, t_hit: &mut f32, isect: &mut SurfaceInteraction, test_alpha_texture: Option<bool>) -> bool;
    fn intersect_p(&self, ray: &Ray, test_alpha_texture: Option<bool>) -> bool {
        let mut t_hit = ray.t_max.get();
        let mut isect = SurfaceInteraction::new();

        self.intersect(ray, &mut t_hit, &mut isect, test_alpha_texture)
    }

    fn area(&self) -> f32;
    fn pdf(&self) -> f32 { 1.0 / self.area() }

    fn sample(&self, u: &Point2) -> Interaction;
    fn sample_interaction(&self, _re: &Interaction, u: &Point2) -> Interaction {
        self.sample(u)
    }

    fn pdf_interaction(&self, re: &Interaction, wi: &Vector3) -> f32 {
        self.pdf()
    }
}