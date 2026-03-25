use crate::{core::{Bounds3, EPSILON, PI, Point2, Point3, Printable, Ray, Transform, Vector3, apply_transform_to_normal, apply_transform_to_ray, gamma, interaction::{Interaction, InteractionBase}, quadratic, random::uniform_sample_sphere, shape::{Shape, ShapeT}, translation}, interaction::surface_interaction::SurfaceInteraction, loader::Manufacturable};

#[derive(Debug, Clone)]
pub struct Sphere {
    object_to_world: Transform,
    world_to_object: Transform,
    reverse_orientation: bool,

    radius: f32,
    z_min: f32,
    z_max: f32,
    theta_min: f32,
    theta_max: f32,
    phi_max: f32
}

impl Sphere {
    pub fn init(object_to_world: Transform, world_to_object: Transform, reverse_orientation: bool, radius: f32, z_min: f32, z_max: f32, phi_max: f32) -> Self {
        let z_min = z_min.min(z_max).clamp(-radius, radius);
        let z_max = z_min.max(z_max).clamp(-radius, radius);

        let theta_min = (z_min / radius).clamp(-1.0, 1.0).acos();
        let theta_max = (z_max / radius).clamp(-1.0, 1.0).acos();
        let phi_max = phi_max.clamp(0.0, 360.0).to_radians();

        Self {
            object_to_world,
            world_to_object,
            reverse_orientation,

            radius,
            z_min,
            z_max,
            theta_min,
            theta_max,
            phi_max
        }
    }
}

impl ShapeT for Sphere {
    fn get_object_to_world(&self) -> &Transform { &self.object_to_world }
    fn get_world_to_object(&self) -> &Transform { &self.world_to_object }
    fn get_reverse_orientation(&self) -> bool { self.reverse_orientation }

    fn object_bounds(&self) -> Bounds3 {
        let p_min = Point3::new(-self.radius, -self.radius, self.z_min);
        let p_max = Point3::new(self.radius, self.radius, self.z_max);

        Bounds3::init_two(&p_min, &p_max)
    }

    // fn intersect(&self, ray: &Ray, t_hit: &mut f32, isect: &mut SurfaceInteraction, test_alpha_texture: Option<bool>) -> bool 

    fn intersect(&self, ray: &Ray, t_hit: &mut f32, isect: &mut SurfaceInteraction, _test_alpha_texture: Option<bool>) -> bool {
        let mut phi: f32;
        let mut p_hit: Point3;

        let ray = apply_transform_to_ray(ray, &self.world_to_object);

        let a = ray.d.x * ray.d.x + ray.d.y * ray.d.y + ray.d.z * ray.d.z;
        let b = 2.0 * (ray.d.x * ray.o.x + ray.d.y * ray.o.y + ray.d.z * ray.o.z);
        let c = ray.o.x * ray.o.x + ray.o.y * ray.o.y + ray.o.z * ray.o.z - self.radius * self.radius;

        let mut t0: f32 = 0.0;
        let mut t1: f32 = 0.0;
        if !quadratic(a, b, c, &mut t0, &mut t1) {
            return false;
        }

        // Degen case
        if t0 > ray.t_max.get() || t1 <= 0.0 {
            return false;
        }

        let mut t_shape_hit = t0;
        if t_shape_hit <= 0.0 {
            t_shape_hit = t1;
            if t_shape_hit > ray.t_max.get() {
                return false;
            }
        }

        p_hit = ray.at(t_shape_hit);
        p_hit *= self.radius / (p_hit - Point3::new(0.0, 0.0, 0.0)).norm();

        if p_hit.x == 0.0 && p_hit.y == 0.0 { p_hit.x = EPSILON * self.radius; }// if x and u are 0 0, then shift x a bit

        phi = p_hit.y.atan2(p_hit.x);
        if phi < 0.0 { phi += 2.0 * PI; }

        // Check against zminmax and phimax
        if (self.z_min > -self.radius && p_hit.z < self.z_min) || (self.z_max < self.radius && p_hit.z > self.z_max) || phi > self.phi_max {
            if t_shape_hit == t1 { return false; }  // If its the second hit, return false
            if t1 > ray.t_max.get() { return false; }
            t_shape_hit = t1;

            p_hit = ray.at(t_shape_hit);
            p_hit *= self.radius / (p_hit - Point3::new(0.0, 0.0, 0.0)).norm();

            if p_hit.x == 0.0 && p_hit.y == 0.0 { p_hit.x = EPSILON * self.radius; }    // if x and u are 0 0, then shift x a bit 

            phi = p_hit.y.atan2(p_hit.x);
            if phi < 0.0 { phi += 2.0 * PI; }

            // if still out of bounds return false
            if (self.z_min > -self.radius && p_hit.z < self.z_min) || (self.z_max < self.radius && p_hit.z > self.z_max) || phi > self.phi_max { return false}
        }

        let u = phi / self.phi_max;
        let theta = (p_hit.z / self.radius).clamp(-1.0, 1.0).acos();
        let v = (theta - self.theta_min) / (self.theta_max - self.theta_min);

        let z_radius = (p_hit.x * p_hit.x + p_hit.y * p_hit.y).sqrt();
        let inv_z_radius = 1.0 / z_radius;
        let cos_phi = p_hit.x * inv_z_radius;
        let sin_phi = p_hit.y * inv_z_radius;

        let dpdu = Vector3::new(-self.phi_max * p_hit.y, self.phi_max * p_hit.x, 0.0);
        let dpdv = (self.theta_max - self.theta_min) * Vector3::new(p_hit.z * cos_phi, p_hit.z * sin_phi, -self.radius * theta.sin());

        let d2pduu = -self.phi_max * self.phi_max * Vector3::new(p_hit.x, p_hit.y, 0.0);
        let d2pduv = (self.theta_max - self.theta_min) * p_hit.z * self.phi_max * Vector3::new(-sin_phi, cos_phi, 0.0);
        let d2pdvv = -(self.theta_max - self.theta_min) * (self.theta_max - self.theta_min) * Vector3::new(p_hit.x, p_hit.y, p_hit.z);

        let big_e = dpdu.dot(&dpdu);
        let big_f = dpdu.dot(&dpdv);
        let big_g = dpdv.dot(&dpdv);

        let n = dpdu.cross(&dpdv).normalize();

        let e = n.dot(&d2pduu);
        let f = n.dot(&d2pduv);
        let g = n.dot(&d2pdvv);

        let inv_efg2 = 1.0 / (big_e * big_g - big_f * big_f);
        let dndu = (f*big_f - e*big_g)*inv_efg2*dpdu + (e*big_f - f*big_e)*inv_efg2*dpdv;
        let dndv = (g*big_f - f*big_g)*inv_efg2*dpdu + (f*big_f - g*big_e)*inv_efg2*dpdv;

        let p_error = gamma(5.0) * p_hit.map(|e| e.abs()) - Point3::new(0.0, 0.0, 0.0);

        *isect = SurfaceInteraction::init(&p_hit, &p_error, &Point2::new(u, v), &(-ray.d), &dpdu, &dpdv, &dndu, &dndv, ray.time, None);
        *t_hit = t_shape_hit;

        true
    }

    fn area(&self) -> f32 {
        self.phi_max * self.radius * (self.z_max - self.z_min)
    }

    fn sample(&self, u: &Point2) -> Interaction {
        let mut p_obj = Point3::origin() + self.radius * uniform_sample_sphere(u);
        let mut it = InteractionBase::new();

        it.n = apply_transform_to_normal(&(p_obj - Point3::origin()), &self.object_to_world);

        if self.reverse_orientation {
            it.n *= -1.0;
        }

        p_obj *= self.radius / (p_obj - Point3::origin()).norm();

        let p_obj_error = gamma(5.0) * (p_obj - Point3::origin()).map(|x| x.abs());

        it.p = self.object_to_world.transform_point(&p_obj);

        Interaction::Base(it)
    }
}

impl Manufacturable<Shape> for Sphere {
    fn create_from_parameters(param: crate::loader::Parameters) -> Shape {
        let t = param.get_transform();

        let object_to_world: Transform = t;
        let world_to_object: Transform = object_to_world.inverse();
        
        let reverse_orientation: bool = param.get_bool("reverse_orientation", Some(false));
        let radius: f32 = param.get_float("radius", Some(1.0));
        let z_min: f32 = param.get_float("z_min", Some(-1.0));
        let z_max: f32 = param.get_float("z_max", Some(1.0));
        let phi_max: f32 = param.get_float("phi_max", Some(360.0));

        let ret = Self::init(object_to_world, world_to_object, reverse_orientation, radius, z_min, z_max, phi_max);

        Shape::Sphere(
            ret
        )
    }
}

impl Printable for Sphere {
    fn to_string(&self) -> String {
        let m = self.object_to_world.matrix();
        let position = Vector3::new(m[(0, 3)], m[(1, 3)], m[(2, 3)]);

        format!(
            "Sphere [\n\
                \tradius: {:.3},\n\
                \tposition: ({:.3}, {:.3}, {:.3})\n\
            ]",
            self.radius,
            position.x, position.y, position.z
        )
    }
}