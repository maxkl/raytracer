
use std::error::Error;
use std::path::PathBuf;

use serde::{Serialize, Deserialize, Deserializer, Serializer};
use cgmath::Vector2;

use crate::math_util::Modulo;
use crate::color::Color;
use crate::image::RgbImage;
use crate::asset_loader;

/// Represents a texture.
///
/// Serializes/deserializes to/from a string, which is the path to the image file
#[derive(Clone)]
pub struct Texture {
    pub path: PathBuf,
    pub img: RgbImage,
}

impl Serialize for Texture {
    /// Serialize this texture to a string, which is the image file path
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        // Serialize file path
        self.path.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Texture {
    /// Deserialize a texture from a string, which is the image file path
    fn deserialize<D>(deserializer: D) -> Result<Texture, D::Error>
    where
        D: Deserializer<'de>
    {
        // Deserialize file path
        let path = PathBuf::deserialize(deserializer)?;
        // Load texture image from path
        Self::load(path.clone()).map_err(|err| {
            serde::de::Error::custom(format!("Unable to open image file \"{}\": {}", path.display(), err))
        })
    }
}

impl Texture {
    /// Load a texture from an image file
    fn load(path: PathBuf) -> Result<Texture, Box<dyn Error>> {
        let a = asset_loader::get_instance();
        let img = a.load_image(&path)?;
        Ok(Texture {
            path,
            img,
        })
    }

    fn sample_nearest(&self, tex_coords: &Vector2<f32>) -> Color {
        let tex_w = self.img.width() as f32;
        let tex_h = self.img.height() as f32;

        let tex_x = (tex_coords.x * tex_w).round().modulo(tex_w) as usize;
        let tex_y = (tex_coords.y * tex_h).round().modulo(tex_h) as usize;

        Color::from_u8(&self.img.get_pixel(tex_x, tex_y))
    }

    fn sample_bilinear(&self, tex_coords: &Vector2<f32>) -> Color {
        let tex_w = self.img.width() as f32;
        let tex_h = self.img.height() as f32;

        let tex_x = tex_coords.x * tex_w;
        let tex_y = tex_coords.y * tex_h;

        let tex_x_1 = tex_x.floor();
        let tex_x_2 = tex_x.ceil();
        let tex_y_1 = tex_y.floor();
        let tex_y_2 = tex_y.ceil();

        let tex_x_1_wrapped = tex_x_1.modulo(tex_w) as usize;
        let tex_x_2_wrapped = tex_x_2.modulo(tex_w) as usize;
        let tex_y_1_wrapped = tex_y_1.modulo(tex_h) as usize;
        let tex_y_2_wrapped = tex_y_2.modulo(tex_h) as usize;

        let color_1_1 = Color::from_u8(&self.img.get_pixel(tex_x_1_wrapped, tex_y_1_wrapped));
        let color_2_1 = Color::from_u8(&self.img.get_pixel(tex_x_2_wrapped, tex_y_1_wrapped));
        let color_1_2 = Color::from_u8(&self.img.get_pixel(tex_x_1_wrapped, tex_y_2_wrapped));
        let color_2_2 = Color::from_u8(&self.img.get_pixel(tex_x_2_wrapped, tex_y_2_wrapped));

        let x_exact = tex_x_1 == tex_x_2;
        let y_exact = tex_y_1 == tex_y_2;
        if x_exact && y_exact {
            color_1_1
        } else if y_exact {
            color_1_1 * (tex_x_2 - tex_x) + color_2_1 * (tex_x - tex_x_1)
        } else if x_exact {
            color_1_1 * (tex_y_2 - tex_y) + color_1_2 * (tex_y - tex_y_1)
        } else {
            color_1_1 * (tex_x_2 - tex_x) * (tex_y_2 - tex_y)
                + color_2_1 * (tex_x - tex_x_1) * (tex_y_2 - tex_y)
                + color_1_2 * (tex_x_2 - tex_x) * (tex_y - tex_y_1)
                + color_2_2 * (tex_x - tex_x_1) * (tex_y - tex_y_1)
        }
    }
}

/// Represents the various ways a point can be colored
#[derive(Clone, Serialize, Deserialize)]
pub enum Coloration {
    /// Uniform color
    Color(Color),
    /// Get color for each point from a texture
    Texture(Texture),
}

impl Coloration {
    /// Calculate color at a specific position
    pub fn color(&self, tex_coords: &Vector2<f32>) -> Color {
        match self {
            Coloration::Color(color) => *color,
            Coloration::Texture(tex) => tex.sample_bilinear(tex_coords),
        }
    }
}

/// Data struct collecting various material properties
#[derive(Clone, Serialize, Deserialize)]
pub struct Material {
    pub color: Coloration,
    pub albedo: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
}
