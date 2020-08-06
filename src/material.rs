
use std::error::Error;
use std::path::PathBuf;
use std::marker::PhantomData;

use serde::{Serialize, Deserialize, Deserializer, Serializer};

use crate::math_util::Modulo;
use crate::color::Color;
use crate::image::RgbImage;
use crate::asset_loader::AssetLoader;

/// Generic texture/UV coordinates
#[derive(Copy, Clone)]
pub struct TexCoords<T> {
    pub u: T,
    pub v: T,
}

/// Represents a texture.
///
/// Serializes/deserializes to/from a string, which is the path to the image file
#[derive(Clone)]
pub struct Texture<L: AssetLoader> {
    pub path: PathBuf,
    pub img: RgbImage,
    phantom: PhantomData<L>,
}

impl<L: AssetLoader> Serialize for Texture<L> {
    /// Serialize this texture to a string, which is the image file path
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        // Serialize file path
        self.path.serialize(serializer)
    }
}

impl<'de, L: AssetLoader> Deserialize<'de> for Texture<L> {
    /// Deserialize a texture from a string, which is the image file path
    fn deserialize<D>(deserializer: D) -> Result<Texture<L>, D::Error>
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

impl<L: AssetLoader> Texture<L> {
    /// Load a texture from an image file
    fn load(path: PathBuf) -> Result<Texture<L>, Box<dyn Error>> {
        let img = L::load_image(&path)?;
        Ok(Texture {
            path,
            img,
            phantom: PhantomData,
        })
    }

    fn sample_nearest(&self, tex_coords: &TexCoords<f32>) -> Color {
        let tex_w = self.img.width() as f32;
        let tex_h = self.img.height() as f32;

        let tex_x = (tex_coords.u * tex_w).round().modulo(tex_w) as usize;
        let tex_y = (tex_coords.v * tex_h).round().modulo(tex_h) as usize;

        Color::from_u8(&self.img.get_pixel(tex_x, tex_y))
    }

    fn sample_bilinear(&self, tex_coords: &TexCoords<f32>) -> Color {
        let tex_w = self.img.width() as f32;
        let tex_h = self.img.height() as f32;

        let tex_x = tex_coords.u * tex_w;
        let tex_y = tex_coords.v * tex_h;

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
pub enum Coloration<L: AssetLoader> {
    /// Uniform color
    Color(Color),
    /// Get color for each point from a texture
    Texture(Texture<L>),
}

impl<L: AssetLoader> Coloration<L> {
    /// Calculate color at a specific position
    pub fn color(&self, tex_coords: &TexCoords<f32>) -> Color {
        match self {
            Coloration::Color(color) => *color,
            Coloration::Texture(tex) => tex.sample_bilinear(tex_coords),
        }
    }
}

/// Data struct collecting various material properties
#[derive(Clone, Serialize, Deserialize)]
pub struct Material<L: AssetLoader> {
    pub color: Coloration<L>,
    pub albedo: f32,
    pub reflectivity: f32,
    pub transparency: f32,
    pub refractive_index: f32,
}
