// slab.rs
use raylib::prelude::Vector3;
use crate::ray_intersect::{Intersect, RayIntersect};
use crate::material::Material;

pub struct Slab {
    pub min_bounds: Vector3,
    pub max_bounds: Vector3,
    pub material: Material,
}

impl Slab {
    /// Crea un nuevo slab (mitad de altura) centrado en `center`.
    /// El slab ocupa la mitad inferior del volumen de un cubo de lado `size`.
    pub fn new(center: Vector3, size: f32, material: Material) -> Self {
        let half_width = size / 2.0;
        let half_depth = size / 2.0;
        let half_height = size / 4.0; // Mitad de la mitad → altura total = size/2

        // El slab está alineado en la parte inferior:
        // - Z y X: centrados como un cubo normal
        // - Y: va desde (center.y - size/2) hasta (center.y)
        let min_bounds = Vector3::new(
            center.x - half_width,
            center.y - half_height * 2.0, // = center.y - size/2
            center.z - half_depth,
        );
        let max_bounds = Vector3::new(
            center.x + half_width,
            center.y, // Solo hasta el centro en Y → mitad de altura
            center.z + half_depth,
        );

        Self {
            min_bounds,
            max_bounds,
            material,
        }
    }

    /// Calcula las coordenadas UV para texturizar, similar al cubo.
    fn get_uv(&self, point: &Vector3, normal: &Vector3) -> (f32, f32) {
        let size = self.max_bounds - self.min_bounds;
        let u: f32;
        let v: f32;

        if normal.x.abs() > 0.5 { // Caras laterales (normal en X)
            u = (point.z - self.min_bounds.z) / size.z;
            v = (point.y - self.min_bounds.y) / size.y;
        } else if normal.y.abs() > 0.5 { // Cara superior o inferior (normal en Y)
            u = (point.x - self.min_bounds.x) / size.x;
            v = (point.z - self.min_bounds.z) / size.z;
        } else { // Caras frontal/trasera (normal en Z)
            u = (point.x - self.min_bounds.x) / size.x;
            v = (point.y - self.min_bounds.y) / size.y;
        }
        (u, v)
    }
}

impl RayIntersect for Slab {
    /// Usa el mismo algoritmo "Slab" que el cubo, ya que es un AABB (Axis-Aligned Bounding Box).
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

        // Determinar la normal
        let epsilon = 1e-4;
        let mut normal = Vector3::zero();

        if (point.x - self.min_bounds.x).abs() < epsilon { normal.x = -1.0; }
        else if (point.x - self.max_bounds.x).abs() < epsilon { normal.x = 1.0; }
        else if (point.y - self.min_bounds.y).abs() < epsilon { normal.y = -1.0; }
        else if (point.y - self.max_bounds.y).abs() < epsilon { normal.y = 1.0; }
        else if (point.z - self.min_bounds.z).abs() < epsilon { normal.z = -1.0; }
        else if (point.z - self.max_bounds.z).abs() < epsilon { normal.z = 1.0; }

        let (u, v) = self.get_uv(&point, &normal);

        Intersect::new(
            self.material.clone(),
            distance,
            normal,
            point,
            u,
            v,
        )
    }
}