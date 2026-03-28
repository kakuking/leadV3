use std::{fmt::Debug, sync::Arc};

use crate::{core::{Printable, Vector2, bsdf::BSDF, interaction::TransportMode, texture::Texture}, interaction::surface_interaction::SurfaceInteraction, material::{glass::GlassMaterial, matte::MatteMaterial, mirror::MirrorMaterial}, registry::Manufacturable};

#[derive(Debug)]
pub enum Material {
    Matte(MatteMaterial),
    Mirror(MirrorMaterial),
    Glass(GlassMaterial),
}

impl Material {
    pub fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_lobes: bool) {
        let bsdf = BSDF::init(&si, 1.0);
        si.bsdf = Some(bsdf);
        
        match self {
            Self::Matte(m) => m.compute_scattering_funcitons(si, mode, allow_multiple_lobes),
            Self::Mirror(m) => m.compute_scattering_funcitons(si, mode, allow_multiple_lobes),
            Self::Glass(m) => m.compute_scattering_funcitons(si, mode, allow_multiple_lobes),
        }
    }
}


pub trait MaterialT: Manufacturable<Material> + Printable {
    fn compute_scattering_funcitons(&self, si: &mut SurfaceInteraction, mode: TransportMode, allow_multiple_lobes: bool);
    fn bump(d: &Arc<Texture>, si: &mut SurfaceInteraction) {
        let mut si_eval = si.clone();

        let mut du = 0.5 * si.dudx.get().abs() + si.dudy.get().abs();
        if du == 0.0 {
            du = 0.01;
        }
        si_eval.base.p = si.base.p + du * si.shading.dpdu;
        si_eval.uv = si.uv + Vector2::new(du, 0.0);
        si_eval.base.n = (si.shading.dpdu.cross(&si.shading.dpdv) + du * si.dndu).normalize();
        let u_displace = d.evaluate(&si_eval).x;

        let mut dv = 0.5 * si.dvdx.get().abs() + si.dvdy.get().abs();
        if dv == 0.0 {
            dv = 0.01;
        }
        si_eval.base.p = si.base.p + dv * si.shading.dpdv;
        si_eval.uv = si.uv + Vector2::new(0.0, dv);
        si_eval.base.n = (si.shading.dpdu.cross(&si.shading.dpdv) + du * si.dndv).normalize();
        let v_displace = d.evaluate(&si_eval).x;

        let displace = d.evaluate(&si).x;

        let dpdu = si.shading.dpdu + (u_displace - displace) / du * si.shading.n + displace * si.shading.dndu;
        let dpdv = si.shading.dpdv + (v_displace - displace) / dv * si.shading.n + displace * si.shading.dndv;

        let dndu = si.shading.dndu;
        let dndv = si.shading.dndv;

        si.set_shading_geometry(&dpdu, &dpdv, &dndu, &dndv, false);
    }
}