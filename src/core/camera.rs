use std::sync::Arc;

use crate::{camera::orthographic::OrthographicCamera, core::{Bounds2, Point2, Printable, Ray, RayDifferential, Spectrum, Transform, Vector3, interaction::Interaction, light::VisibilityTester, medium::Medium, scaling, translation}, loader::Manufacturable};

pub struct Film {
    pub full_resolution: Point2
}

#[derive(Debug, Clone)]
pub struct CameraSample {
    pub p_film: Point2,
    pub p_lens: Point2,
    pub time: f32
}

pub enum Camera {
    Orthographic(OrthographicCamera)
}

impl Camera {
    pub fn get_medium(&self) -> Option<Arc<Medium>> {
        match self {
            Self::Orthographic(cam) => cam.get_medium()
        }
    }
    pub fn get_shutter_open(&self) -> f32 {
        match self {
            Self::Orthographic(cam) => cam.get_shutter_open()
        }
    }

    pub fn get_shutter_close(&self) -> f32 {
        match self {
            Self::Orthographic(cam) => cam.get_shutter_close()
        }
    }

    pub fn get_film(&self) -> Arc<Film> {
        match self {
            Self::Orthographic(cam) => cam.get_film()
        }
    }

    pub fn get_camera_to_world(&self) -> Transform {
        match self {
            Self::Orthographic(cam) => cam.get_camera_to_world()
        }
    }

    pub fn generate_ray(&self, sample: CameraSample, ray: &mut Ray) -> f32 {
        match self {
            Self::Orthographic(cam) => cam.generate_ray(sample, ray)
        }
    }

    pub fn we(&self, ray: &Ray, p_raster2: &mut Point2) -> Spectrum {
        match self {
            Self::Orthographic(cam) => cam.we(ray, p_raster2)
        }
    }

    pub fn pdf_we(&self, ray: &Ray, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum {
        match self {
            Self::Orthographic(cam) => cam.pdf_we(ray, pdf_pos, pdf_dir)
        }
    }

    pub fn sample_wi(&self, reference: &Interaction, u: &Point2, wi: &mut Vector3, pdf: &mut f32, p_raster: &mut Point2, vis: &mut VisibilityTester) -> Spectrum {
        match self {
            Self::Orthographic(cam) => cam.sample_wi(reference, u, wi, pdf, p_raster, vis)
        }
    }

    pub fn generate_ray_differential(&self, sample: CameraSample, ray: &mut Ray) -> f32 {
        let wt = self.generate_ray(sample.clone(), ray);

        let mut s_shift: CameraSample = sample;
        s_shift.p_film.x += 1.0;

        let mut rx = Ray::new();
        let wtx = self.generate_ray(s_shift.clone(), &mut rx);
        if wtx == 0.0 {
            return 0.0;
        }

        let mut diff = RayDifferential::new();
        diff.rx_o = rx.o;
        diff.rx_d = rx.d;

        s_shift.p_film.x -= 1.0;
        s_shift.p_film.y += 1.0;

        let mut ry = Ray::new();
        let wty = self.generate_ray(s_shift, &mut ry);
        if  wty == 0.0 {
            return 0.0;
        }

        diff.ry_o = ry.o;
        diff.ry_d = ry.d;

        ray.differential = Some(diff);

        wt
    }
}

pub trait CameraT: Manufacturable + Printable {
    fn get_medium(&self) -> Option<Arc<Medium>>;
    fn get_shutter_open(&self) -> f32;
    fn get_shutter_close(&self) -> f32;
    fn get_film(&self) -> Arc<Film>;
    fn get_camera_to_world(&self) -> Transform;
    
    fn generate_ray(&self, sample: CameraSample, ray: &mut Ray) -> f32;
    fn we(&self, ray: &Ray, p_raster2: &mut Point2) -> Spectrum;
    fn pdf_we(&self, ray: &Ray, pdf_pos: &mut f32, pdf_dir: &mut f32) -> Spectrum;
    fn sample_wi(&self, reference: &Interaction, u: &Point2, wi: &mut Vector3, pdf: &mut f32, p_raster: &mut Point2, vis: &mut VisibilityTester) -> Spectrum;
}


pub struct ProjectedCameraBase {
    pub camera_to_screen: Transform,
    pub raster_to_camera: Transform,
    pub screen_to_raster: Transform,
    pub raster_to_screen: Transform,
    pub lens_radius: f32,
    pub focal_distance: f32,

    pub camera_to_world: Transform,
    pub shutter_open: f32,
    pub shutter_close: f32,
    pub film: Arc<Film>,
    pub medium: Option<Arc<Medium>> 
}

impl ProjectedCameraBase {
    pub fn init(
        camera_to_world: Transform,
        camera_to_screen: Transform,
        screen_window: Bounds2,
        shutter_open: f32,
        shutter_close: f32,
        lens_radius: f32,
        focal_distance: f32,
        film: Arc<Film>,
        medium: Option<Arc<Medium>> 
    ) -> Self {
        let mut screen_to_raster = scaling(Vector3::new(
            film.full_resolution.x, film.full_resolution.y, 1.0
        ));

        screen_to_raster = screen_to_raster * scaling(Vector3::new(
            1.0 / (screen_window.p_max.x - screen_window.p_min.x), 
            1.0 / (screen_window.p_max.y - screen_window.p_min.y), 
            1.0
        ));

        screen_to_raster = screen_to_raster * translation(Vector3::new(
            -screen_window.p_min.x, 
            -screen_window.p_max.y, 
            0.0
        ));

        let raster_to_screen = screen_to_raster.inverse();

        let raster_to_camera = camera_to_screen.inverse() * raster_to_screen;

        Self {
            camera_to_world,
            camera_to_screen,
            raster_to_camera,
            screen_to_raster,
            raster_to_screen,
            lens_radius,
            focal_distance,
            shutter_open,
            shutter_close,
            film,
            medium
        }
    }
}

impl CameraSample {
    pub fn new() -> Self {
        Self {
            p_film: Point2::origin(),
            p_lens: Point2::origin(),
            time: 0.0
        }
    }
}