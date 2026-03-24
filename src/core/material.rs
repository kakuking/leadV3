use std::{fmt::Debug, sync::Arc};

use nalgebra::Vector3;

use crate::{core::{Vector2, bxdf::BxDF, interaction::TransportMode, spectrum::Spectrum, texture::Texture}, interaction::surface_interaction::SurfaceInteraction, reflection::lambertian::LambertianReflection};

#[derive(Debug)]
pub enum Material {
    Matte(MatteMaterial)
}

impl Material {
    pub fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_lobes: bool) {
        match self {
            Self::Matte(m) => m.compute_scattering_funcitons(si, mode, allow_multiple_lobes),
        }
    }
}


pub trait MaterialT {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_lobes: bool);
    fn bump(d: &Arc<dyn Texture<f32>>, si: &mut SurfaceInteraction) {
        let mut si_eval = si.clone();

        let mut du = 0.5 * si.dudx.get().abs() + si.dudy.get().abs();
        if du == 0.0 {
            du = 0.01;
        }
        si_eval.base.p = si.base.p + du * si.shading.dpdu;
        si_eval.uv = si.uv + Vector2::new(du, 0.0);
        si_eval.base.n = (si.shading.dpdu.cross(&si.shading.dpdv) + du * si.dndu).normalize();
        let u_displace = d.evaluate(&si_eval);

        let mut dv = 0.5 * si.dvdx.get().abs() + si.dvdy.get().abs();
        if dv == 0.0 {
            dv = 0.01;
        }
        si_eval.base.p = si.base.p + dv * si.shading.dpdv;
        si_eval.uv = si.uv + Vector2::new(0.0, dv);
        si_eval.base.n = (si.shading.dpdu.cross(&si.shading.dpdv) + du * si.dndv).normalize();
        let v_displace = d.evaluate(&si_eval);

        let displace = d.evaluate(&si);

        let dpdu = si.shading.dpdu + (u_displace - displace) / du * si.shading.n + displace * si.shading.dndu;
        let dpdv = si.shading.dpdv + (v_displace - displace) / dv * si.shading.n + displace * si.shading.dndv;

        let dndu = si.shading.dndu;
        let dndv = si.shading.dndv;

        si.set_shading_geometry(&dpdu, &dpdv, &dndu, &dndv, false);
    }
}

#[derive(Debug)]
pub struct MatteMaterial {
    kd: Arc<dyn Texture<Spectrum>>,
    sigma: Arc<dyn Texture<f32>>,
    bump_map: Option<Arc<dyn Texture<f32>>>,
}

impl MatteMaterial {
    pub fn init(
        kd: Arc<dyn Texture<Spectrum>>,
        sigma: Arc<dyn Texture<f32>>,
        bump_map: Option<Arc<dyn Texture<f32>>>
    ) -> Self {
        Self {
            kd,
            sigma,
            bump_map
        }
    }
}

impl MaterialT for MatteMaterial {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, _mode: TransportMode, _allow_multiple_lobes: bool) {
        if let Some(b) = &self.bump_map {
            Self::bump(b, si);
        }

        let r = self.kd.evaluate(si);
        let sig = self.sigma.evaluate(&si).clamp(0.0, 90.0);

        if r == Vector3::zeros() {
            let bsdf = si.bsdf.as_mut().unwrap();
            if sig == 0.0 {
                bsdf.add(
                    BxDF::Lambertian(
                        LambertianReflection::init(r))
                    );
            } else {
                todo!("to make OrenNayar")
                // bsdf.add(
                //     BxDF::OrenNayar(
                //         OrenNayarReflection::init(r, sig))
                //     );
            }
        }
    }
}