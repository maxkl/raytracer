
use std::f32;

use cgmath::{InnerSpace, Vector3};
use rand::{thread_rng, Rng};
use rand_distr::Normal;

use crate::color::Color;
use crate::image::RgbImage;
use crate::ray::{Ray, Hit};
use crate::scene::{Scene, Object};

pub struct Renderer {
    scene: Scene,
}

impl Renderer {
    pub fn new(scene: Scene) -> Renderer {
        Renderer {
            scene,
        }
    }

    /// Render the scene to a new image
    pub fn render(&self) -> RgbImage {
        let size = self.scene.camera.resolution;
        self.render_rect(0, 0, size.0, size.1)
    }

    pub fn render_rect(&self, x: usize, y: usize, w: usize, h: usize) -> RgbImage {
        let camera = &self.scene.camera;
        let full_image_size = camera.resolution;

        let mut img = RgbImage::new(w, h);

        let aa_samples = self.scene.aa_samples;
        let mut rng = thread_rng();
        let distr = Normal::new(0.0f32, 0.4).unwrap();

        // Iterate over the entire image pixel by pixel
        for y_local in 0..h {
            for x_local in 0..w {
                let mut color_sum = Color::black();
                for _ in 0..aa_samples {
                    // This is not a true bivariate normal distribution but it's good enough
                    let sample_x = (x + x_local) as f32 + rng.sample::<f32, _>(distr);
                    let sample_y = (y + y_local) as f32 + rng.sample::<f32, _>(distr);
                    // Construct ray
                    let camera_ray = Ray::from_screen_coordinates(sample_x, sample_y, full_image_size.0, full_image_size.1, camera.fov);
                    let world_ray = camera_ray.transform(&camera.transformation_matrix);
                    // Assign appropriate color
                    let color = self.cast_ray(&world_ray, 0);

                    color_sum += color;
                }
                let color = color_sum / aa_samples as f32;
                // Assign pixel value
                img.put_pixel(x_local, y_local, &color.to_u8());
            }
        }

        img
    }

    fn cast_ray(&self, ray: &Ray, depth: u32) -> Color {
        if depth > self.scene.max_recursion_depth {
            return Color::black();
        }

        let base_color = self.scene.trace(ray)
            .map(|(obj, hit)| self.get_color(ray, obj, &hit, depth))
            .unwrap_or(self.scene.clear_color);

        let debug_data = ray.debug_data.borrow();
        let kd_tree_lookups_value = debug_data.kd_tree_lookups.min(100) as f32 * (1.0 / 100.0);
        let debug_color = Color::new(kd_tree_lookups_value, 0.0, 0.0);

        base_color + debug_color
    }

    fn get_color(&self, ray: &Ray, obj: &Object, hit: &Hit, depth: u32) -> Color {
        let material = &self.scene.materials[obj.material_index];

        let is_refractive = material.transparency > 0.0;
        let is_reflective = material.reflectivity > 0.0 || is_refractive;

        let diffuse_color = self.shade_diffuse(obj, hit);

        let reflective_color = if is_reflective {
            let reflection_ray = Ray::create_reflection(&hit.normal, &ray.direction, &hit.point);
            self.cast_ray(&reflection_ray, depth + 1)
        } else {
            Color::black()
        };

        let refractive_color = if is_refractive {
            let k_r = self.calc_fresnel_reflectivity(&hit.normal, &ray.direction, material.refractive_index);

            let transmission_ray = Ray::create_transmission(&hit.normal, &ray.direction, &hit.point, material.refractive_index);
            let refractive_color = transmission_ray
                .map(|transmission_ray| self.cast_ray(&transmission_ray, depth + 1))
                .unwrap_or_else(|| Color::black());

            k_r * reflective_color + (1.0 - k_r) * refractive_color
        } else {
            Color::black()
        };

        (diffuse_color * (1.0 - material.reflectivity - material.transparency) + reflective_color * material.reflectivity + refractive_color * material.transparency).clamp()
    }

    fn shade_diffuse(&self, obj: &Object, hit: &Hit) -> Color {
        let material = &self.scene.materials[obj.material_index];
        let material_color = material.color.color(&hit.tex_coords);

        let mut color = material_color * self.scene.ambient_light_color;

        // Sum contributions by all light sources
        for light in self.scene.lights.iter() {
            // Vector that points towards the light
            let to_light = light.direction_from(&hit.point);

            // Cast ray towards the light to check whether the point lies in the shadow
            let shadow_ray = Ray::new(hit.point + hit.normal * 1e-5, to_light);
            let shadow_hit = self.scene.trace(&shadow_ray);
            // Is there any object in the direction of the light that is closer than the light source?
            let in_light = match shadow_hit {
                Some((_, shadow_hit)) => shadow_hit.distance > light.distance_at(&hit.point),
                None => true,
            };

            if in_light {
                // Calculate color using Lambert's Cosine Law
                let light_power = hit.normal.dot(to_light).max(0.0) * light.intensity_at(&hit.point);
                let reflection_factor = material.albedo / f32::consts::PI;
                color += material_color * light.color() * light_power * reflection_factor;
            }
        }

        // Ensure that color components are between 0.0 and 1.0
        color.clamp()
    }

    fn calc_fresnel_reflectivity(&self, normal: &Vector3<f32>, incident: &Vector3<f32>, refractive_index: f32) -> f32 {
        let eta_t;
        let eta_i;
        let mut i_dot_n = incident.dot(*normal);
        if i_dot_n < 0.0 {
            i_dot_n = -i_dot_n;

            eta_t = refractive_index;
            eta_i = 1.0;
        } else {
            eta_t = 1.0;
            eta_i = refractive_index;
        }

        let sin_theta_t = eta_i / eta_t * (1.0 - i_dot_n.powi(2)).sqrt();

        if sin_theta_t >= 1.0 {
            1.0
        } else {
            let cos_theta_t = (1.0 - sin_theta_t.powi(2)).sqrt();
            let r_s = (eta_t * i_dot_n - eta_i * cos_theta_t) / (eta_t * i_dot_n + eta_i * cos_theta_t);
            let r_p = (eta_i * i_dot_n - eta_t * cos_theta_t) / (eta_i * i_dot_n + eta_t * cos_theta_t);
            0.5 * (r_s.powi(2) + r_p.powi(2))
        }
    }
}