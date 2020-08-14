
use std::f32;

use cgmath::{InnerSpace, Vector3, EuclideanSpace, Vector2};
use serde::{Serialize, Deserialize};

use crate::ray::{Ray, Hit};

/// A plane
#[derive(Clone, Serialize, Deserialize)]
pub struct Plane {}

impl Plane {
    pub fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // The normal has to be inverted for this calculation
        let normal = -Vector3::unit_y();

        // Calculate intersection
        let denominator = normal.dot(ray.direction);
        if denominator > 0.0 {
            let to_p0 = -ray.origin.to_vec();
            let distance = to_p0.dot(normal) / denominator;
            if distance > 0.0 {
                let hit_point = ray.origin + distance * ray.direction;

                // Calculate two perpendicular axes (unit vectors) that lie on the plane
                let x_axis = Vector3::unit_x();
                let y_axis = Vector3::unit_z();

                // Vector from plane origin to hit point
                let hit_vec = hit_point.to_vec();

                // Project onto the two plane axes to get the UV coordinates
                let tex_coords = Vector2::new(hit_vec.dot(x_axis), hit_vec.dot(y_axis));

                return Some(Hit::new(hit_point, distance, Vector3::unit_y(), tex_coords))
            }
        }

        None
    }
}

/// A sphere
#[derive(Clone, Serialize, Deserialize)]
pub struct Sphere {}

impl Sphere {
    pub fn intersect(&self, ray: &Ray) -> Option<Hit> {
        // Calculate vector from ray origin to sphere center (hypotenuse)
        let to_center = -ray.origin.to_vec();

        // Project to_center onto ray direction vector to get length of adjacent side
        let adjacent = to_center.dot(ray.direction);

        // Is the sphere behind the ray origin?
        if adjacent < 0.0 {
            return None;
        }

        // The length of the hypotenuse is just he magnitude of the vector connecting the ray origin and the sphere center
        let center_distance_squared = to_center.magnitude2();
        // Length of opposite side (pythagorean theorem)
        let distance_squared = center_distance_squared - adjacent.powi(2);

        // The opposite side is the smallest distance between the ray and the sphere center
        // Compare the opposite side and the sphere radius to determine whether the ray goes through the sphere
        if distance_squared > 1.0 {
            return None;
        }

        // Calculate how thick the sphere is at the intersection point
        let thickness_half = (1.0 - distance_squared).sqrt();
        // Calculate the distance along the ray of the two intersection points (front and back)
        let t0 = adjacent - thickness_half;
        let t1 = adjacent + thickness_half;

        // If both intersection points are behind us, return
        if t0 < 0.0 && t1 < 0.0 {
            return None;
        }

        // Choose the intersection point that is closer to the ray origin
        let distance = if t0 < 0.0 {
            t1
        } else if t1 < 0.0 {
            t0
        } else if t0 < t1 {
            t0
        } else {
            t1
        };

        let hit_point = ray.origin + distance * ray.direction;

        // Vector from sphere origin to hit point
        let hit_vec = hit_point.to_vec();

        let normal = hit_vec.normalize();

        // Calculate UV coordinates from spherical coordinates
        let tex_x = (1.0 + hit_vec.z.atan2(hit_vec.x) / f32::consts::PI) * 0.5;
        let tex_y = hit_vec.y.acos() / f32::consts::PI;

        let tex_coords = Vector2::new(tex_x, tex_y);

        Some(Hit::new(hit_point, distance, normal, tex_coords))
    }
}
