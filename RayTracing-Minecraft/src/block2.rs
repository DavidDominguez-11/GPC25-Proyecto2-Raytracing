// src/block2.rs

use raylib::prelude::Vector3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct TexturedBlock {
    pub min_bounds: Vector3,
    pub max_bounds: Vector3,
    pub top_bottom_texture: String,
    pub side_texture: String,
    pub material: Material,
}

impl TexturedBlock {
    pub fn new(
        center: Vector3,
        size: f32,
        top_bottom_texture: String,
        side_texture: String,
        material: Material,
    ) -> Self {
        let half_size = Vector3::new(size / 2.0, size / 2.0, size / 2.0);
        Self {
            min_bounds: center - half_size,
            max_bounds: center + half_size,
            top_bottom_texture,
            side_texture,
            material,
        }
    }

    fn get_uv(&self, point: &Vector3, normal: &Vector3) -> (f32, f32) {
        let size = self.max_bounds - self.min_bounds;
        let u: f32;
        let v: f32;

        if normal.x.abs() > 0.5 {
            // Caras en X (laterales)
            u = (point.z - self.min_bounds.z) / size.z;
            v = 1.0 - (point.y - self.min_bounds.y) / size.y; // ← ¡Invertido!
        } else if normal.y.abs() > 0.5 {
            // Caras en Y (arriba/abajo) → NO se invierte
            u = (point.x - self.min_bounds.x) / size.x;
            v = (point.z - self.min_bounds.z) / size.z;
        } else {
            // Caras en Z (frontal/trasera)
            u = (point.x - self.min_bounds.x) / size.x;
            v = 1.0 - (point.y - self.min_bounds.y) / size.y; // ← ¡Invertido!
        }
        (u, v)
    }
}

impl RayIntersect for TexturedBlock {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect {
        let inv_dir = Vector3::new(
            if ray_direction.x.abs() < 1e-8 { f32::INFINITY } else { 1.0 / ray_direction.x },
            if ray_direction.y.abs() < 1e-8 { f32::INFINITY } else { 1.0 / ray_direction.y },
            if ray_direction.z.abs() < 1e-8 { f32::INFINITY } else { 1.0 / ray_direction.z },
        );

        let mut tmin = (self.min_bounds.x - ray_origin.x) * inv_dir.x;
        let mut tmax = (self.max_bounds.x - ray_origin.x) * inv_dir.x;
        if tmin > tmax { std::mem::swap(&mut tmin, &mut tmax); }

        let mut tymin = (self.min_bounds.y - ray_origin.y) * inv_dir.y;
        let mut tymax = (self.max_bounds.y - ray_origin.y) * inv_dir.y;
        if tymin > tymax { std::mem::swap(&mut tymin, &mut tymax); }

        if tmin > tymax || tymin > tmax {
            return Intersect::empty();
        }
        if tymin > tmin { tmin = tymin; }
        if tymax < tmax { tmax = tymax; }

        let mut tzmin = (self.min_bounds.z - ray_origin.z) * inv_dir.z;
        let mut tzmax = (self.max_bounds.z - ray_origin.z) * inv_dir.z;
        if tzmin > tzmax { std::mem::swap(&mut tzmin, &mut tzmax); }

        if tmin > tzmax || tzmin > tmax {
            return Intersect::empty();
        }
        if tzmin > tmin { tmin = tzmin; }
        if tzmax < tmax { tmax = tzmax; }

        let distance = if tmin > 0.001 { tmin } else { tmax };
        if distance < 0.001 {
            return Intersect::empty();
        }

        let point = *ray_origin + *ray_direction * distance;

        let epsilon = 1e-4;
        let mut normal = Vector3::zero();
        if (point.x - self.min_bounds.x).abs() < epsilon { normal.x = -1.0; }
        else if (point.x - self.max_bounds.x).abs() < epsilon { normal.x = 1.0; }
        else if (point.y - self.min_bounds.y).abs() < epsilon { normal.y = -1.0; }
        else if (point.y - self.max_bounds.y).abs() < epsilon { normal.y = 1.0; }
        else if (point.z - self.min_bounds.z).abs() < epsilon { normal.z = -1.0; }
        else if (point.z - self.max_bounds.z).abs() < epsilon { normal.z = 1.0; }

        let (u, v) = self.get_uv(&point, &normal);

        Intersect::textured_block(
            self.material.clone(),
            distance,
            normal,
            point,
            u,
            v,
            self.top_bottom_texture.clone(),
            self.side_texture.clone(),
        )
    }
}