
use std::path::Path;
use std::error::Error;

use once_cell::sync::OnceCell;

use crate::image::RgbImage;
use crate::mesh::MeshData;

pub trait AssetLoader: Send + Sync {
    fn load_image(&self, path: &Path) -> Result<RgbImage, Box<dyn Error>>;

    fn load_obj(&self, path: &Path) -> Result<MeshData, Box<dyn Error>>;
}

static INSTANCE: OnceCell<Box<dyn AssetLoader>> = OnceCell::new();

pub fn set_instance(instance: Box<dyn AssetLoader>) {
    INSTANCE.set(instance)
        .ok()
        .expect("Instance already set");
}

pub fn get_instance() -> &'static Box<dyn AssetLoader> {
    INSTANCE.get()
        .expect("Instance not set")
}
