
use std::error::Error;
use std::path::PathBuf;

use serde::{Serialize, Deserialize, Serializer, Deserializer};
use cgmath::{Vector3, InnerSpace, Zero, EuclideanSpace, Vector2};

use crate::ray::{Intersectable, Hit, Ray};
use crate::asset_loader;
use crate::material::TexCoords;

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

#[derive(Clone)]
pub struct Mesh {
    path: PathBuf,
    data: MeshData,
}

impl Serialize for Mesh {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        self.path.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Mesh {
    fn deserialize<D>(deserializer: D) -> Result<Mesh, D::Error>
    where
        D: Deserializer<'de>
    {
        let path = PathBuf::deserialize(deserializer)?;
        Self::load(path.clone()).map_err(|err| {
            serde::de::Error::custom(format!("Unable to open mesh file \"{}\": {}", path.display(), err))
        })
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

#[typetag::serde]
impl Intersectable for Mesh {
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let mut nearest_hit: Option<(usize, TriangleHit)> = None;

        for (i, triangle) in self.data.triangles.iter().enumerate() {
            let v0 = self.data.get_vertex_position(triangle.position_indices.0);
            let v1 = self.data.get_vertex_position(triangle.position_indices.1);
            let v2 = self.data.get_vertex_position(triangle.position_indices.2);

            if let Some(hit) = intersect_triangle(ray, v0, v1, v2) {
                if let Some((_, current_nearest_hit)) = &nearest_hit {
                    if hit.distance < current_nearest_hit.distance {
                        nearest_hit = Some((i, hit));
                    }
                } else {
                    nearest_hit = Some((i, hit));
                }
            }
        }

        nearest_hit.map(|(triangle_index, triangle_hit)| {
            let triangle = &self.data.triangles[triangle_index];

            let normal = triangle.normal_indices.map_or_else(|| {
                let v0 = self.data.get_vertex_position(triangle.position_indices.0);
                let v1 = self.data.get_vertex_position(triangle.position_indices.1);
                let v2 = self.data.get_vertex_position(triangle.position_indices.2);

                (v1 - v0).cross(v2 - v0).normalize()
            }, |normal_indices| {
                let n0 = self.data.get_vertex_normal(normal_indices.0);
                let n1 = self.data.get_vertex_normal(normal_indices.1);
                let n2 = self.data.get_vertex_normal(normal_indices.2);

                (1.0 - triangle_hit.u - triangle_hit.v) * n0 + triangle_hit.u * n1 + triangle_hit.v * n2
            });

            let tex_coords = triangle.tex_coords_indices.map_or_else(|| {
                Vector2::zero()
            }, |tex_coords_indices| {
                let t0 = self.data.get_vertex_tex_coords(tex_coords_indices.0);
                let t1 = self.data.get_vertex_tex_coords(tex_coords_indices.1);
                let t2 = self.data.get_vertex_tex_coords(tex_coords_indices.2);

                (1.0 - triangle_hit.u - triangle_hit.v) * t0 + triangle_hit.u * t1 + triangle_hit.v * t2
            });

            Hit {
                point: ray.origin + ray.direction * triangle_hit.distance,
                distance: triangle_hit.distance,
                normal,
                tex_coords: TexCoords { u: tex_coords.x, v: tex_coords.y },
            }
        })
    }
}

impl Mesh {
    pub fn new(path: PathBuf, data: MeshData) -> Mesh {
        Mesh {
            path,
            data,
        }
    }

    pub fn load(path: PathBuf) -> Result<Mesh, Box<dyn Error>> {
        let a = asset_loader::get_instance();
        let data = a.load_obj(&path)?;
        Ok(Mesh::new(path, data))
    }
}
