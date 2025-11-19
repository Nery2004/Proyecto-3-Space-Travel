# Sistema Solar 3D con Shaders Procedurales

Motor de renderizado 3D por software desarrollado en Rust que simula un sistema solar completo con 6 planetas únicos, cada uno con shaders procedurales personalizados, una nave espacial estática y controles de cámara libre.

## ✨ Funcionalidades Principales

- **6 Planetas con Shaders Procedurales**: Sol, planeta rocoso, gigante gaseoso, planeta helado, planeta desértico y planeta volcánico
- **Shaders Procedurales Únicos**: Cada cuerpo celeste tiene textura generada mediante algoritmos de ruido (Perlin noise, FBM)
- **Órbitas Realistas**: Los planetas orbitan alrededor del sol a diferentes velocidades y distancias
- **Rotación Planetaria**: Todos los planetas rotan sobre su propio eje
- **Cámara Libre**: Control total de la cámara con movimiento WASD y rotación con mouse
- **Nave Espacial Estática**: TIE Fighter renderizado en color gris uniforme
- **Renderizado Optimizado**: Culling de espacio de clip, backface culling y compilación en modo release

## 🌌 Cuerpos Celestes

### ☀️ Sol
- **Shader**: Amarillo-naranja brillante con turbulencia superficial y efecto de pulsación
- **Características**: Núcleo muy brillante, gradiente radial, manchas solares simuladas
- **Escala**: 5.0 (el más grande del sistema)
- **Comportamiento**: Estático en el centro (0, 0, 0), sin rotación

### 🪨 Planeta Rocoso
- **Shader**: Continentes verdes/marrones y océanos azules generados con FBM
- **Características**: Diferenciación entre tierra y agua mediante threshold de ruido
- **Órbita**: 8.0 unidades del sol
- **Velocidad orbital**: 0.3 rad/s

### 🌀 Gigante Gaseoso  
- **Shader**: Bandas horizontales beige/marrones con tormenta roja característica
- **Características**: Bandas de gas animadas, gran mancha roja similar a Júpiter
- **Órbita**: 12.0 unidades del sol
- **Velocidad orbital**: 0.15 rad/s

### ❄️ Planeta Helado
- **Shader**: Azul y blanco con grietas de hielo y copos de nieve
- **Características**: Superficie helada con detalles de nieve brillante
- **Órbita**: 10.0 unidades del sol
- **Velocidad orbital**: 0.25 rad/s

### 🏜️ Planeta Desértico
- **Shader**: Amarillo/naranja con dunas de arena onduladas
- **Características**: Variación de tonos de arena, patrones de dunas procedurales
- **Órbita**: 6.5 unidades del sol
- **Velocidad orbital**: 0.35 rad/s

### 🌋 Planeta Volcánico
- **Shader**: Roca negra con ríos de lava naranja pulsante
- **Características**: Lava animada que pulsa, contraste dramático roca/lava
- **Órbita**: 14.0 unidades del sol
- **Velocidad orbital**: 0.4 rad/s

### 🚀 Nave Espacial (TIE Fighter)
- **Shader**: Gris uniforme (0.5, 0.5, 0.5)
- **Posición**: Estática en (6.0, 2.0, 9.0)
- **Modelo**: CazaTie.obj

## 🛠️ Stack Tecnológico

- **Rust** - Lenguaje de sistemas para alto rendimiento
- **nalgebra-glm** - Librería de álgebra lineal para gráficos 3D
- **minifb** - Framework para gestión de ventanas y buffer de píxeles
- **Software Rasterization** - Renderizado 3D completamente implementado desde cero

## 📋 Prerrequisitos

- Rust 1.70 o versión posterior
- Modelos 3D en el directorio `assets/`:
  - `planeta.obj` - Usado para todos los cuerpos celestes
  - `CazaTie.obj` - Nave TIE Fighter

## 🚀 Inicio Rápido

```bash
# Clonar este repositorio
git clone https://github.com/Nery2004/Lab-5-Shaders.git
cd Lab-5-Shaders

# Compilar y lanzar en modo optimizado (recomendado)
cargo run --release
```

## 🎮 Controles de Usuario

### Movimiento de Cámara
| Control | Función |
|---------|---------|
| `W` | Mover cámara hacia adelante |
| `S` | Mover cámara hacia atrás |
| `A` | Mover cámara hacia la izquierda |
| `D` | Mover cámara hacia la derecha |
| `Espacio` | Subir cámara |
| `Shift Izquierdo` | Bajar cámara |
| `ESC` | Cerrar aplicación |

### Rotación de Cámara
| Control | Función |
|---------|---------|
| `Botón derecho del mouse + Arrastrar` | Rotar cámara (yaw y pitch) |

## 📁 Arquitectura del Proyecto

```
Lab-5-Shaders/
├── Cargo.toml              # Configuración de dependencias
├── assets/
│   ├── planeta.obj         # Modelo de esfera para planetas
│   ├── CazaTie.obj         # Modelo de nave TIE Fighter
│   ├── planeta.mtl
│   └── CazaTie.mtl
└── src/
    ├── main.rs             # Ciclo principal, cámara, y lógica de órbitas
    ├── shaders.rs          # Vertex shader y 7 fragment shaders procedurales
    ├── triangle.rs         # Rasterización con culling optimizado
    ├── vertex.rs           # Definición de vértices con transformaciones
    ├── framebuffer.rs      # Gestión de buffers de color y profundidad
    ├── fragment.rs         # Estructura de fragmentos con vertex_position
    ├── obj.rs              # Parser de archivos OBJ
    ├── color.rs            # Manejo de colores RGB
    └── line.rs             # Algoritmo de líneas
```

## 🔄 Pipeline de Renderizado

1. **Carga de Modelos**: Lectura de archivos OBJ para planetas y nave
2. **Transformaciones**: Matrices de modelo (órbita + rotación) → vista (cámara) → proyección
3. **Vertex Shader**: Transformación MVP y cálculo de normales
4. **Culling Optimizado**: 
   - Clip space culling (descarta triángulos fuera de vista)
   - Backface culling (descarta caras traseras)
   - Bounding box clamping (limita a pantalla 800x600)
5. **Rasterización**: Conversión a fragmentos con coordenadas baricéntricas
6. **Fragment Shader**: Selección de shader procedural según `shader_type` (0-6)
7. **Z-Buffer**: Test de profundidad para resolver oclusión
8. **Display**: Actualización de ventana con buffer final

## 💡 Detalles de Implementación

### Sistema de Shaders Procedurales

Todos los shaders utilizan funciones de ruido procedural:

```rust
// Ruido Perlin 3D
fn noise(p: Vec3) -> f32

// Fractional Brownian Motion para detalles complejos
fn fbm(p: Vec3, octaves: i32, persistence: f32, lacunarity: f32) -> f32
```

**Tipos de Shader:**
- `0`: `shade_star` - Sol con turbulencia y pulsación
- `1`: `shade_rocky` - Planeta rocoso con continentes/océanos
- `2`: `shade_gas_giant` - Gigante gaseoso con bandas y tormenta
- `3`: `shade_spaceship` - Color gris uniforme
- `4`: `shade_ice_planet` - Planeta helado con grietas
- `5`: `shade_desert_planet` - Planeta desértico con dunas
- `6`: `shade_volcanic_planet` - Planeta volcánico con lava

### Optimizaciones de Rendimiento

```rust
// Culling temprano en espacio de clip
if clip_coords.iter().all(|v| v.x.abs() > v.w.abs() * 1.5 || ...) {
    continue; // Salta triángulos fuera de vista
}

// Backface culling en triangle.rs
let normal = edge1.cross(&edge2);
if normal.z <= 0.0 { return vec![]; }

// Bounding box clamping
let min_x = min_x.max(0);
let max_x = max_x.min(799);
```

**Reducción de octavas en FBM:** De 4-6 octavas a 2-3 para mejor performance

### Sistema de Cámara Libre

```rust
struct Camera {
    position: Vec3,      // Posición en el mundo
    yaw: f32,           // Rotación horizontal
    pitch: f32,         // Rotación vertical (clamped -89° a 89°)
    speed: f32,         // Velocidad de movimiento
}
```

La cámara calcula su dirección mediante ángulos de Euler y genera una matriz de vista con `look_at`.

## 🎨 Configuración Visual

- **Resolución**: 800x600 píxeles
- **Entorno de fondo**: Negro (espacio)
- **FOV**: 45 grados
- **Near plane**: 0.1
- **Far plane**: 100.0
- **Posición inicial de cámara**: (0.0, 4.0, 15.0)

## 📸 Galería Visual

![Sistema Solar 3D - Renderizado con Shaders](Screenshot%202025-11-12%20235655.png)

*Sistema solar completo mostrando el sol, 5 planetas con shaders procedurales únicos y la nave TIE Fighter*

## 🔧 Compilación

```bash
# Modo debug (más lento, útil para desarrollo)
cargo run

# Modo release (optimizado, recomendado para uso)
cargo run --release
```

**Nota**: El modo release es **altamente recomendado** debido a las optimizaciones de compilador que mejoran significativamente el rendimiento del rasterizador.

## 🎓 Propósito Educativo

Este proyecto fue desarrollado como parte del **Laboratorio 5** del curso de Gráficas por Computadora, demostrando:
- Implementación de pipeline de renderizado 3D desde cero
- Shaders procedurales con funciones de ruido
- Optimizaciones de rasterización
- Física orbital simplificada
- Sistema de cámara 3D interactiva

## 📝 Licencia

Este proyecto es de código abierto y está disponible bajo la licencia MIT.




