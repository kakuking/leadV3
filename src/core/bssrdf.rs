use std::sync::Arc;

use crate::{core::{Normal3, PI, Point2, Point3, Printable, Vector3, bsdf::BSDF, bxdf::{BxDF, BxDFT, BxDFType}, cos_theta, distribution::find_interval, interaction::{InteractionBase, InteractionT, TransportMode}, material::Material, scene::Scene, spectrum::Spectrum}, interaction::surface_interaction::SurfaceInteraction, reflection::fresnel::fr_dielectric, registry::Manufacturable};

#[derive(Debug, Clone)]
pub struct TabulatedBSSRDF {
    // BSSRDF data
    pub po_p: Point3,
    pub po_time: f32,
    pub po_wo: Vector3,
    pub eta: f32,

    // Separable BSSRDF data
    pub ns: Normal3,
    pub ss: Vector3,
    pub ts: Vector3,
    pub material: Arc<Material>,
    pub mode: TransportMode,

    // TabulatedBSSRDF data
    pub table: Arc<BSSRDFTable>,
    pub sigma_t: Spectrum,
    pub rho: Spectrum
}

impl TabulatedBSSRDF {
    pub fn init(
        po: &SurfaceInteraction, material: Option<Arc<Material>>,
        mode: TransportMode, eta: f32,
        sigma_a: Spectrum, sigma_s: Spectrum,
        table: Arc<BSSRDFTable>
    ) -> Self {
        let sigma_t = sigma_a + sigma_s;

        let mut rho = Spectrum::zeros();
        for i in 0..3 {
            rho[i] = if sigma_t[i] != 0.0 {
                sigma_s[i] / sigma_t[i]
            } else {
                0.0
            };
        }

        let ns = po.shading.n;
        let ss = po.shading.dpdu.normalize();

        if let Some(mat) = material {
            Self {
                po_p: po.get_p().clone(),
                po_time: po.get_time().clone(),
                po_wo: po.get_wo().clone(),
                eta,
                ns,
                ss,
                ts: ns.cross(&ss),
                material: mat,
                mode,
                table,
                sigma_t,
                rho
            }
        } else {
            panic!("BSSRDF needs a material buddy")
        }
    }

    // Separable BSSRDF stuff
    pub fn sr(&self, r: f32) -> Spectrum {
        let mut ret = Spectrum::zeros();

        for ch in 0..3 {
            let r_optical = r * self.sigma_t[ch];
            let mut rho_offset: usize = 0;
            let mut radius_offset: usize = 0;
            let mut rho_weights = vec![0.0f32; 4];
            let mut radius_weights = vec![0.0f32; 4];

            if !catmull_rom_weights(self.table.n_rho_samples, &self.table.rho_samples, self.rho[ch], &mut rho_offset, &mut rho_weights) || !catmull_rom_weights(self.table.n_radius_samples, &self.table.radius_samples, r_optical, &mut radius_offset, &mut radius_weights) {
                    continue;
            }

            let mut sr = 0.0;
            for i in 0..4 {
                for j in 0..4 {
                    let weight = rho_weights[i] * radius_weights[j];

                    if weight != 0.0 {
                        sr += weight * self.table.eval_profile(rho_offset+i, radius_offset+j);
                    }
                }
            }

            if r_optical != 0.0 {
                sr /= 2.0 * PI * r_optical;
            }

            ret[ch] = sr;
        }

        ret.component_mul_assign(
            &self.sigma_t.component_mul(
                &self.sigma_t
            )
        );

        ret.map(|x| x.clamp(-1.0, 1.0))
    }

    pub fn sample_sr(&self, ch: usize, u: f32) -> f32 {
        if self.sigma_t[ch] == 0.0 {
            return -1.0;
        }
        
        sample_catmull_rom_2d(self.table.n_rho_samples, self.table.n_radius_samples, &self.table.rho_samples, &self.table.radius_samples, &self.table.profile, &self.table.profile_cdf, self.rho[ch], u, None, None)
    }

    pub fn pdf_sr(&self, ch: usize, r: f32) -> f32 {
        let r_optical = r * self.sigma_t[ch];

        let mut rho_offset = 0usize;
        let mut radius_offset = 0usize;
        let mut rho_weights = vec![0.0; 4];
        let mut radius_weights = vec![0.0; 4];

        if !catmull_rom_weights(self.table.n_rho_samples, &self.table.rho_samples, self.rho[ch], &mut rho_offset, &mut rho_weights) ||
        !catmull_rom_weights(self.table.n_radius_samples, &self.table.radius_samples, r_optical, &mut radius_offset, &mut radius_weights) {
            return 0.0;
        }

        let mut sr = 0.0;
        let mut rho_eff = 0.0;

        for i in 0..4 {
            if rho_weights[i] == 0.0 {
                continue;
            }

            rho_eff += self.table.rho_eff[rho_offset + i] * rho_weights[i];

            for j in 0..4 {
                if radius_weights[j] == 0.0 {
                    continue;
                }

                sr += self.table.eval_profile(rho_offset + i, radius_offset + j) * rho_weights[i] * radius_weights[j];
            }
        }

        if r_optical != 0.0 {
            sr /= 2.0 * PI * r_optical;
        }

        (sr * self.sigma_t[ch] * self.sigma_t[ch] / rho_eff).max(0.0)
    }

    fn sw(&self, w: &Vector3) -> Spectrum {
        let c = 1.0 - 2.0 * fresnel_moment1(1.0 / self.eta);
        let ret = (1.0 - fr_dielectric(cos_theta(w), 1.0, self.eta)) / (c * PI);
        
        Spectrum::new(ret, ret, ret)
    }

    fn sp(&self, pi: &SurfaceInteraction) -> Spectrum {
        self.sr((self.po_p - pi.base.p).norm())
    }

    // BSSRDF stuff
    pub fn s(&self, pi: &SurfaceInteraction, wi: &Vector3) -> Spectrum {
        let ft = 1.0 - fr_dielectric(self.po_wo.dot(&self.ns), 1.0, self.eta);
        ft * self.sp(pi).component_mul(&self.sw(wi))
    }

    pub fn sample_s(
        &self,
        // to create a bssrdf
        bssrdf: Arc<TabulatedBSSRDF>,
        // actual args 
        scene: &Scene, 
        u1: f32, u2: &Point2, 
        si: &mut SurfaceInteraction, 
        pdf: &mut f32
    ) -> Spectrum {
        let sp = self.sample_sp(scene, u1, u2, si, pdf);

        if sp == Spectrum::zeros() {
            let mut bsdf = BSDF::init(si, 1.0);
            bsdf.add(
                BxDF::BSSRDFAdapter(
                    SeparableBSSRDFAdapter::new(
                        bssrdf, 
                        self.mode.clone(), 
                        self.eta
                    )
                )
            );
            si.bsdf = Some(bsdf);
            si.base.wo = si.shading.n;
        }

        sp
    }

    pub fn sample_sp(&self, scene: &Scene, u1: f32, u2: &Point2, pi: &mut SurfaceInteraction, pdf: &mut f32) -> Spectrum {
        let vx: Vector3; // = Vector3::zeros();
        let vy: Vector3; // = Vector3::zeros();
        let vz: Vector3; // = Vector3::zeros();

        let mut u1 = u1;
        if u1 < 0.5 {
            vx = self.ss;
            vy = self.ts;
            vz = self.ns;
            u1 *= 2.0;
        } else if u1 < 0.75 {
            vx = self.ts;
            vy = self.ns;
            vz = self.ss;
            u1 = (u1 - 0.5) * 4.0;
        } else {
            vx = self.ns;
            vy = self.ss;
            vz = self.ts;
            u1 = (u1 - 0.75) * 4.0;
        }

        let ch = ((u1 * 3.0) as usize).clamp(0, 2);
        u1 = u1 * 3.0 - ch as f32;

        let r = self.sample_sr(ch, u2.x);
        if r < 0.0 {
            return Spectrum::zeros();
        }

        let phi = 2.0 * PI * u2.y;

        let r_max = self.sample_sr(ch, 0.999);
        if r > r_max {
            return Spectrum::zeros();
        }

        let l = 2.0 * (r_max*r_max - r*r).sqrt();

        let mut base = InteractionBase::new();
        base.p = self.po_p + r * (vx * phi.cos() + vy * phi.sin()) - l * vz * 0.5;
        base.time = self.po_time;
        
        let p_target = base.p + l * vz;

        let mut hits: Vec<SurfaceInteraction> = Vec::new();
        loop {
            let ray = base.spawn_ray_to(p_target);
            let mut si = SurfaceInteraction::new();

            if !scene.intersect(&ray, &mut si) {
                break;
            }

            base = si.get_base().clone();

            if let Some(mat) = si.primitive.get_material() {
                if Arc::ptr_eq(&mat, &self.material) {
                    hits.push(si);
                }
            }
        }

        if hits.is_empty() {
            return Spectrum::zeros();
        }

        let selected = ((u1 * hits.len() as f32) as usize).clamp(0, hits.len() - 1);
        *pi = hits[selected].clone();

        *pdf = self.pdf_sp(pi) / hits.len() as f32;
        self.sp(pi)
    }

    pub fn pdf_sp(&self, pi: &SurfaceInteraction) -> f32 {
        let d = pi.get_p() - self.po_p;
        let d_local = Vector3::new(
            self.ss.dot(&d), self.ts.dot(&d), self.ns.dot(&d)
        );
        let n_local = Vector3::new(
            self.ss.dot(pi.get_n()), self.ts.dot(pi.get_n()), self.ns.dot(pi.get_n())
        );

        let r_proj = [
            (d_local.y * d_local.y + d_local.z * d_local.z).sqrt(),
            (d_local.x * d_local.x + d_local.z * d_local.z).sqrt(),
            (d_local.y * d_local.y + d_local.x * d_local.x).sqrt(),
        ];

        let mut pdf = 0.0;
        let axis_prob = [0.25, 0.25, 0.5];

        let ch_prob = 1.0 / 3.0;

        for axis in 0..3 {
            for ch in 0..3 {
                pdf += self.pdf_sr(ch, r_proj[axis] * n_local[axis].abs() * ch_prob * axis_prob[axis]);
            }
        }

        pdf
    }
}

#[derive(Debug, Clone)]
pub struct BSSRDFTable {
    pub n_rho_samples: usize,
    pub n_radius_samples: usize,
    pub rho_samples: Vec<f32>,
    pub radius_samples: Vec<f32>,
    pub profile: Vec<f32>,
    pub rho_eff: Vec<f32>,
    pub profile_cdf: Vec<f32>,
}

impl BSSRDFTable {
    pub fn init(n_rho_samples: usize, n_radius_samples: usize) -> Self {
        let rho_samples: Vec<f32> = vec![0.0; n_rho_samples];
        let radius_samples: Vec<f32> = vec![0.0; n_radius_samples];
        let profile: Vec<f32> = vec![0.0; n_radius_samples * n_rho_samples];
        let rho_eff: Vec<f32> = vec![0.0; n_rho_samples];
        let profile_cdf: Vec<f32> = vec![0.0; n_rho_samples * n_radius_samples];

        Self {
            n_rho_samples,
            n_radius_samples,
            rho_samples,
            radius_samples,
            profile,
            rho_eff,
            profile_cdf,
        }
    }

    pub fn eval_profile(&self, rho_idx: usize, radius_idx: usize) -> f32 {
        self.profile[rho_idx * self.n_radius_samples + radius_idx]
    }
}

#[derive(Debug, Clone)]
pub struct SeparableBSSRDFAdapter {
    pub bssrdf: Arc<TabulatedBSSRDF>,
    pub mode: TransportMode,
    pub eta2: f32,
    typ: BxDFType,
}

impl SeparableBSSRDFAdapter {
    pub fn new(bssrdf: Arc<TabulatedBSSRDF>, mode: TransportMode, eta: f32) -> Self {
        Self {
            bssrdf,
            mode,
            eta2: eta * eta,
            typ: BxDFType::BSDF_REFLECTION | BxDFType::BSDF_DIFFUSE
        }
    }
}

impl BxDFT for SeparableBSSRDFAdapter {
    fn get_type(&self) -> BxDFType { self.typ }
    fn set_type(&mut self, typ: BxDFType) { self.typ = typ }

    fn f(&self, _wo: &Vector3, wi: &Vector3) -> Spectrum {
        let mut f = self.bssrdf.sw(wi);

        if self.mode == TransportMode::Radiance {
            f *= self.eta2;
        }

        f
    }
}

impl Manufacturable<BxDF> for SeparableBSSRDFAdapter {
    fn create_from_parameters(_param: crate::loader::Parameters) -> BxDF {
        panic!("SeparableBSSRDFAdapter isnot created from parameters")
    }
}

impl Printable for SeparableBSSRDFAdapter {
    fn to_string(&self) -> String {
        format!(
            "Separable BSSRDF Adapter []"
        )
    }
}

pub fn sample_catmull_rom_2d(
    size1: usize,
    size2: usize,
    nodes1: &Vec<f32>,
    nodes2: &Vec<f32>,
    values: &Vec<f32>,
    cdf: &Vec<f32>,
    alpha: f32,
    u: f32,
    fval: Option<&mut f32>,
    pdf: Option<&mut f32>,
) -> f32 {
    let mut offset: usize = 0;
    let mut weights = vec![0.0_f32; 4];

    if !catmull_rom_weights(size1, nodes1, alpha, &mut offset, &mut weights) {
        if let Some(f) = fval {
            *f = 0.0;
        }
        if let Some(p) = pdf {
            *p = 0.0;
        }
        return 0.0;
    }

    let interpolate = |array: &Vec<f32>, idx: usize| -> f32 {
        let mut value = 0.0;
        for i in 0..4 {
            if weights[i] != 0.0 {
                let row = offset + i;
                value += array[row * size2 + idx] * weights[i];
            }
        }
        value
    };

    let maximum = interpolate(cdf, size2 - 1);
    let mut u = u * maximum;

    let idx = find_interval(size2, |i| interpolate(cdf, i) <= u);

    let f0 = interpolate(values, idx);
    let f1 = interpolate(values, idx + 1);
    let x0 = nodes2[idx];
    let x1 = nodes2[idx + 1];
    let width = x1 - x0;

    u = (u - interpolate(cdf, idx)) / width;

    let d0 = if idx > 0 {
        width * (f1 - interpolate(values, idx - 1)) / (x1 - nodes2[idx - 1])
    } else {
        f1 - f0
    };

    let d1 = if idx + 2 < size2 {
        width * (interpolate(values, idx + 2) - f0) / (nodes2[idx + 2] - x0)
    } else {
        f1 - f0
    };

    let mut t = if f0 != f1 {
        let disc = (f0 * f0 + 2.0 * u * (f1 - f0)).max(0.0);
        (f0 - disc.sqrt()) / (f0 - f1)
    } else {
        u / f0
    };

    let mut a = 0.0_f32;
    let mut b = 1.0_f32;
    let mut fhat;
    let mut fhat_integral;

    loop {
        if !(t >= a && t <= b) {
            t = 0.5 * (a + b);
        }

        fhat_integral = t
            * (f0
                + t * (0.5 * d0
                    + t * ((1.0 / 3.0) * (-2.0 * d0 - d1)
                        + f1
                        - f0
                        + t * (0.25 * (d0 + d1) + 0.5 * (f0 - f1)))));

        fhat = f0
            + t * (d0
                + t * (-2.0 * d0 - d1 + 3.0 * (f1 - f0)
                    + t * (d0 + d1 + 2.0 * (f0 - f1))));

        if (fhat_integral - u).abs() < 1.0e-6 || (b - a) < 1.0e-6 {
            break;
        }

        if fhat_integral - u < 0.0 {
            a = t;
        } else {
            b = t;
        }

        t -= (fhat_integral - u) / fhat;
    }

    if let Some(f) = fval {
        *f = fhat;
    }
    if let Some(p) = pdf {
        *p = if maximum > 0.0 { fhat / maximum } else { 0.0 };
    }

    x0 + width * t
}

pub fn catmull_rom_weights(size: usize, nodes: &Vec<f32>, x: f32, offset: &mut usize, weights: &mut Vec<f32>) -> bool {
    if !(x >= nodes[0] && x <= nodes[size-1]) {
        return false;
    }

    let idx = find_interval(size, |i| nodes[i] <= x);

    *offset = idx - 1;
    let x0 = nodes[idx];
    let x1 = nodes[idx+1];

    let t = (x - x0) / (x1 - x0);
    let t2 = t * t;
    let t3 = t2 * t;

    weights[1] = 2.0 * t3 - 3.0 * t2 + 1.0;
    weights[2] = -2.0 * t3 + 3.0 * t2;

    if idx > 0 {
        let w0 = (t3 - 2.0 * t2 + t) * (x1 - x0) / (x1 - nodes[idx - 1]);
        weights[0] = -w0;
        weights[2] += w0;
    } else {
        let w0 = t3 - 2.0 * t2 + t;
        weights[0] = 0.0;
        weights[1] -= w0;
        weights[2] += w0;
    }

    if idx+2 < size {
        let w3 = (t3 - t2) * (x1 - x0) / (nodes[idx+2] - x0);
        weights[1] -= w3;
        weights[3] = w3;
    } else {
        let w3 = t3 - t2;
        weights[1] -= w3;
        weights[2] += w3; 
        weights[3]  = 0.0;
    }

    true
}

pub fn fresnel_moment1(eta: f32) -> f32 {
    let eta2: f32 = eta * eta;
    let eta3: f32 = eta2 * eta;
    let eta4: f32 = eta3 * eta;
    let eta5: f32 = eta4 * eta;
    if eta < 1.0 as f32 {
        0.45966 as f32 - 1.73965 as f32 * eta + 3.37668 as f32 * eta2 - 3.904_945 * eta3
            + 2.49277 as f32 * eta4
            - 0.68441 as f32 * eta5
    } else {
        -4.61686 as f32 + 11.1136 as f32 * eta - 10.4646 as f32 * eta2
            + 5.11455 as f32 * eta3
            - 1.27198 as f32 * eta4
            + 0.12746 as f32 * eta5
    }
}

pub fn fresnel_moment2(eta: f32) -> f32 {
    let eta2: f32 = eta * eta;
    let eta3: f32 = eta2 * eta;
    let eta4: f32 = eta3 * eta;
    let eta5: f32 = eta4 * eta;
    if eta < 1.0 as f32 {
        0.27614 as f32 - 0.87350 as f32 * eta + 1.12077 as f32 * eta2
            - 0.65095 as f32 * eta3
            + 0.07883 as f32 * eta4
            + 0.04860 as f32 * eta5
    } else {
        let r_eta = 1.0 as f32 / eta;
        let r_eta2 = r_eta * r_eta;
        let r_eta3 = r_eta2 * r_eta;
        -547.033 as f32 + 45.3087 as f32 * r_eta3 - 218.725 as f32 * r_eta2
            + 458.843 as f32 * r_eta
            + 404.557 as f32 * eta
            - 189.519 as f32 * eta2
            + 54.9327 as f32 * eta3
            - 9.00603 as f32 * eta4
            + 0.63942 as f32 * eta5
    }
}
