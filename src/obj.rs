use tobj;
use nalgebra_glm::{Vec2, Vec3};
use crate::vertex::Vertex;

pub struct Obj {
    meshes: Vec<Mesh>,
}

struct Mesh {
    vertices: Vec<Vec3>,
    normals: Vec<Vec3>,
    texcoords: Vec<Vec2>,
    indices: Vec<u32>,
}

impl Obj {
    pub fn load(filename: &str) -> Result<Self, tobj::LoadError> {
        let (models, _) = tobj::load_obj(filename, &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        })?;

        let meshes = models.into_iter().map(|model| {
            let mesh = model.mesh;
            Mesh {
                vertices: mesh.positions.chunks(3)
                    .map(|v| Vec3::new(v[0], -v[1], -v[2]))
                    .collect(),
                normals: mesh.normals.chunks(3)
                    .map(|n| Vec3::new(n[0], -n[1], -n[2]))
                    .collect(),
                texcoords: mesh.texcoords.chunks(2)
                    .map(|t| Vec2::new(t[0], 1.0 - t[1]))
                    .collect(),
                indices: mesh.indices,
            }
        }).collect();

        Ok(Obj { meshes })
    }

    pub fn get_vertex_array(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();

        for mesh in &self.meshes {
            for &index in &mesh.indices {
                let position = mesh.vertices[index as usize];
                let normal = mesh.normals.get(index as usize)
                    .cloned()
                    .unwrap_or(Vec3::new(0.0, 1.0, 0.0));
                let tex_coords = mesh.texcoords.get(index as usize)
                    .cloned()
                    .unwrap_or(Vec2::new(0.0, 0.0));

                vertices.push(Vertex::new(position, normal, tex_coords));
            }
        }

        vertices
    }

    // Nuevo método para obtener vértices e índices por separado
    // Esto es necesario para recorrer manualmente las caras como pide el ejercicio
    pub fn get_vertex_and_index_arrays(&self) -> (Vec<Vertex>, Vec<u32>) {
        let mut all_vertices = Vec::new();
        let mut all_indices = Vec::new();
        let mut vertex_offset = 0u32;

        for mesh in &self.meshes {
            // Agregar todos los vértices únicos de este mesh
            for i in 0..mesh.vertices.len() {
                let position = mesh.vertices[i];
                let normal = mesh.normals.get(i)
                    .cloned()
                    .unwrap_or(Vec3::new(0.0, 1.0, 0.0));
                let tex_coords = mesh.texcoords.get(i)
                    .cloned()
                    .unwrap_or(Vec2::new(0.0, 0.0));

                all_vertices.push(Vertex::new(position, normal, tex_coords));
            }

            // Agregar los índices ajustados por el offset
            for &index in &mesh.indices {
                all_indices.push(index + vertex_offset);
            }

            vertex_offset += mesh.vertices.len() as u32;
        }

        (all_vertices, all_indices)
    }

    // Método para obtener información del modelo
    pub fn get_model_info(&self) -> String {
        let total_vertices: usize = self.meshes.iter().map(|m| m.vertices.len()).sum();
        let total_indices: usize = self.meshes.iter().map(|m| m.indices.len()).sum();
        let total_triangles = total_indices / 3;

        format!(
            "Modelo cargado:\n- {} meshes\n- {} vértices\n- {} triángulos",
            self.meshes.len(),
            total_vertices,
            total_triangles
        )
    }
}