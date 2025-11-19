use nalgebra_glm::{Vec3, Vec4, Mat3};
use crate::vertex::Vertex;
use crate::Uniforms;

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position = Vec4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
    
    // Aplicar transformación completa: Model -> View -> Projection
    let transformed = uniforms.projection_matrix * uniforms.view_matrix * uniforms.model_matrix * position;

    // La normal se transforma solo con la matriz del modelo
    let model_mat3 = Mat3::from_columns(&[
        uniforms.model_matrix.column(0).xyz(),
        uniforms.model_matrix.column(1).xyz(),
        uniforms.model_matrix.column(2).xyz(),
    ]);
    let normal_matrix = model_mat3.transpose().try_inverse().unwrap_or_else(Mat3::identity);
    let transformed_normal = (normal_matrix * vertex.normal).normalize();

    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: transformed,
        transformed_normal,
    }
}

// Funciones de ruido procedural
fn noise(p: Vec3) -> f32 {
    let i = p.map(|x| x.floor());
    let f = p.map(|x| x.fract());
    let u = f.component_mul(&f).map(|x| x * (3.0 - 2.0 * x));

    let mix = |a, b, t| a + t * (b - a);

    mix(
        mix(
            mix(rand(i + Vec3::new(0.0, 0.0, 0.0)), rand(i + Vec3::new(1.0, 0.0, 0.0)), u.x),
            mix(rand(i + Vec3::new(0.0, 1.0, 0.0)), rand(i + Vec3::new(1.0, 1.0, 0.0)), u.x),
            u.y,
        ),
        mix(
            mix(rand(i + Vec3::new(0.0, 0.0, 1.0)), rand(i + Vec3::new(1.0, 0.0, 1.0)), u.x),
            mix(rand(i + Vec3::new(0.0, 1.0, 1.0)), rand(i + Vec3::new(1.0, 1.0, 1.0)), u.x),
            u.y,
        ),
        u.z,
    )
}

fn rand(p: Vec3) -> f32 {
    (p.dot(&Vec3::new(12.9898, 78.233, 45.5432)).sin() * 43758.5453).fract()
}

fn fbm(p: Vec3, octaves: i32, persistence: f32, lacunarity: f32) -> f32 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += noise(p * frequency) * amplitude;
        max_value += amplitude;
        amplitude *= persistence;
        frequency *= lacunarity;
    }

    total / max_value
}

// Shaders para los cuerpos celestes
pub fn shade_star(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Colores del sol más realistas: amarillo-naranja brillante
    let mut color = Vec3::new(1.0, 0.9, 0.3); // Amarillo brillante
    let dist_to_center = uv.magnitude();
    color *= 1.2 - (dist_to_center * 0.6).powf(2.0); // Núcleo muy brillante

    // Gradiente radial hacia naranja en los bordes
    let radial_grad = (1.0 - uv.magnitude()).powf(2.5);
    let grad_color = Vec3::new(1.0, 0.6, 0.1); // Naranja intenso
    color = color.lerp(&grad_color, radial_grad * 0.7);

    // Turbulencia de superficie solar (menos que antes)
    let turbulence_freq = 3.0;
    let turbulence_speed = 0.5;
    let turbulence = fbm(uv * turbulence_freq + Vec3::new(0.0, 0.0, time * turbulence_speed), 2, 0.5, 2.0);
    let flame_color = Vec3::new(1.0, 0.5, 0.0); // Naranja-rojo
    color = color.lerp(&flame_color, turbulence * 0.3);

    // Pulsación sutil
    let pulse = ((time * 1.5).sin() * 0.5 + 0.5) * 0.15 + 0.95; // Varía entre 0.95 y 1.1
    color *= pulse;

    color.map(|x| x.max(0.0).min(1.5)) // Permitir valores > 1.0 para brillo extra
}

pub fn shade_rocky(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();

    let base_freq = 2.0;
    let n = fbm(uv * base_freq, 3, 0.5, 2.0);

    let threshold = 0.5;
    let is_land = n > threshold;
    
    let ocean_color_deep = Vec3::new(0.0, 0.1, 0.3);
    let ocean_color_shallow = Vec3::new(0.1, 0.3, 0.7);
    let land_color_low = Vec3::new(0.1, 0.4, 0.1);
    let land_color_high = Vec3::new(0.6, 0.5, 0.3);

    let mut color;
    if is_land {
        let land_factor = (n - threshold) / (1.0 - threshold);
        color = land_color_low.lerp(&land_color_high, land_factor.powf(0.7));
    } else {
        let ocean_factor = n / threshold;
        color = ocean_color_deep.lerp(&ocean_color_shallow, ocean_factor);
    }

    // Simplified detail for better performance
    let detail_noise = noise(uv * 5.0 + Vec3::new(0.0, 0.0, time * 0.1));
    if is_land {
        color = color.lerp(&Vec3::new(0.9, 0.9, 0.9), detail_noise * 0.15);
    }

    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_gas_giant(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    let mut color;

    let band_freq_y = 8.0;
    let band_speed = 0.2;
    let band_noise_freq = 15.0;
    
    let y_component = uv.y + time * band_speed * 0.1;
    let band_noise = noise(uv * band_noise_freq + Vec3::new(time * band_speed, 0.0, 0.0));
    let bands = (y_component * band_freq_y + band_noise * 2.0).sin();

    let band_color1 = Vec3::new(0.8, 0.7, 0.5);
    let band_color2 = Vec3::new(0.6, 0.4, 0.2);
    color = band_color1.lerp(&band_color2, bands * 0.5 + 0.5);

    let gas_texture_noise = noise(uv * 20.0 + Vec3::new(time * 0.3, 0.0, 0.0));
    color = color.lerp(&Vec3::new(1.0, 1.0, 1.0), gas_texture_noise * 0.08);

    // Simplified storm
    let storm_pos = Vec3::new(0.0, -0.4, 0.0);
    let dist_to_storm = (uv - storm_pos).magnitude();
    if dist_to_storm < 0.25 {
        let storm_factor = 1.0 - (dist_to_storm / 0.25);
        let storm_color = Vec3::new(0.95, 0.3, 0.15);
        color = color.lerp(&storm_color, storm_factor.powf(3.0) * 0.6);
    }

    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_spaceship(_point: Vec3, _time: f32) -> Vec3 {
    // Color gris uniforme para toda la nave
    Vec3::new(0.5, 0.5, 0.5)
}

pub fn shade_ice_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Planeta helado con grietas y hielo
    let base_freq = 3.0;
    let n = fbm(uv * base_freq + Vec3::new(0.0, time * 0.05, 0.0), 3, 0.5, 2.0);
    
    let ice_color = Vec3::new(0.8, 0.9, 1.0);
    let crack_color = Vec3::new(0.3, 0.4, 0.6);
    
    let mut color = ice_color.lerp(&crack_color, n * 0.6);
    
    // Añadir detalles de nieve
    let detail = noise(uv * 8.0);
    color = color.lerp(&Vec3::new(1.0, 1.0, 1.0), detail * 0.3);
    
    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_desert_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Planeta desértico con dunas
    let base_freq = 4.0;
    let n = fbm(uv * base_freq + Vec3::new(time * 0.02, 0.0, 0.0), 2, 0.6, 2.0);
    
    let sand_light = Vec3::new(0.9, 0.7, 0.3);
    let sand_dark = Vec3::new(0.6, 0.4, 0.1);
    
    let mut color = sand_dark.lerp(&sand_light, n.powf(0.8));
    
    // Dunas de arena
    let dunes = (uv.y * 10.0 + noise(uv * 6.0) * 2.0).sin() * 0.5 + 0.5;
    color = color.lerp(&Vec3::new(0.95, 0.8, 0.4), dunes * 0.3);
    
    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_volcanic_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Planeta volcánico con lava
    let base_freq = 3.0;
    let n = fbm(uv * base_freq, 3, 0.5, 2.0);
    
    let rock_color = Vec3::new(0.2, 0.15, 0.1);
    let lava_color = Vec3::new(1.0, 0.3, 0.0);
    
    let threshold = 0.45;
    let mut color;
    
    if n > threshold {
        // Zonas de lava
        let lava_factor = (n - threshold) / (1.0 - threshold);
        color = rock_color.lerp(&lava_color, lava_factor.powf(2.0));
        
        // Pulsación de lava
        let pulse = (time * 2.0 + uv.x * 5.0).sin() * 0.5 + 0.5;
        color = color.lerp(&Vec3::new(1.0, 0.5, 0.0), pulse * lava_factor * 0.4);
    } else {
        // Roca oscura
        color = rock_color;
        let detail = noise(uv * 10.0);
        color = color.lerp(&Vec3::new(0.3, 0.25, 0.2), detail * 0.3);
    }
    
    color.map(|x| x.max(0.0).min(1.0))
}