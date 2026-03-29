use std::sync::Arc;

use nalgebra::Matrix4;

use crate::{core::{INFINITY, PI, Point2, Point3, Printable, Ray, Transform, Vector3, apply_transform_to_ray, bounds::Bounds2, camera::{ Camera, CameraSample, CameraT, ProjectedCameraBase}, film::Film, interaction::InteractionBase, lerp, light::VisibilityTester, look_at, medium::Medium, random::concentric_sample_disc, scaling, spectrum::Spectrum}, loader::Parameters, registry::{LeadObject, Manufacturable}};

// #[derive(Clone)]
pub struct PerspectiveCamera {
    pub base: ProjectedCameraBase,

    pub dx_camera: Vector3,
    pub dy_camera: Vector3,
    pub a: f32
}

impl PerspectiveCamera {
    pub fn init(
        camera_to_world: Transform,
        screen_window: Bounds2,
        shutter_open: f32,
        shutter_close: f32,
        lens_radius: f32,
        focal_distance: f32,
        fov: f32,
        film: Film,
        medium: Option<Arc<Medium>>
    ) -> Self {
        let base = ProjectedCameraBase::init(camera_to_world, Self::create_perspective(fov, 0.01, 1000.0), screen_window, shutter_open, shutter_close, lens_radius, focal_distance, film, medium);

        let dx_camera = base.raster_to_camera.transform_vector(&Vector3::new(1.0, 0.0, 0.0));
        let dy_camera = base.raster_to_camera.transform_vector(&Vector3::new(0.0, 1.0, 0.0));

        let res = base.film.full_resolution.clone();
        let mut p_min = base.raster_to_camera.transform_point(&Point3::origin());
        let mut p_max = base.raster_to_camera.transform_point(
            &Point3::new(
                res.x,
                res.y,
                0.0
            )
        );

        p_min = p_min / p_min.z;
        p_max = p_max / p_max.z;

        let a = ((p_max.x - p_min.x)*(p_max.y - p_min.y)).abs();

        Self {
            base,
            
            dx_camera,
            dy_camera,
            a
        }
    }

    fn create_perspective(fov: f32, n: f32, f: f32) -> Transform {
        let persp = Matrix4::new(
            1.0, 0.0, 0.0,               0.0,
            0.0, 1.0, 0.0,               0.0,
            0.0, 0.0, f / (f - n),      -f * n / (f - n),
            0.0, 0.0, 1.0,               0.0,
        );

        let inv_tan_ang = 1.0 / (fov.to_radians() * 0.5).tan();

        let scale = scaling(Vector3::new(inv_tan_ang, inv_tan_ang, 1.0));

        scale * Transform::from_matrix_unchecked(persp)
    }
}

impl CameraT for PerspectiveCamera {
    fn get_medium(&self) -> Option<Arc<Medium>> { self.base.medium.clone() }
    fn get_shutter_open(&self) -> f32 { self.base.shutter_open }
    fn get_shutter_close(&self) -> f32 { self.base.shutter_close }
    fn get_mut_film(&mut self) -> &mut Film { &mut self.base.film }
    fn get_film(&self) -> &Film { &self.base.film }
    fn get_camera_to_world(&self) -> Transform { self.base.camera_to_world }

    fn generate_ray(&self, sample: CameraSample, ray: &mut Ray) -> f32 {
        let p_film = Point3::new(sample.p_film.x, sample.p_film.y, 0.0);
        let p_camera = self.base.raster_to_camera.transform_point(&p_film);
        let p_camera_vec: Vector3 = p_camera.coords;

        *ray = Ray::init(&Point3::origin(), &p_camera_vec.normalize(), INFINITY, 0.0, None, None);


        if self.base.lens_radius > 0.0 {
            let p_lens = self.base.lens_radius * concentric_sample_disc(&sample.p_lens);

            let ft = self.base.focal_distance / ray.d.z;
            let p_focus = ray.at(ft);

            ray.o = Point3::new(p_lens.x, p_lens.y, 0.0);
            ray.d = (p_focus - ray.o).normalize();
        }

        ray.time = lerp(sample.time, self.base.shutter_open, self.base.shutter_close);
        ray.medium = self.base.medium.clone();

        *ray = apply_transform_to_ray(&ray, &self.base.camera_to_world);

        // println!("O: {}, {}, {} ---> {}, {} ,{}", ray.o.x, ray.o.y, ray.o.z, ray.d.x, ray.d.y, ray.d.z);

        1.0
    }

    fn we(&self, ray: &Ray, p_raster2: &mut Point2) -> Spectrum {
        let c2w = self.get_camera_to_world();
        let cos_theta = ray.d.dot(&c2w.transform_vector(&Vector3::z()));

        if cos_theta <= 0.0 {
            return Spectrum::zeros();
        }

        let p_focus = ray.at(
            if self.base.lens_radius > 0.0 {
                self.base.focal_distance 
            } else {
                1.0
            } / cos_theta
        );

        let p_raster = self.base.raster_to_camera.inverse().transform_point(
            &c2w.inverse().transform_point(&p_focus)
        );

        *p_raster2 = Point2::new(p_raster.x, p_raster.y);

        let sample_bounds = self.get_film().get_sample_bounds();
        if  p_raster.x < sample_bounds.p_min.x || 
            p_raster.y < sample_bounds.p_min.y ||
            p_raster.x >= sample_bounds.p_max.x ||
            p_raster.y >= sample_bounds.p_max.y {
                return Spectrum::zeros();
        }

        let lens_area = if self.base.lens_radius != 0.0 {
            PI * self.base.lens_radius * self.base.lens_radius
        } else {
            1.0 
        };

        let cos2_theta = cos_theta * cos_theta;
        let v = 1.0 / (self.a * lens_area * cos2_theta * cos2_theta);

        Spectrum::new(v, v, v)
    }

    fn pdf_we(&self, ray: &Ray, pdf_pos: &mut f32, pdf_dir: &mut f32) {
        let c2w = self.get_camera_to_world();
        let cos_theta = ray.d.dot(&c2w.transform_vector(&Vector3::z()));

        if cos_theta <= 0.0 {
            *pdf_pos = 0.0;
            *pdf_dir = 0.0;
            return;
        }

        let p_focus = ray.at(
            if self.base.lens_radius > 0.0 {
                self.base.focal_distance 
            } else {
                1.0
            } / cos_theta
        );

        let p_raster = self.base.raster_to_camera.inverse().transform_point(
            &c2w.inverse().transform_point(&p_focus)
        );

        let sample_bounds = self.get_film().get_sample_bounds();
        if  p_raster.x < sample_bounds.p_min.x || 
            p_raster.y < sample_bounds.p_min.y ||
            p_raster.x >= sample_bounds.p_max.x ||
            p_raster.y >= sample_bounds.p_max.y {
                *pdf_pos = 0.0;
                *pdf_dir = 0.0;
                return;
        }

        let lens_area = if self.base.lens_radius != 0.0 {
            PI * self.base.lens_radius * self.base.lens_radius
        } else {
            1.0 
        };

        *pdf_pos = 1.0 / lens_area;
        *pdf_dir = 1.0 / (self.a * cos_theta * cos_theta * cos_theta);
    }

    fn sample_wi(&self, re: &InteractionBase, u: &Point2, wi: &mut Vector3, pdf: &mut f32, p_raster: &mut Point2, vis: &mut VisibilityTester) -> Spectrum {
        let p_lens = self.base.lens_radius * concentric_sample_disc(u);
        let p_lens_world = self.get_camera_to_world().transform_point(
            &Point3::new(p_lens.x, p_lens.y, 0.0)
        );

        let mut lens_its = InteractionBase::init_no_wo(&p_lens_world, re.time.clone(), re.medium_interface.clone());
        lens_its.n = self.get_camera_to_world().transform_vector(&Vector3::z());

        *vis = VisibilityTester::init(re, &lens_its);
        *wi = lens_its.p - re.p;
        let dist = wi.norm();
        *wi /= dist;

        let lens_area = if self.base.lens_radius != 0.0 {
            PI * self.base.lens_radius * self.base.lens_radius
        } else {
            1.0
        };

        *pdf = dist * dist / (lens_its.n.dot(&wi).abs() * lens_area);

        self.we(&lens_its.spawn_ray(&-*wi), p_raster)
    }
}

impl Printable for PerspectiveCamera {
    fn to_string(&self) -> String {
        format!(
            "Perspective: [\n
            \tlens radius: {}\n
            \tfocal distance: {}\n
            \ta: {}\n
            ]",
            self.base.lens_radius,
            self.base.focal_distance,
            self.a
        )
    }
}

impl Manufacturable<Camera> for PerspectiveCamera {
    fn create_from_parameters(params: Parameters) -> Camera {
        let mut params = params;
        let eye    = params.get_point3("eye",    Some(Point3::new(0.0, 0.0, -1.0)));
        let target = params.get_point3("target", Some(Point3::origin()));
        let up     = params.get_vector3("up",    Some(Vector3::new(0.0, 1.0, 0.0)));

        let camera_to_world = look_at(&eye, &target, &up);

        let extent = params.get_float("extent", Some(5.0));

        let screen_window = Bounds2::init_two(
            &Point2::new(
                params.get_float("screen_min_x", Some(-extent)),
                params.get_float("screen_min_y", Some(-extent)),
            ),
            &Point2::new(
                params.get_float("screen_max_x", Some(extent)),
                params.get_float("screen_max_y", Some(extent)),
            ),
        );

        let shutter_open   = params.get_float("shutter_open",   Some(0.0));
        let shutter_close  = params.get_float("shutter_close",  Some(1.0));
        let lens_radius    = params.get_float("lens_radius",    Some(0.0));
        let focal_distance = params.get_float("focal_distance", Some(1e6));

        let film = match params.get_lead_object("film") {
            Some(LeadObject::Film(f)) => f,
            _ => panic!("Camera requires a nested film"),
        };

        let fov = params.get_float("fov", Some(45.0));

        // film and medium would typically come from the scene, not params
        // pass them in or use defaults
        Camera::Perspective(
            PerspectiveCamera::init(
                camera_to_world,
                screen_window,
                shutter_open,
                shutter_close,
                lens_radius,
                focal_distance,
                fov,
                // film and medium need to come from somewhere else
                film,
                None,
            )
        )
    }
}