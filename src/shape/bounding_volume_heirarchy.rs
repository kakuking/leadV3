// pub use crate::common::*;

use std::sync::Arc;

use crate::{core::{Bounds3, Point3, Ray, Vector3, interaction::{InteractionT, TransportMode}, material::Material, primitive::Primitive}, interaction::surface_interaction::SurfaceInteraction, light::area_light::AreaLight};

#[derive(Debug)]
pub enum SplitMethod {
    SAH
}

impl SplitMethod {
    pub fn to_string(&self) -> String {
        match self {
            SplitMethod::SAH => "Surface Area Hueristic".to_string()
        }
    }
}

struct BVHPrimitiveInfo {
    primitive_num: usize,
    bounds: Bounds3,
    centroid: Point3
}

impl BVHPrimitiveInfo {
    pub fn init(primitive_num: usize, bounds: Bounds3) -> Self {
        let centroid: Point3 = nalgebra::center(&bounds.p_min, &bounds.p_max);
        Self {
            primitive_num,
            bounds,
            centroid
        }
    }
}

struct BVHBuildNode {
    bounds: Bounds3,
    split_axis: usize,
    first_prim_offset: usize,
    n_primitives: usize,
    children: [Option<Box<BVHBuildNode>>; 2]
}

impl BVHBuildNode {
    pub fn new() -> Self {
        Self {
            bounds: Bounds3::new(),
            split_axis: 0usize,
            first_prim_offset: 0usize,
            n_primitives: 0usize,
            children: [None, None]
        }
    }

    pub fn init_leaf(&mut self, first: usize, n: usize, b: Bounds3) {
        self.first_prim_offset = first;
        self.n_primitives = n;
        self.bounds = b;
        self.children[0] = None;
        self.children[1] = None;
    }

    pub fn init_interior(&mut self, axis: usize, left: Option<Box<BVHBuildNode>>, right: Option<Box<BVHBuildNode>>) {
        self.split_axis = axis;
        self.n_primitives = 0usize;
        
        if let Some(left_bound) = &left {
            if let Some(right_bound) = &right {
                self.bounds = Bounds3::union(&left_bound.bounds, &right_bound.bounds); // if both are not none
            } else {
                self.bounds = left_bound.bounds.clone();    // If right is none left is not
            }
        } else if let Some(right_bound) = &right {
            self.bounds = right_bound.bounds.clone();   // if left is none right is not
        }

        self.children[0] = left;
        self.children[1] = right;
    }
}

#[derive(Debug)]
struct LinearBVHNode {
    bounds: Bounds3,
    primitives_offset: Option<usize>, // leaf
    second_child_offset: Option<usize>, // nterior node
    n_primitives: usize,
    axis: usize,
}

impl LinearBVHNode {
    pub fn new() -> Self {
        Self {
            bounds: Bounds3::new(),
            primitives_offset: None,
            second_child_offset: None,
            n_primitives: 0usize,
            axis: 0usize
        }
    }
}

// arealight, maerial are none and compute scattering func shouldnt be called
#[derive(Debug)]
pub struct BVHAccel {
    max_primitives_in_node: usize,
    splitmethod: SplitMethod,
    primitives: Vec<Arc<Primitive>>,
    nodes: Vec<LinearBVHNode>,
    is_built: bool
}

impl BVHAccel {
    pub fn init(max_primitives_in_node: usize, split_method: SplitMethod) -> Self {
        Self {
            max_primitives_in_node: 255usize.min(max_primitives_in_node),
            splitmethod: split_method,
            primitives: Vec::new(),
            nodes: Vec::new(),
            is_built: false,
        }
    }

    pub fn to_string(&self) -> String {
        format!(
            "BVHAccel [\n
            \tmax_primitives: {},\n
            \tsplit_method: {},\n
            \tnum_primitives: {},\n
            \tnum_nodes: {},\n
            \tis_built: {}\n
            ]
            ",
            self.max_primitives_in_node,
            self.splitmethod.to_string(),
            self.primitives.len(),
            self.nodes.len(),
            self.is_built
        )
    }

    pub fn add_primitive(&mut self, primitive: Arc<Primitive>) {
        if !self.is_built {
            self.primitives.push(primitive);
        }
    }

    pub fn build(&mut self) {
        let num_primitives = self.primitives.len();
        let mut primitive_infos: Vec<BVHPrimitiveInfo> = Vec::new();

        for i in 0..num_primitives {
            let primitive_info = BVHPrimitiveInfo::init(i, self.primitives[i].world_bounds());
            primitive_infos.push(primitive_info);
        }

        let mut total_nodes = 0usize;
        let mut ordered_primitives: Vec<Arc<Primitive>> = Vec::new();
        let mut root: BVHBuildNode = self.recursively_build(&mut primitive_infos, 0, num_primitives, &mut total_nodes, &mut ordered_primitives);
        
        self.primitives.clear();
        self.primitives = ordered_primitives;

        for _ in 0..total_nodes{
            self.nodes.push(LinearBVHNode::new());
        }
        let mut offset = 0usize;
        self.flatten_tree(&mut root, &mut offset);
        self.is_built = true;
    } 

    fn recursively_build(&mut self, primtive_infos: &mut Vec<BVHPrimitiveInfo>, start: usize, end: usize, total_nodes: &mut usize, ordered_primitives: &mut Vec<Arc<Primitive>>) -> BVHBuildNode {
        let mut node: BVHBuildNode = BVHBuildNode::new();
        (*total_nodes) += 1usize;

        let mut bounds: Bounds3 = Bounds3::new();

        for i in start..end {
            bounds = Bounds3::union(&bounds, &primtive_infos[i].bounds);
        }

        let n_primitives = end - start;
        if n_primitives == 1 {
            let first_prim_offset = ordered_primitives.len();
            for i in start..end {
                let prim_num = primtive_infos[i].primitive_num;
                ordered_primitives.push(self.primitives[prim_num].clone());
            }
            node.init_leaf(first_prim_offset, n_primitives, bounds);
            return node;
        }

        let mut centroid_bounds = Bounds3::new();
        for i in start..end {
            centroid_bounds = Bounds3::union_p(&centroid_bounds, &primtive_infos[i].centroid);
        }

        let dim = centroid_bounds.max_extent(); // longer dimension
        let mid;//= (start + end) / 2usize;

        if centroid_bounds.p_max[dim] == centroid_bounds.p_min[dim] {
            // is leaf
            let first_prim_offset = ordered_primitives.len();
            for i in start..end {
                let primitive_num = primtive_infos[i].primitive_num;
                ordered_primitives.push(self.primitives[primitive_num].clone());
            }
            node.init_leaf(first_prim_offset, n_primitives, bounds);
            return node;
        }
        // is interior node
        // partition based on split method
        match self.splitmethod {
        SplitMethod::SAH => {
            if n_primitives <= 4 {
                mid = (start + end) / 2;
                // everything before mid is <= mid and above it is >= it
                (*primtive_infos)[start..end].select_nth_unstable_by(mid - start, |a, b| {
                    a.centroid[dim].partial_cmp(&b.centroid[dim]).unwrap()
                });
            } else {
                const N_BUCKETS: usize = 12;
                
                #[derive(Debug)]
                struct BucketInfo {
                    count: usize,
                    bounds: Bounds3
                }
                let mut buckets: Vec<BucketInfo> = Vec::new();
                
                for _ in 0..N_BUCKETS {
                    buckets.push(BucketInfo { count: 0usize, bounds: Bounds3::new() });
                }

                for i in start..end {
                    let mut b = N_BUCKETS * centroid_bounds.offset(&primtive_infos[i].centroid)[dim] as usize;
                    if b == N_BUCKETS {
                        b = N_BUCKETS - 1;
                    }
                    buckets[b].count += 1;
                    buckets[b].bounds = Bounds3::union(&buckets[b].bounds, &primtive_infos[i].bounds);   
                }

                // find costs of all buckets
                let mut cost = [0.0; N_BUCKETS - 1];
                for i in 0..(N_BUCKETS - 1) {
                    let mut b0 = Bounds3::new();
                    let mut b1 = Bounds3::new();
                    let mut count_0 = 0usize;
                    let mut count_1 = 0usize;

                    for j in 0..=i {
                        b0 = Bounds3::union(&b0, &buckets[j].bounds);
                        count_0 += buckets[j].count;
                    }

                    for j in (i+1)..N_BUCKETS {
                        b1 = Bounds3::union(&b1, &buckets[j].bounds);
                        count_1 += buckets[j].count;
                    }

                    cost[i] = 0.125 + (count_0 as f32 * b0.surface_area() + count_1 as f32 * b1.surface_area()) / bounds.surface_area();
                }

                // find min cosst bukcet
                let mut min_cost = cost[0];
                let mut min_cost_bucket = 0;
                for i in 1..(N_BUCKETS - 1) {
                    if cost[i] < min_cost {
                        min_cost = cost[i];
                        min_cost_bucket = i;
                    }
                }
                

                let leaf_cost = n_primitives as f32;
                if n_primitives > self.max_primitives_in_node || min_cost < leaf_cost {
                    let mut left = start;
                    let mut right = end - 1;
                    
                    while left <= right {
                        let mut b_left = (N_BUCKETS as f32 * centroid_bounds.offset(&primtive_infos[left].centroid)[dim]) as usize;
                        if b_left == N_BUCKETS {
                            b_left -= 1;
                        }
                        
                        if b_left <= min_cost_bucket {
                            left += 1;
                        } else {
                            primtive_infos.swap(left, right);
                            right -= 1;
                        }
                    }
                    mid = left;
                } else {
                    // is a leaf
                    let first_prim_offset = ordered_primitives.len();
                    for i in start..end {
                        let prim_num = primtive_infos[i].primitive_num;
                        ordered_primitives.push(self.primitives[prim_num].clone());
                    }
                    node.init_leaf(first_prim_offset, n_primitives, bounds);

                    return node;
                }
            }
        },
        }

        let left_child = match start == mid {
            true => None,
            _ => Some(Box::from(self.recursively_build(primtive_infos, start, mid, total_nodes, ordered_primitives)))
        };
        let right_child = match mid == end {
            true => None,
            false => Some(Box::from(self.recursively_build(primtive_infos, mid, end, total_nodes, ordered_primitives)))
        };
        node.init_interior(dim, left_child, right_child);

        return node;     
    }

    fn flatten_tree(&mut self, node: &BVHBuildNode, offset: &mut usize) -> usize {
        let cur_idx  = *offset;
        self.nodes[cur_idx].bounds = node.bounds.clone();
        (*offset) += 1;

        if node.n_primitives > 0 {
            self.nodes[cur_idx].primitives_offset = Some(node.first_prim_offset);
            self.nodes[cur_idx].n_primitives = node.n_primitives;
        } else {
            self.nodes[cur_idx].axis = node.split_axis;
            self.nodes[cur_idx].n_primitives = 0;
            if let Some(left_child) = &node.children[0] {
                self.flatten_tree(left_child, offset);
            }
            if let Some(right_child) = &node.children[1] {
                self.nodes[cur_idx].second_child_offset = Some(self.flatten_tree(right_child, offset));
            }
        }

        cur_idx
    }

    pub fn compute_scattering_function(&self, _isect: &mut SurfaceInteraction, _mode: TransportMode, _allow_multiple_lobes: bool) {
        panic!("Should not call this for a aggregate!")
    }

    pub fn get_area_light(&self) -> Option<Arc<AreaLight>> {
        panic!("Should not call this for a aggregate!")
    }

    pub fn get_material(&self) -> Option<Arc<Material>> {
        panic!("Should not call this for a aggregate!")
    }

    pub fn world_bounds(&self) -> Bounds3 {
        self.nodes[0].bounds.clone()
    }

    pub fn intersect(&self, ray: &Ray, isect: &mut SurfaceInteraction) -> bool {
        let mut hit: bool = false;

        let inv_dir = Vector3::new(1.0 / ray.d.x, 1.0 / ray.d.y, 1.0 / ray.d.z);
        // let dir_is_neg: [bool; 3] = [inv_dir.x < 0.0, inv_dir.y < 0.0, inv_dir.z < 0.0];
        let dir_is_neg: [usize; 3] = [if inv_dir.x < 0.0 { 1 } else { 0 }, if inv_dir.y < 0.0 { 1 } else { 0 }, if inv_dir.z < 0.0 { 1 } else { 0 }];
        
        let mut to_visit_offset = 0usize;
        let mut current_node_idx = 0usize;
        let mut nodes_to_visit = [0usize; 64];  // custom stack
        loop {
            let node = &self.nodes[current_node_idx];
            if node.bounds.intersect_p_with_inv_dir(ray, &inv_dir, dir_is_neg) {
                if node.n_primitives > 0 {
                    for i in 0..node.n_primitives {
                        if let Some(prim_offset) = &node.primitives_offset {
                            if self.primitives[prim_offset + i].intersect(ray, isect) {
                                hit = true;
                            }
                        }
                    }

                    if to_visit_offset == 0 {
                        break;
                    }
                    current_node_idx = nodes_to_visit[to_visit_offset-1];
                    to_visit_offset -= 1;
                } else {
                    if dir_is_neg[node.axis] == 1{
                        if let Some(next_offset) = node.second_child_offset {
                            nodes_to_visit[to_visit_offset] = current_node_idx + 1;
                            to_visit_offset += 1;
                            current_node_idx = next_offset;
                        }
                    } else {
                        if let Some(next_offset) = node.second_child_offset {
                            nodes_to_visit[to_visit_offset] = next_offset;
                            to_visit_offset += 1; 
                            current_node_idx += 1;
                        }
                    }
                }
            } else {
                if to_visit_offset == 0 {
                    break;
                }

                to_visit_offset -= 1;
                current_node_idx = nodes_to_visit[to_visit_offset];
            }
        }

        hit
    }

    pub fn intersect_p(&self, r: &Ray) -> bool {
        let mut isect = SurfaceInteraction::new();

        self.intersect(r, &mut isect)
    }
}
