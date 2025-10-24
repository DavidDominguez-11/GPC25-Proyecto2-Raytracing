// src/ray_intersect.rs

use raylib::prelude::{Color, Vector3};
use crate::material::Material;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Intersect {
    pub material: Material,
    pub distance: f32,
    pub is_intersecting: bool,
    pub normal: Vector3,
    pub point: Vector3,
    pub u: f32,
    pub v: f32,
    // Nuevos campos para TexturedBlock
    pub top_bottom_texture: Option<String>,
    pub side_texture: Option<String>,
}

impl Intersect {
    /// Constructor para objetos normales (Cube, Slab, etc.)
    pub fn new(material: Material, distance: f32, normal: Vector3, point: Vector3, u: f32, v: f32) -> Self {
        Intersect {
            material,
            distance,
            is_intersecting: true,
            normal,
            point,
            u,
            v,
            top_bottom_texture: None,
            side_texture: None,
        }
    }

    /// Constructor especial para TexturedBlock
    pub fn textured_block(
        material: Material,
        distance: f32,
        normal: Vector3,
        point: Vector3,
        u: f32,
        v: f32,
        top_bottom_texture: String,
        side_texture: String,
    ) -> Self {
        Intersect {
            material,
            distance,
            is_intersecting: true,
            normal,
            point,
            u,
            v,
            top_bottom_texture: Some(top_bottom_texture),
            side_texture: Some(side_texture),
        }
    }

    pub fn empty() -> Self {
        Intersect {
            material: Material::black(),
            distance: 0.0,
            is_intersecting: false,
            normal: Vector3::zero(),
            point: Vector3::zero(),
            u: 0.0,
            v: 0.0,
            top_bottom_texture: None,
            side_texture: None,
        }
    }
}

pub trait RayIntersect: Send + Sync {
    fn ray_intersect(&self, ray_origin: &Vector3, ray_direction: &Vector3) -> Intersect;
}