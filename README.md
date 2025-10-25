# GPC25 - Proyecto 2: Raytracing

![tempimg](https://github.com/user-attachments/assets/96e211f9-59d0-43a1-8e7f-01165b4f2098)

## Descripción

Este proyecto implementa un **ray tracer en Rust** que renderiza un diorama inspirado en **Minecraft**, con bloques texturizados, efectos avanzados de iluminación y materiales realistas. La escena recrea un **portal del End** rodeado por una estructura de bloques de piedra del End, con islas flotantes y efectos visuales especiales.

## Características Implementadas

### Requerimientos Cumplidos

- **Complejidad de la escena**: Diorama detallado con más de 30 bloques organizados en estructuras complejas (portal del End + islas flotantes)
- **Atractivo visual**: Texturas personalizadas, cielo procedural morado, materiales con propiedades físicas realistas
- **Threads**: Renderizado paralelo usando **Rayon** para aprovechar múltiples núcleos de CPU
- **Rotación y zoom de cámara**: Sistema de cámara orbital con controles de teclado (flechas, A/D, W/S) sin errores de colisión
- **Materiales diferentes**: 6 materiales únicos, cada uno con:
  - Textura propia
  - Parámetros personalizados (albedo, specular, reflectividad, transparencia)
  - Comportamiento óptico distinto
- **Refracción**: Implementada en bloques de portal con `refractive_index = 1.2-1.3`
- **Efectos de portal**: Texturas especiales y materiales transparentes/reflectivos que simulan el efecto de portal
- **Reflexión**: Bloques de portal con `reflectivity = 0.15-0.2`

<img width="1156" height="672" alt="Captura de pantalla 2025-10-24 185519" src="https://github.com/user-attachments/assets/775c0191-d2f2-4a14-964c-5cf81df57bd8" />
<img width="1156" height="672" alt="Captura de pantalla 2025-10-24 185519" src="https://github.com/user-attachments/assets/52c0396d-d792-43db-a0cb-a14488fd1f05" />
<img width="1156" height="672" alt="Captura de pantalla 2025-10-24 185519" src="https://github.com/user-attachments/assets/40195aaa-2436-401f-96bd-bff1c11c2812" />

### Materiales Implementados

| Material | Textura | Albedo | Specular | Reflectividad | Transparencia | Índice de Refracción |
|----------|---------|--------|----------|---------------|---------------|---------------------|
| **Obsidiana** | `OBSIDIAN.png` | `[0.8, 0.2]` | `5.0` | `0.02` | `0.0` | `1.0` |
| **Bloque Purpur** | `PURPURBLOCK.png` | `[0.8, 0.2]` | `10.0` | `0.05` | `0.0` | `1.0` |
| **Portal Lateral** | `LATENDPORTAL.png` | `[0.6, 0.4]` | `25.0` | `0.15` | `0.3` | `1.2` |
| **Portal Central** | `ENDPORTAL.png` | `[0.5, 0.5]` | `50.0` | `0.2` | `0.5` | `1.3` |
| **Bloque del End** | `ENDBLOCK.png` | `[0.7, 0.3]` | `8.0` | `0.05` | `0.0` | `1.0` |
| **Deepslate** | `deepslateBLOCK.png` | `[0.8, 0.2]` | `5.0` | `0.02` | `0.0` | `1.0` |

### Video Demostración
[![Ver Video](https://img.shields.io/badge/YouTube-Video%20de%20Demostración-red?logo=youtube)](https://youtu.be/GisZVV2Koic)


## Controles

- **Flechas ← → ↑ ↓**: Rotar la cámara alrededor de la escena
- **Teclas A/D**: Acercar/Alejar la cámara (zoom)
- **Teclas W/S**: Mover la cámara verticalmente

## Tecnologías Utilizadas

- **Lenguaje**: Rust (sin librerías externas de ray tracing)
- **Biblioteca gráfica**: Raylib (solo para ventana y texturas, **NO** para renderizado)
- **Paralelización**: Rayon para renderizado multihilo
- **Renderizado**: Ray tracing puro implementado desde cero

## Estructura del Proyecto

```
Directory structure:
└── daviddominguez-11-gpc25-proyecto2-raytracing/
    ├── README.md
    └── RayTracing-Minecraft/
        ├── Cargo.toml
        └── src/
            ├── camera.rs
            ├── cube.rs
            ├── framebuffer.rs
            ├── light.rs
            ├── main.rs
            ├── material.rs
            ├── ray_intersect.rs
            ├── snell.rs
            └── textures.rs
```

## Cómo Ejecutar

```bash
# Clonar el repositorio
git clone https://github.com/daviddominguez-11/gpc25-proyecto2-raytracing.git
cd gpc25-proyecto2-raytracing/RayTracing-Minecraft

# Ejecutar (requiere Rust instalado)
cargo run
```

## Notas Técnicas

- **Renderizado**: Cada píxel se calcula independientemente mediante ray tracing
- **Iluminación**: Modelo de iluminación Phong con sombras duras
- **Texturas**: Sistema de mapeo UV por cara con soporte para texturas diferentes en top/sides
- **Optimización**: Early exit conditions, cálculos matemáticos optimizados, reutilización de buffers
- **Recursión**: Máximo 2 niveles de profundidad para reflexión/refracción

## Objetivo del Proyecto

Demostrar la comprensión de los conceptos fundamentales de **ray tracing** mediante la implementación de:
- Intersección de rayos con primitivas 3D
- Modelos de iluminación realistas
- Efectos ópticos (reflexión, refracción)
- Sistemas de materiales complejos
- Renderizado paralelo eficiente

---
