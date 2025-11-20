use nalgebra_glm::{Vec3, Vec4, Mat4, look_at, perspective};
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
            distance: 12.0, // Distancia por defecto (tercera persona) - más lejos
            min_distance: 1.5, // Zoom mínimo para ver la nave completa
            max_distance: 20.0, // Máximo zoom out aumentado
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

    fn update_rotation(&mut self, delta_x: f32) {
        self.yaw += delta_x * 0.3;
        // Solo rotación horizontal, pitch se mantiene fijo
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

    fn check_collision(&self, celestial_bodies: &[(Vec3, f32)]) -> bool {
        // Verificar colisión con cada cuerpo celeste
        for (body_pos, body_radius) in celestial_bodies {
            let distance = (self.position - body_pos).magnitude();
            // Radio de colisión = radio del planeta + margen de seguridad
            if distance < body_radius + 2.0 {
                return true;
            }
        }
        false
    }

    fn move_forward(&mut self, celestial_bodies: &[(Vec3, f32)]) {
        let new_pos = Vec3::new(self.position.x, self.position.y, self.position.z - self.speed);
        let old_pos = self.position;
        self.position = new_pos;
        if self.check_collision(celestial_bodies) {
            self.position = old_pos; // Revertir movimiento si hay colisión
        } else {
            self.target_tilt_z = -0.15;
        }
    }

    fn move_backward(&mut self, celestial_bodies: &[(Vec3, f32)]) {
        let new_pos = Vec3::new(self.position.x, self.position.y, self.position.z + self.speed);
        let old_pos = self.position;
        self.position = new_pos;
        if self.check_collision(celestial_bodies) {
            self.position = old_pos;
        } else {
            self.target_tilt_z = 0.1;
        }
    }

    fn move_left(&mut self, celestial_bodies: &[(Vec3, f32)]) {
        let new_pos = Vec3::new(self.position.x - self.speed, self.position.y, self.position.z);
        let old_pos = self.position;
        self.position = new_pos;
        if self.check_collision(celestial_bodies) {
            self.position = old_pos;
        } else {
            self.target_tilt_x = -0.2;
            self.target_camera_yaw = -15.0;
        }
    }

    fn move_right(&mut self, celestial_bodies: &[(Vec3, f32)]) {
        let new_pos = Vec3::new(self.position.x + self.speed, self.position.y, self.position.z);
        let old_pos = self.position;
        self.position = new_pos;
        if self.check_collision(celestial_bodies) {
            self.position = old_pos;
        } else {
            self.target_tilt_x = 0.2;
            self.target_camera_yaw = 15.0;
        }
    }

    fn move_up(&mut self, celestial_bodies: &[(Vec3, f32)]) {
        let new_pos = Vec3::new(self.position.x, self.position.y + self.speed, self.position.z);
        let old_pos = self.position;
        self.position = new_pos;
        if self.check_collision(celestial_bodies) {
            self.position = old_pos;
        }
    }

    fn move_down(&mut self, celestial_bodies: &[(Vec3, f32)]) {
        let new_pos = Vec3::new(self.position.x, self.position.y - self.speed, self.position.z);
        let old_pos = self.position;
        self.position = new_pos;
        if self.check_collision(celestial_bodies) {
            self.position = old_pos;
        }
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

fn render_orbit(framebuffer: &mut Framebuffer, radius: f32, inclination: f32, view_matrix: &Mat4, projection_matrix: &Mat4, viewport_matrix: &Mat4) {
    let segments = 100;
    let orbit_color = 0x444444; // Gris oscuro para las órbitas
    framebuffer.set_current_color(orbit_color);
    
    for i in 0..segments {
        let angle1 = (i as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        let angle2 = ((i + 1) as f32 / segments as f32) * 2.0 * std::f32::consts::PI;
        
        // Puntos en el espacio 3D con inclinación orbital
        let p1 = Vec3::new(
            angle1.cos() * radius,
            angle1.sin() * radius * inclination.sin(),
            angle1.sin() * radius * inclination.cos()
        );
        let p2 = Vec3::new(
            angle2.cos() * radius,
            angle2.sin() * radius * inclination.sin(),
            angle2.sin() * radius * inclination.cos()
        );
        
        // Transformar a espacio de pantalla
        let p1_4d = Vec4::new(p1.x, p1.y, p1.z, 1.0);
        let p2_4d = Vec4::new(p2.x, p2.y, p2.z, 1.0);
        
        let p1_transformed = projection_matrix * view_matrix * p1_4d;
        let p2_transformed = projection_matrix * view_matrix * p2_4d;
        
        // Perspective divide
        if p1_transformed.w != 0.0 && p2_transformed.w != 0.0 {
            let p1_ndc = p1_transformed / p1_transformed.w;
            let p2_ndc = p2_transformed / p2_transformed.w;
            
            // Solo dibujar si están dentro del frustum
            if p1_ndc.z > 0.0 && p1_ndc.z < 1.0 && p2_ndc.z > 0.0 && p2_ndc.z < 1.0 {
                let p1_screen = viewport_matrix * Vec4::new(p1_ndc.x, p1_ndc.y, p1_ndc.z, 1.0);
                let p2_screen = viewport_matrix * Vec4::new(p2_ndc.x, p2_ndc.y, p2_ndc.z, 1.0);
                
                let x1 = p1_screen.x as i32;
                let y1 = p1_screen.y as i32;
                let x2 = p2_screen.x as i32;
                let y2 = p2_screen.y as i32;
                
                // Dibujar línea más gruesa (3 píxeles de grosor)
                for offset_x in -1..=1 {
                    for offset_y in -1..=1 {
                        let px1 = (x1 + offset_x) as usize;
                        let py1 = (y1 + offset_y) as usize;
                        let px2 = (x2 + offset_x) as usize;
                        let py2 = (y2 + offset_y) as usize;
                        
                        if px1 < framebuffer.width && py1 < framebuffer.height {
                            framebuffer.point(px1, py1, p1_ndc.z);
                        }
                        if px2 < framebuffer.width && py2 < framebuffer.height {
                            framebuffer.point(px2, py2, p2_ndc.z);
                        }
                    }
                }
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

    let projection_matrix = perspective(WIDTH as f32 / HEIGHT as f32, 55.0 * PI / 180.0, 0.1, 150.0);
    let viewport_matrix = create_viewport_matrix(WIDTH as f32, HEIGHT as f32);

    let mut camera = Camera::new();
    let mut spaceship = Spaceship::new(Vec3::new(35.0, 15.0, 40.0));
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

        // Calculate all celestial body positions for collision detection
        let rocky_angle = time * 0.3;
        let rocky_orbit_radius = 45.0;
        let rocky_inclination = 5.0_f32.to_radians(); // 5 grados de inclinación
        let rocky_pos = Vec3::new(
            rocky_angle.cos() * rocky_orbit_radius,
            (rocky_angle.sin() * rocky_orbit_radius * rocky_inclination.sin()),
            rocky_angle.sin() * rocky_orbit_radius * rocky_inclination.cos(),
        );
        
        let gas_angle = time * 0.15;
        let gas_orbit_radius = 60.0;
        let gas_inclination = (-8.0_f32).to_radians(); // -8 grados (inclinación opuesta)
        let gas_pos = Vec3::new(
            -gas_angle.cos() * gas_orbit_radius,
            (gas_angle.sin() * gas_orbit_radius * gas_inclination.sin()),
            gas_angle.sin() * gas_orbit_radius * gas_inclination.cos(),
        );
        
        let ice_angle = time * 0.25;
        let ice_orbit_radius = 53.0;
        let ice_inclination = 12.0_f32.to_radians(); // 12 grados
        let ice_pos = Vec3::new(
            (ice_angle + PI * 0.5).cos() * ice_orbit_radius,
            ((ice_angle + PI * 0.5).sin() * ice_orbit_radius * ice_inclination.sin()),
            (ice_angle + PI * 0.5).sin() * ice_orbit_radius * ice_inclination.cos(),
        );
        
        let desert_angle = time * 0.35;
        let desert_orbit_radius = 38.0;
        let desert_inclination = (-6.0_f32).to_radians(); // -6 grados
        let desert_pos = Vec3::new(
            (desert_angle + PI).cos() * desert_orbit_radius,
            ((desert_angle + PI).sin() * desert_orbit_radius * desert_inclination.sin()),
            (desert_angle + PI).sin() * desert_orbit_radius * desert_inclination.cos(),
        );
        
        let volcanic_angle = time * 0.4;
        let volcanic_orbit_radius = 72.0;
        let volcanic_inclination = 15.0_f32.to_radians(); // 15 grados
        let volcanic_pos = Vec3::new(
            (volcanic_angle + PI * 1.5).cos() * volcanic_orbit_radius,
            ((volcanic_angle + PI * 1.5).sin() * volcanic_orbit_radius * volcanic_inclination.sin()),
            (volcanic_angle + PI * 1.5).sin() * volcanic_orbit_radius * volcanic_inclination.cos(),
        );
        
        let ocean_angle = time * 0.28;
        let ocean_orbit_radius = 49.0;
        let ocean_inclination = (-10.0_f32).to_radians(); // -10 grados
        let ocean_pos = Vec3::new(
            (ocean_angle + PI * 0.25).cos() * ocean_orbit_radius,
            ((ocean_angle + PI * 0.25).sin() * ocean_orbit_radius * ocean_inclination.sin()),
            (ocean_angle + PI * 0.25).sin() * ocean_orbit_radius * ocean_inclination.cos(),
        );
        
        let purple_angle = time * 0.2;
        let purple_orbit_radius = 57.0;
        let purple_inclination = 18.0_f32.to_radians(); // 18 grados
        let purple_pos = Vec3::new(
            (purple_angle + PI * 0.75).cos() * purple_orbit_radius,
            ((purple_angle + PI * 0.75).sin() * purple_orbit_radius * purple_inclination.sin()),
            (purple_angle + PI * 0.75).sin() * purple_orbit_radius * purple_inclination.cos(),
        );
        
        let ringed_angle = time * 0.18;
        let ringed_orbit_radius = 67.0;
        let ringed_inclination = (-14.0_f32).to_radians(); // -14 grados
        let ringed_pos = Vec3::new(
            (ringed_angle + PI * 1.25).cos() * ringed_orbit_radius,
            ((ringed_angle + PI * 1.25).sin() * ringed_orbit_radius * ringed_inclination.sin()),
            (ringed_angle + PI * 1.25).sin() * ringed_orbit_radius * ringed_inclination.cos(),
        );
        
        // Lista de todos los cuerpos celestes (posición, radio)
        let celestial_bodies = vec![
            (Vec3::new(0.0, 0.0, 0.0), 8.0),  // Sol
            (rocky_pos, 0.8),
            (gas_pos, 1.2),
            (ice_pos, 0.7),
            (desert_pos, 3.0),
            (volcanic_pos, 4.5),
            (ocean_pos, 3.8),
            (purple_pos, 4.2),
            (ringed_pos, 5.0),
        ];

        // Spaceship movement controls with collision detection
        if window.is_key_down(Key::W) { spaceship.move_forward(&celestial_bodies); }
        if window.is_key_down(Key::S) { spaceship.move_backward(&celestial_bodies); }
        if window.is_key_down(Key::A) { spaceship.move_left(&celestial_bodies); }
        if window.is_key_down(Key::D) { spaceship.move_right(&celestial_bodies); }
        if window.is_key_down(Key::Space) { spaceship.move_up(&celestial_bodies); }
        if window.is_key_down(Key::LeftShift) || window.is_key_down(Key::RightShift) { spaceship.move_down(&celestial_bodies); }

        // Actualizar animación de la nave
        spaceship.update_animation();

        // Mouse camera rotation with right click (horizontal only)
        if let Some((mouse_x, mouse_y)) = window.get_mouse_pos(minifb::MouseMode::Discard) {
            if window.get_mouse_down(minifb::MouseButton::Right) {
                if let Some((last_x, _last_y)) = last_mouse_pos {
                    let delta_x = mouse_x - last_x;
                    camera.update_rotation(delta_x);
                }
                last_mouse_pos = Some((mouse_x, mouse_y));
            } else {
                last_mouse_pos = None;
            }
        }

        // Scroll wheel zoom control
        if let Some(scroll) = window.get_scroll_wheel() {
            camera.zoom(scroll.1);
        }

        let view_matrix = camera.get_view_matrix(&spaceship.position, spaceship.camera_yaw);

        // Render orbital paths for all planets with their inclinations
        render_orbit(&mut framebuffer, 45.0, 5.0_f32.to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Rocky
        render_orbit(&mut framebuffer, 60.0, (-8.0_f32).to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Gas Giant
        render_orbit(&mut framebuffer, 53.0, 12.0_f32.to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Ice
        render_orbit(&mut framebuffer, 38.0, (-6.0_f32).to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Desert
        render_orbit(&mut framebuffer, 72.0, 15.0_f32.to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Volcanic
        render_orbit(&mut framebuffer, 49.0, (-10.0_f32).to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Ocean
        render_orbit(&mut framebuffer, 57.0, 18.0_f32.to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Purple
        render_orbit(&mut framebuffer, 67.0, (-14.0_f32).to_radians(), &view_matrix, &projection_matrix, &viewport_matrix);  // Ringed

        // Render Sun (center, no rotation, much bigger size)
        let sun_rotation = Vec3::new(0.0, 0.0, 0.0); // No rotation
        let sun_model = create_model_matrix(Vec3::new(0.0, 0.0, 0.0), 8.0, sun_rotation);
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