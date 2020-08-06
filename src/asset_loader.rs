
use std::path::Path;
use std::error::Error;

use crate::image::RgbImage;

pub trait AssetLoader {
    fn load_image(path: &Path) -> Result<RgbImage, Box<dyn Error>>;
}
