
use serde::{Serialize, Deserialize};

use crate::color::Color;
use crate::ray::{Ray, Hit, Intersectable};
use crate::lights::Light;
use crate::material::{Material, ImageLoader};

#[derive(Clone, Serialize, Deserialize)]
pub struct Object<L: ImageLoader> {
    pub shape: Box<dyn Intersectable>,
    pub material: Material<L>,
}

impl<L: ImageLoader> Object<L> {
    pub fn intersect(&self, ray: &Ray) -> Option<(&Object<L>, Hit)> {
        self.shape.intersect(ray)
            .map(|hit| (self, hit))
    }
}

/// Holds all information about the scene
#[derive(Clone, Serialize, Deserialize)]
pub struct Scene<L: ImageLoader> {
    pub image_size: (usize, usize),
    /// Background color, assigned to pixels that are not covered by any object in the scene
    pub clear_color: Color,
    pub objects: Vec<Object<L>>,
    pub ambient_light_color: Color,
    pub lights: Vec<Box<dyn Light>>,
    pub max_recursion_depth: u32,
}

impl<L: ImageLoader> Scene<L> {
    /// Check ray intersections against all objects in the scene and return the closest hit
    pub fn trace(&self, ray: &Ray) -> Option<(&Object<L>, Hit)> {
        self.objects.iter()
            .filter_map(|obj| obj.intersect(ray))
            .min_by(|(_, hit1), (_, hit2)| hit1.cmp(hit2))
    }
}
