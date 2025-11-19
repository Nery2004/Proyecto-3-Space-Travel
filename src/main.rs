use nalgebra_glm::{Vec3, Mat4, look_at, perspective};
use minifb::{Key, Window, WindowOptions, MouseMode};
use std::f32::consts::PI;

mod framebuffer;
mod triangle;
mod vertex;
mod obj;
mod color;
mod fragment;
mod shaders;

use framebuffer::Framebuffer;
use vertex::Vertex;
use obj::Obj;
use triangle::triangle;
use shaders::{vertex_shader, shade_star, shade_rocky, shade_gas_giant, shade_spaceship, 
              shade_ice_planet, shade_desert_planet, shade_volcanic_planet,
              shade_ocean_planet, shade_purple_planet, shade_ringed_planet};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

pub struct Uniforms {
    model_matrix: Mat4,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    viewport_matrix: Mat4,
    time: f32,
    shader_type: u32,
}

struct Camera {
    yaw: f32,
    pitch: f32,
    distance: f32, // Distancia desde la nave
    min_distance: f32,
    max_distance: f32,
}

impl Camera {
    fn new() -> Self {
        Self {
            yaw: 62.0, // Cámara directamente detrás de la nave
            pitch: 10.0, // Ángulo de elevación suave
            distance: 5.0, // Distancia por defecto (tercera persona)
            min_distance: 1.5, // Zoom mínimo para ver la nave completa
            max_distance: 8.0, // Máximo zoom out reducido
        }
    }

    fn get_view_matrix(&self, target: &Vec3, ship_yaw: f32) -> Mat4 {
        let combined_yaw = (self.yaw + ship_yaw).to_radians();
        let pitch_rad = self.pitch.to_radians();
        
        // Calcular posición de la cámara alrededor de la nave
        let camera_pos = Vec3::new(
            target.x + self.distance * combined_yaw.cos() * pitch_rad.cos(),
            target.y + self.distance * pitch_rad.sin(),
            target.z + self.distance * combined_yaw.sin() * pitch_rad.cos(),
        );
        
        look_at(&camera_pos, target, &Vec3::new(0.0, 1.0, 0.0))
    }

    fn update_rotation(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += delta_x * 0.3;
        self.pitch -= delta_y * 0.3;
        self.pitch = self.pitch.clamp(-89.0, 89.0);
    }

    fn zoom(&mut self, delta: f32) {
        self.distance -= delta * 0.5;
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
    }
}

struct Spaceship {
    position: Vec3,
    rotation: Vec3,
    speed: f32,
    tilt_x: f32, // Inclinación lateral (roll)
    tilt_z: f32, // Inclinación frontal (pitch)
    target_tilt_x: f32,
    target_tilt_z: f32,
    camera_yaw: f32, // Ángulo de la cámara que sigue a la nave
    target_camera_yaw: f32,
}

impl Spaceship {
    fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Vec3::new(0.0, 90.0, 0.0),
            speed: 0.15,
            tilt_x: 0.0,
            tilt_z: 0.0,
            target_tilt_x: 0.0,
            target_tilt_z: 0.0,
            camera_yaw: 0.0,
            target_camera_yaw: 0.0,
        }
    }

    fn move_forward(&mut self) {
        self.position.z -= self.speed; // Mover hacia arriba (Z negativo)
        self.target_tilt_z = -0.15; // Inclinación hacia adelante
    }

    fn move_backward(&mut self) {
        self.position.z += self.speed; // Mover hacia abajo (Z positivo)
        self.target_tilt_z = 0.1; // Inclinación hacia atrás
    }

    fn move_left(&mut self) {
        self.position.x -= self.speed; // Mover a la izquierda (X negativo)
        self.target_tilt_x = -0.2; // Inclinación a la izquierda
        self.target_camera_yaw = -15.0; // Rotar cámara a la izquierda
    }

    fn move_right(&mut self) {
        self.position.x += self.speed; // Mover a la derecha (X positivo)
        self.target_tilt_x = 0.2; // Inclinación a la derecha
        self.target_camera_yaw = 15.0; // Rotar cámara a la derecha
    }

    fn update_animation(&mut self) {
        // Suavizar la inclinación con interpolación
        let lerp_factor = 0.1;
        self.tilt_x += (self.target_tilt_x - self.tilt_x) * lerp_factor;
        self.tilt_z += (self.target_tilt_z - self.tilt_z) * lerp_factor;
        
        // Suavizar rotación de cámara
        self.camera_yaw += (self.target_camera_yaw - self.camera_yaw) * lerp_factor;
        
        // Retornar gradualmente a posición neutral
        self.target_tilt_x *= 0.9;
        self.target_tilt_z *= 0.9;
        self.target_camera_yaw *= 0.9;
    }

    fn get_animated_rotation(&self) -> Vec3 {
        Vec3::new(
            self.rotation.x + self.tilt_z,
            self.rotation.y,
            self.rotation.z + self.tilt_x,
        )
    }
}

fn create_model_matrix(translation: Vec3, scale: f32, rotation: Vec3) -> Mat4 {
    let (sin_x, cos_x) = rotation.x.sin_cos();
    let (sin_y, cos_y) = rotation.y.sin_cos();
    let (sin_z, cos_z) = rotation.z.sin_cos();

    let rotation_matrix_x = Mat4::new(
        1.0,  0.0,    0.0,   0.0,
        0.0,  cos_x, -sin_x, 0.0,
        0.0,  sin_x,  cos_x, 0.0,
        0.0,  0.0,    0.0,   1.0,
    );

    let rotation_matrix_y = Mat4::new(
        cos_y,  0.0,  sin_y, 0.0,
        0.0,    1.0,  0.0,   0.0,
        -sin_y, 0.0,  cos_y, 0.0,
        0.0,    0.0,  0.0,   1.0,
    );

    let rotation_matrix_z = Mat4::new(
        cos_z, -sin_z, 0.0, 0.0,
        sin_z,  cos_z, 0.0, 0.0,
        0.0,    0.0,  1.0, 0.0,
        0.0,    0.0,  0.0, 1.0,
    );

    let rotation_matrix = rotation_matrix_z * rotation_matrix_y * rotation_matrix_x;

    let scale_matrix = Mat4::new(
        scale, 0.0,   0.0,   0.0,
        0.0,   scale, 0.0,   0.0,
        0.0,   0.0,   scale, 0.0,
        0.0,   0.0,   0.0,   1.0,
    );

    let translation_matrix = Mat4::new(
        1.0, 0.0, 0.0, translation.x,
        0.0, 1.0, 0.0, translation.y,
        0.0, 0.0, 1.0, translation.z,
        0.0, 0.0, 0.0, 1.0,
    );

    translation_matrix * rotation_matrix * scale_matrix
}

fn create_viewport_matrix(width: f32, height: f32) -> Mat4 {
    Mat4::new(
        width / 2.0, 0.0, 0.0, width / 2.0,
        0.0, -height / 2.0, 0.0, height / 2.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    )
}

fn render_model(framebuffer: &mut Framebuffer, uniforms: &Uniforms, vertices: &[Vertex], indices: &[u32]) {
    let mut transformed_vertices = Vec::with_capacity(vertices.len());
    for vertex in vertices {
        transformed_vertices.push(vertex_shader(vertex, uniforms));
    }

    // Process triangles with early culling
    for i in (0..indices.len()).step_by(3) {
        let v1 = &transformed_vertices[indices[i] as usize];
        let v2 = &transformed_vertices[indices[i+1] as usize];
        let v3 = &transformed_vertices[indices[i+2] as usize];

        // Early clip space culling - skip triangles completely outside view
        let clip_coords = [v1.transformed_position, v2.transformed_position, v3.transformed_position];
        if clip_coords.iter().all(|v| v.x.abs() > v.w.abs() * 1.5 || v.y.abs() > v.w.abs() * 1.5 || v.z < -v.w || v.z > v.w) {
            continue;
        }

        let fragments = triangle(v1, v2, v3, uniforms);
        for fragment in fragments {
            let x = fragment.position.x as usize;
            let y = fragment.position.y as usize;

            if x < WIDTH && y < HEIGHT {
                let color_vec = match uniforms.shader_type {
                    0 => shade_star(fragment.vertex_position, uniforms.time),
                    1 => shade_rocky(fragment.vertex_position, uniforms.time),
                    2 => shade_gas_giant(fragment.vertex_position, uniforms.time),
                    3 => shade_spaceship(fragment.vertex_position, uniforms.time),
                    4 => shade_ice_planet(fragment.vertex_position, uniforms.time),
                    5 => shade_desert_planet(fragment.vertex_position, uniforms.time),
                    6 => shade_volcanic_planet(fragment.vertex_position, uniforms.time),
                    7 => shade_ocean_planet(fragment.vertex_position, uniforms.time),
                    8 => shade_purple_planet(fragment.vertex_position, uniforms.time),
                    9 => shade_ringed_planet(fragment.vertex_position, uniforms.time),
                    _ => Vec3::new(0.5, 0.5, 0.5), // Gris por defecto
                };

                let r = (color_vec.x * 255.0).clamp(0.0, 255.0) as u32;
                let g = (color_vec.y * 255.0).clamp(0.0, 255.0) as u32;
                let b = (color_vec.z * 255.0).clamp(0.0, 255.0) as u32;
                let color = (r << 16) | (g << 8) | b;
                
                framebuffer.set_current_color(color);
                framebuffer.point(x, y, fragment.depth);
            }
        }
    }
}

fn render_starfield(framebuffer: &mut Framebuffer, time: f32) {
    use std::f32::consts::PI;
    let width = framebuffer.width;
    let height = framebuffer.height;
    
    // Estrellas fijas
    for i in 0..800 {
        let seed = i as f32 * 12.9898;
        let x = ((seed.sin() * 43758.5453).fract() * width as f32) as usize;
        let y = (((seed * 1.234).cos() * 43758.5453).fract() * height as f32) as usize;
        
        if x < width && y < height {
            let brightness = ((seed * 2.345).sin() * 0.5 + 0.5) * 255.0;
            let b = brightness as u32;
            let color = (b << 16) | (b << 8) | b;
            framebuffer.set_current_color(color);
            framebuffer.point(x, y, 0.0);
        }
    }
    
    // Galaxias distantes
    for i in 0..5 {
        let seed = i as f32 * 7.321;
        let cx = ((seed.sin() * 43758.5453).fract() * width as f32) as i32;
        let cy = (((seed * 3.456).cos() * 43758.5453).fract() * height as f32) as i32;
        let rotation = time * 0.1 + seed;
        
        // Espiral de galaxia
        for j in 0..100 {
            let angle = j as f32 * 0.3 + rotation;
            let radius = (j as f32 * 0.5).sqrt() * 3.0;
            let x = cx + (angle.cos() * radius) as i32;
            let y = cy + (angle.sin() * radius) as i32;
            
            if x >= 0 && x < width as i32 && y >= 0 && y < height as i32 {
                let intensity = (1.0 - j as f32 / 100.0) * 150.0;
                let r = (intensity * 0.8) as u32;
                let g = (intensity * 0.6) as u32;
                let b = (intensity * 1.0) as u32;
                let color = (r << 16) | (g << 8) | b;
                framebuffer.set_current_color(color);
                framebuffer.point(x as usize, y as usize, 0.0);
            }
        }
    }
}

fn main() {
    let mut window = Window::new(
        "Proyecto 3 - Space Travel (WASD: mover nave, Click derecho: rotar cámara, Scroll: zoom)",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap();

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    
    // Load planet model for celestial bodies
    let planet_obj = Obj::load("assets/planeta.obj").expect("No se pudo cargar planeta.obj");
    let (planet_vertices, planet_indices) = planet_obj.get_vertex_and_index_arrays();

    // Load spaceship model
    let nave_obj = Obj::load("assets/CazaTie.obj").expect("No se pudo cargar CazaTie.obj");
    let (nave_vertices, nave_indices) = nave_obj.get_vertex_and_index_arrays();

    let projection_matrix = perspective(WIDTH as f32 / HEIGHT as f32, 45.0 * PI / 180.0, 0.1, 100.0);
    let viewport_matrix = create_viewport_matrix(WIDTH as f32, HEIGHT as f32);

    let mut camera = Camera::new();
    let mut spaceship = Spaceship::new(Vec3::new(6.0, 4.0, 9.0));
    let mut time = 0.0;
    let mut last_mouse_pos: Option<(f32, f32)> = None;

    println!("Controles:");
    println!("  WASD: Mover nave");
    println!("  Scroll: Zoom in/out (primera/tercera persona)");
    println!("  ESC: Salir");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        framebuffer.clear();
        
        // Renderizar fondo estrellado
        render_starfield(&mut framebuffer, time);
        
        time += 0.01;

        // Spaceship movement controls
        if window.is_key_down(Key::W) { spaceship.move_forward(); }
        if window.is_key_down(Key::S) { spaceship.move_backward(); }
        if window.is_key_down(Key::A) { spaceship.move_left(); }
        if window.is_key_down(Key::D) { spaceship.move_right(); }

        // Actualizar animación de la nave
        spaceship.update_animation();

        // Scroll wheel zoom control
        if let Some(scroll) = window.get_scroll_wheel() {
            camera.zoom(scroll.1);
        }

        let view_matrix = camera.get_view_matrix(&spaceship.position, spaceship.camera_yaw);

        // Render Sun (center, no rotation, much bigger size)
        let sun_rotation = Vec3::new(0.0, 0.0, 0.0); // No rotation
        let sun_model = create_model_matrix(Vec3::new(0.0, 0.0, 0.0), 5.0, sun_rotation);
        let sun_uniforms = Uniforms {
            model_matrix: sun_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 0, // Star shader
        };
        render_model(&mut framebuffer, &sun_uniforms, &planet_vertices, &planet_indices);

        // Render Rocky Planet (orbiting)
        let rocky_angle = time * 0.3;
        let rocky_orbit_radius = 8.0;
        let rocky_pos = Vec3::new(
            rocky_angle.cos() * rocky_orbit_radius,
            0.0,
            rocky_angle.sin() * rocky_orbit_radius,
        );
        let rocky_rotation = Vec3::new(0.0, time * 0.5, 0.0);
        let rocky_model = create_model_matrix(rocky_pos, 0.8, rocky_rotation);
        let rocky_uniforms = Uniforms {
            model_matrix: rocky_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 1, // Rocky shader
        };
        render_model(&mut framebuffer, &rocky_uniforms, &planet_vertices, &planet_indices);

        // Render Gas Giant (orbiting in opposite direction)
        let gas_angle = time * 0.15;
        let gas_orbit_radius = 12.0;
        let gas_pos = Vec3::new(
            -gas_angle.cos() * gas_orbit_radius,
            0.5,
            gas_angle.sin() * gas_orbit_radius,
        );
        let gas_rotation = Vec3::new(0.0, time * 0.3, 0.0);
        let gas_model = create_model_matrix(gas_pos, 1.2, gas_rotation);
        let gas_uniforms = Uniforms {
            model_matrix: gas_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 2, // Gas giant shader
        };
        render_model(&mut framebuffer, &gas_uniforms, &planet_vertices, &planet_indices);

        // Render Ice Planet (orbiting)
        let ice_angle = time * 0.25;
        let ice_orbit_radius = 10.0;
        let ice_pos = Vec3::new(
            (ice_angle + PI * 0.5).cos() * ice_orbit_radius,
            -0.3,
            (ice_angle + PI * 0.5).sin() * ice_orbit_radius,
        );
        let ice_rotation = Vec3::new(0.0, time * 0.4, 0.0);
        let ice_model = create_model_matrix(ice_pos, 0.7, ice_rotation);
        let ice_uniforms = Uniforms {
            model_matrix: ice_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 4, // Ice planet shader
        };
        render_model(&mut framebuffer, &ice_uniforms, &planet_vertices, &planet_indices);

        // Render Desert Planet (orbiting)
        let desert_angle = time * 0.35;
        let desert_orbit_radius = 32.0;
        let desert_pos = Vec3::new(
            (desert_angle + PI).cos() * desert_orbit_radius,
            0.2,
            (desert_angle + PI).sin() * desert_orbit_radius,
        );
        let desert_rotation = Vec3::new(0.0, time * 0.6, 0.0);
        let desert_model = create_model_matrix(desert_pos, 3.0, desert_rotation);
        let desert_uniforms = Uniforms {
            model_matrix: desert_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 5, // Desert planet shader
        };
        render_model(&mut framebuffer, &desert_uniforms, &planet_vertices, &planet_indices);

        // Render Volcanic Planet (orbiting)
        let volcanic_angle = time * 0.4;
        let volcanic_orbit_radius = 70.0;
        let volcanic_pos = Vec3::new(
            (volcanic_angle + PI * 1.5).cos() * volcanic_orbit_radius,
            -0.5,
            (volcanic_angle + PI * 1.5).sin() * volcanic_orbit_radius,
        );
        let volcanic_rotation = Vec3::new(0.0, time * 0.7, 0.0);
        let volcanic_model = create_model_matrix(volcanic_pos, 4.5, volcanic_rotation);
        let volcanic_uniforms = Uniforms {
            model_matrix: volcanic_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 6, // Volcanic planet shader
        };
        render_model(&mut framebuffer, &volcanic_uniforms, &planet_vertices, &planet_indices);

        // Render Ocean Planet (orbiting)
        let ocean_angle = time * 0.28;
        let ocean_orbit_radius = 45.0;
        let ocean_pos = Vec3::new(
            (ocean_angle + PI * 0.25).cos() * ocean_orbit_radius,
            1.0,
            (ocean_angle + PI * 0.25).sin() * ocean_orbit_radius,
        );
        let ocean_rotation = Vec3::new(0.0, time * 0.45, 0.0);
        let ocean_model = create_model_matrix(ocean_pos, 3.8, ocean_rotation);
        let ocean_uniforms = Uniforms {
            model_matrix: ocean_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 7, // Ocean planet shader
        };
        render_model(&mut framebuffer, &ocean_uniforms, &planet_vertices, &planet_indices);

        // Render Purple Alien Planet (orbiting)
        let purple_angle = time * 0.2;
        let purple_orbit_radius = 55.0;
        let purple_pos = Vec3::new(
            (purple_angle + PI * 0.75).cos() * purple_orbit_radius,
            -1.2,
            (purple_angle + PI * 0.75).sin() * purple_orbit_radius,
        );
        let purple_rotation = Vec3::new(0.0, time * 0.55, 0.0);
        let purple_model = create_model_matrix(purple_pos, 4.2, purple_rotation);
        let purple_uniforms = Uniforms {
            model_matrix: purple_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 8, // Purple planet shader
        };
        render_model(&mut framebuffer, &purple_uniforms, &planet_vertices, &planet_indices);

        // Render Ringed Turquoise Planet (orbiting)
        let ringed_angle = time * 0.18;
        let ringed_orbit_radius = 65.0;
        let ringed_pos = Vec3::new(
            (ringed_angle + PI * 1.25).cos() * ringed_orbit_radius,
            0.8,
            (ringed_angle + PI * 1.25).sin() * ringed_orbit_radius,
        );
        let ringed_rotation = Vec3::new(0.0, time * 0.35, 0.0);
        let ringed_model = create_model_matrix(ringed_pos, 5.0, ringed_rotation);
        let ringed_uniforms = Uniforms {
            model_matrix: ringed_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 9, // Ringed planet shader
        };
        render_model(&mut framebuffer, &ringed_uniforms, &planet_vertices, &planet_indices);

        // Render Spaceship (TIE Fighter) - Controlled by player with animation
        let animated_rotation = spaceship.get_animated_rotation();
        let nave_model = create_model_matrix(spaceship.position, 0.3, animated_rotation);
        let nave_uniforms = Uniforms {
            model_matrix: nave_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 3, // Spaceship shader
        };
        render_model(&mut framebuffer, &nave_uniforms, &nave_vertices, &nave_indices);

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}