use std::sync::Arc;

use crate::{core::{Bounds2, INFINITY, Point2, Point3, Printable, Ray, Transform, Vector3, apply_transform_to_ray, camera::{ Camera, CameraSample, CameraT, ProjectedCameraBase}, film::Film, interaction::Interaction, lerp, light::VisibilityTester, look_at, medium::Medium, sampler::concentric_sample_disk, scaling, spectrum::Spectrum, translation}, loader::{LeadObject, Manufacturable, Parameters}};

#[derive(Clone)]
pub struct OrthographicCamera {
    pub base: ProjectedCameraBase,

    pub dx_camera: Vector3,
    pub dy_camera: Vector3
}

impl OrthographicCamera {
    pub fn init(
        camera_to_world: Transform,
        screen_window: Bounds2,
        shutter_open: f32,
        shutter_close: f32,
        lens_radius: f32,
        focal_distance: f32,
        film: Arc<Film>,
        medium: Option<Arc<Medium>>
    ) -> Self {
        let base = ProjectedCameraBase::init(camera_to_world, Self::create_orthographic(0.0, 1.0), screen_window, shutter_open, shutter_close, lens_radius, focal_distance, film, medium);

        let dx_camera = base.raster_to_camera.transform_vector(&Vector3::new(1.0, 0.0, 0.0));
        let dy_camera = base.raster_to_camera.transform_vector(&Vector3::new(0.0, 1.0, 0.0));

        Self {
            base,
            
            dx_camera,
            dy_camera
        }
    }

    fn create_orthographic(z_near: f32, z_far: f32) -> Transform {
        scaling(Vector3::new(
            1.0, 
            1.0, 
            1.0 / (z_far - z_near)
        )) * translation(Vector3::new(
            0.0, 
            0.0, 
            -z_near
        ))
    }
}

impl CameraT for OrthographicCamera {
    fn get_medium(&self) -> Option<Arc<Medium>> { self.base.medium.clone() }
    fn get_shutter_open(&self) -> f32 { self.base.shutter_open }
    fn get_shutter_close(&self) -> f32 { self.base.shutter_close }
    fn get_film(&self) -> Arc<Film> { self.base.film.clone() }
    fn get_camera_to_world(&self) -> Transform { self.base.camera_to_world }

    fn set_film(&mut self, film: Arc<Film>) {
        self.base.set_film(film);
    }

    fn generate_ray(&self, sample: CameraSample, ray: &mut Ray) -> f32 {
        let p_film = Point3::new(sample.p_film.x, sample.p_film.y, 0.0);
        let p_camera = self.base.raster_to_camera.transform_point(&p_film);

        *ray = Ray::init(&p_camera, &Vector3::new(0.0, 0.0, 1.0), INFINITY, 0.0, None, None);


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

    fn we(&self, ray: &Ray, p_raster2: &mut Point2) -> Spectrum {
        todo!("orthographic::we")
    }
    fn pdf_we(&self, ray: &Ray, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum {
        todo!("orthographic::pdf_we")
    }
    fn sample_wi(&self, reference: &Interaction, u: &Point2, wi: &mut Vector3, pdf: &mut f32, p_raster: &mut Point2, vis: &mut VisibilityTester) -> Spectrum {
        todo!("orthographic::sample_wi")
    }
}

impl Printable for OrthographicCamera {
    fn to_string(&self) -> String {
        format!(
            "Orthographic: [\n
            \tlens radius: {}\n
            \tfocal distance: {}\n
            ]",
            self.base.lens_radius,
            self.base.focal_distance
        )
    }
}

impl Manufacturable<Camera> for OrthographicCamera {
    fn create_from_parameters(params: Parameters) -> Camera {
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
            Some(LeadObject::Film(f)) => Arc::clone(f),
            _ => panic!("Camera requires a nested film"),
        };


        // film and medium would typically come from the scene, not params
        // pass them in or use defaults
        Camera::Orthographic(
            OrthographicCamera::init(
                camera_to_world,
                screen_window,
                shutter_open,
                shutter_close,
                lens_radius,
                focal_distance,
                // film and medium need to come from somewhere else
                film,
                None,
            )
        )
    }
}