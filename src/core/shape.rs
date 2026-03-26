use crate::{core::{Point2, Printable, Ray, Transform, Vector3, apply_transform_to_bounds, bounds::Bounds3, interaction::{InteractionBase, InteractionT}, transform_swaps_handedness}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable, shape::triangle_mesh::Triangle};

use crate::shape::{Sphere};

#[derive(Debug, Clone)]
pub enum Shape {
    Empty,
    Sphere(Sphere),
    Triangle(Triangle)
}

impl Shape {
    pub fn get_object_to_world(&self) -> &Transform {
        match self {
            Self::Sphere(s) => s.get_object_to_world(),
            Self::Triangle(s) => s.get_object_to_world(),
            Self::Empty => panic!("Calling get_object_to_world on empty shape"),
        }
    }

    pub fn get_world_to_object(&self) -> &Transform {
        match self {
            Self::Sphere(s) => s.get_world_to_object(),
            Self::Triangle(s) => s.get_world_to_object(),
            Self::Empty => panic!("Calling get_world_to_object on empty shape"),
        }
    }

    pub fn get_reverse_orientation(&self) -> bool {
        match self {
            Self::Sphere(s) => s.get_reverse_orientation(),
            Self::Triangle(s) => s.get_reverse_orientation(),
            Self::Empty => panic!("Calling get_reverse_orientation on empty shape"),
        }
    }

    pub fn get_transform_swaps_handedness(&self) -> bool {
        match self {
            Self::Sphere(s) => s.get_transform_swaps_handedness(),
            Self::Triangle(s) => s.get_transform_swaps_handedness(),
            Self::Empty => panic!("Calling get_transform_swaps_handedness on empty shape"),
        }
    }

    pub fn object_bounds(&self) -> Bounds3 {
        match self {
            Self::Sphere(s) => s.object_bounds(),
            Self::Triangle(s) => s.object_bounds(),
            Self::Empty => panic!("Calling object_bounds on empty shape"),
        }
    }

    pub fn world_bounds(&self) -> Bounds3 {
        apply_transform_to_bounds(&self.object_bounds(), self.get_object_to_world())
    }

    pub fn intersect(&self, ray: &Ray, t_hit: &mut f32, isect: &mut SurfaceInteraction, test_alpha_texture: Option<bool>) -> bool {
        match self {
            Self::Sphere(s) => s.intersect(ray, t_hit, isect, test_alpha_texture),
            Self::Triangle(s) => s.intersect(ray, t_hit, isect, test_alpha_texture),
            Self::Empty => panic!("Calling on empty shape"),
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
            Self::Triangle(s) => s.area(),
            Self::Empty => {0.0},
        }
    }

    pub fn pdf(&self) -> f32 { 1.0 / self.area() }

    pub fn sample(&self, u: &Point2, pdf: &mut f32) -> InteractionBase {
        match self {
            Self::Sphere(s) => s.sample(u, pdf),
            Self::Triangle(s) => s.sample(u, pdf),
            Self::Empty => panic!("Calling on empty shape"),
        }
    }

    pub fn sample_interaction(&self, re: &InteractionBase, u: &Point2, pdf: &mut f32) -> InteractionBase {
        match self {
            Self::Sphere(s) => s.sample_interaction(re, u, pdf),
            Self::Triangle(s) => s.sample_interaction(re, u, pdf),
            Self::Empty => panic!("Calling on empty shape"),
        }
    }

    pub fn pdf_interaction(&self, re: &InteractionBase, wi: &Vector3) -> f32 {
        match self {
            Self::Sphere(s) => s.pdf_interaction(re, wi),
            Self::Triangle(s) => s.pdf_interaction(re, wi),
            Self::Empty => panic!("Calling on empty shape"),
        }
    }
}

impl Printable for Shape {
    fn to_string(&self) -> String {
        match self {
            Shape::Sphere(s) => s.to_string(),
            Shape::Triangle(s) => s.to_string(),
            Self::Empty => "Empty Shape".to_string()
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
    fn pdf_interaction(&self, re: &InteractionBase, wi: &Vector3) -> f32 {
        let ray = re.spawn_ray(wi);
        let mut t_hit = 0.0;
        let mut isect_light = SurfaceInteraction::new();

        if !self.intersect(&ray, &mut t_hit, &mut isect_light, Some(false)) {
            return 0.0;
        }

        let pdf = (re.p - isect_light.get_p()).norm_squared() / (isect_light.get_n().dot(&(-wi)).abs() * self.area());

        pdf
    }

    fn sample(&self, u: &Point2, pdf: &mut f32) -> InteractionBase;
    fn sample_interaction(&self, _re: &InteractionBase, u: &Point2, pdf: &mut f32) -> InteractionBase {
        self.sample(u, pdf)
    }
}