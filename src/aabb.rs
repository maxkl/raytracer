
use cgmath::{Point3, Vector3};

use crate::ray::Ray;
use crate::math_util::Axis;

#[derive(Clone)]
pub struct AABB {
    pub min: Point3<f32>,
    pub max: Point3<f32>,
}

impl AABB {
    pub fn empty() -> AABB {
        AABB {
            min: Point3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: Point3::new(-f32::INFINITY, -f32::INFINITY, -f32::INFINITY),
        }
    }

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

    pub fn from_triangle(p1: &Vector3<f32>, p2: &Vector3<f32>, p3: &Vector3<f32>) -> AABB {
        AABB {
            min: Point3::new(
                p1.x.min(p2.x).min(p3.x),
                p1.y.min(p2.y).min(p3.y),
                p1.z.min(p2.z).min(p3.z),
            ),
            max: Point3::new(
                p1.x.max(p2.x).max(p3.x),
                p1.y.max(p2.y).max(p3.y),
                p1.z.max(p2.z).max(p3.z),
            ),
        }
    }

    pub fn union(&self, other: &AABB) -> AABB {
        AABB {
            min: Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    pub fn maximum_extent(&self) -> Axis {
        let extent = self.max - self.min;

        if extent.x > extent.y && extent.x > extent.z {
            Axis::X
        } else if extent.y > extent.z {
            Axis::Y
        } else {
            Axis::Z
        }
    }

    pub fn intersects_p(&self, ray: &Ray) -> Option<(f32, f32)> {
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
            None
        } else if tmin > tmax {
            None
        } else {
            Some((tmin, tmax))
        }
    }

    pub fn intersects(&self, ray: &Ray) -> bool {
        self.intersects_p(ray).is_some()
    }
}
