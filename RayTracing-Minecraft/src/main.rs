// main.rs
#![allow(unused_imports)]
#![allow(dead_code)]
use raylib::prelude::*;
use std::f32::consts::PI;
use rayon::prelude::*;
use std::mem::size_of;

mod framebuffer;
mod ray_intersect;
mod cube;
mod camera;
mod material;
mod light;
mod snell;
mod textures;
mod slab;
mod block2;
use block2::TexturedBlock;
use framebuffer::Framebuffer;
use ray_intersect::{RayIntersect, Intersect};
use cube::Cube;
use camera::Camera;
use material::{Material, vector3_to_color};
use light::Light;
use snell::{reflect, refract};
use textures::TextureManager;
use slab::Slab;

fn procedural_sky(dir: Vector3) -> Vector3 {
    let d = dir.normalized();
    let t = (d.y + 1.0) * 0.5;
    
    // Colores morados pastel
    let light_lavender = Vector3::new(0.9, 0.8, 1.0);    // Lavanda claro
    let soft_purple = Vector3::new(0.8, 0.7, 1.0);      // Morado suave
    let pale_purple = Vector3::new(0.7, 0.6, 0.95);     // Morado pastel
    let deep_purple = Vector3::new(0.5, 0.4, 0.8);      // Morado más intenso
    
    if t < 0.4 {
        // Horizonte - transición suave
        let k = t / 0.4;
        light_lavender * (1.0 - k) + soft_purple * k
    } else if t < 0.6 {
        // Zona media del cielo
        let k = (t - 0.4) / 0.2;
        soft_purple * (1.0 - k) + pale_purple * k
    } else if t < 0.85 {
        // Hacia el cenit
        let k = (t - 0.6) / 0.25;
        pale_purple * (1.0 - k) + deep_purple * k
    } else {
        // Cenit - morado más profundo
        deep_purple
    }
}

fn cast_shadow(
    intersect: &Intersect,
    light: &Light,
    objects: &[Box<dyn RayIntersect>], //poner sphere si se quiere usar esferas
) -> f32 {
    let light_direction = (light.position - intersect.point).normalized();
    let shadow_ray_origin = intersect.point + intersect.normal * 0.001; // Bias para evitar auto-intersección
    for object in objects {
        let shadow_intersect = object.ray_intersect(&shadow_ray_origin, &light_direction);
        if shadow_intersect.is_intersecting && shadow_intersect.distance < (light.position - shadow_ray_origin).length() {
            return 0.7;
        }
    }
    0.0
}

const ORIGIN_BIAS: f32 = 1e-4;
fn offset_origin(intersect: &Intersect, ray_direction: &Vector3) -> Vector3 {
    let offset = intersect.normal * ORIGIN_BIAS;
    if ray_direction.dot(intersect.normal) < 0.0 {
        intersect.point - offset
    } else {
        intersect.point + offset
    }
}

pub fn cast_ray(
    ray_origin: &Vector3,
    ray_direction: &Vector3,
    objects: &[Box<dyn RayIntersect>], //poner sphere si se quiere usar esferas
    light: &Light,
    depth: u32,
    texture_manager: &TextureManager,
) -> Vector3 {
    if depth > 1 { // Originalmente era 3, reducido para mejor rendimiento
        return procedural_sky(*ray_direction);
    }
    
    let mut intersect = Intersect::empty();
    let mut zbuffer = f32::INFINITY;
    for object in objects {
        let tmp = object.ray_intersect(ray_origin, ray_direction);
        if tmp.is_intersecting && tmp.distance < zbuffer {
            zbuffer = tmp.distance;
            intersect = tmp;
        }
    }
    
    if !intersect.is_intersecting {
        return procedural_sky(*ray_direction);
    }
    
    let light_direction = (light.position - intersect.point).normalized();
    let view_direction = (*ray_origin - intersect.point).normalized();
    
    let mut normal = intersect.normal;
    if let Some(normal_map_path) = &intersect.material.normal_map_id {
        let texture = texture_manager.get_texture(normal_map_path).unwrap();
        let width = texture.width() as u32;
        let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32;
        let ty = (intersect.v * height as f32) as u32;
        
        if let Some(tex_normal) = texture_manager.get_normal_from_map(normal_map_path, tx, ty) {
            let tangent = Vector3::new(normal.y, -normal.x, 0.0).normalized();
            let bitangent = normal.cross(tangent);
            
            let transformed_normal_x = tex_normal.x * tangent.x + tex_normal.y * bitangent.x + tex_normal.z * normal.x;
            let transformed_normal_y = tex_normal.x * tangent.y + tex_normal.y * bitangent.y + tex_normal.z * normal.y;
            let transformed_normal_z = tex_normal.x * tangent.z + tex_normal.y * bitangent.z + tex_normal.z * normal.z;
            
            normal = Vector3::new(transformed_normal_x, transformed_normal_y, transformed_normal_z).normalized();
        }
    }
    
    let reflection_direction = reflect(&-light_direction, &normal).normalized();
    
    let shadow_intensity = cast_shadow(&intersect, light, objects);
    let light_intensity = light.intensity * (1.0 - shadow_intensity);
    
    // Difuso
    let diffuse_intensity = normal.dot(light_direction).max(0.0) * light_intensity;


    let diffuse_color = if intersect.top_bottom_texture.is_some() {
        // Es un TexturedBlock
        let texture_path = if normal.y.abs() > 0.5 {
            intersect.top_bottom_texture.as_ref().unwrap()
        } else {
            intersect.side_texture.as_ref().unwrap()
        };
        let texture = texture_manager.get_texture(texture_path).unwrap();
        let width = texture.width() as u32;
        let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32;
        let ty = (intersect.v * height as f32) as u32;
        texture_manager.get_pixel_color(texture_path, tx, ty)
    } else if let Some(texture_path) = &intersect.material.texture {
        // Es un Cube o Slab con textura uniforme
        let texture = texture_manager.get_texture(texture_path).unwrap();
        let width = texture.width() as u32;
        let height = texture.height() as u32;
        let tx = (intersect.u * width as f32) as u32;
        let ty = (intersect.v * height as f32) as u32;
        texture_manager.get_pixel_color(texture_path, tx, ty)
    } else {
        // Sin textura
        intersect.material.diffuse
    };


    let diffuse = diffuse_color * diffuse_intensity;
    
    // Especular
    let specular_intensity = view_direction.dot(reflection_direction).max(0.0).powf(intersect.material.specular) * light_intensity;
    let specular = light.color * specular_intensity;
    
    // Reflejo
    let mut reflection_color = Vector3::zero(); // Cambiado para rendimiento
    let reflectivity = intersect.material.reflectivity;
    if reflectivity > 0.0 {
        let reflect_direction = reflect(ray_direction, &normal);
        let reflect_origin = offset_origin(&intersect, &reflect_direction);
        reflection_color = cast_ray(&reflect_origin, &reflect_direction, objects, light, depth + 1, texture_manager);
    }
    
    // Transparencia
    let transparency = intersect.material.transparency;
    let mut refraction_color = Vector3::zero(); // Cambiado para rendimiento
    if transparency > 0.0 {
        let refract_direction = refract(ray_direction, &normal, intersect.material.refractive_index);
        let refract_origin = offset_origin(&intersect, &refract_direction);
        refraction_color = cast_ray(&refract_origin, &refract_direction, objects, light, depth + 1, texture_manager);
    }
    
    // Color final
    let color = diffuse * intersect.material.albedo[0] + specular * intersect.material.albedo[1] + 
                reflection_color * reflectivity + refraction_color * transparency;
    color
}

/// Renderiza la escena en paralelo usando múltiples hilos. Devuelve un vector de colores de píxeles que se pueden cargar en el framebuffer.
pub fn render(
    width: i32,
    height: i32,
    objects: &[Box<dyn RayIntersect>],
    camera: &Camera,
    light: &Light,
    texture_manager: &TextureManager,
) -> Vec<Color> {
    let aspect_ratio = width as f32 / height as f32;
    let fov = PI / 3.0;
    let perspective_scale = (fov * 0.5).tan();
    
    // Genera un iterador paralelo para las filas de píxeles (y). flat_map crea un nuevo iterador paralelo para cada píxel (x, y) en la pantalla.
    (0..height)
        .into_par_iter()
        .flat_map(|y| (0..width).into_par_iter().map(move |x| (x, y)))
        .map(|(x, y)| {
            // El cálculo para cada píxel es el mismo que antes.
            let screen_x = (2.0 * x as f32) / width as f32 - 1.0;
            let screen_y = -(2.0 * y as f32) / height as f32 + 1.0;
            let screen_x = screen_x * aspect_ratio * perspective_scale;
            let screen_y = screen_y * perspective_scale;
            let ray_direction = Vector3::new(screen_x, screen_y, -1.0).normalized();
            let rotated_direction = camera.basis_change(&ray_direction);
            let pixel_color_vec = cast_ray(
                &camera.eye,
                &rotated_direction,
                objects,
                light,
                0,
                texture_manager,
            );
            vector3_to_color(pixel_color_vec)
        })
        .collect() // Recolecta los resultados de todos los hilos en un solo Vec<Color>.
}

fn main() {
    let window_width = 800; //1300 para tamaño mas grande con menos fps
    let window_height = 600; //900 para tamaño mas grande con menos fps
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Raytracer Class - Cubes (Optimized)")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    window.set_target_fps(30);
    
    let mut texture_manager = TextureManager::new();
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/TOPENDPORTALwEYE.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/PURPURBLOCK.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/LATENDPORTAL.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/ENDPORTAL.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/ENDBLOCK.png");
    texture_manager.load_texture(&mut window, &raylib_thread, "assets/deepslateBLOCK.png");
    
    // textura de block 2
    //texture_manager.load_texture(&mut window, &raylib_thread, "assets/TOPENDPORTALwEYE.png");
    //texture_manager.load_texture(&mut window, &raylib_thread, "assets/LATENDPORTAL.png");
    
    let top_end_portal_eye = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0), albedo: [0.7, 0.3], specular: 30.0,
        reflectivity: 0.1, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/TOPENDPORTALwEYE.png".to_string()), normal_map_id: None,
    };
    let purpur_block = Material {
        diffuse: Vector3::new(0.7, 0.5, 0.8), albedo: [0.8, 0.2], specular: 10.0,
        reflectivity: 0.05, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/PURPURBLOCK.png".to_string()), normal_map_id: None,
    };
    let lat_end_portal = Material {
        diffuse: Vector3::new(0.3, 0.1, 0.5), albedo: [0.6, 0.4], specular: 25.0,
        reflectivity: 0.15, transparency: 0.3, refractive_index: 1.2,
        texture: Some("assets/LATENDPORTAL.png".to_string()), normal_map_id: None,
    };
    let en_portal = Material {
        diffuse: Vector3::new(0.2, 0.1, 0.4), albedo: [0.5, 0.5], specular: 50.0,
        reflectivity: 0.2, transparency: 0.5, refractive_index: 1.3,
        texture: Some("assets/ENDPORTAL.png".to_string()), normal_map_id: None,
    };
    let end_block = Material {
        diffuse: Vector3::new(0.4, 0.4, 0.4), albedo: [0.7, 0.3], specular: 8.0,
        reflectivity: 0.05, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/ENDBLOCK.png".to_string()), normal_map_id: None,
    };
    let deeslate_block = Material {
        diffuse: Vector3::new(0.3, 0.3, 0.3), albedo: [0.8, 0.2], specular: 5.0,
        reflectivity: 0.02, transparency: 0.0, refractive_index: 1.0,
        texture: Some("assets/deepslateBLOCK.png".to_string()), normal_map_id: None,
    };

    // para el slab
    let stone_slab_mat = Material {
        diffuse: Vector3::new(0.6, 0.6, 0.6),
        albedo: [0.8, 0.2],
        specular: 5.0,
        reflectivity: 0.02,
        transparency: 0.0,
        refractive_index: 1.0,
        texture: Some("assets/deepslateBLOCK.png".to_string()), // o crea una textura específica
        normal_map_id: None,
    };
    let end_stone_slab_mat = Material {
        diffuse: Vector3::new(0.6, 0.6, 0.6),
        albedo: [0.8, 0.2],
        specular: 5.0,
        reflectivity: 0.02,
        transparency: 0.0,
        refractive_index: 1.0,
        texture: Some("assets/ENDBLOCK.png".to_string()), // o crea una textura específica
        normal_map_id: None,
    };
    // Crea un material base (texture debe ser None)
    let block2_material = Material {
        diffuse: Vector3::new(1.0, 1.0, 1.0),
        albedo: [0.8, 0.2],
        specular: 10.0,
        reflectivity: 0.05,
        transparency: 0.0,
        refractive_index: 1.0,
        texture: None, // ¡Importante!
        normal_map_id: None,
    };

    let objects: Vec<Box<dyn RayIntersect>> = vec![
        //Box::new(Cube::new(Vector3::new(-2.5, 0.0, 0.0), 1.0, top_end_portal_eye.clone())),
        Box::new(Cube::new(Vector3::new(-1.5, 0.0, 0.0), 1.0, purpur_block.clone())),
        //Box::new(Cube::new(Vector3::new(-0.5, 0.0, 0.0), 1.0, lat_end_portal.clone())),
        Box::new(Cube::new(Vector3::new(0.5, 0.0, 0.0),  1.0, en_portal.clone())),
        Box::new(Cube::new(Vector3::new(1.5, 0.0, 0.0), 1.0, end_block.clone())),
        Box::new(Cube::new(Vector3::new(2.5, 0.0, 0.0), 1.0, deeslate_block.clone())),
        Box::new(Slab::new(Vector3::new(3.5, 0.0, 0.0), 1.0, stone_slab_mat.clone())),
        Box::new(Slab::new(Vector3::new(4.5, 0.0, 0.0), 1.0, end_stone_slab_mat.clone())),
        Box::new(TexturedBlock::new(
            Vector3::new(5.5, 0.0, 0.0),
            1.0,
            "assets/TOPENDPORTALwEYE.png".to_string(),
            "assets/LATENDPORTAL.png".to_string(),
            block2_material.clone(),
        )),
    ];
    let mut camera = Camera::new(
        Vector3::new(0.0, 0.0, 8.0), //Posicion de la camara
        Vector3::new(0.0, 0.0, 0.0), //Donde mira la camara
        Vector3::new(0.0, 1.0, 0.0), //Donde esta arriba
    );
    let rotation_speed = PI / 100.0;
    let zoom_speed = 0.1;
    let vertical_speed = 0.1;
    let light = Light::new(
        Vector3::new(5.0, 5.0, 5.0), //Posicion de la luz
        Vector3::new(1.0, 1.0, 1.0), //Color de la luz
        1.5, //Intensidad de la luz
    );

    //Creamos la textura una sola vez, fuera del bucle principal.
    let mut texture = window
        .load_texture_from_image(&raylib_thread, &Image::gen_image_color(window_width, window_height, Color::BLACK))
        .expect("No se pudo cargar la textura desde el framebuffer");

    while !window.window_should_close() {
        let start_time = std::time::Instant::now();
        
        if window.is_key_down(KeyboardKey::KEY_LEFT) { camera.orbit(rotation_speed, 0.0); } // left
        if window.is_key_down(KeyboardKey::KEY_RIGHT) { camera.orbit(-rotation_speed, 0.0); } // right
        if window.is_key_down(KeyboardKey::KEY_UP) { camera.orbit(0.0, -rotation_speed); } // up
        if window.is_key_down(KeyboardKey::KEY_DOWN) { camera.orbit(0.0, rotation_speed); } // down
        if window.is_key_down(KeyboardKey::KEY_D) { camera.zoom(zoom_speed); } //Zoom +
        if window.is_key_down(KeyboardKey::KEY_A) { camera.zoom(-zoom_speed); } //Zoom -
        if window.is_key_down(KeyboardKey::KEY_W) { // up
            camera.eye.y += vertical_speed; 
            camera.center.y += vertical_speed;
            camera.update_basis(); 
        }
        if window.is_key_down(KeyboardKey::KEY_S) { // down
            camera.eye.y -= vertical_speed; 
            camera.center.y -= vertical_speed;
            camera.update_basis(); 
        }
        
        // 1. Llama a la función de renderizado en paralelo.
        let pixel_data = render(
            window_width,
            window_height,
            &objects,
            &camera,
            &light,
            &texture_manager,
        );
        
        // 2. Actualizamos la textura de forma segura. Convertimos el vector de colores a un slice de bytes de forma segura
        let pixel_bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                pixel_data.as_ptr() as *const u8,
                pixel_data.len() * size_of::<Color>(),
            )
        };

        // 3. Actualizamos la textura existente en la GPU con los nuevos datos.
        let _ = texture.update_texture(pixel_bytes);
        
        // 4. Inicia el dibujado en pantalla.
        {
            let mut d = window.begin_drawing(&raylib_thread);
            d.clear_background(Color::BLACK);
            // Dibujamos la textura que acabamos de actualizar.
            d.draw_texture(&texture, 0, 0, Color::WHITE);
            
            // 5. Dibuja el contador de FPS.
            let elapsed = start_time.elapsed().as_millis() as f32 / 1000.0;
            let fps = if elapsed > 0.0 { (1.0 / elapsed).round() as i32 } else { 0 };
            d.draw_text(
                &format!("FPS: {}", fps),
                10, // Posición X
                10, // Posición Y
                20, // Tamaño de fuente
                Color::BLACK, // Color del texto
            );
        }
    }
}

