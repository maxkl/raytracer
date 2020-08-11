
use std::cmp::Ordering;
use std::rc::Rc;
use std::cell::RefCell;

use cgmath::{Point3, Vector3, InnerSpace, Matrix4, Transform, MetricSpace};
use dyn_clone::DynClone;

use crate::material::TexCoords;

pub struct RayDebugData {
    pub kd_tree_lookups: usize,
}

/// Represents a single ray with origin and direction
pub struct Ray {
    /// Ray origin
    pub origin: Point3<f32>,
    /// Unit vector representing the rays direction
    pub direction: Vector3<f32>,

    pub debug_data: Rc<RefCell<RayDebugData>>,
}

impl Ray {
    pub fn new(origin: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray {
            origin,
            direction,
            debug_data: Rc::new(RefCell::new(RayDebugData {
                kd_tree_lookups: 0,
            })),
        }
    }

    pub fn transform(&self, transformation: &Matrix4<f32>) -> Ray {
        Ray {
            origin: transformation.transform_point(self.origin),
            direction: transformation.transform_vector(self.direction).normalize(),
            debug_data: self.debug_data.clone(),
        }
    }

    /// Create a ray with the appropriate direction for the specified pixel position and field of view
    pub fn from_screen_coordinates(x: f32, y: f32, width: usize, height: usize, fov: f32) -> Ray {
        let fov_factor = (fov.to_radians() / 2.0).tan();

        let aspect_ratio = width as f32 / height as f32;

        // Calculate screen coordinates between 0 and 1
        let x_01 = (x + 0.5) / width as f32;
        let y_01 = (y + 0.5) / height as f32;

        // Translate screen coordinates in range [0.0, 1.0] to range [-1.0, 1.0]
        let x_relative = x_01 * 2.0 - 1.0;
        let y_relative = -(y_01 * 2.0 - 1.0);

        // Calculate ray direction from screen coordinates
        let ray_x = x_relative * aspect_ratio * fov_factor;
        let ray_y = y_relative * fov_factor;

        let direction_normalized = Vector3::new(ray_x, ray_y, -1.0).normalize();

        Ray::new(
            Point3::new(0.0, 0.0, 0.0),
            direction_normalized,
        )
    }

    pub fn create_reflection(normal: &Vector3<f32>, incident: &Vector3<f32>, hit_point: &Point3<f32>) -> Ray {
        Ray::new(
            hit_point + 1e-5 * normal,
            incident - (2.0 * incident.dot(*normal) * normal),
        )
    }

    pub fn create_transmission(normal: &Vector3<f32>, incident: &Vector3<f32>, hit_point: &Point3<f32>, refractive_index: f32) -> Option<Ray> {
        let ref_n;
        let eta_t;
        let eta_i;
        let mut i_dot_n = incident.dot(*normal);
        if i_dot_n < 0.0 {
            i_dot_n = -i_dot_n;

            ref_n = *normal;
            eta_t = refractive_index;
            eta_i = 1.0;
        } else {
            ref_n = -*normal;
            eta_t = 1.0;
            eta_i = refractive_index;
        }

        let eta = eta_i / eta_t;
        let k = 1.0 - eta.powi(2) * (1.0 - i_dot_n.powi(2));
        if k < 0.0 {
            None
        } else {
            Some(Ray::new(
                hit_point - 1e-5 * ref_n,
                incident * eta + (i_dot_n * eta - k.sqrt()) * ref_n,
            ))
        }
    }
}

pub struct Hit {
    pub point: Point3<f32>,
    pub distance: f32,
    pub normal: Vector3<f32>,
    pub tex_coords: TexCoords<f32>,
}

impl PartialEq for Hit {
    /// Hits are equal when their hit distances are equal
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl Eq for Hit {}

impl PartialOrd for Hit {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hit {
    /// Compare hits by their hit distance
    fn cmp(&self, other: &Self) -> Ordering {
        // Hit distances should never be NaN or Infinity
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}

impl Hit {
    pub fn new(point: Point3<f32>, distance: f32, normal: Vector3<f32>, tex_coords: TexCoords<f32>) -> Hit {
        Hit { point, distance, normal, tex_coords }
    }

    pub fn transform(&self, transformation: &Matrix4<f32>, ray_origin: &Point3<f32>) -> Hit {
        let transformed_point = transformation.transform_point(self.point);
        let transformed_distance = ray_origin.distance(transformed_point);

        Hit {
            point: transformed_point,
            distance: transformed_distance,
            normal: transformation.transform_vector(self.normal).normalize(),
            tex_coords: self.tex_coords,
        }
    }
}

/// Implement for objects that a ray can intersect with
#[typetag::serde(tag = "type")]
pub trait Intersectable: DynClone + Send {
    /// Cast a ray at the object. Returns true if it hits
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}

dyn_clone::clone_trait_object!(Intersectable);
