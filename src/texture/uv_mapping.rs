use crate::{core::{Point2, Printable, Vector2, texture::{TextureMapping2D, TextureMapping2DT}}, interaction::surface_interaction::SurfaceInteraction, registry::Manufacturable};


#[derive(Debug, PartialEq)]
pub struct UVMapping2D {
    su: f32,
    sv: f32,
    du: f32,
    dv: f32,
}

impl UVMapping2D {
    pub fn new() -> Self {
        Self {
            su: 0.0,
            sv: 0.0,
            du: 0.0,
            dv: 0.0
        }
    }

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

impl Manufacturable<TextureMapping2D> for UVMapping2D {
    fn create_from_parameters(param: crate::loader::Parameters) -> TextureMapping2D {
        let su = param.get_float("su", Some(1.0));
        let sv = param.get_float("sv", Some(1.0));
        let du = param.get_float("du", Some(0.0));
        let dv = param.get_float("dv", Some(0.0));

        TextureMapping2D::UV(
            UVMapping2D::init(su, sv, du, dv)
        )
    }
}

impl Printable for UVMapping2D {
    fn to_string(&self) -> String {
        format!("UV Mapping []")
    }
}
