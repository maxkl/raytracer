
use cgmath::{Point3, Vector3};

use crate::ray::Ray;

#[derive(Clone)]
pub struct AABB {
    min: Point3<f32>,
    max: Point3<f32>,
}

impl AABB {
    pub fn new(p1: &Point3<f32>, p2: &Point3<f32>) -> AABB {
        AABB {
            min: Point3::new(
                p1.x.min(p2.x),
                p1.y.min(p2.y),
                p1.z.min(p2.z),
            ),
            max: Point3::new(
                p1.x.max(p2.x),
                p1.y.max(p2.y),
                p1.z.max(p2.z),
            ),
        }
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        let dirfrac: Vector3<f32> = 1.0 / ray.direction;

        let t1 = (self.min.x - ray.origin.x) * dirfrac.x;
        let t2 = (self.max.x - ray.origin.x) * dirfrac.x;
        let t3 = (self.min.y - ray.origin.y) * dirfrac.y;
        let t4 = (self.max.y - ray.origin.y) * dirfrac.y;
        let t5 = (self.min.z - ray.origin.z) * dirfrac.z;
        let t6 = (self.max.z - ray.origin.z) * dirfrac.z;

        let tmin = f32::max(f32::max(f32::min(t1, t2), f32::min(t3, t4)), f32::min(t5, t6));
        let tmax = f32::min(f32::min(f32::max(t1, t2), f32::max(t3, t4)), f32::max(t5, t6));

        if tmax < 0.0 {
            false
        } else if tmin > tmax {
            false
        } else {
            true
        }
    }
}
