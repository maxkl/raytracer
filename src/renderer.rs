
use std::f32;

use image::{DynamicImage, GenericImage, Pixel};
use cgmath::{InnerSpace, Vector3};

use crate::color::Color;
use crate::ray::{Ray, Hit};
use crate::scene::Scene;

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
    pub fn render(&self) -> DynamicImage {
        let size = self.scene.image_size;
        self.render_rect(0, 0, size.0, size.1)
    }

    pub fn render_rect(&self, x: u32, y: u32, w: u32, h: u32) -> DynamicImage {
        let full_image_size = self.scene.image_size;

        let mut img = DynamicImage::new_rgb8(w, h);

        // Iterate over the entire image pixel by pixel
        for y_local in 0..h {
            for x_local in 0..w {
                // Construct ray
                let ray = Ray::from_screen_coordinates(x + x_local, y + y_local, full_image_size.0, full_image_size.1, 45.0);
                // Assign appropriate color
                let color = self.cast_ray(&ray, 0);
                // Assign pixel value
                img.put_pixel(x_local, y_local, color.to_image_color().to_rgba());
            }
        }

        img
    }

    fn cast_ray(&self, ray: &Ray, depth: u32) -> Color {
        if depth > self.scene.max_recursion_depth {
            return Color::black();
        }

        self.scene.trace(ray)
            .map(|hit| self.get_color(ray, &hit, depth))
            .unwrap_or(self.scene.clear_color)
    }

    fn get_color(&self, ray: &Ray, hit: &Hit, depth: u32) -> Color {
        let is_refractive = hit.material.transparency > 0.0;
        let is_reflective = hit.material.reflectivity > 0.0 || is_refractive;

        let diffuse_color = self.shade_diffuse(hit);

        let reflective_color = if is_reflective {
            let reflection_ray = Ray::create_reflection(&hit.normal, &ray.direction, &hit.point);
            self.cast_ray(&reflection_ray, depth + 1)
        } else {
            Color::black()
        };

        let refractive_color = if is_refractive {
            let k_r = self.calc_fresnel_reflectivity(&hit.normal, &ray.direction, hit.material.refractive_index);

            let transmission_ray = Ray::create_transmission(&hit.normal, &ray.direction, &hit.point, hit.material.refractive_index);
            let refractive_color = transmission_ray
                .map(|transmission_ray| self.cast_ray(&transmission_ray, depth + 1))
                .unwrap_or_else(|| Color::black());

            k_r * reflective_color + (1.0 - k_r) * refractive_color
        } else {
            Color::black()
        };

        (diffuse_color * (1.0 - hit.material.reflectivity - hit.material.transparency) + reflective_color * hit.material.reflectivity + refractive_color * hit.material.transparency).clamp()
    }

    fn shade_diffuse(&self, hit: &Hit) -> Color {
        let material_color = hit.material.color.color(&hit.tex_coords);

        let mut color = material_color * self.scene.ambient_light_color;

        // Sum contributions by all light sources
        for light in self.scene.lights.iter() {
            // Vector that points towards the light
            let to_light = light.direction_from(&hit.point);

            // Cast ray towards the light to check whether the point lies in the shadow
            let shadow_ray = Ray { origin: hit.point, direction: to_light };
            let shadow_hit = self.scene.trace(&shadow_ray);
            // Is there any object in the direction of the light that is closer than the light source?
            let in_light = match shadow_hit {
                Some(shadow_hit) => shadow_hit.distance > light.distance_at(&hit.point),
                None => true,
            };

            if in_light {
                // Calculate color using Lambert's Cosine Law
                let light_power = hit.normal.dot(to_light).max(0.0) * light.intensity_at(&hit.point);
                let reflection_factor = hit.material.albedo / f32::consts::PI;
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