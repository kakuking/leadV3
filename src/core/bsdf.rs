use crate::{core::{Normal3, Point2, Vector3, bxdf::{BxDF, BxDFType}, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, reflection::lambertian::LambertianReflection};

pub const MAX_BXDFS: usize = 8;

#[derive(Debug, Clone)]
pub struct BSDF {
    pub eta: f32,

    ns: Normal3,
    ng: Normal3,
    ss: Vector3,
    ts: Vector3,
    n_bxdfs: usize,
    bxdfs: Vec<BxDF>,
}

impl BSDF {
    pub fn init(si: &SurfaceInteraction, eta: f32) -> Self {
        let ns = si.shading.n;
        let ng = si.base.n;
        let ss = si.shading.dpdu.normalize();
        let ts = ns.cross(&ss);

        let mut bxdfs = Vec::new();
        for _ in 0..MAX_BXDFS {
            let lamb =  BxDF::Lambertian(
                LambertianReflection::init(Spectrum::x())
            );
            
            bxdfs.push(lamb);
        }

        Self {
            eta,

            ns,
            ng,
            ss,
            ts,
            n_bxdfs: 0,
            bxdfs: bxdfs
        }
    }

    pub fn add(&mut self, b: BxDF) {
        assert!(self.n_bxdfs < MAX_BXDFS);

        self.bxdfs[self.n_bxdfs] = b;
        self.n_bxdfs += 1;
    }

    pub fn num_components(&self, flags: BxDFType) -> usize {
        let mut ret = 0;

        for i in 0..self.n_bxdfs {
            let typ = self.bxdfs[i].get_type();
            if typ.contains(flags) {
                ret += 1;
            }
        }

        ret
    }

    pub fn world_to_local(&self, v: Vector3) -> Vector3 {
        Vector3::new(
            v.dot(&self.ss),
            v.dot(&self.ts),
            v.dot(&self.ns),
        )
    }

    pub fn local_to_world(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            self.ss.x * v.x + self.ts.x * v.y + self.ns.x * v.z,
            self.ss.y * v.x + self.ts.y * v.y + self.ns.y * v.z,
            self.ss.z * v.x + self.ts.z * v.y + self.ns.z * v.z
        )
    }

    pub fn f(&self, wo_w: &Vector3, wi_w: &Vector3, flags: Option<BxDFType>) -> Spectrum {
        let flags = flags.unwrap_or(BxDFType::BSDF_ALL);

        let wi = self.world_to_local(*wi_w);
        let wo = self.world_to_local(*wo_w);

        let reflect = wi_w.dot(&self.ng) * wo_w.dot(&self.ng) > 0.0;
        let mut f = Spectrum::zeros();

        for i in 0..self.n_bxdfs {
            let bxdf = &self.bxdfs[i];
            if bxdf.matches_flags(flags) && 
            ((reflect && (bxdf.get_type().contains(BxDFType::BSDF_REFLECTION))) ||
            (!reflect && (bxdf.get_type().contains(BxDFType::BSDF_TRANSMISSION)))) {
                f += bxdf.f(&wo, &wi);
            }
        }
        f
    }

    pub fn sample_f(
        &self,
        wo_world: &Vector3,
        wi_world: &mut Vector3,
        u: &Point2,
        pdf: &mut f32,
        typ: &mut BxDFType,
        flags: Option<BxDFType>,
    ) -> Spectrum {
        let flags = flags.unwrap_or(BxDFType::BSDF_ALL);

        let matching_comps = self.num_components(flags);
        if matching_comps == 0 {
            *pdf = 0.0;
            return Spectrum::zeros();
        }

        let comp = ((u[0] * matching_comps as f32).floor() as usize).min(matching_comps - 1);

        // Choose which BxDF to sample
        let mut chosen: Option<(usize, &BxDF)> = None;
        let mut count = comp;

        for i in 0..self.n_bxdfs {
            if self.bxdfs[i].matches_flags(flags) {
                if count == 0 {
                    chosen = Some((i, &self.bxdfs[i]));
                    break;
                }
                count -= 1;
            }
        }

        let (cur_bxdf_idx, cur_bxdf) = chosen.expect("matching_comps > 0 but no matching BxDF found");

        // Remap sample to [0,1)^2 for chosen component
        let u_remapped = Point2::new(u[0] * matching_comps as f32 - comp as f32, u[1]);

        // Sample chosen BxDF
        let wo = self.world_to_local(*wo_world);
        let mut wi = Vector3::zeros();
        *pdf = 0.0;

        *typ = cur_bxdf.get_type();
        let mut f = cur_bxdf.sample_f(&wo, &mut wi, &u_remapped, pdf, Some(*typ));

        if *pdf == 0.0 {
            return Spectrum::zeros();
        }

        *wi_world = self.local_to_world(&wi);

        // Compute overall PDF with all matching BxDFs
        if !cur_bxdf.get_type().contains(BxDFType::BSDF_SPECULAR) && matching_comps > 1 {
            for i in 0..self.n_bxdfs {
                if i != cur_bxdf_idx && self.bxdfs[i].matches_flags(flags) {
                    *pdf += self.bxdfs[i].pdf(&wo, &wi);
                }
            }
        }

        if matching_comps > 1 {
            *pdf /= matching_comps as f32;
        }

        // Compute value of BSDF for sampled direction
        if !cur_bxdf.get_type().contains(BxDFType::BSDF_SPECULAR) && matching_comps > 1 {
            let reflect = wi_world.dot(&self.ng) * wo_world.dot(&self.ng) > 0.0;
            f = Spectrum::zeros();

            for i in 0..self.n_bxdfs {
                if self.bxdfs[i].matches_flags(flags)
                    && ((reflect && self.bxdfs[i].get_type().contains(BxDFType::BSDF_REFLECTION))
                        || (!reflect
                            && self.bxdfs[i].get_type().contains(BxDFType::BSDF_TRANSMISSION)))
                {
                    f += self.bxdfs[i].f(&wo, &wi);
                }
            }
        }

        f
    }

    pub fn rho(
        &self,
        wo: &Vector3,
        n_samples: usize,
        samples: &mut Vec<Point2>,
        flags: Option<BxDFType>,
    ) -> Spectrum {
        let flags = flags.unwrap_or(BxDFType::BSDF_ALL);

        let wo_local = self.world_to_local(*wo);
        let mut ret = Spectrum::zeros();

        for i in 0..self.n_bxdfs {
            let bxdf = &self.bxdfs[i];
            if bxdf.matches_flags(flags) {
                ret += bxdf.rho(&wo_local, n_samples, samples);
            }
        }

        ret
    }

    pub fn rho_2(
        &self,
        n_samples: usize,
        samples1: &mut Vec<Point2>,
        samples2: &mut Vec<Point2>,
        flags: Option<BxDFType>,
    ) -> Spectrum {
        let flags = flags.unwrap_or(BxDFType::BSDF_ALL);

        let mut ret = Spectrum::zeros();

        for i in 0..self.n_bxdfs {
            let bxdf = &self.bxdfs[i];
            if bxdf.matches_flags(flags) {
                ret += bxdf.rho_2(n_samples, samples1, samples2);
            }
        }

        ret
    }

    pub fn pdf(&self, wi: &Vector3, wo: &Vector3, flags: Option<BxDFType>) -> f32 {
        let flags = flags.unwrap_or(BxDFType::BSDF_ALL);

        if self.n_bxdfs == 0 {
            return 0.0;
        }

        let wi_local = self.world_to_local(*wi);
        let wo_local = self.world_to_local(*wo);

        if wo_local.z == 0.0 {
            return 0.0;
        }

        let mut pdf = 0.0;
        let mut matching_comps = 0;

        for i in 0..self.n_bxdfs {
            let bxdf = &self.bxdfs[i];
            if bxdf.matches_flags(flags) {
                matching_comps += 1;
                pdf += bxdf.pdf(&wo_local, &wi_local);
            }
        }

        if matching_comps > 0 {
            pdf / matching_comps as f32
        } else {
            0.0
        }
    }
}

#[derive(Debug)]
pub struct BSSRDF {
    
}

impl BSSRDF {
    pub fn new() -> Self {
        Self {
            
        }
    }
}

