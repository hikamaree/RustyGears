#![allow(non_snake_case)]
#![allow(dead_code)]

use std::path::Path;
use std::ffi::CStr;

use cgmath::{vec2, vec3, vec4, Vector3, Matrix4};
use tobj;

use super::mesh::{ Mesh, Texture, Vertex };
use super::shader::Shader;
use super::utils::*;

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub textures_loaded: Vec<Texture>,
    directory: String,
    position: Vector3<f32>
}

impl Model {
    fn default() -> Self {
        Self {
            meshes: Vec::default(),
            textures_loaded: Vec::default(),
            directory: String::default(),
            position: vec3(0.0, 0.0, 0.0)
        }
    }

    pub fn new(path: &str, position: Vector3<f32>) -> Model {
        let mut model = Model::default();
        model.loadModel(path);
        model.position = position;
        model
    }

    pub fn Draw(&self, shader: &Shader) {
        let mmodel = Matrix4::from_translation(self.position);
        shader.setMat4(c_str!("model"), &mmodel);

        unsafe {
            for mesh in &self.meshes {
                mesh.Draw(shader);
            }
        }
    }

    fn loadModel(&mut self, path: &str) {
        let path = Path::new(path);

        self.directory = path.parent().unwrap_or_else(|| Path::new("")).to_str().unwrap().into();
        let obj = tobj::load_obj(path);

        let (models, materials) = obj.unwrap();
        for model in models {
            let mesh = &model.mesh;
            let num_vertices = mesh.positions.len() / 3;

            let mut vertices: Vec<Vertex> = Vec::with_capacity(num_vertices);
            let indices: Vec<u32> = mesh.indices.clone();

            let (p, n, t) = (&mesh.positions, &mesh.normals, &mesh.texcoords);
            for i in 0..num_vertices {
                let position = vec3(p[i*3], p[i*3+1], p[i*3+2]);

                let normal = if n.len() > i*3+2 {
                    vec3(n[i*3], n[i*3+1], n[i*3+2])
                } else {
                    vec3(0.0, 0.0, 0.0)
                };

                let texcoords = if t.len() > i*2+1 {
                    vec2(t[i*2], t[i*2+1])
                } else {
                    vec2(0.0, 0.0)
                };

                vertices.push(Vertex {
                    Position: position,
                    Normal: normal,
                    TexCoords: texcoords,
                    Color: vec4(1.0, 1.0, 1.0, 1.0),
                    ..Vertex::default()
                });
            }

            let mut textures = Vec::new();
            if let Some(material_id) = mesh.material_id {
                let material = &materials[material_id];

                if !material.diffuse_texture.is_empty() {
                    let texture = self.loadMaterialTexture(&material.diffuse_texture, "texture_diffuse");
                    textures.push(texture);
                } else {
                    for vertex in vertices.iter_mut() {
                        if material.diffuse.len() > 3 {
                            vertex.Color = vec4(material.diffuse[0], material.diffuse[1], material.diffuse[2], material.diffuse[3]);
                        } else {
                            vertex.Color = vec4(material.diffuse[0], material.diffuse[1], material.diffuse[2], 1.0);
                        }
                    }
                }
                if !material.specular_texture.is_empty() {
                    let texture = self.loadMaterialTexture(&material.specular_texture, "texture_specular");
                    textures.push(texture);
                }
                if !material.normal_texture.is_empty() {
                    let texture = self.loadMaterialTexture(&material.normal_texture, "texture_normal");
                    textures.push(texture);
                }
            }

            self.meshes.push(Mesh::new(vertices, indices, textures));
        }

    }

    fn loadMaterialTexture(&mut self, path: &str, typeName: &str) -> Texture {
        {
            let texture = self.textures_loaded.iter().find(|t| t.path == path);
            if let Some(texture) = texture {
                return texture.clone();
            }
        }

        let texture = Texture {
            id: loadTexture(format!("{}/{}", self.directory, path).as_str()),
            type_: typeName.into(),
            path: path.into()
        };
        self.textures_loaded.push(texture.clone());
        texture
    }
}
