
use std::cmp::Ordering;

use cgmath::{Point3, Vector3, InnerSpace};
use dyn_clone::DynClone;

use crate::material::{Material, TexCoords};

/// Represents a single ray with origin and direction
pub struct Ray {
    /// Ray origin
    pub origin: Point3<f32>,
    /// Unit vector representing the rays direction
    pub direction: Vector3<f32>,
}

impl Ray {
    /// Create a ray with the appropriate direction for the specified pixel position and field of view
    pub fn from_screen_coordinates(x: u32, y: u32, width: u32, height: u32, fov: f32) -> Ray {
        let fov_factor = (fov.to_radians() / 2.0).tan();

        let aspect_ratio = width as f32 / height as f32;

        // Calculate screen coordinates between 0 and 1
        let x_01 = (x as f32 + 0.5) / width as f32;
        let y_01 = (y as f32 + 0.5) / height as f32;

        // Translate screen coordinates in range [0.0, 1.0] to range [-1.0, 1.0]
        let x_relative = x_01 * 2.0 - 1.0;
        let y_relative = -(y_01 * 2.0 - 1.0);

        // Calculate ray direction from screen coordinates
        let ray_x = x_relative * aspect_ratio * fov_factor;
        let ray_y = y_relative * fov_factor;

        let direction_normalized = Vector3::new(ray_x, ray_y, -1.0).normalize();

        Ray {
            origin: Point3::new(0.0, 0.0, 0.0),
            direction: direction_normalized,
        }
    }

    pub fn create_reflection(normal: &Vector3<f32>, incident: &Vector3<f32>, hit_point: &Point3<f32>) -> Ray {
        Ray {
            origin: (hit_point + 1e-5 * normal),
            direction: incident - (2.0 * incident.dot(*normal) * normal),
        }
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
            Some(Ray {
                origin: (hit_point - 1e-5 * ref_n),
                direction: incident * eta + (i_dot_n * eta - k.sqrt()) * ref_n,
            })
        }
    }
}

pub struct Hit<'a> {
    pub point: Point3<f32>,
    pub distance: f32,
    pub normal: Vector3<f32>,
    pub material: &'a Material,
    pub tex_coords: TexCoords<f32>,
}

impl<'a> PartialEq for Hit<'a> {
    /// Hits are equal when their hit distances are equal
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl<'a> Eq for Hit<'a> {}

impl<'a> PartialOrd for Hit<'a> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a> Ord for Hit<'a> {
    /// Compare hits by their hit distance
    fn cmp(&self, other: &Self) -> Ordering {
        // Hit distances should never be NaN or Infinity
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}

impl<'a> Hit<'a> {
    pub fn new(point: Point3<f32>, distance: f32, normal: Vector3<f32>, material: &Material, tex_coords: TexCoords<f32>) -> Hit {
        Hit { point, distance, normal, material, tex_coords }
    }
}

/// Implement for objects that a ray can intersect with
#[typetag::serde(tag = "type")]
pub trait Intersectable: DynClone + Send {
    /// Cast a ray at the object. Returns true if it hits
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
}

dyn_clone::clone_trait_object!(Intersectable);
