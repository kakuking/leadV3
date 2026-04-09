use std::sync::Arc;

use nalgebra::{Vector3, Vector4};

use pyo3::prelude::*;
use pyo3::types::PyModule;

use crate::{core::{Point3, Printable, Ray, Transform, apply_transform_to_ray, bounds::Bounds3, interaction::MediumInteraction, lerp, medium::{Medium, MediumInterface, MediumT, PhaseFunction}, rotate_angle_axis, sampler::Sampler, spectrum::Spectrum, translation}, medium::hg_phase::HenyeyGreenstein, registry::Manufacturable};

#[derive(Debug, PartialEq, Clone)]
pub struct HeterogeneousMedium {
    sigma_a: Spectrum,
    sigma_s: Spectrum,
    g: f32,

    nx: usize,
    ny: usize,
    nz: usize,
    bounds_max: Point3,
    density: Vec<f32>,

    world_to_medium: Transform,
    sigma_t: f32,
    inv_max_density: f32
}

impl HeterogeneousMedium {
    pub fn init(sigma_a: Spectrum, sigma_s: Spectrum, g: f32, nx: usize, ny: usize, nz: usize, medium_to_world: Transform, density: Vec<f32>) -> Self {
        let sigma_t = (sigma_a + sigma_s).x;
        let mut max_density: f32 = 0.0;

        for i in 0..nx * ny * nz {
            max_density = max_density.max(density[i]);
        }

        let inv_max_density = 1.0 / max_density;

        let max_dim = nx.max(ny).max(nz) as f32;
        let bounds_max = Point3::new(
            nx as f32 / max_dim,
            ny as f32 / max_dim,
            nz as f32 / max_dim,
        );

        Self {
            sigma_a,
            sigma_s,
            g,
            nx,
            ny,
            nz,
            bounds_max,
            density,
            world_to_medium: medium_to_world.inverse(),
            sigma_t,
            inv_max_density
        }
    }

    pub fn get_density(&self, p: &Point3) -> f32 {
        let pn = Point3::new(
            p.x / self.bounds_max.x,
            p.y / self.bounds_max.y,
            p.z / self.bounds_max.z,
        );

        let p_samples = Point3::new(
            pn.x * self.nx as f32 - 0.5,
            pn.y * self.ny as f32 - 0.5,
            pn.z * self.nz as f32 - 0.5,
        );

        let pi = p_samples.map(|x| x.floor());
        let d = p_samples - pi;

        let d00 = lerp(d.x, self.d(&pi), self.d(&(pi + Vector3::new(1.0, 0.0, 0.0))));
        let d10 = lerp(d.x, self.d(&(pi + Vector3::new(0.0, 1.0, 0.0))), self.d(&(pi + Vector3::new(1.0, 1.0, 0.0))));
        let d01 = lerp(d.x, self.d(&(pi + Vector3::new(0.0, 0.0, 1.0))), self.d(&(pi + Vector3::new(1.0, 0.0, 1.0))));
        let d11 = lerp(d.x, self.d(&(pi + Vector3::new(0.0, 1.0, 1.0))), self.d(&(pi + Vector3::new(1.0, 1.0, 1.0))));

        let d0 = lerp(d.y, d00, d10);
        let d1 = lerp(d.y, d01, d11);

        let ret = lerp(d.z, d0, d1);

        ret
    }

    pub fn d(&self, p: &Point3) -> f32 {
        let sample_bounds = Bounds3::init_two(
            &Point3::origin(), 
            &Point3::new(self.nx as f32, self.ny as f32, self.nz as f32)
        );

        if !sample_bounds.inside_exclusive(p) {
            return 0.0
        }

        let idx = (p.z as usize * self.ny + p.y as usize) * self.nx + p.x as usize;

        self.density[idx]
    }

    pub fn set_world_to_medium(&mut self, world_to_medium: Transform) {
        self.world_to_medium = world_to_medium;
    }
    
    // 0, 0, 0 is at bottom left corner, act accordingly when applying transform
    pub fn load_vdb_dense(path: &str, grid_name: &str) -> PyResult<(usize, usize, usize, Vec<f32>)> {
        Python::attach(|py| {
            let code = r#"
import openvdb

def load_dense(path, grid_name):
    grid = openvdb.read(path, grid_name)

    bmin, bmax = grid.evalActiveVoxelBoundingBox()
    minx, miny, minz = bmin
    maxx, maxy, maxz = bmax

    nx = maxx - minx + 1
    ny = maxy - miny + 1
    nz = maxz - minz + 1

    data = [0.0] * (nx * ny * nz)

    acc = grid.getConstAccessor()

    for z in range(minz, maxz + 1):
        for y in range(miny, maxy + 1):
            for x in range(minx, maxx + 1):
                value, active = acc.probeValue((x, y, z))
                if active:
                    lx = x - minx
                    ly = y - miny
                    lz = z - minz
                    idx = (lz * ny + ly) * nx + lx
                    data[idx] = float(value)

    return nx, ny, nz, data
"#;

            use std::ffi::CString;
            let code_c = CString::new(code).unwrap();
            let file_c = CString::new("embedded_vdb_loader.py").unwrap();
            let name_c = CString::new("embedded_vdb_loader").unwrap();

            let module = PyModule::from_code(
                py,
                code_c.as_c_str(),
                file_c.as_c_str(),
                name_c.as_c_str(),
            )?;

            let result = module
                .getattr("load_dense")?
                .call1((path, grid_name))?;

            let (nx, ny, nz, density): (usize, usize, usize, Vec<f32>) = result.extract()?;

            Ok((nx, ny, nz, density))
        })
    }
}

impl MediumT for HeterogeneousMedium {
    fn sample(&self, r_world: &Ray, sampler: &mut Sampler, mi: &mut MediumInteraction, medium: Arc<Medium>) -> Spectrum {
        let ray = apply_transform_to_ray(
            &Ray::init(
                &r_world.o, 
                &r_world.d.normalize(), 
                r_world.t_max.get() * r_world.d.norm(), 
                0.0, 
                None, 
                None
            ), 
            &self.world_to_medium
        );

        // let b = Bounds3::init_two(&Point3::origin(), &Point3::new(1.0, 1.0, 1.0));
        let b = Bounds3::init_two(&Point3::origin(), &self.bounds_max);
        let mut t_min: f32 = 0.0;
        let mut t_max: f32 = 0.0;

        if !b.intersect_p(&ray, &mut t_min, &mut t_max) {
            return Spectrum::new(1.0, 1.0, 1.0);
        }

        let mut t = t_min;

        loop {
            t -= (1.0 - sampler.get_1d()).ln() * self.inv_max_density / self.sigma_t;

            if t >= t_max {
                break;
            }

            if self.get_density(&ray.at(t)) * self.inv_max_density > sampler.get_1d() {
                let phase = Arc::new(
                    PhaseFunction::HG(
                        HenyeyGreenstein::init(self.g)
                    )
                );

                *mi = MediumInteraction::init_no_normal_one_medium(&r_world.at(t), &-r_world.d, r_world.time, MediumInterface::init_one(Some(medium)), Some(phase));

                return self.sigma_s / self.sigma_t;
            }
        }

        Spectrum::new(1.0, 1.0, 1.0)
    }

    fn tr(&self, r_world: &Ray, sampler: &mut Sampler) -> Spectrum {
        let ray = apply_transform_to_ray(
            &Ray::init(
                &r_world.o, 
                &r_world.d.normalize(), 
                r_world.t_max.get() * r_world.d.norm(), 
                0.0, 
                None, 
                None
            )
            , 
            &self.world_to_medium
        );

        // let b = Bounds3::init_two(
        //     &Point3::origin(), 
        //     &Point3::new(1.0, 1.0, 1.0)
        // );
        let b = Bounds3::init_two(&Point3::origin(), &self.bounds_max);

        let mut t_min: f32 = 0.0;
        let mut t_max: f32 = 0.0;

        if !b.intersect_p(&ray, &mut t_min, &mut t_max) {
            // println!("Returning 1.0");
            return Spectrum::new(1.0, 1.0, 1.0);
        }

        let mut tr = 1.0;
        let mut t = t_min;

        // println!("t_min={} t_max={}", t_min, t_max);
        // println!("sigma_t={} inv_max_density={}", self.sigma_t, self.inv_max_density);
        loop {
            t -= (1.0 - sampler.get_1d()).ln() * self.inv_max_density / self.sigma_t;

            // println!("t: {}, max_t: {}", t, t_max);

            if t >= t_max {
                break;
            }

            let density = self.get_density(&ray.at(t));
            tr *= 1.0 - (density * self.inv_max_density).max(0.0);
        }

        // println!("Returning {}", tr);
        Spectrum::new(tr, tr, tr)
    }
}

impl Manufacturable<Medium> for HeterogeneousMedium {
    fn create_from_parameters(param: crate::loader::Parameters) -> Medium {
        let filename = param.get_string("filename", Some("z_input/volume.vdb".to_string()));
        let grid_name = param.get_string("grid", Some("density".to_string()));

        let (nx, ny, nz, density) = Self::load_vdb_dense(&filename, &grid_name).unwrap();

        let sigma_a = param.get_vector3("sigma_a", Some(Vector3::new(1.0, 1.0, 1.0)));
        let sigma_s = param.get_vector3("sigma_s", Some(Vector3::new(1.0, 1.0, 1.0)));
        let g = param.get_float("g", Some(0.0));

        let user_medium_to_world = param.get_transform();

        let to_center = translation(Vector3::new(-0.5, -0.5, -0.5));
        let to_y_up = rotate_angle_axis(Vector4::new(1.0, 0.0, 0.0, -90.0));

        let medium_to_world = user_medium_to_world * to_y_up * to_center;

        let med = Self::init(sigma_a, sigma_s, g, nx, ny, nz, medium_to_world, density);

        Medium::Heterogeneous(
            med
        )
    }
}

impl Printable for HeterogeneousMedium {
    fn to_string(&self) -> String {
        format!(
            "Heterogeneous Medium: [\n
            \tsigma_a: {}, {}, {}\n
            \tsigma_s: {}, {}, {}\n
            \tg: {},\n
            \tnums: {}, {}, {},\n
            \tdensity len: {},\n
            \tsigma_t: {},\n
            \tinv_max_density: {}\n
            ]",
            self.sigma_a.x, self.sigma_a.y, self.sigma_a.z,
            self.sigma_s.x, self.sigma_s.y, self.sigma_s.z,
            self.g,
            self.nx, self.ny, self.nz,
            self.density.len(),
            self.sigma_t,
            self.inv_max_density
        )
    }
}