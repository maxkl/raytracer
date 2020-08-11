
use std::ops::{Index, IndexMut};

use cgmath::{VectorSpace, InnerSpace, BaseFloat, Vector3, Point3};
use serde::{Deserialize, Deserializer};

/// Deserialize a vector and normalize it
///
/// Usage example:
/// ```ignore
/// #[derive(Deserialize)]
/// struct Car {
///     color: (f64, f64, f64),
///     #[serde(deserialize_with = "deserialize_normalized")]
///     direction: Vector3<f64>,
///     speed: f64,
/// }
/// ```
pub fn deserialize_normalized<'de, T, D>(deserializer: D) -> Result<T, D::Error>
    where
        T: InnerSpace + Deserialize<'de>,
        <T as VectorSpace>::Scalar: BaseFloat,
        D: Deserializer<'de>
{
    Ok(T::deserialize(deserializer)?.normalize())
}

/// The mathematically correct modulo operation
pub trait Modulo<RHS=Self> {
    /// Calculate `self mod rhs`
    fn modulo(&self, rhs: RHS) -> Self;
}

impl Modulo for f32 {
    fn modulo(&self, rhs: f32) -> f32 {
        ((self % rhs) + rhs) % rhs
    }
}

#[derive(Copy, Clone)]
pub enum Axis {
    X = 0,
    Y = 1,
    Z = 2
}

impl From<u32> for Axis {
    fn from(axis_int: u32) -> Axis {
        match axis_int {
            0 => Axis::X,
            1 => Axis::Y,
            2 => Axis::Z,
            _ => panic!("Invalid integer value for axis")
        }
    }
}

impl<S> Index<Axis> for Vector3<S> {
    type Output = S;

    fn index(&self, axis: Axis) -> &S {
        AsRef::<[S; 3]>::as_ref(self).index(axis as usize)
    }
}

impl<S> IndexMut<Axis> for Vector3<S> {
    fn index_mut(&mut self, axis: Axis) -> &mut S {
        AsMut::<[S; 3]>::as_mut(self).index_mut(axis as usize)
    }
}

impl<S> Index<Axis> for Point3<S> {
    type Output = S;

    fn index(&self, axis: Axis) -> &S {
        AsRef::<[S; 3]>::as_ref(self).index(axis as usize)
    }
}

impl<S> IndexMut<Axis> for Point3<S> {
    fn index_mut(&mut self, axis: Axis) -> &mut S {
        AsMut::<[S; 3]>::as_mut(self).index_mut(axis as usize)
    }
}
