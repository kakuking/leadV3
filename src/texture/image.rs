use std::sync::Arc;

use crate::{core::{Printable, Vector2, spectrum::Spectrum, texture::{Texture, TextureMapping2D, TextureT}}, interaction::surface_interaction::SurfaceInteraction, loader::Parameters, registry::{LeadObject, Manufacturable}};

#[derive(Debug, Clone, PartialEq)]
pub struct ImageTexture {
    mapping: Arc<TextureMapping2D>,

    filename: String,
    image: Arc<Vec<Spectrum>>,
    width: usize,
    height: usize
}

impl ImageTexture {
    pub fn init(filename: String, mapping: Arc<TextureMapping2D>) -> Self {
        let img = image::open(&filename).unwrap().to_rgb8();

        let (w, h) = img.dimensions();

        let data: Vec<Spectrum> = img.pixels().map(|p| {
            Spectrum::new(
                p[0] as f32 / 255.0, 
                p[1] as f32 / 255.0, 
                p[2] as f32 / 255.0
            )
        }).collect();

        Self {
            mapping,

            filename,
            image: Arc::new(data),
            width: w as usize,
            height: h as usize,
        }
    }

    fn get_pixel(&self, x: usize, y: usize) -> Spectrum {
        self.image[y * self.width + x]
    }
}

impl TextureT for ImageTexture {
    fn evaluate(&self, si: &SurfaceInteraction) -> Spectrum {
        let mut dsdtx = Vector2::zeros();
        let mut dsdty = Vector2::zeros();

        let st = self.mapping.map(si, &mut dsdtx, &mut dsdty);

        let mut u = st[0];
        let mut v = st[1];

        u = u.rem_euclid(1.0);
        v = v.rem_euclid(1.0);

        let x = (u * self.width as f32) as usize;
        let y = ((1.0 - v) * self.height as f32) as usize;

        let x = x.min(self.width - 1);
        let y = y.min(self.height - 1);

        let ret= self.get_pixel(x, y);

        // println!("{}, {} ==> {:?}", x, y, ret);
        ret
    }
}

impl Manufacturable<Texture> for ImageTexture {
    fn create_from_parameters(param: Parameters) -> Texture {
        let mut param = param;

        let mapping = match param.get_lead_object("mapping") {
            Some(LeadObject::TextureMapping(m)) => m,
            _ => panic!("Checkerboard Texture requires a mapping")
        };

        let filename = param.get_string("filename", None);

        Texture::Image(
            Self::init(
                filename, 
                Arc::new(mapping)
            )
        )
    }
}

impl Printable for ImageTexture {
    fn to_string(&self) -> String {
        format!(
            "Image Texture: [\n
            \tmapping: {},\n
            \tfilename: {},\n
            \tw: {},\n
            \th: {},\n
            ",
            self.mapping.to_string(),
            self.filename,
            self.width,
            self.height
        )
    }
}