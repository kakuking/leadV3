use std::sync::Arc;

use nalgebra::Matrix4;

use crate::{core::{bounds::Bounds2, INFINITY, Point2, Point3, Printable, Ray, Transform, Vector3, apply_transform_to_ray, camera::{ Camera, CameraSample, CameraT, ProjectedCameraBase}, film::Film, interaction::Interaction, lerp, light::VisibilityTester, look_at, medium::Medium, sampler::concentric_sample_disk, scaling, spectrum::Spectrum}, loader::Parameters, registry::{LeadObject, Manufacturable}};

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
            let p_lens = self.base.lens_radius * concentric_sample_disk(&sample.p_lens);

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

    fn we(&self, _ray: &Ray, _p_raster2: &mut Point2) -> Spectrum {
        todo!("orthographic::we")
    }
    fn pdf_we(&self, _ray: &Ray, _pdf_pos: &mut f32, _pdf_dir: &mut f32) -> Spectrum {
        todo!("orthographic::pdf_we")
    }
    fn sample_wi(&self, _reference: &Interaction, _u: &Point2, _wi: &mut Vector3, _pdf: &mut f32, _p_raster: &mut Point2, _vis: &mut VisibilityTester) -> Spectrum {
        todo!("orthographic::sample_wi")
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