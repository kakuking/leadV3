use std::ops::Index;

use crate::core::{MAX, MIN, Point2, Point3, Ray, Vector2, Vector3, gamma, lerp};


#[derive(Debug, Clone, PartialEq)]
pub struct Bounds2 {
    pub p_min: Point2,
    pub p_max: Point2
}

pub struct Bounds2Iterator {
    x: i32,
    y: i32,
    x_start: i32,
    x_end: i32,
    y_end: i32,
}

impl Bounds2 {
    pub fn new() -> Self {
        Self {
            p_min: Point2::new(MAX, MAX),
            p_max: Point2::new(MIN, MIN)
        }
    }

    pub fn init_one(p: &Point2) -> Self {
        Self {
            p_min: p.clone(),
            p_max: p.clone()
        }
    }

    pub fn init_two(p1: &Point2, p2: &Point2) -> Self {
        Self {
            p_min: Point2::new(p1.x.min(p2.x), p1.y.min(p2.y)),
            p_max: Point2::new(p1.x.max(p2.x), p1.y.max(p2.y)),
        }
    }

    pub fn diagonal(&self) -> Vector2 {
        self.p_max - self.p_min
    }

    pub fn area(&self) -> f32 {
        let d = self.diagonal();

        d.x * d.y
    }

    // Returns which extent is longer
    pub fn max_extent(&self) -> usize {
        let d = self.diagonal();

        if d.x > d.y {
            0
        } else {
            1
        }
    }

    pub fn lerp(&self, t: &Point2) -> Point2 {
        let x = lerp(t.x, self.p_min.x, self.p_max.x);
        let y = lerp(t.y, self.p_min.y, self.p_max.y);

        Point2::new(x, y)
    }

    pub fn offset(&self, p: Point2) -> Point2 {
        let mut o: Point2 = (p - self.p_min).into();

        if self.p_max.x > self.p_min.x {
            o.x /= self.p_max.x - self.p_min.x;
        }

        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }

        o
    }

    pub fn bounding_sphere(&self, c: &mut Point2, r: &mut f32) {
        *c = (self.p_min -  (-1.0) * self.p_max).into();
        *c /= 2.0;

        *r = if self.inside(&c) {
            (self.p_max - *c).norm()
        } else {
            0.0
        };
    }

    pub fn inside(&self, p: &Point2) -> bool {
        self.p_min.x < p.x && p.x < self.p_max.x &&
        self.p_min.y < p.y && p.y < self.p_max.y
    }

    pub fn inside_exclusive(&self, p: &Point2) -> bool {
        p.x >= self.p_min.x && p.x < self.p_max.x &&
        p.y >= self.p_min.y && p.y < self.p_max.y
    }

    pub fn intersect(&self, b: &Bounds2) -> Self {
        let p_minx = self.p_min.x.max(b.p_min.x);
        let p_miny = self.p_min.y.max(b.p_min.y);

        let p_maxx = self.p_max.x.min(b.p_max.x);
        let p_maxy = self.p_max.y.min(b.p_max.y);

        Self {
            p_min: Point2::new(p_minx, p_miny),
            p_max: Point2::new(p_maxx, p_maxy),
        }
    }

    pub fn is_integer(&self) -> bool {
        fn is_int(v: f32) -> bool {
            (v - v.round()).abs() < 1e-6
        }

        is_int(self.p_min.x) &&
        is_int(self.p_min.y) &&
        is_int(self.p_max.x) &&
        is_int(self.p_max.y)
    }
}

impl Index<usize> for Bounds2 {
    type Output = Point2;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index == 0 || index == 1);

        if index == 0 {
            &self.p_min
        } else {
            &self.p_max
        }
    }
}

impl Iterator for Bounds2Iterator {
    type Item = Point2;

    fn next(&mut self) -> Option<Self::Item> {
        if self.y >= self.y_end {
            return None;
        }

        let p = Point2::new(self.x as f32, self.y as f32);

        self.x += 1;
        if self.x >= self.x_end {
            self.x = self.x_start;
            self.y += 1;
        }

        Some(p)
    }
}

impl IntoIterator for &Bounds2 {
    type Item = Point2;
    type IntoIter = Bounds2Iterator;

    fn into_iter(self) -> Self::IntoIter {
        let x_start = self.p_min.x as i32;
        let y_start = self.p_min.y as i32;
        let x_end = self.p_max.x as i32;
        let y_end = self.p_max.y as i32;

        Bounds2Iterator {
            x: x_start,
            y: y_start,
            x_start,
            x_end,
            y_end,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Bounds3 {
    pub p_min: Point3,
    pub p_max: Point3
}

impl Bounds3 {
    pub fn new() -> Self {
        Self {
            p_min: Point3::new(MAX, MAX, MAX),
            p_max: Point3::new(MIN, MIN, MIN)
        }
    }

    pub fn init_one(p: &Point3) -> Self {
        Self {
            p_min: p.clone(),
            p_max: p.clone()
        }
    }

    pub fn init_two(p1: &Point3, p2: &Point3) -> Self {
        Self {
            p_min: Point3::new(p1.x.min(p2.x), p1.y.min(p2.y), p1.z.min(p2.z)),
            p_max: Point3::new(p1.x.max(p2.x), p1.y.max(p2.y), p1.z.max(p2.z)),
        }
    }

    pub fn diagonal(&self) -> Vector3 {
        self.p_max - self.p_min
    }

    pub fn corner(&self, corner: usize) -> Point3 {
        let x_idx = corner & 1;
        let y_idx = if (corner & 2) != 0 { 1 } else { 0 };
        let z_idx = if (corner & 4) != 0 { 1 } else { 0 };

        let x = self[x_idx].x;
        let y = self[y_idx].y;
        let z = self[z_idx].z;

        Point3::new(x, y, z)
    }

    pub fn volume(&self) -> f32 {
        let d = self.diagonal();

        d.x * d.y * d.z
    }

    pub fn surface_area(&self) -> f32 {
        let d = self.diagonal();

        2.0 * (d.x * d.y + d.x * d.z + d.y * d.z)
    }

    // Returns which extent is longer
    pub fn max_extent(&self) -> usize {
        let d = self.diagonal();

        if d.x > d.y && d.x > d.z {
            0
        } else if d.y > d.z {
            1
        } else {
            2
        }
    }

    pub fn lerp(&self, t: &Point3) -> Point3 {
        let x = lerp(t.x, self.p_min.x, self.p_max.x);
        let y = lerp(t.y, self.p_min.y, self.p_max.y);
        let z = lerp(t.z, self.p_min.z, self.p_max.z);

        Point3::new(x, y, z)
    }

    pub fn offset(&self, p: &Point3) -> Point3 {
        let mut o: Point3 = (p - self.p_min).into();

        if self.p_max.x > self.p_min.x {
            o.x /= self.p_max.x - self.p_min.x;
        }

        if self.p_max.y > self.p_min.y {
            o.y /= self.p_max.y - self.p_min.y;
        }

        if self.p_max.z > self.p_min.z {
            o.z /= self.p_max.z - self.p_min.z;
        }

        o
    }

    pub fn bounding_sphere(&self, c: &mut Point3, r: &mut f32) {
        *c = (self.p_min -  (-1.0) * self.p_max).into();
        *c /= 2.0;

        *r = if self.inside(&c) {
            (self.p_max - *c).norm()
        } else {
            0.0
        };
    }

    pub fn inside(&self, p: &Point3) -> bool {
        self.p_min.x < p.x && p.x < self.p_max.x &&
        self.p_min.y < p.y && p.y < self.p_max.y &&
        self.p_min.z < p.z && p.z < self.p_max.z
    }

    pub fn inside_exclusive(&self, p: &Point3) -> bool {
        p.x >= self.p_min.x && p.x < self.p_max.x &&
        p.y >= self.p_min.y && p.y < self.p_max.y &&
        p.z >= self.p_min.z && p.z < self.p_max.z
    }

    pub fn union_p(&self, p: &Point3) -> Self {
        let p_minx = self.p_min.x.min(p.x);
        let p_miny = self.p_min.y.min(p.y);
        let p_minz = self.p_min.z.min(p.z);

        let p_maxx = self.p_max.x.max(p.x);
        let p_maxy = self.p_max.y.max(p.y);
        let p_maxz = self.p_max.z.max(p.z);

        Self {
            p_min: Point3::new(p_minx, p_miny, p_minz),
            p_max: Point3::new(p_maxx, p_maxy, p_maxz),
        }
    }

    pub fn union(&self, b: &Bounds3) -> Self {
        let p_minx = self.p_min.x.min(b.p_min.x);
        let p_miny = self.p_min.y.min(b.p_min.y);
        let p_minz = self.p_min.z.min(b.p_min.z);

        let p_maxx = self.p_max.x.max(b.p_max.x);
        let p_maxy = self.p_max.y.max(b.p_max.y);
        let p_maxz = self.p_max.z.max(b.p_max.z);

        Self {
            p_min: Point3::new(p_minx, p_miny, p_minz),
            p_max: Point3::new(p_maxx, p_maxy, p_maxz),
        }
    }

    pub fn intersect(&self, b: &Bounds3) -> Self {
        let p_minx = self.p_min.x.max(b.p_min.x);
        let p_miny = self.p_min.y.max(b.p_min.y);
        let p_minz = self.p_min.z.max(b.p_min.z);

        let p_maxx = self.p_max.x.min(b.p_max.x);
        let p_maxy = self.p_max.y.min(b.p_max.y);
        let p_maxz = self.p_max.z.min(b.p_max.z);

        Self {
            p_min: Point3::new(p_minx, p_miny, p_minz),
            p_max: Point3::new(p_maxx, p_maxy, p_maxz),
        }
    }

    pub fn overlaps(&self, b: Bounds3) -> bool {
        let x = self.p_max.x >= b.p_min.x && self.p_min.x <= b.p_max.x;
        let y = self.p_max.y >= b.p_min.y && self.p_min.y <= b.p_max.y;
        let z = self.p_max.z >= b.p_min.z && self.p_min.z <= b.p_max.z;

        x && y && z
    }

    pub fn intersect_p(&self, r: &Ray, hit_t0: &mut f32, hit_t1: &mut f32) -> bool {
        let mut t0 = 0.0f32;
        let mut t1 = r.t_max.get();

        for i in  0..3 {
            let inv_ray_dir =  1.0 / r.d[i];

            let  mut t_near = (self.p_min[i] - r.o[i]) * inv_ray_dir;
            let  mut t_far = (self.p_max[i] - r.o[i]) * inv_ray_dir;

            if t_near > t_far {
                std::mem::swap(&mut t_near, &mut t_far);
            }

            t_far *= 1.0 + 2.0 * gamma(3.0);

            t0 = if t_near > t0 { t_near } else { t0 };
            t1 = if  t_far < t1 { t_far } else { t1 };
            if t0 > t1 {
                return false;
            }
        }

        *hit_t0 = t0;
        *hit_t1 = t1;

        true
    }
    
    pub fn intersect_p_with_inv_dir(&self, r: &Ray, inv_dir: &Vector3, dir_is_neg: [usize; 3]) -> bool {
        let mut t_min = (self[dir_is_neg[0]].x - r.o.x) * inv_dir.x;
        let mut t_max = (self[1 - dir_is_neg[0]].x - r.o.x) * inv_dir.x;
        let ty_min = (self[dir_is_neg[1]].y - r.o.y) * inv_dir.y;
        let mut ty_max = (self[1 - dir_is_neg[1]].y - r.o.y) * inv_dir.y;

        t_max *= 1.0 + 2.0 * gamma(3.0);
        ty_max *= 1.0 + 2.0 * gamma(3.0);

        if  t_min > ty_max || ty_min > t_max {
            return false;
        }

        if ty_min > t_min {
            t_min = ty_min;
        }

        if ty_max < t_max {
            t_max = ty_max;
        }

        let tz_min = (self[dir_is_neg[2]].z - r.o.z) * inv_dir.z;
        let mut tz_max = (self[1 - dir_is_neg[2]].z - r.o.z) * inv_dir.z;

        tz_max *= 1.0 + 2.0 * gamma(3.0);

        if t_min > tz_max || tz_min > t_max {
            return false;
        }

        if tz_min > t_min {
            t_min = tz_min;
        }

        if tz_max < t_max {
            t_max = tz_max;
        }

        t_min < r.t_max.get() && t_max > 0.0
    }
}


impl Index<usize> for Bounds3 {
    type Output = Point3;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index == 0 || index == 1);

        if index == 0 {
            &self.p_min
        } else {
            &self.p_max
        }
    }
}
