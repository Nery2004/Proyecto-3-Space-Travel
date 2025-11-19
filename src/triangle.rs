use nalgebra_glm::{Vec3, dot};
use crate::fragment::Fragment;
use crate::vertex::Vertex;
use crate::color::Color;
use crate::Uniforms;

pub fn triangle(v1: &Vertex, v2: &Vertex, v3: &Vertex, uniforms: &Uniforms) -> Vec<Fragment> {
  let mut fragments = Vec::new();

  // Perform perspective division to get screen-space coordinates
  let a_w = v1.transformed_position.w;
  let b_w = v2.transformed_position.w;
  let c_w = v3.transformed_position.w;

  if a_w.abs() < 1e-6 || b_w.abs() < 1e-6 || c_w.abs() < 1e-6 {
      return fragments;
  }

  let a = Vec3::new(
      v1.transformed_position.x / a_w,
      v1.transformed_position.y / a_w,
      v1.transformed_position.z / a_w,
  );
  let b = Vec3::new(
      v2.transformed_position.x / b_w,
      v2.transformed_position.y / b_w,
      v2.transformed_position.z / b_w,
  );
  let c = Vec3::new(
      v3.transformed_position.x / c_w,
      v3.transformed_position.y / c_w,
      v3.transformed_position.z / c_w,
  );

  // Apply viewport transformation
  let transform_to_screen = |pos: Vec3| -> Vec3 {
      let screen_x = (pos.x * 0.5 + 0.5) * 800.0;
      let screen_y = (1.0 - (pos.y * 0.5 + 0.5)) * 600.0;
      Vec3::new(screen_x, screen_y, pos.z)
  };

  let a_screen = transform_to_screen(a);
  let b_screen = transform_to_screen(b);
  let c_screen = transform_to_screen(c);

  let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&a_screen, &b_screen, &c_screen);

  // Clamp to screen bounds
  let min_x = min_x.max(0);
  let min_y = min_y.max(0);
  let max_x = max_x.min(799);
  let max_y = max_y.min(599);

  // Skip if completely outside screen
  if min_x > 799 || min_y > 599 || max_x < 0 || max_y < 0 {
      return fragments;
  }

  let triangle_area = edge_function(&a_screen, &b_screen, &c_screen);

  if triangle_area.abs() < 1e-6 {
      return fragments;
  }

  // Backface culling
  if triangle_area < 0.0 {
      return fragments;
  }

  for y in min_y..=max_y {
    for x in min_x..=max_x {
      let point = Vec3::new(x as f32 + 0.5, y as f32 + 0.5, 0.0);

      let (w1, w2, w3) = barycentric_coordinates(&point, &a_screen, &b_screen, &c_screen, triangle_area);

      if w1 >= 0.0 && w2 >= 0.0 && w3 >= 0.0 {
        let inv_w = 1.0/a_w * w1 + 1.0/b_w * w2 + 1.0/c_w * w3;
        let w = 1.0/inv_w;

        let vertex_position = (v1.position * (w1 / a_w) + v2.position * (w2 / b_w) + v3.position * (w3 / c_w)) * w;
        
        let depth = a_screen.z * w1 + b_screen.z * w2 + c_screen.z * w3;

        fragments.push(Fragment::new_with_vertex_position(
            x as f32, 
            y as f32, 
            Color::new(255, 255, 255),
            depth,
            vertex_position
        ));
      }
    }
  }

  fragments
}

fn calculate_bounding_box(v1: &Vec3, v2: &Vec3, v3: &Vec3) -> (i32, i32, i32, i32) {
    let min_x = v1.x.min(v2.x).min(v3.x).floor() as i32;
    let min_y = v1.y.min(v2.y).min(v3.y).floor() as i32;
    let max_x = v1.x.max(v2.x).max(v3.x).ceil() as i32;
    let max_y = v1.y.max(v2.y).max(v3.y).ceil() as i32;

    (min_x, min_y, max_x, max_y)
}

fn barycentric_coordinates(p: &Vec3, a: &Vec3, b: &Vec3, c: &Vec3, area: f32) -> (f32, f32, f32) {
    let w1 = edge_function(b, c, p) / area;
    let w2 = edge_function(c, a, p) / area;
    let w3 = edge_function(a, b, p) / area;

    (w1, w2, w3)
}

fn edge_function(a: &Vec3, b: &Vec3, c: &Vec3) -> f32 {
    (c.x - a.x) * (b.y - a.y) - (c.y - a.y) * (b.x - a.x)
}