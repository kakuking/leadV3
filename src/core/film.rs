use std::sync::{Arc, Mutex};

use atomic_float::AtomicF32;

use crate::{core::{bounds::Bounds2, Point2, Vector2, filter::Filter, image::{write_image, write_ppm}, spectrum::{Spectrum, rgb_to_xyz, xyz_to_rgb}}, loader::Parameters, registry::{LeadObject, Manufacturable}};

use std::sync::atomic::Ordering;

pub const FILTER_TABLE_WIDTH: usize = 16;

#[derive(Debug)]
struct Pixel {
    xyz: [f32; 3],
    filter_weight_sum: f32,
    splat_xyz: [AtomicF32; 3],
    pad: f32
}

impl Pixel {
    pub fn new() -> Self {
        Self {
            xyz: [0.0, 0.0, 0.0],
            filter_weight_sum: 0.0,
            splat_xyz: [AtomicF32::new(0.0), AtomicF32::new(0.0), AtomicF32::new(0.0)],
            pad: 0.0
        }
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct FilmTilePixel {
    contrib_sum: Spectrum,
    filter_weight_sum: f32,
}

impl FilmTilePixel {
    pub fn new() -> Self {
        Self {
            contrib_sum: Spectrum::zeros(),
            filter_weight_sum: 0.0
        }
    }
}

pub struct FilmTile {
    pixel_bounds: Bounds2,
    filter_radius: Vector2,
    inv_filter_radius: Vector2,
    filter_table: Vec<f32>,
    filter_table_size: usize,
    pixels: Vec<FilmTilePixel>,
}

impl FilmTile {
    pub fn init(pixel_bounds: Bounds2, filter_radius: Vector2, filter_table: Vec<f32>, filter_table_size: usize) -> Self {
        let num_pixels = (pixel_bounds.area() as usize).max(0);
        let pixels = vec![FilmTilePixel::new(); num_pixels];

        Self {
            pixel_bounds,
            filter_radius,
            inv_filter_radius: Vector2::new(1.0 / filter_radius.x, 1.0 / filter_radius.y),
            filter_table,
            filter_table_size,
            pixels: pixels
        }
    }

    pub fn add_sample(&mut self, p_film: &Point2, l: &Spectrum, sample_weight: f32) {
        // Compute sample's raster bounds
        let p_film_discrete = p_film - Vector2::new(0.5, 0.5);

        let mut p0 = (p_film_discrete - self.filter_radius).map(|x| x.ceil());
        let mut p1 =
            (p_film_discrete + self.filter_radius).map(|x| x.floor()) + Vector2::new(1.0, 1.0);

        p0 = Point2::new(
            p0.x.max(self.pixel_bounds.p_min.x),
            p0.y.max(self.pixel_bounds.p_min.y),
        );

        p1 = Point2::new(
            p1.x.min(self.pixel_bounds.p_max.x),
            p1.y.min(self.pixel_bounds.p_max.y),
        );

        let p0x = p0.x as usize;
        let p0y = p0.y as usize;
        let p1x = p1.x as usize;
        let p1y = p1.y as usize;

        // Precompute x and y filter table offsets
        let mut ifx: Vec<usize> = Vec::with_capacity(p1x.saturating_sub(p0x));
        for x in p0x..p1x {
            let fx = ((x as f32 - p_film_discrete.x)
                * self.inv_filter_radius.x
                * self.filter_table_size as f32)
                .abs();

            ifx.push(fx.floor().min(self.filter_table_size as f32 - 1.0) as usize);
        }

        let mut ify: Vec<usize> = Vec::with_capacity(p1y.saturating_sub(p0y));
        for y in p0y..p1y {
            let fy = ((y as f32 - p_film_discrete.y)
                * self.inv_filter_radius.y
                * self.filter_table_size as f32)
                .abs();

            ify.push(fy.floor().min(self.filter_table_size as f32 - 1.0) as usize);
        }

        // Loop over filter support and add sample to pixel arrays
        let tile_width = (self.pixel_bounds.p_max.x - self.pixel_bounds.p_min.x) as usize;

        for y in p0y..p1y {
            for x in p0x..p1x {
                // Evaluate filter value at (x, y) pixel
                let offset =
                    ify[y - p0y] * self.filter_table_size +
                    ifx[x - p0x];
                let filter_weight = self.filter_table[offset];

                // Update pixel values with filtered sample contribution
                let pixel_index =
                    (y - self.pixel_bounds.p_min.y as usize) * tile_width +
                    (x - self.pixel_bounds.p_min.x as usize);

                let pixel = &mut self.pixels[pixel_index];
                pixel.contrib_sum += *l * sample_weight * filter_weight;
                pixel.filter_weight_sum += filter_weight;
            }
        }
    }

    pub fn get_mut_pixel(&mut self, p: &Point2) -> &mut FilmTilePixel {
        let width = self.pixel_bounds.p_max.x - self.pixel_bounds.p_min.x;
        let offset = (p.x - self.pixel_bounds.p_min.x) + (p.y - self.pixel_bounds.p_min.y) * width;

        &mut self.pixels[offset as usize]
    }

    pub fn get_pixel(&self, p: &Point2) -> &FilmTilePixel {
        let width = self.pixel_bounds.p_max.x - self.pixel_bounds.p_min.x;
        let offset = (p.x - self.pixel_bounds.p_min.x) + (p.y - self.pixel_bounds.p_min.y) * width;

        &self.pixels[offset as usize]
    }

    pub fn get_pixel_bounds(&self) -> Bounds2 {
        self.pixel_bounds.clone()
    }
}

pub struct Film {
    pub full_resolution: Point2,
    pub diagonal: f32,
    pub filter: Arc<Filter>,
    pub filename: String,
    pub cropped_pixel_bounds: Bounds2,

    pixels: Mutex<Vec<Pixel>>,
    filter_table: Vec<f32>,
    scale: f32
}

impl Film {
    // pub fn new() -> Self {
    //     Self {
    //         full_resolution: Point2::new(1.0, 1.0)
    //     }
    // }

    pub fn init(full_resolution: Point2, crop_window: Bounds2, filter: Arc<Filter>, diagonal: f32, filename: String, scale: f32) -> Self {
        let cropped_min = Point2::new(
            (full_resolution.x * crop_window.p_min.x).ceil(),
            (full_resolution.y * crop_window.p_min.y).ceil(),
        );
        let cropped_max = Point2::new(
            (full_resolution.x * crop_window.p_max.x).ceil(),
            (full_resolution.y * crop_window.p_max.y).ceil(),
        );

        let cropped_pixel_bounds = Bounds2::init_two(&cropped_min, &cropped_max);

        let pixels = {
            let mut vector: Vec<Pixel> = Vec::new();
            for _ in 0..cropped_pixel_bounds.area() as usize {
                vector.push(Pixel::new());
            }

            vector
        };

        let mut offset: usize = 0;
        let mut filter_table = vec![0.0; FILTER_TABLE_WIDTH*FILTER_TABLE_WIDTH];

        for y in 0..FILTER_TABLE_WIDTH {
            for x in 0..FILTER_TABLE_WIDTH {
                let mut p = Point2::origin();
                p.x = (x as f32 + 0.5) * filter.get_radius().x / FILTER_TABLE_WIDTH as f32;
                p.y = (y as f32 + 0.5) * filter.get_radius().y / FILTER_TABLE_WIDTH as f32;

                filter_table[offset] = filter.evaluate(&p);
                offset += 1;
            }
        }

        Self {
            full_resolution, 
            diagonal, 
            filter, 
            filename, 
            cropped_pixel_bounds, 
            pixels: Mutex::new(pixels), 
            filter_table, 
            scale
        }
    }

    pub fn get_sample_bounds(&self) -> Bounds2 {
        let p_min: Point2 = (self.cropped_pixel_bounds.p_min + Vector2::new(0.5, 0.5) - self.filter.get_radius()).map(|x| x.floor());
        let p_max = (self.cropped_pixel_bounds.p_max - Vector2::new(0.5, 0.5) + self.filter.get_radius()).map(|x| x.ceil());

        Bounds2::init_two(&p_min, &p_max)
    }

    pub fn get_physical_extent(&self) -> Bounds2 {
        let aspect = self.full_resolution.y / self.full_resolution.x;
        let x = (self.diagonal * self.diagonal / (1.0 + aspect * aspect)).sqrt();
        let y = aspect * x;

        Bounds2::init_two(
            &Point2::new(-x / 2.0, -y / 2.0), 
            &Point2::new(x / 2.0, y / 2.0)
        )
    }

    pub fn get_film_tile(&self, sample_bounds: &Bounds2) -> FilmTile {
        let half_pixel = Vector2::new(0.5, 0.5);
        let float_bounds = sample_bounds.clone();

        let p0 = (float_bounds.p_min - half_pixel - self.filter.get_radius()).map(|x| x.ceil());
        let p1 = (float_bounds.p_max - half_pixel + self.filter.get_radius()).map(|x| x.floor());

        let tile_pixel_bounds = Bounds2::init_two(&p0, &p1).intersect(&self.cropped_pixel_bounds);

        FilmTile::init(
            tile_pixel_bounds,
            self.filter.get_radius(),
            self.filter_table.clone(),
            FILTER_TABLE_WIDTH
        )
    }

    pub fn merge_film_tile(&self, tile: &FilmTile) {
        let pixel_bounds = tile.get_pixel_bounds();
        let mut pixels = self.pixels.lock().unwrap();

        let width = self.cropped_pixel_bounds.p_max.x - self.cropped_pixel_bounds.p_min.x;

        for y in pixel_bounds.p_min.y as usize..pixel_bounds.p_max.y as usize {
        for x in pixel_bounds.p_min.x as usize..pixel_bounds.p_max.x as usize {
            let pixel = Point2::new(x as f32, y as f32);
            let tile_pixel = tile.get_pixel(&pixel);

            let offset =
                (pixel.x - self.cropped_pixel_bounds.p_min.x)
                + (pixel.y - self.cropped_pixel_bounds.p_min.y) * width;

            let merge_pixel = &mut pixels[offset as usize];

            let mut xyz = [0.0, 0.0, 0.0];
            rgb_to_xyz(&tile_pixel.contrib_sum, &mut xyz);

            for i in 0..3 {
                merge_pixel.xyz[i] += xyz[i];
            }

            merge_pixel.filter_weight_sum += tile_pixel.filter_weight_sum;
        }
        }
    }
    
    pub fn set_image(&self, img: Vec<Spectrum>) {
        let n_pixels = self.cropped_pixel_bounds.area() as usize;
        let mut pixels = self.pixels.lock().unwrap();

        for i in 0..n_pixels {
            let p = &mut pixels[i];

            rgb_to_xyz(&img[i], &mut p.xyz);
            p.filter_weight_sum = 1.0;
            p.splat_xyz[0] = AtomicF32::new(0.0);
            p.splat_xyz[1] = AtomicF32::new(0.0);
            p.splat_xyz[2] = AtomicF32::new(0.0);
        }
    }

    pub fn add_splat(&mut self, p: Point2, v: &Spectrum) {
        if !self.cropped_pixel_bounds.inside_exclusive(&p) {
            return;
        }

        let mut xyz = [0.0, 0.0, 0.0];
        rgb_to_xyz(v, &mut xyz);
        
        let pixel_offset = self.get_pixel_offset(&p);
        let mut pixels = self.pixels.lock().unwrap();
        let pixel = &mut pixels[pixel_offset];

        for i in 0..3 {
            pixel.splat_xyz[i].fetch_add(xyz[i], Ordering::Relaxed);
        }
    }

    pub fn write_image(&mut self, splat_scale: f32) {
        let num_pixels = self.cropped_pixel_bounds.area() as usize;
        let mut rgb: Vec<f32> = Vec::new();
        for _ in 0..num_pixels {
            rgb.push(0.0);
            rgb.push(0.0);
            rgb.push(0.0);
        }

        let mut offset: usize = 0;
        let pixels = self.pixels.lock().unwrap();

        for p in &self.cropped_pixel_bounds {
            let pixel_offset = self.get_pixel_offset(&p);
            let pixel = &pixels[pixel_offset];

            xyz_to_rgb(pixel.xyz, &mut rgb[3*offset..]);

            let filter_weight_sum = pixel.filter_weight_sum;
            if filter_weight_sum != 0.0 {
                let inv_wt = 1.0 / filter_weight_sum;
                rgb[3*offset  ] = (rgb[3*offset  ] * inv_wt).max(0.0);
                rgb[3*offset+1] = (rgb[3*offset+1] * inv_wt).max(0.0);
                rgb[3*offset+2] = (rgb[3*offset+2] * inv_wt).max(0.0);
            }

            let mut splat_rgb = [0.0, 0.0, 0.0];
            let splat_xyz = [
                pixel.splat_xyz[0].load(Ordering::Relaxed), 
                pixel.splat_xyz[1].load(Ordering::Relaxed), 
                pixel.splat_xyz[2].load(Ordering::Relaxed)
            ];

            xyz_to_rgb(splat_xyz, &mut splat_rgb);

            rgb[3*offset  ] += splat_scale * splat_rgb[0];
            rgb[3*offset+1] += splat_scale * splat_rgb[1];
            rgb[3*offset+2] += splat_scale * splat_rgb[2];

            rgb[3*offset  ] *= self.scale;
            rgb[3*offset+1] *= self.scale;
            rgb[3*offset+2] *= self.scale;

            offset += 1;
        }

        match write_image(&self.filename, &rgb, self.cropped_pixel_bounds.clone(), self.full_resolution) {
            Ok(_) => println!("Successfully saved image to {}", &self.filename),
            Err(_) => println!("Could not save image to {}", &self.filename),
        }
    }

    pub fn clear(&mut self) {
        let mut pixels = self.pixels.lock().unwrap();

        for p in &self.cropped_pixel_bounds {
            let pixel_offset = self.get_pixel_offset(&p);
            let pixel = &mut pixels[pixel_offset];

            for c in 0..3 {
                pixel.splat_xyz[c] = AtomicF32::new(0.0);
                pixel.xyz[c] = 0.0;
            }

            pixel.filter_weight_sum = 0.0;
        }
    }

    fn get_pixel_offset(&self, p: &Point2) -> usize {
        let width = self.cropped_pixel_bounds.p_max.x - self.cropped_pixel_bounds.p_min.x;
        let offset = (p.x - self.cropped_pixel_bounds.p_min.x) + (p.y - self.cropped_pixel_bounds.p_min.y) * width;

        offset as usize
    }
}

impl Manufacturable<Film> for Film {
    fn create_from_parameters(params: Parameters) -> Film {
        let mut params = params;

        let full_resolution =
            params.get_point2("resolution", Some(Point2::new(1000.0, 1000.0)));

        let diagonal =
            params.get_float("diagonal", Some(35.0));

        let filename =
            params.get_string("filename", Some("output.ppm".to_string()));

        let scale =
            params.get_float("scale", Some(1.0));

        let crop_window = Bounds2::init_two(
            &Point2::new(
                params.get_float("crop_min_x", Some(0.0)),
                params.get_float("crop_min_y", Some(0.0)),
            ),
            &Point2::new(
                params.get_float("crop_max_x", Some(1.0)),
                params.get_float("crop_max_y", Some(1.0)),
            ),
        );

        let filter = match params.get_lead_object("filter") {
            Some(LeadObject::Filter(f)) => f,
            Some(_) => panic!("Parameter 'filter' exists but is not a filter"),
            None => panic!("Film requires a nested filter"),
        };

        Film::init(
            full_resolution,
            crop_window,
            Arc::new(filter),
            diagonal,
            filename,
            scale,
        )
    }
}