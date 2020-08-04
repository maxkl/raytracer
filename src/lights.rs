
use std::f32;

use cgmath::{Vector3, Point3, InnerSpace};
use serde::{Serialize, Deserialize};
use dyn_clone::DynClone;

use crate::color::Color;
use crate::math_util::deserialize_normalized;

#[typetag::serde(tag = "type")]
pub trait Light: DynClone + Send {
    fn direction_from(&self, point: &Point3<f32>) -> Vector3<f32>;
    fn color(&self, ) -> Color;
    fn intensity_at(&self, point: &Point3<f32>) -> f32;
    fn distance_at(&self, point: &Point3<f32>) -> f32;
}

dyn_clone::clone_trait_object!(Light);

/// A light that only has a direction, e.g. from the sun
#[derive(Clone, Serialize, Deserialize)]
pub struct DirectionalLight {
    #[serde(deserialize_with = "deserialize_normalized")]
    pub direction: Vector3<f32>,
    pub color: Color,
    pub intensity: f32,
}

#[typetag::serde]
impl Light for DirectionalLight {
    #[allow(unused_variables)]
    fn direction_from(&self, point: &Point3<f32>) -> Vector3<f32> {
        -self.direction
    }

    fn color(&self) -> Color {
        self.color
    }

    #[allow(unused_variables)]
    fn intensity_at(&self, point: &Point3<f32>) -> f32 {
        self.intensity
    }

    #[allow(unused_variables)]
    fn distance_at(&self, point: &Point3<f32>) -> f32 {
        f32::INFINITY
    }
}

/// A light that's only a single point and radiates uniformly in all directions
#[derive(Clone, Serialize, Deserialize)]
pub struct PointLight {
    pub point: Point3<f32>,
    pub color: Color,
    pub intensity: f32,
}

#[typetag::serde]
impl Light for PointLight {
    fn direction_from(&self, point: &Point3<f32>) -> Vector3<f32> {
        (self.point - point).normalize()
    }

    fn color(&self) -> Color {
        self.color
    }

    fn intensity_at(&self, point: &Point3<f32>) -> f32 {
        // Inverse Square Law
        let distance_squared = (self.point - point).magnitude2();
        self.intensity / (4.0 * f32::consts::PI * distance_squared)
    }

    fn distance_at(&self, point: &Point3<f32>) -> f32 {
        (self.point - point).magnitude()
    }
}
