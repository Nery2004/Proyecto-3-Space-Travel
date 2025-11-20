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
    let dist_to_center = uv.magnitude();
    
    // Núcleo ultra brillante con gradiente suave
    let core_brightness = (1.0 - (dist_to_center * 1.2).powf(2.0)).max(0.0);
    let core_color = Vec3::new(1.5, 1.4, 1.2); // Blanco-amarillo ultra brillante
    
    // Capa intermedia: amarillo-naranja intenso
    let middle_layer = Vec3::new(1.3, 0.9, 0.3);
    
    // Borde exterior: naranja-rojo
    let outer_layer = Vec3::new(1.2, 0.5, 0.1);
    
    // Mezclar capas según distancia al centro
    let mut color = core_color;
    if dist_to_center > 0.3 {
        let middle_factor = ((dist_to_center - 0.3) / 0.3).min(1.0);
        color = color.lerp(&middle_layer, middle_factor);
    }
    if dist_to_center > 0.6 {
        let outer_factor = ((dist_to_center - 0.6) / 0.4).min(1.0);
        color = color.lerp(&outer_layer, outer_factor.powf(0.5));
    }
    
    // Manchas solares (sunspots) - regiones más oscuras
    let sunspot_freq = 4.0;
    let sunspot_pattern = fbm(uv * sunspot_freq + Vec3::new(time * 0.1, 0.0, 0.0), 3, 0.6, 2.0);
    if sunspot_pattern > 0.65 {
        let spot_intensity = (sunspot_pattern - 0.65) * 2.0;
        color *= 1.0 - (spot_intensity * 0.4);
    }
    
    // Turbulencia de plasma solar
    let turbulence_freq = 8.0;
    let turbulence = fbm(
        uv * turbulence_freq + Vec3::new(time * 0.3, time * 0.2, 0.0),
        4,
        0.5,
        2.5
    );
    let plasma_color = Vec3::new(1.4, 0.6, 0.05);
    color = color.lerp(&plasma_color, turbulence * 0.25);
    
    // Llamaradas solares (solar flares)
    let flare_angle = (uv.y.atan2(uv.x) + time * 0.5).sin();
    let flare_distance = dist_to_center + flare_angle * 0.1;
    let flare_noise = noise(uv * 15.0 + Vec3::new(time * 0.8, 0.0, 0.0));
    if flare_noise > 0.8 && flare_distance > 0.85 {
        let flare_intensity = (flare_noise - 0.8) * 5.0;
        color = color.lerp(&Vec3::new(1.6, 0.8, 0.2), flare_intensity * 0.5);
    }
    
    // Corona solar - brillo difuso en los bordes
    if dist_to_center > 0.8 {
        let corona_factor = ((dist_to_center - 0.8) / 0.2).powf(0.3);
        let corona_color = Vec3::new(1.3, 0.7, 0.3);
        let corona_flicker = (time * 3.0 + uv.x * 10.0).sin() * 0.5 + 0.5;
        color = color.lerp(&corona_color, corona_factor * corona_flicker * 0.4);
    }
    
    // Pulsación energética global
    let pulse = ((time * 1.2).sin() * 0.5 + 0.5) * 0.12 + 0.96;
    color *= pulse;
    
    // Aumentar brillo cerca del núcleo
    color *= 1.0 + core_brightness * 0.8;

    color.map(|x| x.max(0.0).min(2.0)) // Permitir valores muy brillantes
}

pub fn shade_rocky(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();

    // Generación mejorada de continentes
    let continent_freq = 2.5;
    let continent_noise = fbm(uv * continent_freq, 4, 0.55, 2.1);
    
    let threshold = 0.48;
    let is_land = continent_noise > threshold;
    
    let ocean_deep = Vec3::new(0.02, 0.15, 0.35);
    let ocean_mid = Vec3::new(0.08, 0.28, 0.55);
    let ocean_shallow = Vec3::new(0.15, 0.45, 0.75);
    
    let land_beach = Vec3::new(0.76, 0.70, 0.50);
    let land_grass = Vec3::new(0.15, 0.52, 0.15);
    let land_forest = Vec3::new(0.08, 0.35, 0.10);
    let land_mountain = Vec3::new(0.45, 0.45, 0.47);
    let land_snow = Vec3::new(0.92, 0.95, 0.98);

    let mut color;
    if is_land {
        let elevation = (continent_noise - threshold) / (1.0 - threshold);
        
        // Biomas basados en altura
        if elevation < 0.1 {
            // Playa
            color = land_beach;
        } else if elevation < 0.4 {
            // Pradera
            color = land_grass;
            let grass_detail = noise(uv * 20.0);
            color = color.lerp(&land_forest, grass_detail * 0.3);
        } else if elevation < 0.7 {
            // Bosque/colinas
            color = land_forest.lerp(&land_mountain, (elevation - 0.4) / 0.3);
        } else {
            // Montañas nevadas
            color = land_mountain.lerp(&land_snow, (elevation - 0.7) / 0.3);
        }
        
        // Detalle de terreno
        let terrain_detail = fbm(uv * 15.0, 2, 0.5, 2.0);
        color *= 0.85 + terrain_detail * 0.3;
    } else {
        // Océanos con profundidad variable
        let depth = 1.0 - (continent_noise / threshold);
        if depth < 0.3 {
            color = ocean_shallow;
        } else if depth < 0.7 {
            color = ocean_mid;
        } else {
            color = ocean_deep;
        }
        
        // Olas y corrientes oceánicas
        let wave_pattern = fbm(uv * 25.0 + Vec3::new(time * 0.5, time * 0.3, 0.0), 2, 0.6, 2.0);
        color = color.lerp(&ocean_shallow, wave_pattern * 0.15);
    }
    
    // Nubes atmosféricas
    let cloud_freq = 6.0;
    let cloud_pattern = fbm(uv * cloud_freq + Vec3::new(time * 0.15, 0.0, 0.0), 3, 0.5, 2.0);
    if cloud_pattern > 0.62 {
        let cloud_density = (cloud_pattern - 0.62) * 2.5;
        let cloud_color = Vec3::new(0.95, 0.95, 1.0);
        color = color.lerp(&cloud_color, cloud_density.min(0.85));
    }

    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_gas_giant(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();

    // Bandas atmosféricas múltiples
    let band_freq = 10.0;
    let band_turbulence = fbm(uv * 18.0 + Vec3::new(time * 0.25, 0.0, 0.0), 3, 0.6, 2.0);
    let band_position = uv.y * band_freq + band_turbulence * 1.5;
    let bands = (band_position.sin() + 1.0) * 0.5;
    
    // Paleta de colores para las bandas
    let color1 = Vec3::new(0.85, 0.75, 0.55); // Crema claro
    let color2 = Vec3::new(0.72, 0.58, 0.38); // Beige
    let color3 = Vec3::new(0.58, 0.42, 0.25); // Marrón claro
    let color4 = Vec3::new(0.45, 0.30, 0.18); // Marrón oscuro
    
    let mut color;
    if bands < 0.25 {
        color = color1.lerp(&color2, bands * 4.0);
    } else if bands < 0.5 {
        color = color2.lerp(&color3, (bands - 0.25) * 4.0);
    } else if bands < 0.75 {
        color = color3.lerp(&color4, (bands - 0.5) * 4.0);
    } else {
        color = color4.lerp(&color1, (bands - 0.75) * 4.0);
    }
    
    // Turbulencia de gas atmosférico
    let turbulence_detail = fbm(uv * 30.0 + Vec3::new(time * 0.4, 0.0, 0.0), 2, 0.5, 2.0);
    color *= 0.88 + turbulence_detail * 0.24;
    
    // Vórtices y remolinos
    let vortex_pattern = noise(uv * 22.0 + Vec3::new(time * 0.35, time * 0.2, 0.0));
    if vortex_pattern > 0.7 {
        let vortex_intensity = (vortex_pattern - 0.7) * 3.0;
        color = color.lerp(&Vec3::new(0.95, 0.85, 0.7), vortex_intensity * 0.25);
    }

    // Gran Mancha Roja (tormenta característica)
    let storm_center = Vec3::new(0.0, -0.35, 0.0);
    let dist_to_storm = (uv - storm_center).magnitude();
    if dist_to_storm < 0.28 {
        let storm_factor = 1.0 - (dist_to_storm / 0.28);
        let storm_swirl = fbm(
            (uv - storm_center) * 15.0 + Vec3::new(time * 0.8, 0.0, 0.0),
            3,
            0.6,
            2.0
        );
        
        let storm_color_center = Vec3::new(0.95, 0.25, 0.12); // Rojo intenso
        let storm_color_edge = Vec3::new(0.88, 0.45, 0.28); // Naranja-rojo
        let storm_color = storm_color_center.lerp(&storm_color_edge, storm_swirl);
        
        color = color.lerp(&storm_color, storm_factor.powf(2.5) * 0.75);
    }

    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_spaceship(_point: Vec3, _time: f32) -> Vec3 {
    // Nave completamente gris uniforme
    Vec3::new(0.5, 0.5, 0.5)
}

pub fn shade_ice_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Base de hielo con variación
    let ice_base = fbm(uv * 4.0 + Vec3::new(time * 0.03, 0.0, 0.0), 4, 0.55, 2.0);
    
    let ice_bright = Vec3::new(0.92, 0.95, 1.0);  // Hielo brillante
    let ice_normal = Vec3::new(0.75, 0.85, 0.95); // Hielo azulado
    let ice_dark = Vec3::new(0.55, 0.65, 0.80);   // Hielo en sombra
    let ice_crack = Vec3::new(0.25, 0.35, 0.55);  // Grietas profundas
    
    let mut color;
    if ice_base > 0.7 {
        // Áreas de hielo muy brillante
        color = ice_bright;
    } else if ice_base > 0.45 {
        color = ice_normal.lerp(&ice_bright, (ice_base - 0.45) / 0.25);
    } else if ice_base > 0.25 {
        color = ice_dark.lerp(&ice_normal, (ice_base - 0.25) / 0.2);
    } else {
        // Grietas
        color = ice_crack.lerp(&ice_dark, ice_base / 0.25);
    }
    
    // Capas de nieve fresca
    let snow_pattern = fbm(uv * 12.0, 2, 0.6, 2.0);
    if snow_pattern > 0.65 {
        let snow_intensity = (snow_pattern - 0.65) * 2.8;
        color = color.lerp(&Vec3::new(0.98, 0.99, 1.0), snow_intensity.min(0.7));
    }
    
    // Grietas profundas con detalle
    let crack_detail = fbm(uv * 18.0 + Vec3::new(time * 0.05, 0.0, 0.0), 2, 0.5, 2.0);
    if crack_detail < 0.25 {
        let crack_depth = 1.0 - (crack_detail / 0.25);
        color = color.lerp(&Vec3::new(0.15, 0.25, 0.45), crack_depth * 0.6);
    }
    
    // Casquetes polares extra brillantes
    let polar_factor = uv.y.abs();
    if polar_factor > 0.7 {
        let polar_intensity = (polar_factor - 0.7) / 0.3;
        color = color.lerp(&Vec3::new(0.98, 0.99, 1.0), polar_intensity * 0.5);
    }
    
    // Cristales de hielo brillantes
    let crystal_noise = noise(uv * 35.0 + Vec3::new(0.0, time * 0.1, 0.0));
    if crystal_noise > 0.82 {
        let sparkle = (crystal_noise - 0.82) * 5.0;
        color = color.lerp(&Vec3::new(1.0, 1.0, 1.0), sparkle.min(0.4));
    }
    
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
    
    // Terreno volcánico base
    let terrain = fbm(uv * 3.5, 4, 0.55, 2.0);
    
    let rock_dark = Vec3::new(0.12, 0.10, 0.08);    // Roca volcánica oscura
    let rock_normal = Vec3::new(0.25, 0.20, 0.15);  // Roca gris
    let rock_hot = Vec3::new(0.45, 0.25, 0.15);     // Roca caliente
    let lava_dark = Vec3::new(0.8, 0.25, 0.05);     // Lava enfriándose
    let lava_bright = Vec3::new(1.2, 0.45, 0.0);    // Lava fundida
    let lava_core = Vec3::new(1.5, 0.8, 0.1);       // Núcleo de lava
    
    let threshold = 0.42;
    let mut color;
    
    if terrain > threshold {
        // Zonas de lava activa
        let lava_intensity = (terrain - threshold) / (1.0 - threshold);
        
        // Flujo de lava con turbulencia
        let lava_flow = fbm(
            uv * 8.0 + Vec3::new(time * 0.5, time * 0.3, 0.0),
            3,
            0.6,
            2.0
        );
        
        if lava_intensity > 0.7 {
            // Lava ultra caliente (núcleo)
            color = lava_bright.lerp(&lava_core, (lava_intensity - 0.7) / 0.3);
        } else if lava_intensity > 0.4 {
            // Lava fundida
            color = lava_dark.lerp(&lava_bright, (lava_intensity - 0.4) / 0.3);
        } else {
            // Lava enfriándose
            color = rock_hot.lerp(&lava_dark, lava_intensity / 0.4);
        }
        
        // Pulsación de lava (efecto de flujo)
        let pulse = (time * 2.5 + uv.x * 8.0 + uv.y * 6.0).sin() * 0.5 + 0.5;
        let pulse_color = Vec3::new(1.4, 0.6, 0.05);
        color = color.lerp(&pulse_color, pulse * lava_intensity * 0.35);
        
        // Agregar variación por flujo
        color *= 0.85 + lava_flow * 0.3;
    } else {
        // Roca volcánica
        if terrain > 0.25 {
            color = rock_normal.lerp(&rock_hot, (terrain - 0.25) / 0.17);
        } else {
            color = rock_dark.lerp(&rock_normal, terrain / 0.25);
        }
        
        // Textura de roca volcánica
        let rock_detail = fbm(uv * 15.0, 2, 0.5, 2.0);
        color *= 0.8 + rock_detail * 0.4;
        
        // Grietas con resplandor de lava
        let crack_pattern = noise(uv * 20.0 + Vec3::new(time * 0.2, 0.0, 0.0));
        if crack_pattern < 0.15 {
            let crack_glow = (0.15 - crack_pattern) * 6.0;
            color = color.lerp(&Vec3::new(1.0, 0.35, 0.0), crack_glow.min(0.5));
        }
    }
    
    // Ceniza volcánica flotante
    let ash_pattern = noise(uv * 25.0 + Vec3::new(time * 0.4, time * 0.6, 0.0));
    if ash_pattern > 0.78 {
        let ash_density = (ash_pattern - 0.78) * 4.0;
        color = color.lerp(&Vec3::new(0.35, 0.30, 0.28), ash_density.min(0.3));
    }
    
    color.map(|x| x.max(0.0).min(1.5))
}

pub fn shade_ocean_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Planeta oceánico con olas
    let wave_freq = 15.0;
    let wave_speed = 0.5;
    let waves = fbm(uv * wave_freq + Vec3::new(time * wave_speed, time * wave_speed * 0.5, 0.0), 3, 0.6, 2.0);
    
    let deep_ocean = Vec3::new(0.0, 0.2, 0.5);
    let shallow_ocean = Vec3::new(0.1, 0.5, 0.8);
    let foam = Vec3::new(0.7, 0.9, 1.0);
    
    let mut color = deep_ocean.lerp(&shallow_ocean, waves);
    
    // Espuma en las crestas
    if waves > 0.7 {
        color = color.lerp(&foam, (waves - 0.7) * 3.0);
    }
    
    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_purple_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Planeta alienígena púrpura con cristales
    let crystal_freq = 8.0;
    let n = fbm(uv * crystal_freq + Vec3::new(0.0, time * 0.1, 0.0), 4, 0.5, 2.5);
    
    let dark_purple = Vec3::new(0.3, 0.1, 0.5);
    let bright_purple = Vec3::new(0.7, 0.2, 0.9);
    let crystal_color = Vec3::new(0.9, 0.5, 1.0);
    
    let mut color = dark_purple.lerp(&bright_purple, n);
    
    // Cristales brillantes
    let crystal_noise = noise(uv * 20.0);
    if crystal_noise > 0.75 {
        color = color.lerp(&crystal_color, (crystal_noise - 0.75) * 4.0);
    }
    
    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_ringed_planet(point: Vec3, time: f32) -> Vec3 {
    let uv = point.normalize();
    
    // Planeta con atmósfera turquesa
    let base_freq = 5.0;
    let n = fbm(uv * base_freq + Vec3::new(time * 0.15, 0.0, 0.0), 3, 0.5, 2.0);
    
    let turquoise_dark = Vec3::new(0.1, 0.4, 0.5);
    let turquoise_light = Vec3::new(0.3, 0.8, 0.9);
    let white_clouds = Vec3::new(0.9, 0.95, 1.0);
    
    let mut color = turquoise_dark.lerp(&turquoise_light, n);
    
    // Nubes brillantes
    let cloud_noise = fbm(uv * 10.0 + Vec3::new(time * 0.2, 0.0, 0.0), 2, 0.6, 2.0);
    if cloud_noise > 0.6 {
        color = color.lerp(&white_clouds, (cloud_noise - 0.6) * 2.5);
    }
    
    color.map(|x| x.max(0.0).min(1.0))
}

pub fn shade_starfield(_point: Vec3, _time: f32) -> Vec3 {
    // Fondo negro del espacio
    Vec3::new(0.0, 0.0, 0.0)
}
