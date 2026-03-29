use std::sync::Arc;

use crate::core::scene::Scene;

pub struct Distribution1D {
    pub func_int: f32,
    pub func: Vec<f32>,
    pub cdf: Vec<f32>,
}

pub fn find_interval<F>(size: usize, pred: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let mut first = 0usize;
    let mut len = size;

    while len > 0 {
        let half = len >> 1;
        let middle = first + half;

        if pred(middle) {
            first = middle + 1;
            len -= half + 1;
        } else {
            len = half;
        }
    }

    let i = first.saturating_sub(1);
    i.clamp(0, size - 2)
}

pub fn compute_light_power_distribution(scene: &Scene) -> Arc<Distribution1D> {
    let mut light_power: Vec<f32> = Vec::new();
    for light in &scene.lights {
        light_power.push(
            light.power().y
        );
    }

    Arc::new(
        Distribution1D::init(
            &light_power,
            light_power.len()
        )
    )
}

impl Distribution1D {
    pub fn init(f: &Vec<f32>, n: usize) -> Self {
        let func: Vec<f32> = f.clone();
        let mut cdf = vec![0f32; n+1];
        for i in 1..n {
            cdf[i] = cdf[i-1] + func[i-1] / n as f32;
        }

        let func_int = cdf[n];

        if func_int == 0.0 {
            for i in 1..n {
                cdf[i] = i as f32 / n as f32;
            }
        } else {
            for i in 1..n {
                cdf[i] /= func_int;
            }
        }

        Self {
            func_int,
            func,
            cdf
        }
    }

    pub fn count(&self) -> usize { self.func.len() }

    pub fn sample_countinous(&self, u: f32, pdf: &mut f32, off: &mut usize) -> f32 {
        let offset = find_interval(self.cdf.len(), |x| self.cdf[x] <= u);

        *off = offset;
        let mut du = u - self.cdf[offset];

        if self.cdf[offset+1] - self.cdf[offset] > 0.0 {
            du /= self.cdf[offset+1] - self.cdf[offset];
        }

        *pdf = self.func[offset] / self.func_int;

        (offset as f32 + du) / self.count() as f32
    }

    pub fn sample_discrete(&self, u: f32, pdf: &mut f32, u_remapped: &mut f32) -> usize {
        let offset = find_interval(self.cdf.len(), |x| self.cdf[x] <= u);

        *pdf = self.func[offset] / (self.func_int * self.count() as f32);
        
        *u_remapped = (u - self.cdf[offset]) / (self.cdf[offset+1] - self.cdf[offset]);

        offset
    }

    pub fn discrete_pdf(&self, idx: usize) -> f32 {
        self.func[idx] / (self.func_int * self.count() as f32)
    }
}