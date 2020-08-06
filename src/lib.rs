mod math_util;
mod color;
mod image;
mod material;
mod ray;
mod primitives;
mod lights;
mod scene;
mod asset_loader;
mod renderer;

pub use image::RgbImage;
pub use asset_loader::AssetLoader;
pub use scene::Scene;
pub use renderer::Renderer;
