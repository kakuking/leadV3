use crate::{core::{Point2, Vector2, texture::TextureMapping2DT}, interaction::surface_interaction::SurfaceInteraction};


#[derive(Debug)]
pub struct UVMapping2D {
    su: f32,
    sv: f32,
    du: f32,
    dv: f32,
}

impl UVMapping2D {
    pub fn init(su: f32, sv: f32, du: f32, dv: f32) -> Self {
        Self {
            su,
            sv,
            du,
            dv
        }
    }
}

impl TextureMapping2DT for UVMapping2D {
    fn map(&self, si: &SurfaceInteraction, dsdtx: &mut Vector2, dsdty: &mut Vector2) -> Point2 {
        *dsdtx = Vector2::new(
            self.su * si.dudx.get(),
            self.sv * si.dvdx.get()
        );

        *dsdty = Vector2::new(
            self.su * si.dudy.get(),
            self.sv * si.dvdy.get()
        );

        Point2::new(
            self.su * si.uv[0] + self.du, 
            self.sv * si.uv[1] + self.dv
        )
    }
}
