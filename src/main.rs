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
              shade_ice_planet, shade_desert_planet, shade_volcanic_planet};

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
    position: Vec3,
    yaw: f32,
    pitch: f32,
    speed: f32,
}

impl Camera {
    fn new(position: Vec3) -> Self {
        Self {
            position,
            yaw: -90.0,
            pitch: 0.0,
            speed: 0.1,
        }
    }

    fn get_direction(&self) -> Vec3 {
        let yaw_rad = self.yaw.to_radians();
        let pitch_rad = self.pitch.to_radians();
        
        Vec3::new(
            yaw_rad.cos() * pitch_rad.cos(),
            pitch_rad.sin(),
            yaw_rad.sin() * pitch_rad.cos(),
        ).normalize()
    }

    fn get_view_matrix(&self) -> Mat4 {
        let direction = self.get_direction();
        let target = self.position + direction;
        look_at(&self.position, &target, &Vec3::new(0.0, 1.0, 0.0))
    }

    fn update_rotation(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw += delta_x * 0.1;
        self.pitch -= delta_y * 0.1;
        self.pitch = self.pitch.clamp(-89.0, 89.0);
    }

    fn move_forward(&mut self) {
        let direction = self.get_direction();
        self.position += direction * self.speed;
    }

    fn move_backward(&mut self) {
        let direction = self.get_direction();
        self.position -= direction * self.speed;
    }

    fn move_left(&mut self) {
        let direction = self.get_direction();
        let right = direction.cross(&Vec3::new(0.0, 1.0, 0.0)).normalize();
        self.position -= right * self.speed;
    }

    fn move_right(&mut self) {
        let direction = self.get_direction();
        let right = direction.cross(&Vec3::new(0.0, 1.0, 0.0)).normalize();
        self.position += right * self.speed;
    }

    fn move_up(&mut self) {
        self.position.y += self.speed;
    }

    fn move_down(&mut self) {
        self.position.y -= self.speed;
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

fn main() {
    let mut window = Window::new(
        "Lab 5 - Sistema Solar (WASD: mover, Space/Shift: arriba/abajo, Mouse: rotar cámara)",
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

    let mut camera = Camera::new(Vec3::new(0.0, 4.0, 15.0));
    let mut time = 0.0;
    let mut last_mouse_pos: Option<(f32, f32)> = None;

    println!("Controles:");
    println!("  WASD: Mover cámara");
    println!("  Space/Shift: Subir/Bajar");
    println!("  Mouse: Rotar cámara");
    println!("  ESC: Salir");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        framebuffer.clear();
        time += 0.01;

        // Camera movement controls
        if window.is_key_down(Key::W) { camera.move_forward(); }
        if window.is_key_down(Key::S) { camera.move_backward(); }
        if window.is_key_down(Key::A) { camera.move_left(); }
        if window.is_key_down(Key::D) { camera.move_right(); }
        if window.is_key_down(Key::Space) { camera.move_up(); }
        if window.is_key_down(Key::LeftShift) { camera.move_down(); }

        // Mouse camera control - solo cuando se presiona botón derecho
        if window.get_mouse_down(minifb::MouseButton::Right) {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Discard) {
                if let Some((last_x, last_y)) = last_mouse_pos {
                    let delta_x = mx - last_x;
                    let delta_y = my - last_y;
                    camera.update_rotation(delta_x, delta_y);
                }
                last_mouse_pos = Some((mx, my));
            }
        } else {
            // Resetear posición del mouse cuando se suelta el botón
            last_mouse_pos = None;
        }

        let view_matrix = camera.get_view_matrix();

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
        let desert_orbit_radius = 6.5;
        let desert_pos = Vec3::new(
            (desert_angle + PI).cos() * desert_orbit_radius,
            0.2,
            (desert_angle + PI).sin() * desert_orbit_radius,
        );
        let desert_rotation = Vec3::new(0.0, time * 0.6, 0.0);
        let desert_model = create_model_matrix(desert_pos, 0.6, desert_rotation);
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
        let volcanic_orbit_radius = 14.0;
        let volcanic_pos = Vec3::new(
            (volcanic_angle + PI * 1.5).cos() * volcanic_orbit_radius,
            -0.5,
            (volcanic_angle + PI * 1.5).sin() * volcanic_orbit_radius,
        );
        let volcanic_rotation = Vec3::new(0.0, time * 0.7, 0.0);
        let volcanic_model = create_model_matrix(volcanic_pos, 0.9, volcanic_rotation);
        let volcanic_uniforms = Uniforms {
            model_matrix: volcanic_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 6, // Volcanic planet shader
        };
        render_model(&mut framebuffer, &volcanic_uniforms, &planet_vertices, &planet_indices);

        // Render Spaceship (TIE Fighter) - Static position
        let nave_pos = Vec3::new(6.0, 4.0, 9.0); // Posición fija más alejada y arriba
        let nave_rotation = Vec3::new(0.0, PI * 0.75, 0.0); // Ángulo fijo
        let nave_model = create_model_matrix(nave_pos, 0.3, nave_rotation);
        let nave_uniforms = Uniforms {
            model_matrix: nave_model,
            view_matrix,
            projection_matrix,
            viewport_matrix,
            time,
            shader_type: 3, // No shader (default color)
        };
        render_model(&mut framebuffer, &nave_uniforms, &nave_vertices, &nave_indices);

        window
            .update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}