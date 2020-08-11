
use std::error::Error;
use std::path::PathBuf;
use std::time::Instant;

use serde::{Serialize, Deserialize, Deserializer};
use cgmath::{Vector3, InnerSpace, Zero, EuclideanSpace, Vector2};

use crate::ray::{Intersectable, Hit, Ray};
use crate::asset_loader;
use crate::material::TexCoords;
use crate::aabb::AABB;
use crate::math_util::Axis;

#[derive(Clone)]
pub struct IndexedTriangle {
    pub position_indices: (usize, usize, usize),
    pub normal_indices: Option<(usize, usize, usize)>,
    pub tex_coords_indices: Option<(usize, usize, usize)>,
}

#[derive(Clone)]
pub struct MeshData {
    pub vertex_positions: Vec<(f32, f32, f32)>,
    pub vertex_normals: Vec<(f32, f32, f32)>,
    pub vertex_tex_coords: Vec<(f32, f32)>,
    pub triangles: Vec<IndexedTriangle>,
}

impl MeshData {
    fn get_vertex_position(&self, index: usize) -> &Vector3<f32> {
        (&self.vertex_positions[index]).into()
    }

    fn get_vertex_normal(&self, index: usize) -> &Vector3<f32> {
        (&self.vertex_normals[index]).into()
    }

    fn get_vertex_tex_coords(&self, index: usize) -> &Vector2<f32> {
        (&self.vertex_tex_coords[index]).into()
    }
}

struct TriangleHit {
    distance: f32,
    u: f32,
    v: f32,
}

fn intersect_triangle(ray: &Ray, v0: &Vector3<f32>, v1: &Vector3<f32>, v2: &Vector3<f32>) -> Option<TriangleHit> {
    // Möller-Trumbore ray-triangle intersection algorithm

    let v0v1: Vector3<_> = v1 - v0;
    let v0v2: Vector3<_> = v2 - v0;
    let pvec = ray.direction.cross(v0v2);
    let det = v0v1.dot(pvec);

    if det.abs() < f32::EPSILON {
        return None;
    }

    let inv_det = 1.0 / det;

    let tvec = ray.origin.to_vec() - v0;
    let u = tvec.dot(pvec) * inv_det;
    if u < 0.0 || u > 1.0 {
        return None;
    }

    let qvec = tvec.cross(v0v1);
    let v = ray.direction.dot(qvec) * inv_det;
    if v < 0.0 || u + v > 1.0 {
        return None;
    }

    let t = v0v2.dot(qvec) * inv_det;

    if t < 0.0 {
        return None;
    }

    Some(TriangleHit {
        distance: t,
        u,
        v,
    })
}

pub struct KDTreeOptions {
    max_depth: Option<usize>,
    max_leaf_size: usize,
    debug: bool,
}

impl Default for KDTreeOptions {
    fn default() -> Self {
        KDTreeOptions {
            max_depth: None,
            max_leaf_size: 16,
            debug: false,
        }
    }
}

#[derive(Clone)]
pub enum LinearKDTreeNode {
    Inner { split_axis: Axis, split_position: f32, above_child_index: usize },
    Leaf { triangle_indices: Vec<usize> },
}

#[derive(Clone)]
pub struct LinearKDTree {
    /// All nodes are stored depth-first in this vector to improve traversal speed
    nodes: Vec<LinearKDTreeNode>,
    bounding_box: AABB,
    data: MeshData,
    debug: bool,
}

/// Edge of a bounding box projected onto an axis
struct BoundEdge {
    position: f32,
    triangle_index: usize,
    is_end: bool,
}

/// Node that still has to be traversed during K-D tree intersection test
struct ToDoItem {
    node_index: usize,
    t_min: f32,
    t_max: f32,
}

impl LinearKDTree {
    pub fn build(data: MeshData, options: &KDTreeOptions) -> LinearKDTree {
        let triangle_count = data.triangles.len();

        // Formula taken from "Physically Based Rendering: From Theory To Implementation"
        let max_depth = options.max_depth
            .unwrap_or_else(|| 8 + (1.3 * (triangle_count as f32).log2()).round() as usize);

        let mut root_bounding_box = AABB::empty();
        let mut triangle_bounding_boxes = Vec::with_capacity(triangle_count);
        for triangle in &data.triangles {
            let v0 = data.get_vertex_position(triangle.position_indices.0);
            let v1 = data.get_vertex_position(triangle.position_indices.1);
            let v2 = data.get_vertex_position(triangle.position_indices.2);
            let bounding_box = AABB::from_triangle(v0, v1, v2);
            root_bounding_box = root_bounding_box.union(&bounding_box);
            triangle_bounding_boxes.push(bounding_box);
        }

        // All required working memory is allocated up front

        // Initialize with indices of all triangles
        let mut indices_below: Vec<_> = (0..triangle_count).collect();
        // Reserve size for worst case
        let mut indices_above = vec![0; (max_depth + 1) * triangle_count];
        let mut edges = Vec::with_capacity(triangle_count * 2);

        let mut nodes = Vec::new();

        LinearKDTree::build_node(
            &mut nodes,
            &mut indices_below,
            &mut indices_above,
            // The initial set of triangles is passed in `indices_below`
            false,
            triangle_count,
            &root_bounding_box,
            &triangle_bounding_boxes,
            max_depth,
            options,
            &mut edges,
        );

        LinearKDTree {
            nodes,
            bounding_box: root_bounding_box,
            data,
            debug: options.debug,
        }
    }

    /// Construct a new node in place
    ///
    /// Arguments:
    ///
    /// * `nodes`: All nodes in depth-first, left-to-right order
    /// * `triangle_indices_below`: Heap space for nodes below the previous split
    /// * `triangle_indices_above`: Heap space for nodes above the previous split
    /// * `is_above`: Whether `triangle_indices_below` or `triangle_indices_above` contains the triangle indices for this node
    /// * `triangle_count`: Number of triangles in this node, also determines how many items of `triangle_indices_below` or `triangle_indices_above` are valid
    /// * `node_bounding_box`: Bounding box of all triangles in this node
    /// * `triangle_bounding_boxes`: Bounding boxes of all triangles
    /// * `depth_remaining`: Decremented with each level of recursion
    /// * `options`: Build options
    /// * `edges`: Pre-allocated heap space for bounding box edges
    fn build_node(
        nodes: &mut Vec<LinearKDTreeNode>,
        triangle_indices_below: &mut [usize],
        triangle_indices_above: &mut [usize],
        is_above: bool,
        triangle_count: usize,
        node_bounding_box: &AABB,
        triangle_bounding_boxes: &[AABB],
        depth_remaining: usize,
        options: &KDTreeOptions,
        edges: &mut Vec<BoundEdge>,
    ) {
        let triangle_indices = if is_above {
            &triangle_indices_above[..triangle_count]
        } else {
            &triangle_indices_below[..triangle_count]
        };

        if triangle_count <= options.max_leaf_size || depth_remaining == 0 {
            nodes.push(LinearKDTreeNode::Leaf {
                triangle_indices: triangle_indices[0..triangle_count].to_vec(),
            });

            return;
        }

        let split_axis = node_bounding_box.maximum_extent();

        edges.clear();
        for &triangle_index in triangle_indices {
            let bounding_box = &triangle_bounding_boxes[triangle_index];
            edges.push(BoundEdge { position: bounding_box.min[split_axis], triangle_index, is_end: false });
            edges.push(BoundEdge { position: bounding_box.max[split_axis], triangle_index, is_end: true });
        }

        edges.sort_unstable_by(|a, b| {
            a.position.partial_cmp(&b.position).unwrap()
        });

        // TODO: replace median with SAH
        let split_position = (edges[edges.len() / 2].position + edges[edges.len() / 2 + 1].position) * 0.5;

        let mut n_below = 0;
        let mut n_above = 0;

        // Edges are sorted by their position -> edges below split come first
        let mut i = 0;
        while i < edges.len() && edges[i].position <= split_position {
            // All triangles whose lower edge is below the split
            if !edges[i].is_end {
                triangle_indices_below[n_below] = edges[i].triangle_index;
                n_below += 1;
            }
            i += 1;
        }
        // The remaining edges are all above the split
        while i < edges.len() {
            // All triangles whose upper edge is above the split
            if edges[i].is_end {
                triangle_indices_above[n_above] = edges[i].triangle_index;
                n_above += 1;
            }
            i += 1;
        }

        let node_index = nodes.len();
        nodes.push(LinearKDTreeNode::Inner {
            split_axis,
            split_position,
            // We don't know the index of the second child node yet
            above_child_index: 0
        });

        let mut bounding_box_below = node_bounding_box.clone();
        bounding_box_below.max[split_axis] = split_position;
        LinearKDTree::build_node(
            nodes,
            triangle_indices_below,
            // The first `n_above` items of `triangle_indices_above` need to be preserved for construction of the second child node
            &mut triangle_indices_above[n_above..],
            false,
            n_below,
            &bounding_box_below,
            triangle_bounding_boxes,
            depth_remaining - 1,
            options,
            edges,
        );

        // Update index of the second child node now that we know it
        let second_child_index = nodes.len();
        match &mut nodes[node_index] {
            LinearKDTreeNode::Inner { above_child_index, .. } => {
                *above_child_index = second_child_index;
            },
            _ => unreachable!(),
        }

        let mut bounding_box_above = node_bounding_box.clone();
        bounding_box_above.min[split_axis] = split_position;
        LinearKDTree::build_node(
            nodes,
            triangle_indices_below,
            triangle_indices_above,
            true,
            n_above,
            &bounding_box_above,
            triangle_bounding_boxes,
            depth_remaining - 1,
            options,
            edges,
        );
    }

    pub fn intersect(&self, ray: &Ray) -> Option<Hit> {
        if let Some((bb_t_min, bb_t_max)) = self.bounding_box.intersects_p(ray) {
            // Push root node onto stack
            let mut todo_stack = vec![ToDoItem {
                node_index: 0,
                t_min: bb_t_min,
                t_max: bb_t_max,
            }];

            let mut nearest_hit: Option<(usize, TriangleHit)> = None;

            // Number of nodes we had to look up, for debugging purposes
            let mut lookups = 1;

            let inv_dir: Vector3<f32> = 1.0 / ray.direction;

            while let Some(ToDoItem { node_index, t_min, t_max }) = todo_stack.pop() {
                // Bail out if this node is behind the nearest hit that was found so far
                if let Some((_, nearest_hit)) = &nearest_hit {
                    if nearest_hit.distance < t_min {
                        break;
                    }
                }

                lookups += 1;

                let node = &self.nodes[node_index];
                match node {
                    &LinearKDTreeNode::Inner { split_axis, split_position, above_child_index } => {
                        let origin_position = ray.origin[split_axis];

                        // Find distance at which the ray intersects the split plane
                        let t_split = (split_position - origin_position) * inv_dir[split_axis];

                        // Determine which child the ray crosses first
                        let first_child_index;
                        let second_child_index;
                        if origin_position < split_position || (origin_position == split_position && ray.direction[split_axis] <= 0.0) {
                            first_child_index = node_index + 1;
                            second_child_index = above_child_index;
                        } else {
                            first_child_index = above_child_index;
                            second_child_index = node_index + 1;
                        }

                        if t_split > t_max || t_split <= 0.0 {
                            // The ray leaves this node before it intersects the second child (t_split > t_max) or
                            //  the ray points away from the splitting plane (t_split <= 0)
                            //  -> only the first child is intersected
                            todo_stack.push(ToDoItem {
                                node_index: first_child_index,
                                t_min,
                                t_max,
                            });
                        } else if t_split < t_min {
                            // The ray intersects the splitting plane before it enters the node
                            //  -> only the second child is intersected
                            todo_stack.push(ToDoItem {
                                node_index: second_child_index,
                                t_min,
                                t_max,
                            });
                        } else {
                            // Stack is LIFO -> node at `first_child_index` will be processed next
                            todo_stack.push(ToDoItem {
                                node_index: second_child_index,
                                t_min: t_split,
                                t_max,
                            });
                            todo_stack.push(ToDoItem {
                                node_index: first_child_index,
                                t_min,
                                t_max: t_split,
                            });
                        }
                    }
                    LinearKDTreeNode::Leaf { triangle_indices } => {
                        // Test ray against all triangles in this node
                        for &triangle_index in triangle_indices {
                            let triangle = &self.data.triangles[triangle_index];
                            let v0 = self.data.get_vertex_position(triangle.position_indices.0);
                            let v1 = self.data.get_vertex_position(triangle.position_indices.1);
                            let v2 = self.data.get_vertex_position(triangle.position_indices.2);

                            if let Some(hit) = intersect_triangle(ray, v0, v1, v2) {
                                // Update `nearest_hit` only if it really is the nearest one
                                if let Some((_, current_nearest_hit)) = &nearest_hit {
                                    if hit.distance < current_nearest_hit.distance {
                                        nearest_hit = Some((triangle_index, hit));
                                    }
                                } else {
                                    nearest_hit = Some((triangle_index, hit));
                                }
                            }
                        }
                    }
                }
            }

            if self.debug {
                let mut debug_data = ray.debug_data.borrow_mut();
                debug_data.kd_tree_lookups += lookups;
            }

            // Calculate coordinates, normal and texture coordinates of the hit point
            nearest_hit.map(|(triangle_index, triangle_hit)| {
                let triangle = &self.data.triangles[triangle_index];

                let normal = triangle.normal_indices.map_or_else(|| {
                    let v0 = self.data.get_vertex_position(triangle.position_indices.0);
                    let v1 = self.data.get_vertex_position(triangle.position_indices.1);
                    let v2 = self.data.get_vertex_position(triangle.position_indices.2);

                    // Calculate face normal from vertex positions
                    (v1 - v0).cross(v2 - v0).normalize()
                }, |normal_indices| {
                    let n0 = self.data.get_vertex_normal(normal_indices.0);
                    let n1 = self.data.get_vertex_normal(normal_indices.1);
                    let n2 = self.data.get_vertex_normal(normal_indices.2);

                    // Interpolate vertex normals using the barycentric coordinates of the hit point
                    (1.0 - triangle_hit.u - triangle_hit.v) * n0 + triangle_hit.u * n1 + triangle_hit.v * n2
                });

                let tex_coords = triangle.tex_coords_indices.map_or_else(|| {
                    Vector2::zero()
                }, |tex_coords_indices| {
                    let t0 = self.data.get_vertex_tex_coords(tex_coords_indices.0);
                    let t1 = self.data.get_vertex_tex_coords(tex_coords_indices.1);
                    let t2 = self.data.get_vertex_tex_coords(tex_coords_indices.2);

                    // Interpolate vertex texture coordinates using the barycentric coordinates of the hit point
                    (1.0 - triangle_hit.u - triangle_hit.v) * t0 + triangle_hit.u * t1 + triangle_hit.v * t2
                });

                Hit {
                    point: ray.origin + ray.direction * triangle_hit.distance,
                    distance: triangle_hit.distance,
                    normal,
                    tex_coords: TexCoords { u: tex_coords.x, v: tex_coords.y },
                }
            })
        } else {
            None
        }
    }
}

fn default_debug() -> bool {
    false
}

#[derive(Serialize, Deserialize)]
struct DeserializableMesh {
    path: PathBuf,
    #[serde(default = "default_debug")]
    debug: bool,
}

impl From<Mesh> for DeserializableMesh {
    fn from(mesh: Mesh) -> DeserializableMesh {
        DeserializableMesh {
            path: mesh.path,
            debug: mesh.debug,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(into = "DeserializableMesh")]
pub struct Mesh {
    path: PathBuf,
    kdtree: LinearKDTree,
    debug: bool,
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Mesh, D::Error>
        where
            D: Deserializer<'de>
    {
        let dmesh = DeserializableMesh::deserialize(deserializer)?;
        Self::load(dmesh.path.clone(), dmesh.debug).map_err(|err| {
            serde::de::Error::custom(format!("Unable to open mesh file \"{}\": {}", dmesh.path.display(), err))
        })
    }
}

#[typetag::serde]
impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        self.kdtree.intersect(ray)
    }
}

impl Mesh {
    pub fn new(path: PathBuf, data: MeshData, debug: bool) -> Mesh {
        let start = Instant::now();
        let kdtree = LinearKDTree::build(data, &KDTreeOptions {
            debug,
            ..KDTreeOptions::default()
        });
        let duration = start.elapsed().as_secs_f64();
        if debug {
            println!("K-D tree for {} built in {} s", path.display(), duration);
        }

        Mesh {
            path,
            kdtree,
            debug,
        }
    }

    pub fn load(path: PathBuf, debug: bool) -> Result<Mesh, Box<dyn Error>> {
        let a = asset_loader::get_instance();
        let data = a.load_obj(&path)?;
        Ok(Mesh::new(path, data, debug))
    }
}
