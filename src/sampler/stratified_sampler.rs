use crate::{core::{ONE_MINUS_EPSILON, Point2, Printable, sampler::{Sampler, SamplerBase, SamplerT}}, loader::Manufacturable, sampler::rng::RNG};

#[derive(Clone)]
pub struct StratifiedSampler {
    // base
    base: SamplerBase,

    // pixel sampler stuff
    samples_1d: Vec<Vec<f32>>,
    samples_2d: Vec<Vec<Point2>>,
    current_1d_dim: usize,
    current_2d_dim: usize,

    rng: RNG,

    // stratified stuff
    x_pixel_samples: usize,
    y_pixel_samples: usize,
    jitter_samples: bool
}

impl StratifiedSampler {
    pub fn new() -> Self {
        Self {
            base: SamplerBase::new(),

            samples_1d: Vec::new(),
            samples_2d: Vec::new(),
            current_1d_dim: 0,
            current_2d_dim: 0,
            
            rng: RNG::init(0),

            x_pixel_samples: 0,
            y_pixel_samples: 0,
            jitter_samples: false
        }
    }

    pub fn init(x_pixel_samples: usize, y_pixel_samples: usize, jitter_samples: bool, n_sampled_dimensions: usize) -> Self {
        let mut ret = StratifiedSampler::new();

        let samples_per_pixel = x_pixel_samples * y_pixel_samples;
        ret.base.samples_per_pixel = samples_per_pixel;

        for _ in 0..n_sampled_dimensions {
            ret.samples_1d.push(vec![0.0; samples_per_pixel]);
            ret.samples_2d.push(vec![Point2::origin(); samples_per_pixel]);
        }

        ret.x_pixel_samples = x_pixel_samples;
        ret.y_pixel_samples = y_pixel_samples;
        ret.jitter_samples = jitter_samples;

        ret
    }

    fn stratified_sample_1d(samp: &mut [f32], n_samples: usize, rng: &mut RNG, jitter: bool) {
        let inv_n_samples = 1.0 / n_samples as f32;

        for i in 0..n_samples {
            let delta = if jitter {
                rng.uniform_float()
            } else {
                0.5
            };

            samp[i] = ((i as f32 + delta) * inv_n_samples).min(ONE_MINUS_EPSILON);
        }
    }

    fn stratified_sample_2d(samp: &mut Vec<Point2>, nx: usize, ny: usize, rng: &mut RNG, jitter: bool) {
        let dx = 1.0 / nx as f32;
        let dy = 1.0 / ny as f32;

        let mut cur_sample: usize = 0;

        for y in 0..ny {
            for x in 0..nx {
                let jx = if jitter { rng.uniform_float() } else { 0.5 };
                let jy = if jitter { rng.uniform_float() } else { 0.5 };

                samp[cur_sample].x = ((x as f32 + jx) * dx).min(ONE_MINUS_EPSILON);
                samp[cur_sample].y = ((y as f32 + jy) * dy).min(ONE_MINUS_EPSILON);

                cur_sample += 1;
            }
        }
    }

    fn shuffle<T>(samp: &mut [T], count: usize, n_dim: usize, rng: &mut RNG) {
        for i in 0..count {
            let other = i + rng.uniform_u32_bounded((count - i) as u32) as usize;
            for j in 0..n_dim {
                samp.swap(n_dim * i + j, n_dim * other + j);
            }
        }
    }

    fn latin_hypercube(samp: &mut [Point2], n_samples: usize, rng: &mut RNG) {
    // Generate LHS samples along diagonal
        let inv_n_samples = 1.0 / n_samples as f32;
        for i in 0..n_samples {
            let sx = (i as f32 + rng.uniform_float()) * inv_n_samples;
            let sy = (i as f32 + rng.uniform_float()) * inv_n_samples;
            samp[i].x = sx.min(ONE_MINUS_EPSILON);
            samp[i].y = sy.min(ONE_MINUS_EPSILON);
        }

        // Permute LHS samples in each dimension (x, then y)
        for j in 0..n_samples {
            let other = j + rng.uniform_u32_bounded((n_samples - j) as u32) as usize;
            let tmp = samp[j].x;
            samp[j].x = samp[other].x;
            samp[other].x = tmp;
        }
        for j in 0..n_samples {
            let other = j + rng.uniform_u32_bounded((n_samples - j) as u32) as usize;
            let tmp = samp[j].y;
            samp[j].y = samp[other].y;
            samp[other].y = tmp;
        }
    }
}

impl SamplerT for StratifiedSampler {
    fn get_samples_per_pixel(&self) -> usize { self.base.samples_per_pixel }
    fn get_array_1d_offset(&self) -> usize { self.base.array_1d_offset }
    fn get_array_2d_offset(&self) -> usize { self.base.array_2d_offset }
    fn get_current_pixel(&self) -> Point2 { self.base.current_pixel }
    fn get_current_pixel_sampler_index(&self) -> usize { self.base.current_pixel_sampler_index }

    fn set_samples_per_pixel(&mut self, new_value: usize) { self.base.samples_per_pixel = new_value; }
    fn set_array_1d_offset(&mut self, new_value: usize) { self.base.array_1d_offset = new_value; }
    fn set_array_2d_offset(&mut self, new_value: usize) { self.base.array_2d_offset = new_value; }
    fn set_current_pixel(&mut self, new_value: Point2) { self.base.current_pixel = new_value; }
    fn set_current_pixel_sampler_index(&mut self, new_value: usize) { self.base.current_pixel_sampler_index = new_value; }

    fn get_samples_1d_array_sizes(&mut self) -> &mut Vec<usize> { &mut self.base.samples_1d_array_sizes }
    fn get_samples_2d_array_sizes(&mut self) -> &mut Vec<usize> { &mut self.base.samples_2d_array_sizes }
    fn get_samples_1d_array(&mut self) -> &mut Vec<Vec<f32>> { &mut self.base.samples_1d_array }
    fn get_samples_2d_array(&mut self) -> &mut Vec<Vec<Point2>> { &mut self.base.samples_2d_array }

    fn start_next_sample(&mut self) -> bool {
        self.current_1d_dim = 0;
        self.current_2d_dim = 0;

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);

        let idx = self.get_current_pixel_sampler_index();
        self.set_current_pixel_sampler_index(idx + 1);

        self.get_current_pixel_sampler_index() < self.get_samples_per_pixel()
    }

    fn set_sample_number(&mut self, sample_num: usize) -> bool {
        self.current_1d_dim = 0;
        self.current_2d_dim = 0;

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);

        self.set_current_pixel_sampler_index(sample_num);

        self.get_current_pixel_sampler_index() < self.get_samples_per_pixel()
    }

    fn get_1d(&mut self) -> f32 {
        if self.current_1d_dim < self.samples_1d.len() {
            let idx = self.current_1d_dim;
            self.current_1d_dim += 1;
            self.samples_1d[idx][self.base.current_pixel_sampler_index]
        } else {
            self.rng.uniform_float()
        }
    }

    fn get_2d(&mut self) -> Point2 {
        if self.current_1d_dim < self.samples_1d.len() {
            let idx = self.current_1d_dim;
            self.current_1d_dim += 1;
            self.samples_2d[idx][self.base.current_pixel_sampler_index]
        } else {
            Point2::new(self.rng.uniform_float(), self.rng.uniform_float())
        }
    }

    fn start_pixel(&mut self, p: Point2) {
        for i in 0..self.samples_1d.len() {
            Self::stratified_sample_1d(
                &mut self.samples_1d[i], 
                self.x_pixel_samples * self.y_pixel_samples, 
                &mut self.rng, 
                self.jitter_samples
            );

            Self::shuffle(
                &mut self.samples_1d[i], 
                self.x_pixel_samples * self.y_pixel_samples, 
                1, 
                &mut self.rng
            );
        }

        for i in 0..self.samples_2d.len() {
            Self::stratified_sample_2d(
                &mut self.samples_2d[i], 
                self.x_pixel_samples, 
                self.y_pixel_samples, 
                &mut self.rng, 
                self.jitter_samples
            );

            Self::shuffle(
                &mut self.samples_2d[i], 
                self.x_pixel_samples * self.y_pixel_samples, 
                1, 
                &mut self.rng
            );
        }

        let samples_per_pixel = self.get_samples_per_pixel();

        for i in 0..self.get_samples_1d_array_sizes().len() {
            for j in 0..samples_per_pixel {
                let count = self.get_samples_1d_array_sizes()[i];

                Self::stratified_sample_1d(
                    &mut self.base.samples_1d_array[i][j*count..], 
                    count, 
                    &mut self.rng, 
                    self.jitter_samples
                );

                Self::shuffle(
                    &mut self.base.samples_1d_array[i][j*count..], 
                    count, 
                    1, 
                    &mut self.rng
                );
            }
        }

        for i in 0..self.get_samples_2d_array_sizes().len() {
            for j in 0..samples_per_pixel {
                let count = self.get_samples_2d_array_sizes()[i];
                let offset = j * count;

                Self::latin_hypercube(&mut self.base.samples_2d_array[i][offset..], count, &mut self.rng);
            }
        }

        // pixel sampler :: start pixel
        self.set_current_pixel(p);
        self.set_current_pixel_sampler_index(0);

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);
    }

    fn clone_with_seed(&self, _seed: usize) -> Self {
        // Self::init(self.samples_per_pixel, self.samples_1d.len())
        todo!("stratified::clone_with_seed")
    }
}

impl Printable for StratifiedSampler {
    fn to_string(&self) -> String {
        format!(
            "StratifiedSampler: [\n
            \tSamples Per Pixel: {}\n
            \tJitter: {}\n
            ]",
            self.get_samples_per_pixel(),
            self.jitter_samples
        )
    }
}

impl Manufacturable<Sampler> for StratifiedSampler {
    fn create_from_parameters(param: crate::loader::Parameters) -> Sampler {
        let x_pixel_samples = param.get_int("x_pixel_samples", Some(1)) as usize;
        let y_pixel_samples = param.get_int("y_pixel_samples", Some(1)) as usize;
        let jitter_samples = param.get_bool("jitter_samples", Some(true));
        let n_sampled_dimensions = param.get_int("n_sampled_dimensions", Some(4)) as usize;

        Sampler::Stratified(
            Self::init(
                x_pixel_samples,
                y_pixel_samples,
                jitter_samples,
                n_sampled_dimensions,
            )
        )
    }
}