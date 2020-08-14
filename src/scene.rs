
use serde::{Serialize, Deserialize};
use cgmath::{Matrix4, SquareMatrix, Vector3, Euler, Deg, Point3};

use crate::color::Color;
use crate::ray::{Ray, Hit};
use crate::lights::Light;
use crate::material::Material;
use crate::primitives::{Plane, Sphere};
use crate::mesh::Mesh;

#[derive(Clone, Serialize, Deserialize)]
pub struct Transformation {
    translation: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: f32,
}

impl Transformation {
    fn to_matrix(&self) -> Matrix4<f32> {
        let translation_matrix = Matrix4::from_translation(self.translation);
        let rotation_matrix = Matrix4::from(Euler {
            x: Deg(self.rotation.x),
            y: Deg(self.rotation.y),
            z: Deg(self.rotation.z),
        });
        let scale_matrix = Matrix4::from_scale(self.scale);

        let transform_matrix = translation_matrix * rotation_matrix * scale_matrix;

        transform_matrix
    }
}

#[derive(Serialize, Deserialize)]
struct DeserializableObject {
    pub shape: Shape,
    pub material_index: usize,
    pub transform: Transformation,
}

impl From<Object> for DeserializableObject {
    fn from(o: Object) -> DeserializableObject {
        DeserializableObject {
            shape: o.shape,
            material_index: o.material_index,
            transform: o.transformation,
        }
    }
}

impl From<DeserializableObject> for Object {
    fn from(d: DeserializableObject) -> Object {
        let transform_matrix = d.transform.to_matrix();
        let inv_transform_matrix = transform_matrix.invert().unwrap();
        Object {
            shape: d.shape,
            material_index: d.material_index,
            transformation: d.transform,
            transformation_matrix: transform_matrix,
            inv_transformation_matrix: inv_transform_matrix,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub enum Shape {
    Plane(Plane),
    Sphere(Sphere),
    Mesh(Mesh),
}

impl Shape {
    pub fn intersect(&self, ray: &Ray) -> Option<Hit> {
        match self {
            Shape::Plane(plane) => plane.intersect(ray),
            Shape::Sphere(sphere) => sphere.intersect(ray),
            Shape::Mesh(mesh) => mesh.intersect(ray),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "DeserializableObject")]
#[serde(into = "DeserializableObject")]
pub struct Object {
    pub shape: Shape,
    pub material_index: usize,
    pub transformation: Transformation,
    pub transformation_matrix: Matrix4<f32>,
    pub inv_transformation_matrix: Matrix4<f32>,
}

impl Object {
    pub fn intersect(&self, ray: &Ray) -> Option<(&Object, Hit)> {
        // Transform ray origin and direction into object space
        let object_ray = ray.transform(&self.inv_transformation_matrix);
        let object_hit = self.shape.intersect(&object_ray);
        // Transform the hit point back to world space
        let world_hit = object_hit.map(|hit| {
            hit.transform(&self.transformation_matrix, &ray.origin)
        });

        world_hit.map(|hit| (self, hit))
    }
}

#[derive(Serialize, Deserialize)]
struct DeserializableCamera {
    pub resolution: (usize, usize),
    pub fov: f32,
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub up: Vector3<f32>,
}

impl From<Camera> for DeserializableCamera {
    fn from(o: Camera) -> DeserializableCamera {
        DeserializableCamera {
            resolution: o.resolution,
            fov: o.fov,
            position: o.position,
            direction: o.direction,
            up: o.up,
        }
    }
}

impl From<DeserializableCamera> for Camera {
    fn from(d: DeserializableCamera) -> Camera {
        let transformation_matrix = Matrix4::look_at_dir(d.position, d.direction, d.up).invert().unwrap();
        Camera {
            resolution: d.resolution,
            fov: d.fov,
            position: d.position,
            direction: d.direction,
            up: d.up,
            transformation_matrix,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(from = "DeserializableCamera")]
#[serde(into = "DeserializableCamera")]
pub struct Camera {
    pub resolution: (usize, usize),
    pub fov: f32,
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
    pub up: Vector3<f32>,
    pub transformation_matrix: Matrix4<f32>,
}

/// Holds all information about the scene
#[derive(Clone, Serialize, Deserialize)]
pub struct Scene {
    pub camera: Camera,
    pub aa_samples: usize,
    /// Background color, assigned to pixels that are not covered by any object in the scene
    pub clear_color: Color,
    pub materials: Vec<Material>,
    pub objects: Vec<Object>,
    pub ambient_light_color: Color,
    pub lights: Vec<Light>,
    pub max_recursion_depth: u32,
}

impl Scene {
    /// Check ray intersections against all objects in the scene and return the closest hit
    pub fn trace(&self, ray: &Ray) -> Option<(&Object, Hit)> {
        self.objects.iter()
            .filter_map(|obj| obj.intersect(ray))
            .min_by(|(_, hit1), (_, hit2)| hit1.cmp(hit2))
    }
}
