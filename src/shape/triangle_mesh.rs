use std::sync::Arc;

use crate::{core::{bounds::Bounds3, Normal3, Point2, Point3, Printable, Transform, Vector3, apply_transform_to_normal, coordinate_system, face_forward, gamma, interaction::{Interaction, InteractionBase}, permute_p, permute_v, random::uniform_sample_triangle, shape::{Shape, ShapeT}, texture::Texture}, interaction::surface_interaction::SurfaceInteraction, loader::Parameters, registry::Manufacturable};

#[derive(Debug)]
pub struct TriangleMesh {
    pub object_to_world: Arc<Transform>,
    pub n_triangles: usize,
    pub n_vertices: usize,
    pub vertex_indices: Vec<usize>,
    pub p: Vec<Point3>,
    pub n: Vec<Normal3>,
    pub s: Vec<Vector3>,
    pub uv: Vec<Point2>,
    pub name: String,
    pub alpha_mask: Option<Arc<dyn Texture<f32>>>,
}

impl TriangleMesh {
    pub fn init(
        object_to_world: Arc<Transform>,
        n_triangles: usize,
        vertex_indices: Vec<usize>,
        n_vertices: usize,
        p: Vec<Point3>,
        s: Vec<Vector3>,
        n: Vec<Normal3>,
        uv: Vec<Point2>,
        name: Option<String>,
        // alpha_mask: Option<Arc<dyn Texture>>
    ) -> Self {
        // Transform vertices to world space
        let world_p: Vec<Point3> = p.iter()
            .map(|pt| object_to_world.transform_point(pt))
            .collect();

        // Copy UV if present
        let world_uv = if uv.is_empty() { vec![] } else { uv };

        // Transform normals to world space if present
        let world_n: Vec<Normal3> = if n.is_empty() {
            vec![]
        } else {
            n.iter()
                .map(|normal| apply_transform_to_normal(normal, &object_to_world))
                .collect()
        };

        // Transform tangents to world space if present
        let world_s: Vec<Vector3> = if s.is_empty() {
            vec![]
        } else {
            s.iter()
                .map(|tangent| object_to_world.transform_vector(tangent))
                .collect()
        };

        Self {
            object_to_world: Arc::new(Transform::identity()),
            n_triangles,
            n_vertices,
            vertex_indices,
            p: world_p,
            s: world_s,
            n: world_n,
            uv: world_uv,
            name: name.unwrap_or("Default_Mesh".to_string()),
            alpha_mask: None
        }
    }

    pub fn create_from_parameters(params: Parameters) -> Vec<Shape> {
        let filename = params.get_string("filename", Some("cube.obj".to_string()));
        let object_to_world = params.get_transform();

        let mesh = Self::load_from_file(filename, object_to_world);
        Self::to_triangles(&Arc::new(mesh))
    }

    pub fn load_from_file(filename: String, object_to_world: Transform) -> Self {
        let load_options = tobj::LoadOptions {
            triangulate: true,   // IMPORTANT: ensures triangles
            single_index: true,  // positions/normals/uv share indices
            ..Default::default()
        };

        let (models, _materials) =
            tobj::load_obj(&filename, &load_options)
                .expect("Failed to load OBJ file");

        // For simplicity: take first mesh
        let mesh = &models[0].mesh;

        // Positions
        let mut p = Vec::new();
        for i in 0..mesh.positions.len() / 3 {
            p.push(Point3::new(
                mesh.positions[3 * i],
                mesh.positions[3 * i + 1],
                mesh.positions[3 * i + 2],
            ));
        }

        // Normals
        let mut n = Vec::new();
        if !mesh.normals.is_empty() {
            for i in 0..mesh.normals.len() / 3 {
                n.push(Normal3::new(
                    mesh.normals[3 * i],
                    mesh.normals[3 * i + 1],
                    mesh.normals[3 * i + 2],
                ));
            }
        }

        // UVs
        let mut uv = Vec::new();
        if !mesh.texcoords.is_empty() {
            for i in 0..mesh.texcoords.len() / 2 {
                uv.push(Point2::new(
                    mesh.texcoords[2 * i],
                    mesh.texcoords[2 * i + 1],
                ));
            }
        }

        // Tangents (PBRT often computes these later → leave empty)
        let s: Vec<Vector3> = vec![];

        // Indices
        let vertex_indices: Vec<usize> =
            mesh.indices.iter().map(|&i| i as usize).collect();

        let n_triangles = vertex_indices.len() / 3;
        let n_vertices = p.len();

        // Identity transform for now
        Self::init(
            Arc::new(object_to_world),
            n_triangles,
            vertex_indices,
            n_vertices,
            p,
            s,
            n,
            uv,
            Some(filename),
        )
    }

    pub fn to_triangles(self: &Arc<Self>) -> Vec<Shape> {
        let mut tris = Vec::with_capacity(self.n_triangles);

        for i in 0..self.n_triangles {
            let idx = 3 * i;

            let v0 = self.vertex_indices[idx];
            let v1 = self.vertex_indices[idx + 1];
            let v2 = self.vertex_indices[idx + 2];

            let tri = Triangle::init(
                Arc::clone(self),
                [v0, v1, v2],
            );

            tris.push(Shape::Triangle(tri));
        }

        tris
    }

    pub fn to_string(&self) -> String {
        format!(
            "TriangleMesh:[\n
            \tName: {},\n
            \tNum Triangles: {},\n
            \tNum Vertices: {},\n
            ",
            self.name,
            self.n_triangles,
            self.n_vertices
        )
    }
}

#[derive(Debug, Clone)]
pub struct Triangle {
    // object_to_world: Arc<Transform>,
    // world_to_object: Arc<Transform>,
    // reverse_orientation: bool,
    
    v: Vec<usize>,
    mesh: Arc<TriangleMesh>,
}

impl Triangle {
    pub fn init(mesh: Arc<TriangleMesh>, tri_number: [usize; 3]) -> Self {
        let v = tri_number.to_vec();

        Self {
            v,
            mesh: mesh,
        }
    }

    fn get_uvs(&self, uv: &mut [Point2; 3]) {
        if self.mesh.uv.len() > 0 {
            uv[0] = self.mesh.uv[self.v[0]];
            uv[1] = self.mesh.uv[self.v[1]];
            uv[2] = self.mesh.uv[self.v[2]];
        } else {
            uv[0] = Point2::new(0.0, 0.0);
            uv[1] = Point2::new(1.0, 0.0);
            uv[2] = Point2::new(1.0, 1.0);
        }
    }
}

impl Printable for Triangle {
    fn to_string(&self) -> String {
        format!(
            "Triangle: [\n
            \tv: {}, {}, {}\n
            ]",
            self.v[0],
            self.v[1],
            self.v[2],
        )
    }
}

impl Manufacturable<Shape> for Triangle {
    fn create_from_parameters(_param: crate::loader::Parameters) -> Shape {
        panic!("Triangle is not manufacturable!")
    }
}

impl ShapeT for Triangle {
    fn get_object_to_world(&self) -> &Transform { &self.mesh.object_to_world }
    fn get_world_to_object(&self) -> &Transform { panic!("Dont call get_world_to_object on a triangle"); }
    fn get_reverse_orientation(&self) -> bool { panic!("Dont call get_reverse_orientation on a triangle"); }

    fn object_bounds(&self) -> Bounds3 {
        self.world_bounds()
    }

    fn world_bounds(&self) -> Bounds3 {
        let p0 = &self.mesh.p[self.v[0]];
        let p1 = &self.mesh.p[self.v[1]];
        let p2 = &self.mesh.p[self.v[2]];

        Bounds3::init_two(p0, p1).union_p(p2)
    }
    
    fn intersect(&self, ray: &crate::core::Ray, t_hit: &mut f32, isect: &mut crate::interaction::surface_interaction::SurfaceInteraction, _test_alpha_texture: Option<bool>) -> bool {
        let p0 = &self.mesh.p[self.v[0]];
        let p1 = &self.mesh.p[self.v[1]];
        let p2 = &self.mesh.p[self.v[2]];
        let mut p0_t: Point3 = (p0 - ray.o).into();
        let mut p1_t: Point3 = (p1 - ray.o).into();
        let mut p2_t: Point3 = (p2 - ray.o).into();
        let kz = ray.d.abs().imax();
        let kx = if kz + 1 != 3 { kz + 1 } else { 0 };
        let ky = if kx + 1 != 3 { kx + 1 } else { 0 };
        let d = permute_v(&ray.d, kx, ky, kz);
        p0_t = permute_p(&p0_t, kx, ky, kz);
        p1_t = permute_p(&p1_t, kx, ky, kz);
        p2_t = permute_p(&p2_t, kx, ky, kz);
        let sx = -d.x / d.z;
        let sy = -d.y / d.z;
        let sz = 1.0 / d.z;
        p0_t.x += sx * p0_t.z;
        p0_t.y += sy * p0_t.z;
        p1_t.x += sx * p1_t.z;
        p1_t.y += sy * p1_t.z;
        p2_t.x += sx * p2_t.z;
        p2_t.y += sy * p2_t.z;
        let mut e_0 = p1_t.x * p2_t.y - p1_t.y * p2_t.x;
        let mut e_1 = p2_t.x * p0_t.y - p2_t.y * p0_t.x;
        let mut e_2 = p0_t.x * p1_t.y - p0_t.y * p1_t.x;
        if e_0 == 0.0 || e_1 == 0.0 || e_2 == 0.0 {
            let p2txp1ty: f64 = p2_t.x as f64 * p1_t.y as f64;
            let p2typ1tx: f64 = p2_t.y as f64 * p1_t.x as f64;
            e_0 = (p2typ1tx - p2txp1ty) as f32;
            let p0txp2ty: f64 = p0_t.x as f64 * p2_t.y as f64;
            let p0typ2tx: f64 = p0_t.y as f64 * p2_t.x as f64;
            e_1 = (p0typ2tx - p0txp2ty) as f32;
            let p1txp0ty: f64 = p1_t.x as f64 * p0_t.y as f64;
            let p1typ0tx: f64 = p1_t.y as f64 * p0_t.x as f64;
            e_2 = (p1typ0tx - p1txp0ty) as f32;
        }
        if (e_0 < 0.0 || e_1 < 0.0 || e_2 < 0.0) && (e_0 > 0.0 || e_1 > 0.0 || e_2 > 0.0) {
            return false;
        }
        let det = e_0 + e_1 + e_2;
        if det == 0.0 { return false; }
        p0_t.z *= sz;
        p1_t.z *= sz;
        p2_t.z *= sz;
        let t_scaled = e_0 * p0_t.z + e_1 * p1_t.z + e_2 * p2_t.z;
        if det < 0.0 && (t_scaled >= 0.0 || t_scaled < ray.t_max.get() * det) {
            return false;
        } else if det > 0.0 && (t_scaled <= 0.0 || t_scaled > ray.t_max.get() * det) {
            return false;
        }

        // Barycentric coordinates and t
        let inv_det = 1.0 / det;
        let b0 = e_0 * inv_det;
        let b1 = e_1 * inv_det;
        let b2 = e_2 * inv_det;
        let t = t_scaled * inv_det;

        // Error bounds
        let max_zt = Vector3::new(p0_t.z, p1_t.z, p2_t.z).abs().max();
        let delta_z = gamma(3.0) * max_zt;
        let max_xt = Vector3::new(p0_t.x, p1_t.x, p2_t.x).abs().max();
        let max_yt = Vector3::new(p0_t.y, p1_t.y, p2_t.y).abs().max();
        let delta_x = gamma(5.0) * (max_xt + max_zt);
        let delta_y = gamma(5.0) * (max_yt + max_zt);
        let delta_e = 2.0 * (gamma(2.0) * max_xt * max_yt + delta_y * max_xt + delta_x * max_yt);
        let max_e = Vector3::new(e_0, e_1, e_2).abs().max();
        let delta_t = 3.0 * (gamma(3.0) * max_e * max_zt + delta_e * max_zt + delta_z * max_e) * inv_det.abs();
        if t <= delta_t { return false; }

        // Partial derivatives
        let mut uv = [Point2::origin(), Point2::origin(), Point2::origin()];
        let mut dpdu: Vector3 = Vector3::zeros();
        let mut dpdv: Vector3 = Vector3::zeros();
        self.get_uvs(&mut uv);
        let duv02 = uv[0] - uv[2];
        let duv12 = uv[1] - uv[2];
        let dp02 = p0 - p2;
        let dp12 = p1 - p2;
        let determinant = duv02[0] * duv12[1] - duv02[1] * duv12[0];
        if determinant == 0.0 {
            coordinate_system(&(p2 - p0).cross(&(p1 - p0)).normalize(), &mut dpdu, &mut dpdv);
        } else {
            let inv_det = 1.0 / determinant;
            dpdu = ( duv12[1] * dp02 - duv02[1] * dp12) * inv_det;
            dpdv = (-duv12[0] * dp02 + duv02[0] * dp12) * inv_det;
        };

        // Error bounds for hit point
        let x_abs_sum = (b0 * p0.x).abs() + (b1 * p1.x).abs() + (b2 * p2.x).abs();
        let y_abs_sum = (b0 * p0.y).abs() + (b1 * p1.y).abs() + (b2 * p2.y).abs();
        let z_abs_sum = (b0 * p0.z).abs() + (b1 * p1.z).abs() + (b2 * p2.z).abs();
        let p_error = gamma(7.0) * Vector3::new(x_abs_sum, y_abs_sum, z_abs_sum);

        // Hit point and UV
        let p_hit = Point3::from(b0 * p0.coords + b1 * p1.coords + b2 * p2.coords);
        let uv_hit = Point2::from(
            b0 * uv[0].coords + b1 * uv[1].coords + b2 * uv[2].coords
        );

        // Fill SurfaceInteraction
        *isect = SurfaceInteraction::init(
            &p_hit, &p_error, &uv_hit, &(-ray.d),
            &dpdu, &dpdv,
            &Normal3::zeros(), &Normal3::zeros(),
            ray.time, None
        );

        // Override normal
        let geo_n = Normal3::from(dp02.cross(&dp12).normalize());
        isect.base.n = geo_n;
        isect.shading.n = geo_n;

        // Shading geometry from mesh normals/tangents
        if !self.mesh.n.is_empty() || !self.mesh.s.is_empty() {
            let ns = if !self.mesh.n.is_empty() {
                Normal3::from((b0 * self.mesh.n[self.v[0]] + b1 * self.mesh.n[self.v[1]] + b2 * self.mesh.n[self.v[2]]).normalize())
            } else {
                isect.base.n
            };

            let mut ss = if !self.mesh.s.is_empty() {
                (b0 * self.mesh.s[self.v[0]] + b1 * self.mesh.s[self.v[1]] + b2 * self.mesh.s[self.v[2]]).normalize()
            } else {
                dpdu.normalize()
            };

            let mut ts = ns.cross(&ss);
            if ts.norm_squared() > 0.0 {
                ts = ts.normalize();
                ss = ts.cross(&ns);
            } else {
                coordinate_system(&Vector3::from(ns), &mut ss, &mut ts);
            }

            let (dndu, dndv) = if !self.mesh.n.is_empty() {
                let dn1 = self.mesh.n[self.v[0]] - self.mesh.n[self.v[2]];
                let dn2 = self.mesh.n[self.v[1]] - self.mesh.n[self.v[2]];
                let det = duv02[0] * duv12[1] - duv02[1] * duv12[0];
                if det == 0.0 {
                    (Normal3::zeros(), Normal3::zeros())
                } else {
                    let inv_det = 1.0 / det;
                    (
                        Normal3::from(( duv12[1] * dn1 - duv02[1] * dn2) * inv_det),
                        Normal3::from((-duv12[0] * dn1 + duv02[0] * dn2) * inv_det),
                    )
                }
            } else {
                (Normal3::zeros(), Normal3::zeros())
            };

            isect.set_shading_geometry(&ss, &ts, &dndu, &dndv, true);
        }

        // Ensure correct orientation of geometric normal
        if !self.mesh.n.is_empty() {
            isect.base.n = face_forward(&isect.base.n, &isect.shading.n);
        } else if self.get_reverse_orientation() ^ self.get_transform_swaps_handedness() {
            isect.base.n = -isect.base.n;
            isect.shading.n = -isect.shading.n;
        }

        *t_hit = t;
        true
    }

    fn area(&self) -> f32 {
        let p0 = &self.mesh.p[self.v[0]];
        let p1 = &self.mesh.p[self.v[1]];
        let p2 = &self.mesh.p[self.v[2]];
        
        0.5 * (p1 - p0).cross(&(p2 - p0)).norm()
    }

    fn sample(&self, u: &Point2, pdf: &mut f32) -> InteractionBase {
        let b = uniform_sample_triangle(u);
        
        let p0 = self.mesh.p[self.v[0]];
        let p1 = self.mesh.p[self.v[1]];
        let p2 = self.mesh.p[self.v[2]];

        let mut it = InteractionBase::new();
        
        let w = 1.0 - b.x - b.y;
        it.p = Point3::from(p0.coords * b.x + p1.coords * b.y + p2.coords * w);

        if self.mesh.n.len() > 0 {
            it.n = b.x * self.mesh.n[self.v[0]] + b.y * self.mesh.n[self.v[1]] + (1.0 - b.x - b.y) * self.mesh.n[self.v[2]];
        } else {
            it.n = (p1 - p0).cross(&(p2 - p0)).normalize();
        }

        if self.get_reverse_orientation() {
            it.n *= -1.0;
        }

        let p_abs_sum: Vector3 = (b.x * p0.coords).map(|x| x.abs())
        - (-b.y * p1.coords).map(|x| x.abs())
        - (-(1.0 - b.x - b.y) * p2.coords).map(|x| x.abs());

        it.p_error = gamma(6.0) * p_abs_sum;

        *pdf = self.pdf();

        it
    }
}