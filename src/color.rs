
use std::ops::{Add, AddAssign, Mul};

use image::{Rgb, Pixel};
use serde::{Serialize, Deserialize};

/// Represents RGB colors
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Add for Color {
    type Output = Color;

    fn add(self, rhs: Color) -> Color {
        Color {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
        }
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Color) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        Color {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
        }
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(self, rhs: f32) -> Color {
        Color {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
        }
    }
}

impl Mul<Color> for f32 {
    type Output = Color;

    fn mul(self, rhs: Color) -> Color {
        rhs * self
    }
}

impl Color {
    /// Construct a new Color struct
    pub fn new(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b }
    }

    pub fn from_rgb(rgb: Rgb<u8>) -> Color {
        let data = rgb.channels();
        Color {
            r: data[0] as f32 / 255.0,
            g: data[1] as f32 / 255.0,
            b: data[2] as f32 / 255.0,
        }
    }

    /// Construct a Color struct with all components set to 0.0
    pub fn black() -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    pub fn clamp(&self) -> Color {
        Color {
            r: self.r.min(1.0).max(0.0),
            g: self.g.min(1.0).max(0.0),
            b: self.b.min(1.0).max(0.0),
        }
    }

    /// Convert to 8-bit Rgb struct from `image` crate
    pub fn to_image_color(&self) -> Rgb<u8> {
        Rgb([
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        ])
    }
}
