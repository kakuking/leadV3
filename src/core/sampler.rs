use crate::{core::{Point2, Printable, camera::CameraSample}, registry::Manufacturable, sampler::{halton::HaltonSampler, stratified::StratifiedSampler}};

#[derive(Clone)]
pub enum Sampler {
    Empty,
    Stratified(StratifiedSampler),
    Halton(HaltonSampler)
}

impl Sampler {
    pub fn get_samples_per_pixel(&self) -> usize {
        match self {
            Self::Stratified(s) => s.get_samples_per_pixel(),
            Self::Halton(s) => s.get_samples_per_pixel(),
            Self::Empty => panic!("No sampler specified. Calling get_spp")
        }
    }

    pub fn get_array_1d_offset(&self) -> usize {
        match self {
            Self::Stratified(s) => s.get_array_1d_offset(),
            Self::Halton(s) => s.get_array_1d_offset(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_array_2d_offset(&self) -> usize {
        match self {
            Self::Stratified(s) => s.get_array_2d_offset(),
            Self::Halton(s) => s.get_array_2d_offset(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_current_pixel(&self) -> Point2 {
        match self {
            Self::Stratified(s) => s.get_current_pixel(),
            Self::Halton(s) => s.get_current_pixel(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_current_pixel_sampler_index(&self) -> usize {
        match self {
            Self::Stratified(s) => s.get_current_pixel_sampler_index(),
            Self::Halton(s) => s.get_current_pixel_sampler_index(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn set_samples_per_pixel(&mut self, new_value: usize) {
        match self {
            Self::Stratified(s) => s.set_samples_per_pixel(new_value),
            Self::Halton(s) => s.set_samples_per_pixel(new_value),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn set_array_1d_offset(&mut self, new_value: usize) {
        match self {
            Self::Stratified(s) => s.set_array_1d_offset(new_value),
            Self::Halton(s) => s.set_array_1d_offset(new_value),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn set_array_2d_offset(&mut self, new_value: usize) {
        match self {
            Self::Stratified(s) => s.set_array_2d_offset(new_value),
            Self::Halton(s) => s.set_array_2d_offset(new_value),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn set_current_pixel(&mut self, new_value: Point2) {
        match self {
            Self::Stratified(s) => s.set_current_pixel(new_value),
            Self::Halton(s) => s.set_current_pixel(new_value),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn set_current_pixel_sampler_index(&mut self, new_value: usize) {
        match self {
            Self::Stratified(s) => s.set_current_pixel_sampler_index(new_value),
            Self::Halton(s) => s.set_current_pixel_sampler_index(new_value),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_samples_1d_array_sizes(&mut self) -> &mut Vec<usize> {
        match self {
            Self::Stratified(s) => s.get_samples_1d_array_sizes(),
            Self::Halton(s) => s.get_samples_1d_array_sizes(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_samples_2d_array_sizes(&mut self) -> &mut Vec<usize> {
        match self {
            Self::Stratified(s) => s.get_samples_2d_array_sizes(),
            Self::Halton(s) => s.get_samples_2d_array_sizes(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_samples_1d_array(&mut self) -> &mut Vec<Vec<f32>> {
        match self {
            Self::Stratified(s) => s.get_samples_1d_array(),
            Self::Halton(s) => s.get_samples_1d_array(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_samples_2d_array(&mut self) -> &mut Vec<Vec<Point2>> {
        match self {
            Self::Stratified(s) => s.get_samples_2d_array(),
            Self::Halton(s) => s.get_samples_2d_array(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_1d(&mut self) -> f32 {
        match self {
            Self::Stratified(s) => s.get_1d(),
            Self::Halton(s) => s.get_1d(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_2d(&mut self) -> Point2 {
        match self {
            Self::Stratified(s) => s.get_2d(),
            Self::Halton(s) => s.get_2d(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn clone_with_seed(&self, seed: usize) -> Self {
        match self {
            Self::Stratified(s) => Self::Stratified(s.clone_with_seed(seed)),
            Self::Halton(s) => Self::Halton(s.clone_with_seed(seed)),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn start_pixel(&mut self, p: Point2) {
        match self {
            Self::Stratified(s) => s.start_pixel(p),
            Self::Halton(s) => s.start_pixel(p),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_camera_sample(&mut self, p_raster: &Point2) -> CameraSample {
        match self {
            Self::Stratified(s) => s.get_camera_sample(p_raster),
            Self::Halton(s) => s.get_camera_sample(p_raster),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn request_1d_array(&mut self, n: usize) {
        match self {
            Self::Stratified(s) => s.request_1d_array(n),
            Self::Halton(s) => s.request_1d_array(n),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn request_2d_array(&mut self, n: usize) {
        match self {
            Self::Stratified(s) => s.request_2d_array(n),
            Self::Halton(s) => s.request_2d_array(n),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn round_count(&self, n: usize) -> usize {
        match self {
            Self::Stratified(s) => s.round_count(n),
            Self::Halton(s) => s.round_count(n),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_1d_array(&mut self, n: usize) -> Vec<f32> {
        match self {
            Self::Stratified(s) => s.get_1d_array(n),
            Self::Halton(s) => s.get_1d_array(n),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn get_2d_array(&mut self, n: usize) -> Vec<Point2> {
        match self {
            Self::Stratified(s) => s.get_2d_array(n),
            Self::Halton(s) => s.get_2d_array(n),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn start_next_sample(&mut self) -> bool {
        match self {
            Self::Stratified(s) => s.start_next_sample(),
            Self::Halton(s) => s.start_next_sample(),
            Self::Empty => panic!("No sampler specified.")
        }
    }

    pub fn set_sample_number(&mut self, sample_num: usize) -> bool {
        match self {
            Self::Stratified(s) => s.set_sample_number(sample_num),
            Self::Halton(s) => s.set_sample_number(sample_num),
            Self::Empty => panic!("No sampler specified.")
        }
    }
}

impl Printable for Sampler {
    fn to_string(&self) -> String {
        match self {
            Self::Stratified(s) => s.to_string(),
            Self::Halton(s) => s.to_string(),
            Self::Empty => "Empty Sampler".to_string()
        }
    }
}

#[derive(Clone)]
pub struct SamplerBase {
    pub samples_per_pixel: usize,

    pub array_1d_offset: usize,
    pub array_2d_offset: usize,
    pub current_pixel: Point2,
    pub current_pixel_sampler_index: usize,
    pub samples_1d_array_sizes: Vec<usize>,
    pub samples_2d_array_sizes: Vec<usize>,
    pub samples_1d_array: Vec<Vec<f32>>,
    pub samples_2d_array: Vec<Vec<Point2>>,
}

impl SamplerBase {
    pub fn new() -> Self {
        Self{
            samples_per_pixel: 0,
            array_1d_offset: 0,
            array_2d_offset: 0,
            current_pixel: Point2::origin(),
            current_pixel_sampler_index: 0,
            samples_1d_array_sizes: Vec::new(),
            samples_2d_array_sizes: Vec::new(),
            samples_1d_array: Vec::new(),
            samples_2d_array: Vec::new(),
        }
    }
}
    
pub trait PixelSamplerT: Manufacturable<Sampler> + Printable {
    fn get_samples_per_pixel(&self) -> usize;
    fn get_array_1d_offset(&self) -> usize;
    fn get_array_2d_offset(&self) -> usize;
    fn get_current_pixel(&self) -> Point2;
    fn get_current_pixel_sampler_index(&self) -> usize;

    fn set_samples_per_pixel(&mut self, new_value: usize);
    fn set_array_1d_offset(&mut self, new_value: usize);
    fn set_array_2d_offset(&mut self, new_value: usize);
    fn set_current_pixel(&mut self, new_value: Point2);
    fn set_current_pixel_sampler_index(&mut self, new_value: usize);

    fn get_samples_1d_array_sizes(&mut self) -> &mut Vec<usize>;
    fn get_samples_2d_array_sizes(&mut self) -> &mut Vec<usize>;
    fn get_samples_1d_array(&mut self) -> &mut Vec<Vec<f32>>;
    fn get_samples_2d_array(&mut self) -> &mut Vec<Vec<Point2>>;

    fn get_1d(&mut self) -> f32;
    fn get_2d(&mut self) -> Point2;
    fn clone_with_seed(&self, seed: usize) -> Self;

    fn start_pixel(&mut self, p: Point2) {
        self.set_current_pixel(p);
        self.set_current_pixel_sampler_index(0);

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);
    }

    fn get_camera_sample(&mut self, p_raster: &Point2) -> CameraSample {
        let mut cs = CameraSample::new();
        cs.p_film = (p_raster - self.get_2d()).into();
        cs.time = self.get_1d();
        cs.p_lens = self.get_2d().into();

        cs
    }

    fn request_1d_array(&mut self, n: usize) {
        self.get_samples_1d_array_sizes().push(n);
        let spp = self.get_samples_per_pixel();
        self.get_samples_1d_array().push(
            vec![0.0; n * spp]
        );
    }

    fn request_2d_array(&mut self, n: usize) {
        self.get_samples_2d_array_sizes().push(n);
        let spp = self.get_samples_per_pixel();
        self.get_samples_2d_array().push(
            vec![Point2::origin(); n * spp]
        );
    }

    fn round_count(&self, n: usize) -> usize { n }
    
    fn get_1d_array(&mut self, n: usize) -> Vec<f32> {
        if self.get_array_1d_offset() == self.get_samples_1d_array().len() {
            return Vec::new();
        }

        let idx = self.get_array_1d_offset();
        self.set_array_1d_offset(idx + 1);

        let start = self.get_current_pixel_sampler_index() * n;
        let end = start + n;

        self.get_samples_1d_array()[idx][start..end].to_vec()
    }

    fn get_2d_array(&mut self, n: usize) -> Vec<Point2> {
        if self.get_array_2d_offset() == self.get_samples_2d_array().len() {
            return Vec::new();
        }

        let idx = self.get_array_2d_offset();
        self.set_array_2d_offset(idx + 1);

        let start = self.get_current_pixel_sampler_index() * n;
        let end = start + n;

        self.get_samples_2d_array()[idx][start..end].to_vec()
    }

    fn start_next_sample(&mut self) -> bool {
        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);

        let idx = self.get_current_pixel_sampler_index();
        self.set_current_pixel_sampler_index(idx + 1);

        self.get_current_pixel_sampler_index() < self.get_samples_per_pixel()
    }

    fn set_sample_number(&mut self, sample_num: usize) -> bool {
        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);

        self.set_current_pixel_sampler_index(sample_num);

        self.get_current_pixel_sampler_index() < self.get_samples_per_pixel()
    }
}

pub trait GlobalSamplerT: Manufacturable<Sampler> + Printable {
    fn get_samples_per_pixel(&self) -> usize;
    fn get_array_1d_offset(&self) -> usize;
    fn get_array_2d_offset(&self) -> usize;
    fn get_current_pixel(&self) -> Point2;
    fn get_current_pixel_sampler_index(&self) -> usize;
    fn get_dimension(&self) -> usize;
    fn get_interval_sample_index(&self) -> usize;
    fn get_array_start_dim(&self) -> usize { 5 }
    fn get_array_end_dim(&self) -> usize;

    fn set_samples_per_pixel(&mut self, new_value: usize);
    fn set_array_1d_offset(&mut self, new_value: usize);
    fn set_array_2d_offset(&mut self, new_value: usize);
    fn set_current_pixel(&mut self, new_value: Point2);
    fn set_current_pixel_sampler_index(&mut self, new_value: usize);
    fn set_dimension(&mut self, value: usize);
    fn set_interval_sample_index(&mut self, value: usize);
    fn set_array_end_dim(&mut self, value: usize);

    fn get_samples_1d_array_sizes(&mut self) -> &mut Vec<usize>;
    fn get_samples_2d_array_sizes(&mut self) -> &mut Vec<usize>;
    fn get_samples_1d_array(&mut self) -> &mut Vec<Vec<f32>>;
    fn get_samples_2d_array(&mut self) -> &mut Vec<Vec<Point2>>;

    fn clone_with_seed(&self, seed: usize) -> Self;

    fn get_1d(&mut self) -> f32 {
        if self.get_dimension() >= self.get_array_start_dim() && self.get_dimension() < self.get_array_end_dim() {
            self.set_dimension(self.get_array_end_dim());
        }

        self.set_dimension(self.get_dimension()+1);
        
        self.sample_dimension(
            self.get_interval_sample_index(), 
            self.get_dimension()-1
        )
    }
    
    fn get_2d(&mut self) -> Point2 {
        if self.get_dimension() + 1 >= self.get_array_start_dim() && self.get_dimension() < self.get_array_end_dim() {
            self.set_dimension(self.get_array_end_dim());
        }

        let p = Point2::new(
            self.sample_dimension(
                self.get_interval_sample_index(), 
                self.get_dimension()
            ),
            self.sample_dimension(
                self.get_interval_sample_index(), 
                self.get_dimension()+1
            )
        );
        self.set_dimension(self.get_dimension()+2);
        p
    }

    fn start_pixel(&mut self, p: Point2) {
        // Sampler::start_pixel
        self.set_current_pixel(p);
        self.set_current_pixel_sampler_index(0);

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);
        self.set_dimension(0);
        let idx_for_samp = self.get_index_for_sample(0);
        self.set_interval_sample_index(idx_for_samp);

        let new_array_end_dim = self.get_array_start_dim() + self.get_samples_1d_array().len() +  2 * self.get_samples_2d_array().len();

        self.set_array_end_dim(new_array_end_dim);

        for i in 0..self.get_samples_1d_array_sizes().len() {
            let n_samples = self.get_samples_1d_array_sizes()[i] * self.get_samples_per_pixel();

            for j in 0..n_samples {
                let idx = self.get_index_for_sample(j);

                self.get_samples_1d_array()[i][j] = self.sample_dimension(idx, self.get_array_start_dim() + i);
            }
        }

        let mut dim = self.get_array_start_dim() + self.get_samples_1d_array_sizes().len();
        for i in 0..self.get_samples_2d_array_sizes().len() {
            let n_samples = self.get_samples_2d_array_sizes()[i] * self.get_samples_per_pixel();

            for j in 0..n_samples {
                let idx = self.get_index_for_sample(j);
                self.get_samples_2d_array()[i][j].x = self.sample_dimension(idx, dim);
                self.get_samples_2d_array()[i][j].y = self.sample_dimension(idx, dim+1);
            }
            dim += 2;
        }

        assert!(dim == self.get_array_end_dim())
    }

    fn get_camera_sample(&mut self, p_raster: &Point2) -> CameraSample {
        let mut cs = CameraSample::new();
        cs.p_film = (p_raster - self.get_2d()).into();
        cs.time = self.get_1d();
        cs.p_lens = self.get_2d().into();

        cs
    }

    fn request_1d_array(&mut self, n: usize) {
        self.get_samples_1d_array_sizes().push(n);
        let spp = self.get_samples_per_pixel();
        self.get_samples_1d_array().push(
            vec![0.0; n * spp]
        );
    }

    fn request_2d_array(&mut self, n: usize) {
        self.get_samples_2d_array_sizes().push(n);
        let spp = self.get_samples_per_pixel();
        self.get_samples_2d_array().push(
            vec![Point2::origin(); n * spp]
        );
    }
    
    fn round_count(&self, n: usize) -> usize { n }
    
    fn get_1d_array(&mut self, n: usize) -> Vec<f32> {
        if self.get_array_1d_offset() == self.get_samples_1d_array().len() {
            return Vec::new();
        }

        let idx = self.get_array_1d_offset();
        self.set_array_1d_offset(idx + 1);

        let start = self.get_current_pixel_sampler_index() * n;
        let end = start + n;

        self.get_samples_1d_array()[idx][start..end].to_vec()
    }

    fn get_2d_array(&mut self, n: usize) -> Vec<Point2> {
        if self.get_array_2d_offset() == self.get_samples_2d_array().len() {
            return Vec::new();
        }

        let idx = self.get_array_2d_offset();
        self.set_array_2d_offset(idx + 1);

        let start = self.get_current_pixel_sampler_index() * n;
        let end = start + n;

        self.get_samples_2d_array()[idx][start..end].to_vec()
    }

    fn start_next_sample(&mut self) -> bool {
        self.set_dimension(0);
        let samp = self.get_current_pixel_sampler_index() + 1;
        let idx_for_samp = self.get_index_for_sample(samp);
        self.set_interval_sample_index(idx_for_samp);

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);

        let idx = self.get_current_pixel_sampler_index();
        self.set_current_pixel_sampler_index(idx + 1);

        self.get_current_pixel_sampler_index() < self.get_samples_per_pixel()
    }

    fn set_sample_number(&mut self, sample_num: usize) -> bool {
        self.set_dimension(0);
        let idx_for_sample = self.get_index_for_sample(sample_num);
        self.set_interval_sample_index(idx_for_sample);

        self.set_array_1d_offset(0);
        self.set_array_2d_offset(0);

        self.set_current_pixel_sampler_index(sample_num);

        self.get_current_pixel_sampler_index() < self.get_samples_per_pixel()
    }

    fn get_index_for_sample(&mut self, sample_num: usize) -> usize;
    fn sample_dimension(&self, idx: usize, dimension: usize) -> f32;
}